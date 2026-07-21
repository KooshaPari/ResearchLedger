# ResearchLedger

Local-first research ledger and LLM knowledge base.

The first slice is a Tauri/React desktop shell. The application will keep Markdown and
SQLite data local, preserve source provenance, and expose import/search/export adapters.

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

The desktop app currently supports local vault setup, GitHub starred-repository import,
manual LinkedIn HTML export import, and offline FTS5 search. Imported documents are written
as Markdown under the selected vault and indexed into a local SQLite database.

See [security](docs/SECURITY.md) and [testing](docs/TESTING.md) for data-handling and
verification rules.
