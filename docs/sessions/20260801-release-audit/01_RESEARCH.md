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
- Retrieval combines FTS5 and optional Ollama vectors. Ollama does not provide a native rerank
  API, so the cross-encoder implementation uses the existing `reqwest` dependency against an
  opt-in, loopback-only OpenAI-compatible `/v1/rerank` endpoint (for example, local
  `llama-server --rerank` with a separately installed BGE GGUF). It refuses remote endpoints,
  has a three-second timeout and strict response validation, and falls back deterministically.
  The persisted cross-encoder contract fixture verifies semantic order without a model download.
