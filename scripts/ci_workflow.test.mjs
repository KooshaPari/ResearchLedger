import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const readWorkflow = (name) => readFile(path.join(root, ".github", "workflows", name), "utf8");

describe("hosted CI contracts", () => {
  test("uses the Bun lock and discovers the nested Tauri crate", async () => {
    const workflow = await readWorkflow("ci.yml");

    expect(workflow).toContain("oven-sh/setup-bun@0c5077e51419868618aeaa5fe8019c62421857d6");
    expect(workflow).toContain("bun install --frozen-lockfile");
    expect(workflow).not.toContain("npm ci");
    expect(workflow).toContain("-name Cargo.toml -type f");
    expect(workflow).toContain("--manifest-path apps/desktop/src-tauri/Cargo.toml");
    expect(workflow).toContain("needs.dep-review.result");
    expect(workflow).not.toContain("needs.dependency-review.result");
  });

  test("gives history-based scanners the complete PR history", async () => {
    const workflow = await readWorkflow("ci.yml");
    const security = workflow.slice(workflow.indexOf("  security:"), workflow.indexOf("  dep-review:"));

    expect(security).toContain("fetch-depth: 0");
  });

  test("runs the desktop Rust gate on its supported macOS target", async () => {
    const workflow = await readWorkflow("ci.yml");
    const rust = workflow.slice(workflow.indexOf("  rust:"), workflow.indexOf("  cargo-deny:"));

    expect(rust).toContain("runs-on: macos-latest");
    expect(rust).toContain("RESEARCHLEDGER_SKIP_BUNDLE_RESOURCE_VALIDATION: \"1\"");
    expect(rust).not.toContain("Swatinem/rust-cache");
  });

  test("pins a valid Trunk action commit", async () => {
    const workflow = await readWorkflow("trunk-check.yml");

    expect(workflow).toContain("trunk-io/trunk-action@04ba50e7658c81db7356da96657e6e77f220bfa3");
    expect(workflow).not.toContain("d90b9166660d5e5afae248a58172a3a0e99d56d5");
  });

  test("keeps release verification on the Bun toolchain", async () => {
    const workflow = await readWorkflow("release.yml");

    expect(workflow).toContain("oven-sh/setup-bun@0c5077e51419868618aeaa5fe8019c62421857d6");
    expect(workflow).toContain("bun install --frozen-lockfile");
    expect(workflow).toContain("bun run build");
    expect(workflow).toContain("bun run test");
    expect(workflow).not.toContain("actions/setup-node");
    expect(workflow).not.toContain("npm ");
  });

  test("uses the current Trunk configuration schema", async () => {
    const config = await readFile(path.join(root, ".trunk", "trunk.yaml"), "utf8");

    expect(config).toContain("version: 0.1");
    expect(config).toContain("lint:");
    expect(config).not.toContain("linters:");
    expect(config).not.toContain("formatters:");
    expect(config).toContain("- git-diff-check");
    expect(config).not.toContain("- actionlint");
    expect(config).not.toContain("- prettier");
    expect(config).not.toContain("- taplo");
  });

  test("runs the pinned workflow linter independently of Trunk plugins", async () => {
    const workflow = await readWorkflow("trunk-check.yml");
    const ci = await readWorkflow("ci.yml");

    expect(workflow).toContain("actions/setup-go@b7ad1dad31e06c5925ef5d2fc7ad053ef454303e");
    expect(ci).toContain("actions/setup-go@b7ad1dad31e06c5925ef5d2fc7ad053ef454303e");
    expect(workflow).toContain("github.com/rhysd/actionlint/cmd/actionlint@v1.7.10");
    expect(workflow).toContain("find .github/workflows -type f");
    expect(workflow).toContain("xargs -0 -r \"$(go env GOPATH)/bin/actionlint\"");
  });

  test("keeps all repository workflows valid for actionlint", async () => {
    const scorecard = await readWorkflow("scorecard.yml");
    const config = await readFile(path.join(root, ".github", "actionlint.yaml"), "utf8");

    expect(scorecard).not.toContain("    security:\n      permissions: read-all");
    expect(config).toContain("blacksmith-2vcpu-ubuntu-2204");
  });
});
