#!/usr/bin/env node
/**
 * ResearchLedger – LinkedIn reactions capture (authenticated).
 *
 * LinkedIn does not expose a public API for the reactions tab outside of
 * enterprise partner contracts. We sign in via a *dedicated persistent
 * Chromium profile* (managed entirely by the user) and scroll the
 * `/recent-activity/reactions/` page. Each post is keyed by URL so re-runs
 * are idempotent. The script writes a JSON capture consumable by the
 * `import_linkedin_capture` / `import_linkedin_html` Tauri commands.
 *
 * Privacy guarantees:
 *   - No credentials, cookies, or text leave this machine.
 *   - The persistent profile lives at the configured `profile` path; if it
 *     does not yet exist Playwright will create it on first launch.
 *   - The capture JSON contains only public LinkedIn post URLs and text
 *     scraped from the reactions view.
 *
 * Usage:
 *   linkedin_capture.mjs \
 *     --output <path/to/linkedin-capture.json> \
 *     --profile <persistent-chromium-profile-dir> \
 *     --url    https://www.linkedin.com/in/me/recent-activity/reactions/
 *
 * Flags (all optional except --output):
 *   --profile      Persistent profile directory. Defaults to
 *                   $HOME/Library/Application Support/ResearchLedger/linkedin-profile.
 *   --output       Destination .json file (defaults to linkedin-capture.json).
 *   --url          Reactions page URL (default above).
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
const profile = resolveProfile(args, "linkedin-profile");
const output = args.get("--output") ?? "linkedin-capture.json";
const url =
  args.get("--url") ?? "https://www.linkedin.com/in/me/recent-activity/reactions/";
const maxRounds = readInt(args, "--max-rounds", 240);
const waitMs = readInt(args, "--wait-ms", 1200);
const minLength = readInt(args, "--min-length", 40);

const { chromium } = await loadPlaywright();
const { context, page } = await openAuthenticatedSession({
  chromium,
  profile,
  url,
  logMessage:
    "ResearchLedger LinkedIn capture is running in the authenticated profile. Complete login if prompted.",
});

const probe = getProbe({ mode: "linkedin-article" });

const build = ({ href, text }) => ({ url: canonicalUrl(href), text });

const posts = await scrollAndCollect({
  page,
  selector: "a[href*='feed/update/urn:li:activity:']",
  probe,
  build,
  minLength,
  maxRounds,
  waitMs,
});

const payload = {
  version: 1,
  capturedAt: new Date().toISOString(),
  source: "linkedin-playwright-authenticated-session",
  activityUrl: url,
  posts: [...posts.values()],
};

await writeCapture(
  output,
  payload,
  `Captured ${payload.posts.length} unique LinkedIn posts to ${output}`,
);
await context.close();
