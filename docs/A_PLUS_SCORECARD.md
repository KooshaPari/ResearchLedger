# ResearchLedger A+ scorecard

Verified on 2026-07-23.

| Requirement | Evidence | Status |
|---|---|---|
| Local-first Tauri desktop | `apps/desktop/src-tauri`, packaged macOS app and DMG | PASS |
| SQLite + Markdown canonical vault | `storage.rs`, migrations, atomic Markdown writes | PASS |
| GitHub starred repositories and READMEs | GitHub client, device OAuth, importer tests | PASS |
| LinkedIn, Reddit saved posts, and X bookmarks capture | Persistent Playwright connectors per provider, bounded scrolling, JSON imports, packaged resource, shared renderer with hash-stable re-imports | PASS |
| Search/RAG | FTS5, citation context, cosine vector search, RRF fusion | PASS |
| Frontend workflows | Accessible workspace tabs, unified per-provider capture panels, vault/import/search/distill actions | PASS |
| Interoperable knowledge format | OKF v0.1 frontmatter, citations, generated `index.md` | PASS |
| Enrichment | Link extraction, enrichment jobs, deterministic distillation, pending-job processor | PASS |
| Privacy/security | In-memory tokens, dedicated local profiles per provider, path traversal guard, no raw HTML rendering | PASS |
| Verification | Rust 21/21, frontend 2/2, TypeScript/build, macOS app + DMG | PASS |

Known operational prerequisite: the packaged browser connectors require Node.js/Playwright
availability on the host, plus one user-authenticated persistent browser profile per
provider. Each connector is read-only for saved-posts/activity pages. If Ollama is offline,
ResearchLedger falls back to FTS5 and deterministic distillation without losing data.
