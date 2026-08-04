/** @vitest-environment node */

import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, it } from "vitest";

const scriptsDirectory = path.dirname(fileURLToPath(import.meta.url));

it("verifies every packaged browser resource, including LinkedIn sign-in", () => {
  const output = execFileSync(process.execPath, ["verify_resources.mjs"], {
    cwd: scriptsDirectory,
    encoding: "utf8",
  });

  expect(output).toContain("Resource parity passed: 9 source declarations.");
});
