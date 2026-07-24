# Security and privacy

ResearchLedger is local-first. Vault Markdown, SQLite databases, imported source content,
tokens, and browser state are local artifacts and must not be committed or uploaded.

- GitHub tokens are accepted only in memory for an import and cleared from the UI after use.
- GitHub OAuth device authorization is the primary path; the client ID is not a secret, and
  the returned user token is held in memory only for the current import session.
- The GitHub adapter is read-only: it lists stars and reads repository READMEs.
- LinkedIn, Reddit, and X support use a user-directed local browser capture for
  read-only collection of activity / saved posts / bookmarks. None of them automate
  posting, messaging, reactions, follows, votes, or other account actions.
- The LinkedIn, Reddit, and X browser connectors each use a user-selected persistent
  local profile only for reading the appropriate saved-posts page, apply bounded
  scrolling and deduplication, and store only extracted post text/URLs in the
  capture file; browser cookies remain in the dedicated profile.
- Tauri filesystem commands must validate user-selected roots before reading or writing.
- Logs and error messages must redact tokens and authentication artifacts.
- CI uses synthetic fixtures and never requires real credentials or personal data.
- Users can delete imported Markdown and rebuild SQLite indexes from the vault.

Before release, review Tauri capabilities, macOS signing/notarization settings, dependency
licenses, and the exact external service terms for every enabled connector.
