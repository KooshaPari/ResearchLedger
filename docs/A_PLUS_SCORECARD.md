# ResearchLedger release scorecard

Status as of 2026-08-10: **release candidate, not A+ yet**. The local-first shell,
packaging path, consent registry, and claim-span evidence path are verified; installed-app
and provider-boundary gates still prevent an A+ claim.

| Requirement | Evidence | Status |
|---|---|---|
| Local-first Tauri desktop | `apps/desktop/src-tauri`; app-only bundle built and installed during this release pass | PASS |
| SQLite + Markdown canonical vault | `storage.rs`, migration, atomic Markdown writes | PASS |
| GitHub starred repositories and READMEs | GitHub client, device OAuth, importer tests | PASS |
| LinkedIn, Reddit, X, and Hacker News capture | Persistent Playwright connectors, bounded scroll, JSON import, packaged resource parity | READY / live-account unverified |
| GitHub auth and Markdown export UX | Device-flow polling, client-id validation, native destination picker; frontend suite 7/7 | PASS |
| Provenance | `provenance` rows now write on create, update, and unchanged re-import; Rust suite 57/57 | PASS |
| Search/RAG | SQLite FTS5 lexical search, optional Ollama vectors, rank fusion, citation context | PARTIAL: no reranker or versioned embedding invalidation |
| Frontend workflows | Workspace tabs, provider actions, vault/import/search/distill/export flows | PASS |
| Interoperable knowledge format | OKF-style frontmatter, citations, generated index | PASS: schema coverage still needs fixture validation |
| Enrichment | Bounded reference worker, queued deterministic distillation, persisted provenance, structured claims with source citations and reproducible byte spans | PASS: local tests cover retry/guard behavior and claim-span persistence |
| Privacy/security | In-memory auth, consent registry with scoped/expiring/revocable grants, hashed consent audit targets, provider profiles, URL/path guards, login redirect detection | PASS implementation/tests; installed-app/provider-policy verification remains |
| Verification | `npm run verify:resources`, `npm test` (65/65), `npm run build`, Rust (57/57) | PASS |

## Remaining A+ gates

1. Chunk documents for embeddings, record model/version and input hash, and fix vector-only
   retrieval fusion coverage; add a local reranker contract.
2. Add document-detail/provenance inspection and persisted collection/graph views.
3. Re-run an authenticated LinkedIn capture and a GitHub starred-repository import on the
   installed app, recording counts and hashes without retaining credentials.

Operational prerequisite: packaged browser connectors require Node.js and Playwright
Chromium on the host. Ollama is optional; FTS5 and deterministic distillation remain
available when it is offline.
