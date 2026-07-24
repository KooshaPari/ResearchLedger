#!/usr/bin/env node
/**
 * ResearchLedger – X (Twitter) bookmarks capture (authenticated).
 *
 * X does not expose a public bookmarks API outside of paid enterprise tiers.
 * We sign in via a *dedicated persistent Chromium profile* (managed entirely
 * by the user) and scroll the Bookmarks page. Each post is keyed by status
 * URL so re-runs are idempotent. The script writes a JSON capture consumable
 * by the `import_x_capture` / `import_x_html` Tauri commands.
 *
 * Privacy guarantees:
 *   - No credentials, cookies, or text leave this machine.
 *   - The persistent profile lives at the configured `profile` path; if it
 *     does not yet exist Playwright will create it on first launch.
 *   - The capture JSON contains only public X status URLs and text
 *     scraped from the bookmarks view.
 *
 * Usage:
 *   x_capture.mjs \
 *     --output <path/to/x-capture.json> \
 *     --profile <persistent-chromium-profile-dir> \
 *     --url    https://x.com/i/bookmarks
 *
 * Flags (all optional except --output):
 *   --profile      Persistent profile directory. Defaults to
 *                   $HOME/Library/Application Support/ResearchLedger/x-profile.
 *   --output       Destination .json file (required).
 *   --url          Bookmarks page URL (default above).
 *   --max-rounds   Bound on scroll iterations (default 240; ~8 with no growth aborts).
 *   --wait-ms      Pause between scrolls in ms (default 1200).
 *   --min-length   Skip posts with body text shorter than this (default 40 chars).
 */
import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";

const playwrightModule = process.env.RESEARCHLEDGER_PLAYWRIGHT_MODULE ?? "playwright";
const { chromium } = await import(
  playwrightModule.startsWith("/") ? pathToFileURL(playwrightModule).href : playwrightModule,
);

const args = new Map();
for (let index = 2; index < process.argv.length; index += 2) args.set(process.argv[index], process.argv[index + 1]);

const profile =
  args.get("--profile") ?? `${process.env.HOME ?? ""}/Library/Application Support/ResearchLedger/x-profile`;
const output = args.get("--output");
if (!output) throw new Error("Missing required flag: --output <path>");
const url = args.get("--url") ?? "https://x.com/i/bookmarks";
const maxRounds = Number(args.get("--max-rounds") ?? 240);
const waitMs = Number(args.get("--wait-ms") ?? 1200);
const minLength = Number(args.get("--min-length") ?? 40);

const context = await chromium.launchPersistentContext(profile, { headless: false });
const page = context.pages()[0] ?? (await context.newPage());
await page.goto(url, { waitUntil: "domcontentloaded" });
console.error(
  "ResearchLedger X capture is running in the authenticated profile. Complete login if prompted.",
);
await page.waitForTimeout(2500);

const posts = new Map();
let unchangedRounds = 0;
const selector = "a[href*='/status/']";
for (let round = 0; round < maxRounds && unchangedRounds < 8; round += 1) {
  const before = posts.size;
  const rows = await page.locator(selector).evaluateAll((links) =>
    links.map((link) => {
      const href = link.href.split("?")[0].replace(/\/$/, "");
      if (!/\/status\/\d+/.test(href)) return null;
      const article = link.closest("article") ?? link.parentElement;
      const text = (article?.innerText ?? link.innerText ?? "")
        .replace(/\s+/g, " ")
        .trim();
      const author = text.match(/@([A-Za-z0-9_]{1,15})/)?.[1] ?? "";
      return { url: href, text, author };
    }),
  );
  for (const post of rows) {
    if (!post || post.text.length < minLength) continue;
    if (!posts.has(post.url)) posts.set(post.url, post);
  }
  unchangedRounds = posts.size === before ? unchangedRounds + 1 : 0;
  await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
  await page.waitForTimeout(waitMs);
}

const payload = {
  version: 1,
  capturedAt: new Date().toISOString(),
  source: "x-playwright-authenticated-session",
  bookmarksUrl: url,
  posts: [...posts.values()],
};
await fs.mkdir(path.dirname(path.resolve(output)), { recursive: true });
await fs.writeFile(output, `${JSON.stringify(payload, null, 2)}\n`, "utf8");
console.error(`Captured ${payload.posts.length} unique X bookmarks to ${output}`);
await context.close();
