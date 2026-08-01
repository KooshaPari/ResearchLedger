# Research findings

- `npm run verify:resources` reports parity for 8 capture-resource declarations.
- `npm test` passes 65 tests (58 capture-common, 7 frontend); React `act` warnings remain.
- Rust `cargo test` passes 57 tests; only existing dead-code warnings remain.
- Provenance is stored in SQLite and now refreshes on create, update, and unchanged imports.
- The current implementation extracts URLs and queues deterministic distillation; it does
  not yet fetch referenced pages or produce claim-level citations.
- Retrieval combines FTS5 and optional Ollama vectors, but embeddings are single-chunk and
  there is no reranker or embedding-version invalidation.
