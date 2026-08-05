#!/usr/bin/env node
/**
 * Run a local cross-encoder smoke test against the versioned fixture and print
 * deterministic evidence (endpoint, model, elapsed time, response hash,
 * and returned order). This verifies an installed local reranker is available
 * and compliant with the on-disk contract.
 */
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const fixturePath = path.join(
  repoRoot,
  "apps",
  "desktop",
  "src-tauri",
  "tests",
  "fixtures",
  "retrieval",
  "cross_encoder_contract.json",
);

const fixture = JSON.parse(fs.readFileSync(fixturePath, "utf8"));
const platformKey = process.platform === "darwin" ? "mlx" : "tei";

function hashString(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function normalizeEngine(input) {
  if (!input) {
    return platformKey;
  }
  const key = input.toLowerCase().trim();
  if (key === "mlx" || key === "tei" || key === "onnx") {
    return key;
  }
  throw new Error(`unsupported reranker engine ${input}; expected mlx, tei, or onnx`);
}

function endpointForEngine(engine) {
  if (process.env.RESEARCHLEDGER_RERANK_ENDPOINT) {
    return process.env.RESEARCHLEDGER_RERANK_ENDPOINT;
  }
  const fixtureEntry = fixture[engine];
  if (!fixtureEntry || !fixtureEntry.endpoint) {
    throw new Error(`fixture is missing endpoint for engine ${engine}`);
  }
  return fixtureEntry.endpoint;
}

function modelForEngine(engine) {
  if (process.env.RESEARCHLEDGER_RERANK_MODEL) {
    return process.env.RESEARCHLEDGER_RERANK_MODEL;
  }
  if (engine === "mlx") {
    return fixture.response?.model ?? "local-cross-encoder";
  }
  if (engine === "tei" || engine === "onnx") {
    return "BAAI/bge-reranker-large";
  }
  return "local-cross-encoder";
}

function requestBody(engine, query, documents, model) {
  if (engine === "mlx") {
    return {
      model,
      query,
      documents,
    };
  }
  return {
    query,
    texts: documents,
    raw_scores: false,
    truncate: true,
  };
}

function expectedOrder() {
  return fixture.expected_order.slice();
}

function parseResponse(engine, body, documents) {
  const parsed = JSON.parse(body);
  let items = [];
  if (engine === "mlx") {
    items = Array.isArray(parsed?.results) ? parsed.results : [];
  } else {
    items = Array.isArray(parsed) ? parsed : [];
  }
  const normalized = items.map((item) => ({
    index: Number(item.index),
    score: Number(item.relevance_score ?? item.score),
  }));
  const seen = new Set();
  for (const item of normalized) {
    if (!Number.isFinite(item.score)) {
      throw new Error(`non-finite score for index ${item.index}`);
    }
    if (!Number.isInteger(item.index) || item.index < 0 || item.index >= documents.length) {
      throw new Error(`response index out of range: ${item.index}`);
    }
    if (seen.has(item.index)) {
      throw new Error(`duplicate response index: ${item.index}`);
    }
    seen.add(item.index);
  }
  normalized.sort((a, b) => (b.score === a.score ? a.index - b.index : b.score - a.score));
  return normalized.map((item) => item.index);
}

async function main() {
  const start = performance.now();
  const engine = normalizeEngine(process.env.RESEARCHLEDGER_RERANK_ENGINE);
  const endpoint = endpointForEngine(engine);
  const model = modelForEngine(engine);

  const query = fixture.query;
  const documents = fixture.documents;
  const body = requestBody(engine, query, documents, model);
  const requestText = JSON.stringify(body);

  let response;
  try {
    response = await fetch(endpoint, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: requestText,
    });
  } catch (error) {
    throw new Error(`RERANK_ENDPOINT_UNREACHABLE ${endpoint}: ${error.message}`);
  }

  const responseText = await response.text();
  if (!response.ok) {
    throw new Error(
      `RERANK_ENDPOINT_ERROR ${endpoint}: HTTP ${response.status} ${response.statusText}\n${responseText}`.trim(),
    );
  }

  const responseOrder = parseResponse(engine, responseText, documents);
  const expected = expectedOrder();
  if (
    responseOrder.length !== expected.length ||
    responseOrder.some((value, index) => value !== expected[index])
  ) {
    throw new Error(
      `fixture order mismatch for engine ${engine}.\nexpected: ${JSON.stringify(expected)}\ngot: ${JSON.stringify(responseOrder)}`,
    );
  }

  const elapsedMs = Math.round(performance.now() - start);
  const report = {
    status: "PASS",
    engine,
    endpoint,
    model,
    query,
    document_count: documents.length,
    request_hash: hashString(requestText),
    response_hash: hashString(responseText),
    order: responseOrder,
    elapsed_ms: elapsedMs,
  };
  console.log(JSON.stringify(report, null, 2));
}

main().catch((error) => {
  console.error(error.message);
  process.exitCode = 1;
});
