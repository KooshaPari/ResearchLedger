# Research findings

- `npm run verify:resources` reports parity for 8 capture-resource declarations.
- `npm test` passes 65 tests (58 capture-common, 7 frontend); React `act` warnings remain.
- Rust `cargo test` passes 69 tests; only existing dead-code warnings remain.
- Provenance is stored in SQLite and now refreshes on create, update, and unchanged imports.
- The release branch now persists a `reference_fetches` queue and exposes a bounded worker:
  robots-aware public HTTP, private-host rejection, 1 MB body cap, 15 s timeout, atomic raw
  artifacts, and resumable status/hash metadata.
- Deterministic distillation now emits claims, definitions, alternatives, open questions,
  and `[1]` source citations; fetched artifacts are promoted to `reference:*` Markdown
  source documents with provenance.
- Documents are split into bounded heading-aware chunks; embeddings persist model,
  embedding-version, and input-hash metadata, and vector-only RRF hits are retained.
- Retrieval combines FTS5 and optional Ollama vectors with a deterministic overlap reranker;
  a local cross-encoder remains an explicit future seam. Library exposes persisted claims,
  source URIs, and citation IDs for document inspection.
