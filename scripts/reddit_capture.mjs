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
const profile = resolveProfile(args, "reddit-profile");
const output = args.get("--output");
if (!output) throw new Error("Missing required flag: --output <path>");
const url = args.get("--url") ?? "https://www.reddit.com/user/me/saved";
const maxRounds = readInt(args, "--max-rounds", 240);
const waitMs = readInt(args, "--wait-ms", 1200);
const minLength = readInt(args, "--min-length", 40);

const { chromium } = await loadPlaywright();
const { context, page } = await openAuthenticatedSession({
  chromium,
  profile,
  url,
  logMessage:
    "ResearchLedger Reddit capture is running in the authenticated profile. Complete login if prompted.",
});

const probe = getProbe({ mode: "reddit-article" });

const build = ({ href, text }) => {
  const u = canonicalUrl(href);
  const subreddit = u.match(/\/r\/([^/]+)\//)?.[1] ?? null;
  return { url: u, text, subreddit };
};

const posts = await scrollAndCollect({
  page,
  selector: "a[href*='/r/'][href*='/comments/']",
  probe,
  build,
  minLength,
  maxRounds,
  waitMs,
});

const payload = {
  version: 1,
  capturedAt: new Date().toISOString(),
  source: "reddit-playwright-authenticated-session",
  savedUrl: url,
  posts: [...posts.values()],
};

await writeCapture(
  output,
  payload,
  `Captured ${payload.posts.length} unique Reddit saved posts to ${output}`,
);
await context.close();
