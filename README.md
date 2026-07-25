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
starred-repository import, browser-capture imports for LinkedIn, Reddit, and X
bookmarks, offline FTS5 search, and retrieval context with aligned citations.
Imported documents are written as Markdown under
the selected vault and indexed into a local SQLite database. Markdown export is compatible
with Obsidian and Logseq-style vault workflows.

GitHub uses the OAuth device flow when a GitHub App client ID is configured. The UI displays
GitHub’s verification URL/code and polls only at the interval supplied by GitHub; a pasted
token is retained solely as an advanced fallback.

For LinkedIn’s personal reaction feed, Reddit saved posts, and X bookmarks, use the
authenticated browser connectors (each uses a dedicated persistent browser profile):

```bash
npm run linkedin:capture -- --profile "$HOME/Library/Application Support/ResearchLedger/linkedin-profile" \
  --output "$PWD/linkedin-capture.json"

npm run reddit:capture -- --profile "$HOME/Library/Application Support/ResearchLedger/reddit-profile" \
  --output "$PWD/reddit-capture.json"

npm run x:capture -- --profile "$HOME/Library/Application Support/ResearchLedger/x-profile" \
  --output "$PWD/x-capture.json"
```

Each connector opens a persistent local browser profile, waits for the user’s normal login
if needed, scrolls at a bounded rate, deduplicates post URLs, and writes a deterministic
capture file for import into the selected vault. None of them automate posting, messaging,
reactions, follows, votes, or other account actions.

### First-run: Playwright browser install

The capture scripts use Playwright’s Chromium binary. On first capture (or after a Playwright
version bump), the app will detect a missing browser and run `npx playwright install chromium`
automatically — one time. Subsequent runs use the cached install under
`~/Library/Caches/ms-playwright/`. If auto-install fails (offline, restrictive network),
re-run manually:

```bash
npx playwright install chromium
```

See [security](docs/SECURITY.md) and [testing](docs/TESTING.md) for data-handling and
verification rules.
