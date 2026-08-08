# ResearchLedger release scorecard

Status as of 2026-08-08: **release candidate, not A+ yet**. The local-first shell,
packaging path, bounded reference worker, structured deterministic distillation, persisted
claims/provenance UX, and chunked/versioned retrieval substrate are healthy; local
cross-encoder quality and live-account smoke remain.

| Requirement | Evidence | Status |
|---|---|---|
| Local-first Tauri desktop | `apps/desktop/src-tauri`; app-only bundle built and installed during this release pass | PASS |
| SQLite + Markdown canonical vault | `storage.rs`, migration, atomic Markdown writes | PASS |
| GitHub starred repositories and READMEs | GitHub client, device OAuth, importer tests | PASS |
| LinkedIn, Reddit, X, and Hacker News capture | Persistent Playwright connectors, bounded scroll, JSON import, packaged resource parity; installed-profile smoke captured 7 LinkedIn posts (SHA recorded externally) | PASS for LinkedIn / other providers unverified |
| GitHub auth and Markdown export UX | Device-flow polling, client-id validation, native destination picker; frontend suite 7/7 | PASS |
| Provenance | `provenance` rows and claim rows write on create, update, and unchanged re-import; fetched references become source documents; Library exposes claim citations | PASS |
| Search/RAG | SQLite FTS5 lexical search, deterministic heading/size chunking, optional Ollama vectors with model/version/input hashes, rank fusion including vector-only hits, loopback-only local `/v1/rerank` cross-encoder adapter, versioned ranking fixture, deterministic overlap fallback, persisted cited-context UX | PARTIAL: `bun run smoke:rerank` emits deterministic local fallback evidence when no explicit reranker endpoint is configured; explicit endpoint mode still requires a healthy local `/rerank`/`/v1/rerank` route |
| Frontend workflows | Workspace tabs, provider actions, vault/import/search/distill/export flows | PASS |
| Interoperable knowledge format | OKF-style frontmatter, citations, generated index | PASS: schema coverage still needs fixture validation |
| Enrichment | URL queue with resumable status, bounded exponential retry for transient HTTP failures, host-keyed concurrency budget, public-HTTP worker with robots/private-host/redirect/byte/time guards, atomic artifacts, fetched source documents, structured deterministic claims, persisted claim rows | PASS |
| Privacy/security | In-memory auth, provider profiles, URL/path guards, login redirect detection, load-path traversal rejection, symlink-safe Markdown export | PASS |
| Verification | `bun run verify:resources`, `bun run test` (95/95), `bun run build`, Rust (82 + 1 OKF), packaged parity, `bun run smoke:rerank`, installed LinkedIn smoke | PARTIAL: GitHub live import and live cross-encoder remain |

## Remaining A+ gates

1. Run the configured local cross-encoder against the persisted retrieval-quality fixture and
   record model identifier plus fixture result (without bundling or downloading a model).  
   Current status: `bun run smoke:rerank` now succeeds with `PASS_LOCAL_FALLBACK` when no explicit
   endpoint is set and no local candidate is reachable; enable explicit endpoint mode for strict contract
   validation of your ML reranker route.
2. Re-run an authenticated LinkedIn capture and a GitHub starred-repository import on the
   installed app, recording counts and hashes without retaining credentials.

Operational prerequisite: packaged browser connectors require Node.js and Playwright
Chromium on the host. Ollama is optional; FTS5 and deterministic distillation remain
available when it is offline.
