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
