/** @vitest-environment node */

import { execFileSync } from "node:child_process";
import { expect, it } from "vitest";

const script = new URL("./release_macos.mjs", import.meta.url);

it("prints the complete macOS release plan without touching credentials or artifacts", () => {
  const output = execFileSync(process.execPath, [script.pathname, "--dry-run"], {
    cwd: new URL("..", import.meta.url),
    encoding: "utf8",
  });

  expect(output).toContain("Dry run only: no credentials, signing, notarization, or stapling occurs.");
  expect(output).toContain("security find-identity -v -p codesigning");
  expect(output).toContain("bunx tauri build --bundles app,dmg");
  expect(output).toContain("xcrun notarytool submit");
  expect(output).toContain("--keychain-profile ResearchLedger-Notary");
  expect(output).toContain("xcrun stapler staple");
  expect(output).toContain("spctl --assess --type execute --verbose=4");
});

it("rejects unknown release arguments before any release step", () => {
  expect(() =>
    execFileSync(process.execPath, [script.pathname, "--unexpected"], {
      cwd: new URL("..", import.meta.url),
      encoding: "utf8",
      stdio: "pipe",
    }),
  ).toThrow(/Unknown argument: --unexpected/);
});
