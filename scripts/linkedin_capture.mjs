#!/usr/bin/env node
import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import {
  loadPlaywright,
  assertNonEmptyCapture,
  openAuthenticatedSession,
  parseFlags,
  readInt,
  resolveProfile,
} from "./_capture_common.mjs";

const flags = parseFlags(process.argv);
const profile = resolveProfile(flags, "linkedin-profile");
const output = flags.get("--output") ?? "linkedin-capture.json";
const url = flags.get("--url") ?? "https://www.linkedin.com/in/me/recent-activity/reactions/";
const maxRounds = readInt(flags, "--max-rounds", 240);
const waitMs = readInt(flags, "--wait-ms", 1200);
const { chromium } = await loadPlaywright();
const { context, page } = await openAuthenticatedSession({
  chromium,
  profile,
  url,
  logMessage:
    "ResearchLedger LinkedIn capture is running in the authenticated profile. Complete login if prompted.",
});

const posts = new Map();
let unchangedRounds = 0;
for (let round = 0; round < maxRounds && unchangedRounds < 8; round += 1) {
  const before = posts.size;
  const rows = await page.locator("a[href*='feed/update/urn:li:activity:']").evaluateAll((links) => links.map((link) => {
    const url = link.href.split("?")[0].replace(/\/$/, "");
    const article = link.closest("article") ?? link.parentElement;
    const text = (article?.innerText ?? link.innerText ?? "").replace(/\s+/g, " ").trim();
    return { url, text };
  }));
  // A short reaction is still a real post. Filtering on rendered text length
  // made a loaded feed look like an empty/authenticated failure.
  for (const post of rows) if (post.url) posts.set(post.url, post);
  unchangedRounds = posts.size === before ? unchangedRounds + 1 : 0;
  await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
  await page.waitForTimeout(waitMs);
}

try {
  assertNonEmptyCapture({ providerName: "LinkedIn", posts });
} catch (error) {
  await context.close();
  throw error;
}

const payload = {
  version: 1,
  capturedAt: new Date().toISOString(),
  source: "linkedin-playwright-authenticated-session",
  activityUrl: url,
  posts: [...posts.values()],
};
await fs.mkdir(path.dirname(path.resolve(output)), { recursive: true });
await fs.writeFile(output, `${JSON.stringify(payload, null, 2)}\n`, "utf8");
console.error(`Captured ${payload.posts.length} unique LinkedIn posts to ${output}`);
await context.close();
