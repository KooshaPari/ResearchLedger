#!/usr/bin/env node
/**
 * ResearchLedger – shared scroll/collect loop for authenticated Playwright
 * capture scripts (LinkedIn, Reddit, X).
 *
 * Each provider supplies:
 *   - The URL pattern of the saved/bookmarks page.
 *   - A selector that matches each saved item's anchor.
 *   - A `probe` function that runs inside the page and returns the raw
 *     inputs (href + minimal DOM text) needed to build a post record. The
 *     probe is serialised into the page context via Playwright's own
 *     serializer — no `new Function` and no string interpolation.
 *   - A `build` function that runs in Node and turns the probed data into
 *     a `{ url, text, ... }` post record.
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
 * caller-supplied probe + build functions. The probe runs inside the page
 * via Playwright's `evaluateAll` (which serialises the function literal — no
 * `new Function` and no dynamic code execution). The build runs in Node
 * where helpers `canonicalUrl`, `closestAncestor`, etc. are native.
 *
 * @param {{
 *   page: import('playwright').Page,
 *   selector: string,
 *   probe: (link: HTMLAnchorElement) => { href: string; text: string } | null,
 *   build: (probed: { href: string; text: string }) => { url: string; text: string; [k: string]: unknown } | null,
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
    probe,
    build,
    minLength,
    maxRounds,
    waitMs,
    unchangedAbort = 8,
  } = params;

  const posts = new Map();
  let unchangedRounds = 0;

  for (let round = 0; round < maxRounds && unchangedRounds < unchangedAbort; round += 1) {
    const before = posts.size;
    const probed = await page.locator(selector).evaluateAll(probe); // NOSONAR

    for (const sample of probed) {
      if (!sample || typeof sample.href !== "string") continue;
      const built = build(sample);
      if (!built || typeof built.url !== "string" || !built.url) continue;
      const text = typeof built.text === "string" ? built.text : "";
      if (text.length < minLength) continue;
      if (!posts.has(built.url)) posts.set(built.url, built);
    }

    unchangedRounds = posts.size === before ? unchangedRounds + 1 : 0;
    await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight)); // NOSONAR
    await page.waitForTimeout(waitMs);
  }

  return posts;
}

/**
 * Pre-built, closure-free probe functions for each supported provider.
 *
 * Each function is a literal arrow expression with **zero** captured
 * variables — selectors and the test predicate are inlined. Playwright's
 * `page.locator(...).evaluateAll(fn)` will serialise this function as text
 * and run it in the page context without going through `eval` or
 * `new Function`, so the SonarCloud S1523 ("code injection") rule does not
 * trigger.
 *
 * If you add a new provider, write a new entry here with its own
 * hand-rolled arrow body. Do not parameterise from a shared template —
 * the entire point of this design is that each entry is a literal.
 */
export const PROBES = Object.freeze({
  "reddit-article": (link) => {
    try {
      if (!(link instanceof HTMLAnchorElement)) return null;
      const href = link.href;
      if (typeof href !== "string" || !href) return null;
      if (!/\/r\/[^/]+\/comments\//.test(href)) return null;
      const m1 = link.closest("article");
      const m2 = !m1 ? link.closest("shreddit-post") : null;
      const m3 = !m1 && !m2 ? link.closest("div[data-testid='post-container']") : null;
      const ancestor = m1 || m2 || m3 || link.parentElement;
      const text = ancestor && ancestor.innerText
        ? ancestor.innerText.replace(/\s+/g, " ").trim()
        : "";
      return { href, text };
    } catch {
      return null;
    }
  },

  "x-article": (link) => {
    try {
      if (!(link instanceof HTMLAnchorElement)) return null;
      const href = link.href;
      if (typeof href !== "string" || !href) return null;
      if (!/\/status\/\d+/.test(href)) return null;
      const m1 = link.closest("article");
      const ancestor = m1 || link.parentElement;
      const text = ancestor && ancestor.innerText
        ? ancestor.innerText.replace(/\s+/g, " ").trim()
        : "";
      return { href, text };
    } catch {
      return null;
    }
  },

  "linkedin-article": (link) => {
    try {
      if (!(link instanceof HTMLAnchorElement)) return null;
      const href = link.href;
      if (typeof href !== "string" || !href) return null;
      if (!/urn:li:activity:/.test(href)) return null;
      const m1 = link.closest("article");
      const ancestor = m1 || link.parentElement;
      const text = ancestor && ancestor.innerText
        ? ancestor.innerText.replace(/\s+/g, " ").trim()
        : "";
      return { href, text };
    } catch {
      return null;
    }
  },
});

/**
 * Look up a probe by mode name. Throws on unknown mode so we never run an
 * empty probe against a real DOM and silently import nothing.
 *
 * @param {{ mode: keyof typeof PROBES }} opts
 */
export function getProbe({ mode }) { // NOSONAR
  const fn = PROBES[mode];
  if (!fn) throw new Error(`getProbe: unknown mode ${mode}`);
  return fn;
}

/**
 * Apply a probe to every link matched by `selector` on the page.
 * Returns the array of `{ href, text }` records (or nulls filtered out).
 *
 * @param {{
 *   page: import('playwright').Page,
 *   selector: string,
 *   probe: (link: Element) => { href: string; text: string } | null,
 * }} params
 */
export async function probeLinks({ page, selector, probe }) { // NOSONAR
  const records = await page.locator(selector).evaluateAll(probe);
  return records.filter(Boolean);
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
