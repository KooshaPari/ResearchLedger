#!/usr/bin/env node
/**
 * ResearchLedger – Reddit saved-posts capture (authenticated).
 *
 * Reddit does not expose a general-purpose public API for the "Saved" tab.
 * We sign in via a *dedicated persistent Chromium profile* (managed entirely
 * by the user) and scroll the Saved page. Each post is keyed by URL so
 * re-runs are idempotent. The script writes a JSON capture consumable by
 * the `import_reddit_capture` / `import_reddit_html` Tauri commands.
 *
 * Privacy guarantees:
 *   - No credentials, cookies, or text leave this machine.
 *   - The persistent profile lives at the configured `profile` path; if it
 *     does not yet exist Playwright will create it on first launch.
 *   - The capture JSON contains only public Reddit post URLs and text
 *     scraped from the saved view.
 *
 * Usage:
 *   reddit_capture.mjs \
 *     --output <path/to/reddit-capture.json> \
 *     --profile <persistent-chromium-profile-dir> \
 *     --url    https://www.reddit.com/user/me/saved
 *
 * Flags (all optional except --output):
 *   --profile      Persistent profile directory. Defaults to
 *                   $HOME/Library/Application Support/ResearchLedger/reddit-profile.
 *   --output       Destination .json file (required).
 *   --url          Saved page URL (defaults above).
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
  args.get("--profile") ??
  `${process.env.HOME ?? ""}/Library/Application Support/ResearchLedger/reddit-profile`;
const output = args.get("--output");
if (!output) throw new Error("Missing required flag: --output <path>");
const url = args.get("--url") ?? "https://www.reddit.com/user/me/saved";
const maxRounds = Number(args.get("--max-rounds") ?? 240);
const waitMs = Number(args.get("--wait-ms") ?? 1200);
const minLength = Number(args.get("--min-length") ?? 40);

const context = await chromium.launchPersistentContext(profile, { headless: false });
const page = context.pages()[0] ?? (await context.newPage());
await page.goto(url, { waitUntil: "domcontentloaded" });
console.error(
  "ResearchLedger Reddit capture is running in the authenticated profile. Complete login if prompted.",
);
await page.waitForTimeout(2500);

const posts = new Map();
let unchangedRounds = 0;
const selector = "a[href*='/r/'][href*='/comments/']";
for (let round = 0; round < maxRounds && unchangedRounds < 8; round += 1) {
  const before = posts.size;
  const rows = await page.locator(selector).evaluateAll((links) =>
    links.map((link) => {
      const href = link.href.split("?")[0].replace(/\/$/, "");
      const article =
        link.closest("article, shreddit-post, div[data-testid='post-container']") ??
        link.parentElement;
      const text = (article?.innerText ?? link.innerText ?? "")
        .replace(/\s+/g, " ")
        .trim();
      const subreddit = href.match(/\/r\/([^/]+)\//)?.[1] ?? null;
      return { url: href, text, subreddit };
    }),
  );
  for (const post of rows) {
    if (!post.url || post.text.length < minLength) continue;
    if (!posts.has(post.url)) posts.set(post.url, post);
  }
  unchangedRounds = posts.size === before ? unchangedRounds + 1 : 0;
  await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
  await page.waitForTimeout(waitMs);
}

const payload = {
  version: 1,
  capturedAt: new Date().toISOString(),
  source: "reddit-playwright-authenticated-session",
  savedUrl: url,
  posts: [...posts.values()],
};
await fs.mkdir(path.dirname(path.resolve(output)), { recursive: true });
await fs.writeFile(output, `${JSON.stringify(payload, null, 2)}\n`, "utf8");
console.error(`Captured ${payload.posts.length} unique Reddit saved posts to ${output}`);
await context.close();
