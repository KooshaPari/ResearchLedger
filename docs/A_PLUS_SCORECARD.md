# ResearchLedger release scorecard

Status as of 2026-08-08: **release candidate, not A+ yet**. The local-first shell,
packaging path, bounded reference worker, structured deterministic distillation, persisted
claims/provenance UX, chunked/versioned retrieval substrate, and strict local cross-encoder
quality are green; only installed-app GitHub import remains.

| Requirement | Evidence | Status |
|---|---|---|
| Local-first Tauri desktop | `apps/desktop/src-tauri`; app-only bundle built and installed during this release pass | PASS |
| SQLite + Markdown canonical vault | `storage.rs`, migration, atomic Markdown writes | PASS |
| GitHub starred repositories and READMEs | GitHub client, device OAuth, importer tests, and installed-app `Use authenticated gh` action backed by the OS credential store; keychain-authenticated API smoke fetched 371/371 README payloads (canonical corpus SHA recorded externally) | PASS implementation/API / installed invocation pending |
| LinkedIn, Reddit, X, and Hacker News capture | Persistent Playwright connectors, bounded scroll, JSON import, packaged resource parity; installed-profile smoke captured 7 LinkedIn posts (SHA recorded externally) | PASS for LinkedIn / other providers unverified |
| GitHub auth and Markdown export UX | Device-flow polling, client-id validation, native destination picker; frontend suite 7/7 | PASS |
| Provenance | `provenance` rows and claim rows write on create, update, and unchanged re-import; fetched references become source documents; Library exposes claim citations | PASS |
| Search/RAG | SQLite FTS5 lexical search, deterministic heading/size chunking, optional Ollama vectors with model/version/input hashes, rank fusion including vector-only hits, loopback-only local `/v1/rerank` cross-encoder adapter, versioned ranking fixture, deterministic overlap fallback, persisted cited-context UX; strict smoke passed with `cross-encoder/ms-marco-MiniLM-L-6-v2` on loopback (`d4e3de77826d02b515c08a4cc55a4bd5668d093de5d669f2fb96b61973d9bad4` -> `aa9bd1d064d1303cf6acedbddca3cfa8b0b8be51c7a15b6dc00fbc491f440e16`, order `[2,1,0]`) | PASS (explicit local model; deterministic fallback remains the offline default) |
| Frontend workflows | Workspace tabs, provider actions, vault/import/search/distill/export flows | PASS |
| Interoperable knowledge format | OKF-style frontmatter, citations, generated index | PASS: schema coverage still needs fixture validation |
| Enrichment | URL queue with resumable status, bounded exponential retry for transient HTTP failures, host-keyed concurrency budget, public-HTTP worker with robots/private-host/redirect/byte/time guards, atomic artifacts, fetched source documents, structured deterministic claims, persisted claim rows | PASS |
| Privacy/security | In-memory auth, provider profiles, URL/path guards, login redirect detection, load-path traversal rejection, symlink-safe Markdown export | PASS |
| Verification | `bun run verify:resources`, `bun run test` (96/96), `bun run build`, Rust (83/83 + 1 OKF), packaged parity, installed binary parity, strict `bun run smoke:rerank`, installed LinkedIn smoke (7 posts), GitHub API corpus smoke (371/371 READMEs; SHA `c230e2a92f0c4e458a1e60d3becd9774511445d6f50557183e26a4f289352c96`), `cargo audit` | PARTIAL: installed UI GitHub import invocation remains; audit reports 18 allowed upstream maintenance/unsoundness advisories |

## Remaining A+ gate

Run the installed UI GitHub import, recording repository and README counts plus hashes without
retaining credentials. When its token field is empty, the importer now obtains the authenticated
`gh` credential in memory automatically; the visible keychain control remains optional. The
API-compatible corpus path and keychain-token command are verified separately.

Operational prerequisite: packaged browser connectors require Node.js and Playwright
Chromium on the host. Ollama is optional; FTS5 and deterministic distillation remain
available when it is offline.
