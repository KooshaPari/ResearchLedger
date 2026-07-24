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
import {
  loadPlaywright,
  parseFlags,
  resolveProfile,
  readInt,
  canonicalUrl,
  openAuthenticatedSession,
  scrollAndCollect,
  getProbe,
  writeCapture,
} from "./_capture_common.mjs";

const args = parseFlags(process.argv);
const profile = resolveProfile(args, "x-profile");
const output = args.get("--output");
if (!output) throw new Error("Missing required flag: --output <path>");
const url = args.get("--url") ?? "https://x.com/i/bookmarks";
const maxRounds = readInt(args, "--max-rounds", 240);
const waitMs = readInt(args, "--wait-ms", 1200);
const minLength = readInt(args, "--min-length", 40);

const { chromium } = await loadPlaywright();
const { context, page } = await openAuthenticatedSession({
  chromium,
  profile,
  url,
  logMessage:
    "ResearchLedger X capture is running in the authenticated profile. Complete login if prompted.",
});

const probe = getProbe({ mode: "x-article" });

const build = ({ href, text }) => {
  const u = canonicalUrl(href);
  const author = text.match(/@([A-Za-z0-9_]{1,15})/)?.[1] ?? "";
  return { url: u, text, author };
};

const posts = await scrollAndCollect({
  page,
  selector: "a[href*='/status/']",
  probe,
  build,
  minLength,
  maxRounds,
  waitMs,
});

const payload = {
  version: 1,
  capturedAt: new Date().toISOString(),
  source: "x-playwright-authenticated-session",
  bookmarksUrl: url,
  posts: [...posts.values()],
};

await writeCapture(
  output,
  payload,
  `Captured ${payload.posts.length} unique X bookmarks to ${output}`,
);
await context.close();
