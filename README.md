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
  --output "$HOME/.phenotype/researchledger/captures/linkedin-capture.json"

npm run reddit:capture -- --profile "$HOME/Library/Application Support/ResearchLedger/reddit-profile" \
  --output "$HOME/.phenotype/researchledger/captures/reddit-capture.json"

npm run x:capture -- --profile "$HOME/Library/Application Support/ResearchLedger/x-profile" \
  --output "$HOME/.phenotype/researchledger/captures/x-capture.json"
```

Each connector opens a persistent local browser profile, waits for the user’s normal login
if needed, scrolls at a bounded rate, deduplicates post URLs, and writes a deterministic
capture file for import into the selected vault. None of them automate posting, messaging,
reactions, follows, votes, or other account actions.

### First-run: Playwright browser install

The capture scripts use Playwright’s Chromium binary. `npm install` runs a
`postinstall` hook (`scripts/postinstall.mjs`) that pre-fetches the browser so
the first capture feels instant — there is no prompt, no spinner, and no
~150 MB download during capture. The hook is platform-aware (Linux also pulls
system deps via `--with-deps`), idempotent (Playwright’s install is a no-op
when the browser is already cached at `~/Library/Caches/ms-playwright/`), and
skipped automatically when `CI=1` or `PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1`.

If the install-time hook fails (offline, restrictive network, missing sudo on
Linux) the capture scripts still detect a missing browser and run
`npx playwright install chromium` automatically — one time, as a safety net.
Subsequent runs use the cached install under `~/Library/Caches/ms-playwright/`.
You can also re-run manually:

```bash
npx playwright install chromium
```

See [security](docs/SECURITY.md) and [testing](docs/TESTING.md) for data-handling and
verification rules.
