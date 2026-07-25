# ResearchLedger A+ scorecard
Verified on 2026-07-24.

| Requirement | Evidence | Status |
|---|---|---|
| Local-first Tauri desktop | `apps/desktop/src-tauri`, packaged macOS app and DMG | PASS |
| SQLite + Markdown canonical vault | `storage.rs`, migrations, atomic Markdown writes | PASS |
| GitHub starred repositories and READMEs | GitHub client, device OAuth, importer tests | PASS |
| LinkedIn, Reddit saved posts, and X bookmarks capture | Persistent Playwright connectors per provider with auto-install on first capture, bounded scrolling, JSON imports, packaged resource, shared renderer with hash-stable re-imports, tightened path-shape filters (no /user/X/comments/ or /i/status/ leakage) | PASS |
| Search/RAG | FTS5, citation context, cosine vector search, RRF fusion | PASS |
| Frontend workflows | Accessible workspace tabs, unified per-provider capture panels, vault/import/search/distill actions, friendly error formatting, persistent-profile management | PASS |
| Interoperable knowledge format | OKF v0.1 frontmatter, citations, generated `index.md` | PASS |
| Enrichment | Link extraction, enrichment jobs, deterministic distillation, pending-job processor | PASS |
| Privacy/security | In-memory tokens, dedicated local profiles per provider, path traversal guard, safe-path validation on imports and capture, login-redirect detection, no raw HTML rendering | PASS |
| Verification | Rust 38/38, frontend 2/2, TypeScript/build, macOS app + DMG | PASS |

Known operational prerequisite: the packaged browser connectors require Node.js on the
host, plus Playwright Chromium (auto-installed on first capture if missing). Each
connector is read-only for saved-posts/activity pages. If Ollama is offline, ResearchLedger
falls back to FTS5 and deterministic distillation without losing data.]]
