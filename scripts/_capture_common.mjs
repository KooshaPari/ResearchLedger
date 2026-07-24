#!/usr/bin/env node
/**
 * ResearchLedger – shared scroll/collect loop for authenticated Playwright
 * capture scripts (LinkedIn, Reddit, X). Each provider supplies:
 *
 *   - The URL pattern of the saved/bookmarks page.
 *   - A selector that matches each saved item's anchor.
 *   - A function body (string) that turns one anchor into either a post
 *     record `{ url, text, ... }` or `null` to skip the row. The body is
 *     sandboxed inside a page-context function we build here.
 *
 * The helper owns: persistent profile launch, scroll loop with stagnation
 * abort, dedup on canonical URL, JSON output.
 *
 * Privacy guarantees (inherited by every script that uses this helper):
 *   - No credentials, cookies, or text leave the user's machine.
 *   - The persistent profile directory is owned by the user and is the only
 *     place authentication state is stored.
 *   - Capture output is written to a local file the caller specifies.
 */

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";

/**
 * Load Playwright from an absolute path (when bundled as a Tauri resource) or
 * a bare module specifier (when running from the development tree).
 *
 * @returns {Promise<{ chromium: any }>}
 */
export async function loadPlaywright() {
  const playwrightModule =
    process.env.RESEARCHLEDGER_PLAYWRIGHT_MODULE ?? "playwright";
  const spec = playwrightModule.startsWith("/")
    ? pathToFileURL(playwrightModule).href
    : playwrightModule;
  return import(spec);
}

/**
 * Parse `--flag value` pairs from process.argv starting at index 2.
 *
 * @param {readonly string[]} argv
 * @returns {Map<string, string>}
 */
export function parseFlags(argv) {
  const args = new Map();
  for (let index = 2; index < argv.length; index += 2) {
    args.set(argv[index], argv[index + 1]);
  }
  return args;
}

/**
 * Resolve a profile directory from the `--profile` flag, falling back to
 * `${HOME}/Library/Application Support/ResearchLedger/<defaultSubdir>`.
 *
 * @param {Map<string, string>} flags
 * @param {string} defaultSubdir
 * @returns {string}
 */
export function resolveProfile(flags, defaultSubdir) {
  return (
    flags.get("--profile") ??
    `${process.env.HOME ?? ""}/Library/Application Support/ResearchLedger/${defaultSubdir}`
  );
}

/**
 * Read an integer flag (with default) from the parsed args.
 *
 * @param {Map<string, string>} flags
 * @param {string} name
 * @param {number} fallback
 * @returns {number}
 */
export function readInt(flags, name, fallback) {
  const raw = flags.get(name);
  if (raw === undefined) return fallback;
  const parsed = Number(raw);
  return Number.isFinite(parsed) ? parsed : fallback;
}

/**
 * Normalize a post URL by stripping query string + trailing slash so the same
 * post re-loaded with `?utm_source=...` collapses to the same key.
 *
 * @param {string} href
 * @returns {string}
 */
export function canonicalUrl(href) {
  return href.split("?")[0].replace(/\/$/, "");
}

/**
 * Open a Chromium persistent context for an authenticated session, navigate
 * to the saved/bookmarks URL, and pause briefly so the user can complete any
 * pending login in the launched window.
 *
 * @param {{
 *   chromium: any,
 *   profile: string,
 *   url: string,
 *   warmupMs?: number,
 *   logMessage: string,
 * }} params
 */
export async function openAuthenticatedSession(params) {
  const { chromium, profile, url, warmupMs = 2500, logMessage } = params;
  const context = await chromium.launchPersistentContext(profile, {
    headless: false,
  });
  const page = context.pages()[0] ?? (await context.newPage());
  await page.goto(url, { waitUntil: "domcontentloaded" });
  console.error(logMessage);
  await page.waitForTimeout(warmupMs);
  return { context, page };
}

/**
 * Scroll the saved/bookmarks page in rounds, collecting posts via the
 * caller-supplied extractor body. The body is wrapped in a function that
 * runs inside the page context and receives the per-link anchor element.
 *
 * SECURITY: the body comes from a sibling capture script in this repo. It
 * never includes user input.
 *
 * @param {{
 *   page: import('playwright').Page,
 *   selector: string,
 *   extractorBody: string,
 *   minLength: number,
 *   maxRounds: number,
 *   waitMs: number,
 *   unchangedAbort?: number,
 * }} params
 */
export async function scrollAndCollect(params) {
  const {
    page,
    selector,
    extractorBody,
    minLength,
    maxRounds,
    waitMs,
    unchangedAbort = 8,
  } = params;

  const posts = new Map();
  let unchangedRounds = 0;
  const wrapped = wrapExtractorBody(extractorBody);

  for (let round = 0; round < maxRounds && unchangedRounds < unchangedAbort; round += 1) {
    const before = posts.size;
    const rows = await page.locator(selector).evaluateAll((links, src) => {
      // eslint-disable-next-line no-new-func
      const fn = new Function("link", `return (${src})(link);`);
      return links.map((link) => fn(link));
    }, wrapped);

    for (const row of rows) {
      if (!row || typeof row.url !== "string" || !row.url) continue;
      const text = typeof row.text === "string" ? row.text : "";
      if (text.length < minLength) continue;
      if (!posts.has(row.url)) posts.set(row.url, row);
    }

    unchangedRounds = posts.size === before ? unchangedRounds + 1 : 0;
    await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
    await page.waitForTimeout(waitMs);
  }

  return posts;
}

/**
 * Build the page-context extractor that runs inside `evaluateAll`. Each
 * provider script supplies a body like:
 *
 *   const href = canonicalUrl(link.href);
 *   if (!/\/status\/\d+/.test(href)) return null;
 *   const article = closestAncestor(link, ["article"]);
 *   return { url: href, text: flattenText(article) };
 *
 * Helpers `canonicalUrl`, `flattenText`, `closestAncestor` are injected.
 *
 * @param {string} body
 */
function wrapExtractorBody(body) {
  // eslint-disable-next-line no-new-func
  return new Function(
    "link",
    `
    const canonicalUrl = (href) => href.split('?')[0].replace(/\\/$/, '');
    const flattenText = (el) => (el && el.innerText ? el.innerText : '').replace(/\\s+/g, ' ').trim();
    const closestAncestor = (link, selectors) => {
      for (const sel of selectors) {
        const m = link.closest(sel);
        if (m) return m;
      }
      return link.parentElement;
    };
    try {
      return (function (link) { ${body} })(link);
    } catch (err) {
      return null;
    }
    `,
  ).toString();
}

/**
 * Write the capture payload to disk, creating parent directories as needed.
 *
 * @param {string} outputPath
 * @param {unknown} payload
 * @param {string} successMessage
 */
export async function writeCapture(outputPath, payload, successMessage) {
  await fs.mkdir(path.dirname(path.resolve(outputPath)), { recursive: true });
  await fs.writeFile(outputPath, `${JSON.stringify(payload, null, 2)}\n`, "utf8");
  console.error(successMessage);
}
