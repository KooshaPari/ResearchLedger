import { spawn, spawnSync } from "node:child_process";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "vitest";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

async function unusedPort() {
  const server = createServer();
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const { port } = server.address();
  await new Promise((resolve) => server.close(resolve));
  return port;
}

async function startTestServer() {
  const fixtureRoot = await mkdtemp(path.join(tmpdir(), "researchledger-reranker-"));
  await writeFile(path.join(fixtureRoot, "sentence_transformers.py"), [
    "class CrossEncoder:",
    "    def __init__(self, *_args, **_kwargs): pass",
    "    def predict(self, pairs): return [float(index) for index, _ in enumerate(pairs)]",
    "",
  ].join("\n"));
  const port = await unusedPort();
  const child = spawn("python3", ["scripts/local_reranker_server.py"], {
    cwd: root,
    env: {
      ...process.env,
      PYTHONPATH: fixtureRoot,
      RESEARCHLEDGER_RERANK_MODEL_PATH: "test-model",
      RESEARCHLEDGER_RERANK_PORT: String(port),
    },
    stdio: "ignore",
  });
  const endpoint = `http://127.0.0.1:${port}/v1/rerank`;
  for (let attempt = 0; attempt < 40; attempt += 1) {
    try {
      await fetch(endpoint, { method: "POST", body: "{}" });
      break;
    } catch {
      await new Promise((resolve) => setTimeout(resolve, 25));
    }
  }
  return {
    endpoint,
    async stop() {
      child.kill();
      await new Promise((resolve) => child.once("exit", resolve));
      await rm(fixtureRoot, { recursive: true, force: true });
    },
  };
}

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

test("rejects malformed and structurally invalid requests with bounded JSON 4xx errors", async () => {
  const server = await startTestServer();
  try {
    for (const body of ["{", "[]", "{}", '{"query":"   ","documents":["doc"]}', '{"query":"q","documents":[]}']) {
      const response = await fetch(server.endpoint, { method: "POST", body });
      expect(response.status).toBe(400);
      expect(response.headers.get("content-type")).toContain("application/json");
      expect(await response.json()).toEqual({ error: expect.any(String) });
    }
  } finally {
    await server.stop();
  }
});

test("keeps the valid rerank response contract", async () => {
  const server = await startTestServer();
  try {
    const response = await fetch(server.endpoint, {
      method: "POST",
      body: JSON.stringify({ query: "q", documents: ["first", "second"] }),
    });
    expect(response.status).toBe(200);
    expect(await response.json()).toEqual({
      model: "test-model",
      results: [
        { index: 1, relevance_score: 1 },
        { index: 0, relevance_score: 0 },
      ],
    });
  } finally {
    await server.stop();
  }
});
