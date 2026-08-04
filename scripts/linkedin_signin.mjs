#!/usr/bin/env node
import process from "node:process";
import { loadPlaywright, openAuthenticatedSession, parseFlags, resolveProfile } from "./_capture_common.mjs";

const flags = parseFlags(process.argv);
const profile = resolveProfile(flags, "linkedin-profile");
const url = flags.get("--url") ?? "https://www.linkedin.com/in/me/recent-activity/reactions/";
const { chromium } = await loadPlaywright();
const { context } = await openAuthenticatedSession({
  chromium,
  profile,
  url,
  logMessage: "ResearchLedger LinkedIn sign-in window is ready; close it when finished.",
});
await new Promise((resolve) => context.on("close", resolve));
