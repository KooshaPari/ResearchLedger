/** @vitest-environment node */

import { expect, test } from "vitest";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const buildScript = fs.readFileSync(
  path.join(repoRoot, "apps/desktop/src-tauri/build.rs"),
  "utf8",
);

test("only the explicit Rust-test switch removes packaged browser resources", () => {
  expect(buildScript).toContain('std::env::var_os("RESEARCHLEDGER_SKIP_BUNDLE_RESOURCE_VALIDATION")');
  expect(buildScript).toContain('std::env::set_var("TAURI_CONFIG", r#"{\"bundle\":{\"resources\":null}}"#)');
  expect(buildScript).toContain("tauri_build::build()");
});
