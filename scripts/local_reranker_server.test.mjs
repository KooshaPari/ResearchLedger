import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "vitest";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

test("fails clearly before importing optional model dependencies without a model path", () => {
  const environment = { ...process.env };
  delete environment.RESEARCHLEDGER_RERANK_MODEL_PATH;
  const result = spawnSync("python3", ["scripts/local_reranker_server.py"], {
    cwd: root,
    env: environment,
    encoding: "utf8",
  });

  expect(result.status).toBe(2);
  expect(result.stderr).toContain("RESEARCHLEDGER_RERANK_MODEL_PATH must be set");
  expect(result.stderr).not.toContain("ModuleNotFoundError");
});
