# ResearchLedger

Local-first research ledger and LLM knowledge base.

The desktop app keeps Markdown and SQLite data local, preserves source provenance, and
exposes import/search/RAG/export adapters through a Tauri command boundary.

## Development

```bash
npm install
npm run dev
```

Tests and production build:

```bash
npm test -- --run
npm run build
```

The desktop app supports a native local-vault picker, persisted vault status, GitHub
starred-repository import, manual LinkedIn HTML export import, offline FTS5 search, and
retrieval context with aligned citations. Imported documents are written as Markdown under
the selected vault and indexed into a local SQLite database. Markdown export is compatible
with Obsidian and Logseq-style vault workflows.

See [security](docs/SECURITY.md) and [testing](docs/TESTING.md) for data-handling and
verification rules.
