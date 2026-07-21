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

For LinkedIn’s personal reaction feed, use the authenticated browser connector:

```bash
npm run linkedin:capture -- --profile "$HOME/Library/Application Support/ResearchLedger/linkedin-profile" \
  --output "$PWD/linkedin-capture.json"
```

The connector opens a persistent local browser profile, waits for the user’s normal login
if needed, scrolls at a bounded rate, deduplicates activity URLs, and writes a deterministic
capture file for import into the selected vault. It does not automate posting, messaging,
reactions, follows, or other account actions.

See [security](docs/SECURITY.md) and [testing](docs/TESTING.md) for data-handling and
verification rules.
