#!/usr/bin/env node
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
const profile = args.get("--profile") ?? `${process.env.HOME}/Library/Application Support/ResearchLedger/linkedin-profile`;
const output = args.get("--output") ?? "linkedin-capture.json";
const url = args.get("--url") ?? "https://www.linkedin.com/in/me/recent-activity/reactions/";
const maxRounds = Number(args.get("--max-rounds") ?? 240);
const waitMs = Number(args.get("--wait-ms") ?? 1200);

const context = await chromium.launchPersistentContext(profile, { headless: false });
const page = context.pages()[0] ?? await context.newPage();
await page.goto(url, { waitUntil: "domcontentloaded" });
console.error("ResearchLedger LinkedIn capture is running in the authenticated profile. Complete login if prompted.");
await page.waitForTimeout(2500);

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
  for (const post of rows) if (post.url && post.text.length >= 40) posts.set(post.url, post);
  unchangedRounds = posts.size === before ? unchangedRounds + 1 : 0;
  await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
  await page.waitForTimeout(waitMs);
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
