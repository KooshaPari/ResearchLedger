# ResearchLedger release scorecard

Status as of 2026-08-08: **release candidate, not A+ yet**. The local-first shell,
packaging path, bounded reference worker, structured deterministic distillation, persisted
claims/provenance UX, chunked/versioned retrieval substrate, and strict local cross-encoder
quality are green. Provider-policy retirement and an installed-app GitHub import remain.

| Requirement | Evidence | Status |
|---|---|---|
| Local-first Tauri desktop | `apps/desktop/src-tauri`; app-only bundle built and installed during this release pass | PASS |
| SQLite + Markdown canonical vault | `storage.rs`, migration, atomic Markdown writes | PASS |
| GitHub starred repositories and READMEs | Rust-owned local `gh` credential import, importer tests, and prior keychain-authenticated API corpus evidence | PASS implementation/API / installed invocation pending |
| LinkedIn import | Official export/manual permalink path only; no browser sign-in, persistent profile, DOM capture, or reaction-feed crawler is shipped | PASS policy boundary / API capability adapter pending |
| GitHub auth and Markdown export UX | Rust-owned local CLI import and native destination picker; frontend boundary tests | PARTIAL: first-party Device Flow fallback remains planned |
| Provenance | `provenance` rows and claim rows write on create, update, and unchanged re-import; fetched references become source documents; Library exposes claim citations | PASS |
| Search/RAG | SQLite FTS5 lexical search, deterministic heading/size chunking, optional Ollama vectors with model/version/input hashes, rank fusion including vector-only hits, loopback-only local `/v1/rerank` cross-encoder adapter, versioned ranking fixture, deterministic overlap fallback, persisted cited-context UX; strict smoke passed with `cross-encoder/ms-marco-MiniLM-L-6-v2` on loopback (`d4e3de77826d02b515c08a4cc55a4bd5668d093de5d669f2fb96b61973d9bad4` -> `aa9bd1d064d1303cf6acedbddca3cfa8b0b8be51c7a15b6dc00fbc491f440e16`, order `[2,1,0]`) | PASS (explicit local model; deterministic fallback remains the offline default) |
| Frontend workflows | Workspace tabs, provider actions, vault/import/search/distill/export flows | PASS |
| Interoperable knowledge format | OKF-style frontmatter, citations, generated index | PASS: schema coverage still needs fixture validation |
| Enrichment | URL queue with resumable status, bounded exponential retry for transient HTTP failures, host-keyed concurrency budget, public-HTTP worker with robots/private-host/redirect/byte/time guards, atomic artifacts, fetched source documents, structured deterministic claims, persisted claim rows | PASS |
| Privacy/security | Rust-owned GitHub credentials, provider profiles only for permitted sources, URL/path guards, login redirect detection, load-path traversal rejection, symlink-safe Markdown export | PARTIAL: consent registry and URL redaction lifecycle remain |
| Verification | Existing local checks and historical corpus evidence | PARTIAL: re-run all checks, package parity, installed binary parity, and authenticated GitHub invocation after provider-policy retirement; audit reports 18 allowed upstream maintenance/unsoundness advisories |

## Remaining A+ gate

Run the installed UI GitHub import, recording repository and README counts plus hashes without
retaining credentials. The renderer must never receive a GitHub credential. Complete the consent
registry, URL scope/redaction lifecycle, claim-span evidence, and first-party Device Flow fallback
before an A+ release claim.

Operational prerequisite: permitted packaged browser connectors require Bun and Playwright
Chromium on the host. Ollama is optional; FTS5 and deterministic distillation remain
available when it is offline.
