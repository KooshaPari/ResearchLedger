# WBS — ResearchLedger (2026-08-09)

**Repo:** ResearchLedger
**Status:** initial skeleton; expand as scope grows.
**Owner:** forge (agent CLI). **Driver:** `proc` / `proc <id>`.

## Phase overview

ResearchLedger is a **local-first research ledger and LLM knowledge
base**. The desktop app keeps Markdown and SQLite data local,
preserves source provenance, and exposes import/search/RAG/export
adapters through a Tauri command boundary.

| Phase | Tasks | Theme | Outcome |
|-------|-------|-------|---------|
| 0 | 1–5 | audit close-out + inventory | reproducible baseline |
| 1 | 6–15 | Tauri command boundary hardening | IPC verified |
| 2 | 16–25 | SQLite schema migration audit | forward+backward compat |
| 3 | 26–35 | Markdown import adapters (web/local/arxiv) | 3 adapters verified |
| 4 | 36–50 | RAG pipeline (chunk → embed → retrieve) | recall@k validated |
| 5 | 51–60 | export adapters (PDF / Markdown / JSON) | 3 formats verified |
| 6 | 61–75 | source provenance audit | every doc has lineage |
| 7 | 76–85 | desktop release (Tauri build + signing) | signed bundle |

---

## Phase 0 — Audit (tasks 1–5)

| ID | Title | depends_on | ac |
|----|-------|------------|----|
| 1 | inventory `src-tauri/`, `src/`, `app/` modules | — | ac_v1 |
| 2 | scan remaining clippy warnings (Rust side) | 1 | ac_v1 |
| 3 | scan remaining mypy errors (Python glue) | 1 | ac_v1 |
| 4 | scan bandit MEDIUM findings | 1 | ac_v1 |
| 5 | tag current HEAD as `research-ledger-v0.x` baseline | 1–4 | ac_v1 |

---

## Ac conventions

- `ac_v1`: commit on `main` with conventional subject + DAG id in footer.
- `ac_test`: `cargo test` (Rust) + `pytest -q tests/` (Python) exit 0.
- `ac_clippy`: `cargo clippy -- -D warnings` exits 0.

---

## Notes

- Part of the **Phenotype Fleet** (cross-repo audit at
  `pheno-harness/_cockpit/XREPO_BACKLOG.json`).
- Local-first design — no data leaves the device unless explicitly
  exported by the user.
- Source provenance is a hard requirement: every document must carry
  its origin URL/identifier in metadata.
- AMC / Agentora remains paused per `pheno-harness/AGENTS.md §3.2`.
- Branch taxonomy: 8-prefix (feat/, fix/, chore/, docs/, test/, refactor/,
  perf/, build/).
