# ResearchLedger A+ scorecard

Status: verified on 2026-07-20

| Requirement | Evidence | Status |
|---|---|---|
| Local-first Tauri desktop | `apps/desktop/src-tauri`, packaged macOS app and DMG | PASS |
| SQLite + Markdown canonical vault | `storage.rs`, migrations, atomic Markdown writes | PASS |
| GitHub starred repositories and READMEs | GitHub client, device OAuth, importer tests | PASS |
| LinkedIn personal activity capture | Persistent Playwright connector, bounded scrolling, JSON import, packaged resource | PASS |
| Search/RAG | FTS5, citation context, cosine vector search, RRF fusion | PASS |
| Frontend workflows | Accessible workspace tabs, vault/import/search/distill actions | PASS |
| Interoperable knowledge format | OKF v0.1 frontmatter, citations, generated `index.md` | PASS |
| Enrichment | Link extraction, enrichment jobs, deterministic distillation, pending-job processor | PASS |
| Privacy/security | In-memory tokens, local profile, path traversal guard, no raw HTML rendering | PASS |
| Verification | Rust 11/11, frontend 2/2, TypeScript/build, macOS app + DMG | PASS |

Known operational prerequisite: the packaged browser connector requires Node.js/Playwright
availability on the host and a user-authenticated LinkedIn profile. If Ollama is offline,
ResearchLedger falls back to FTS5 and deterministic distillation without losing data.
