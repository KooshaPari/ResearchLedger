#!/usr/bin/env node
/**
 * ResearchLedger – LinkedIn saved-posts capture (authenticated).
 *
 * LinkedIn does not expose a public API for the "Saved" tab. We sign in via a
 * dedicated persistent Chromium profile (managed entirely by the user) and
 * scroll the Saved page. Each post is keyed by activity URN so re-runs are
 * idempotent. The script writes a JSON capture consumable by the
 * `import_linkedin_capture` / `import_linkedin_html` Tauri commands.
 *
 * Usage:
 *   linkedin_capture.mjs \
 *     --output <path/to/linkedin-capture.json> \
 *     --profile <persistent-chromium-profile-dir> \
 *     --url    https://www.linkedin.com/my-items/saved-posts/
 *
 * Flags (all optional except --output):
 *   --profile      Persistent profile directory. Defaults to
 *                  $HOME/Library/Application Support/ResearchLedger/linkedin-profile.
 *   --output       Destination .json file (required).
 *   --url          Saved page URL (defaults above).
 *   --max-rounds   Bound on scroll iterations (default 240; ~8 with no growth aborts).
 *   --wait-ms      Pause between scrolls in ms (default 1200).
 *   --min-length   Skip posts with body text shorter than this (default 40 chars).
 */
import { makeProviderBuilder, runCaptureSession } from "./_capture_common.mjs";

await runCaptureSession({
  providerName: "LinkedIn",
  argv: process.argv,
  profileSubdir: "linkedin",
  defaultUrl: "https://www.linkedin.com/my-items/saved-posts/",
  probeMode: "linkedin-article",
  selector: "a[href*='urn:li:activity:']",
  build: makeProviderBuilder("linkedin"),
});
