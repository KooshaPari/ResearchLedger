# Known issues

- The prior scorecard overstated enrichment and retrieval as complete; it is now corrected.
- `cargo fmt --check` currently reports pre-existing formatting drift across provider files;
  the provenance patch itself compiles and its focused/full tests pass.
- Frontend tests emit React `act(...)` warnings; they do not fail the suite.
- Installed app now matches the latest bundle (resource parity passed; executable hash is
  recorded in the release handoff). Rebuild after future source changes remains required.
- Reference fetch, structured claims, fetched source documents, versioned chunk embeddings,
  and persisted Library claim/provenance inspection are implemented; the worker now has
  bounded retry/backoff and host-keyed concurrency accounting.
- Deterministic overlap reranking is implemented. The MLX-first/TEI/ONNX local cross-encoder
  contract and versioned quality fixture are implemented, but an installed-model quality smoke
  remains an A+ release gate. Current smoke status: `npm run smoke:rerank` returns
  `RERANK_ENDPOINT_ERROR http://127.0.0.1:9000/v1/rerank: HTTP 404 Not Found`.
- Path traversal and symlink export checks are now guarded in storage; the remaining
  security caveat is the lack of an authenticated live-provider smoke record.
