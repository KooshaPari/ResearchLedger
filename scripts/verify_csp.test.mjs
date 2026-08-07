/** @vitest-environment node */

import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, it } from "vitest";

const scriptsDirectory = path.dirname(fileURLToPath(import.meta.url));

it("keeps production CSP limited to Tauri assets and IPC", () => {
  const output = execFileSync(process.execPath, ["verify_csp.mjs"], {
    cwd: scriptsDirectory,
    encoding: "utf8",
  });

  expect(output).toContain("CSP verification passed");
});
