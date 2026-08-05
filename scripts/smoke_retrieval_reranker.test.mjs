#!/usr/bin/env node
/**
 * Deterministic unit tests for smoke_retrieval_reranker helpers and transport
 * control flow (timeouts, retries, parse validation).
 */

import { describe, expect, it } from "vitest";

import {
  fetchWithTimeout,
  isRetryableEndpointError,
  normalizeEngine,
  parsePositiveInt,
  parseResponse,
  postRerankRequest,
  requestBody,
} from "./smoke_retrieval_reranker.mjs";

function makeResponse({ ok, status = 200, statusText = "OK", body = "{}" }) {
  return {
    ok,
    status,
    statusText,
    text: async () => body,
  };
}

describe("smoke_retrieval_reranker helper parsing and transport helpers", () => {
  it("normalizes supported engines and falls back to current platform key", () => {
    expect(normalizeEngine("TeI")).toBe("tei");
    expect(normalizeEngine(undefined)).toBe(process.platform === "darwin" ? "mlx" : "tei");
  });

  it("parses positive integers and returns fallback when invalid", () => {
    expect(parsePositiveInt("45", 10)).toBe(45);
    expect(parsePositiveInt("0", 10)).toBe(10);
    expect(parsePositiveInt("-7", 10)).toBe(10);
    expect(parsePositiveInt("n/a", 7)).toBe(7);
  });

  it("builds engine-specific request bodies", () => {
    expect(requestBody("mlx", "q", ["a", "b"], "local-cross-encoder")).toEqual({
      model: "local-cross-encoder",
      query: "q",
      documents: ["a", "b"],
    });
    expect(requestBody("tei", "q", ["a", "b"], "BAAI/bge-reranker-large")).toEqual({
      query: "q",
      texts: ["a", "b"],
      raw_scores: false,
      truncate: true,
    });
    expect(requestBody("onnx", "q", ["a"], "m")).toEqual({
      query: "q",
      texts: ["a"],
      raw_scores: false,
      truncate: true,
    });
  });

  it("treats retryable and non-retryable endpoint errors explicitly", () => {
    expect(isRetryableEndpointError("RERANK_ENDPOINT_TIMEOUT http://localhost:1: timed out")).toBe(true);
    expect(isRetryableEndpointError("RERANK_ENDPOINT_UNREACHABLE x: nope")).toBe(true);
    expect(isRetryableEndpointError("RERANK_ENDPOINT_RETRYABLE_HTTP x: HTTP 429")).toBe(false);
    expect(isRetryableEndpointError("RERANK_ENDPOINT_ERROR x: HTTP 404 Not Found")).toBe(false);
  });

  it("orders and validates parsed rerank payloads", () => {
    const response = JSON.stringify({
      results: [
        { index: 1, relevance_score: 0.2 },
        { index: 2, score: 0.7 },
        { index: 0, score: 0.7 },
      ],
    });
    expect(parseResponse("mlx", response, ["a", "b", "c"])).toEqual([2, 0, 1]);
  });

  it("raises a deterministic parse error for malformed response bodies", () => {
    expect(() => parseResponse("mlx", "{ not-json", ["a", "b"])).toThrow(
      /RERANK_RESPONSE_JSON_PARSE_ERROR/,
    );
  });

  it("retries on 5xx and then succeeds with the final successful payload", async () => {
    let attempts = 0;

    const fetchImpl = async () => {
      attempts += 1;
      if (attempts === 1) {
        return makeResponse({
          ok: false,
          status: 500,
          statusText: "Internal Server Error",
          body: "temporary failure",
        });
      }

      return makeResponse({
        ok: true,
        body: JSON.stringify([
          { index: 2, score: 0.42 },
          { index: 0, score: 0.31 },
          { index: 1, score: 0.12 },
        ]),
      });
    };

    const result = await postRerankRequest({
      endpoint: "http://127.0.0.1:0/rerank",
      bodyText: "{}",
      timeoutMs: 1000,
      maxRetries: 2,
      retryDelayMs: 1,
      fetchImpl,
    });

    expect(attempts).toBe(2);
    expect(result.attemptNumber).toBe(2);
    expect(result.responseText).toContain("index");
  });

  it("retries transient network errors with explicit timeout-cap behavior", async () => {
    const hangingFetch = async () => new Promise(() => {});
    await expect(
      fetchWithTimeout(
        "http://127.0.0.1:0/rerank",
        "{}",
        5,
        hangingFetch,
      ),
    ).rejects.toThrow(/RERANK_ENDPOINT_TIMEOUT/);
  });

  it("fails non-retryable HTTP status without retry", async () => {
    let attempts = 0;
    const fetchImpl = async () => {
      attempts += 1;
      return makeResponse({
        ok: false,
        status: 404,
        statusText: "Not Found",
        body: "not found",
      });
    };

    await expect(
      postRerankRequest({
        endpoint: "http://127.0.0.1:0/rerank",
        bodyText: "{}",
        timeoutMs: 1000,
        maxRetries: 2,
        retryDelayMs: 1,
        fetchImpl,
      }),
    ).rejects.toThrow("RERANK_ENDPOINT_ERROR");
    expect(attempts).toBe(1);
  });
});
