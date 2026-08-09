import { describe, it, expect } from "vitest";

describe("smoke", () => {
  it("loads package", async () => {
    const pkg = await import("../package.json");
    expect(pkg.name).toBeDefined();
    expect(pkg.version).toBeDefined();
  });
});

