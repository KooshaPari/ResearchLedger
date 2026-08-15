#!/usr/bin/env python3
"""Loopback-only CrossEncoder adapter for the ResearchLedger rerank contract.

The model directory is supplied explicitly so the app never downloads a model
implicitly and never sends document text off-device.
"""

import json
import os
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

os.environ.setdefault("KMP_DUPLICATE_LIB_OK", "TRUE")

MODEL = os.environ.get("RESEARCHLEDGER_RERANK_MODEL_PATH")
if not MODEL:
    print(
        "RESEARCHLEDGER_RERANK_MODEL_PATH must be set to a local model directory",
        file=sys.stderr,
    )
    raise SystemExit(2)

from sentence_transformers import CrossEncoder

PORT = int(os.environ.get("RESEARCHLEDGER_RERANK_PORT", "8082"))
encoder = CrossEncoder(MODEL, max_length=512)


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        if self.path not in ("/rerank", "/v1/rerank"):
            self.send_error(404)
            return
        length = int(self.headers.get("content-length", "0"))
        payload = json.loads(self.rfile.read(length))
        query = payload["query"]
        documents = payload.get("documents", payload.get("texts", []))
        scores = encoder.predict([(query, document) for document in documents])
        results = [
            {"index": index, "relevance_score": float(score)}
            for index, score in enumerate(scores)
        ]
        results.sort(key=lambda item: (-item["relevance_score"], item["index"]))
        body = json.dumps({"model": os.path.basename(MODEL), "results": results}).encode()
        self.send_response(200)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_args):
        return


ThreadingHTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
