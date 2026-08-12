# ResearchLedger — AGENTS.md

## Project Overview

ResearchLedger is a **local-first research ledger and LLM knowledge
base**. The desktop app keeps Markdown and SQLite data local,
preserves source provenance, and serves as the durable store for
research notes, citations, and machine-readable evidence.

| Aspect | Value |
|---|---|
| Language stack | TypeScript + Node (npm), Electron desktop app (BrowserWindow) |
| Storage | Local SQLite + local Markdown files |
| Audience | Researchers, knowledge workers, agents writing research notes |
| Distribution | Local desktop app (no telemetry) |

## Workspace Layout

```
ResearchLedger/
├── apps/                       # Workspace apps (TS + Electron)
├── docs/                       # User-facing docs
├── dist/                       # Built artifacts
├── index.html                  # Entry point
├── package.json                # Root package manifest
├── package-lock.json           # npm lockfile
├── bun.lock                    # Bun lockfile (cross-runtime)
├── linkedin-capture.json       # Sample data (template)
├── LICENSE                     # License
├── CHANGELOG.md                # Release notes
└── audits/                     # Audit-trail artifacts (added 2026-08-11)
    ├── README.md
    ├── org-audit-snapshots/
    ├── postmortems/
    ├── ci-exceptions/
    ├── boundary-reconciliation/
    └── absorption-justifications/
```

## Branch Discipline

- `main` is protected; all changes flow through PRs.
- Branch naming: `feat/`, `fix/`, `chore/`, `docs/`, `refactor/`.
- Knowledge-base content updates (notes, citations) commit via
  `docs(content):` prefix so the changelog can grep them.

## Conventions

- TypeScript with strict mode; prefer named exports.
- Markdown notes are dated `YYYY-MM-DD-<slug>.md` and live under
  `docs/notes/`.
- Cite sources inline using `[[wikilink]]` notation; the build
  resolves these to `linkedin-capture.json` (or similar) at link time.

## Quality Gates

- `npm install` (or `bun install`)
- `npm run typecheck` (tsc --noEmit)
- `npm run lint` (eslint)
- `npm run test` (vitest)
- `npm run build` (tsc + electron-builder)

## Key Commands

- `npm run dev` — start the desktop app in dev mode
- `npm run build` — produce a packaged build
- `npm run note:new` — create a new dated note from a template
- `npm run search` — query the SQLite knowledge base

## Important Notes

- ResearchLedger is **local-first**. Do not add features that
  require sending data to a remote server.
- Source provenance is non-negotiable. Every note must link back to
  at least one source (URL, paper, conversation reference).
- The `linkedin-capture.json` is a sample file; real captures go
  under a per-user directory.

## Cross-references

- Parent context: `/Users/kooshapari/CodeProjects/Phenotype/repos/_cockpit/audit-ResearchLedger.json`
- Backlog: `X-DOCS-022` (closes "Missing AGENTS.md" gap from
  `_cockpit/XREPO_BACKLOG.json`).
- Sister audit-dir commits: Benchora `bd8b717`, PhenoPlugins
  `0fc70fb`, Eidolon `cc20a5e`, RepoLedger `11fde57`.
- Phenotype root: `/Users/kooshapari/CodeProjects/Phenotype/repos/CLAUDE.md`.
