# Security and privacy

ResearchLedger is local-first. Vault Markdown, SQLite databases, imported source content,
tokens, and browser state are local artifacts and must not be committed or uploaded.

- GitHub tokens are accepted only in memory for an import and cleared from the UI after use.
- The GitHub adapter is read-only: it lists stars and reads repository READMEs.
- LinkedIn support imports user-provided local HTML/export files. It does not automate login,
  MFA, CAPTCHA, posting, reactions, follows, messages, or browser-state extraction.
- Tauri filesystem commands must validate user-selected roots before reading or writing.
- Logs and error messages must redact tokens and authentication artifacts.
- CI uses synthetic fixtures and never requires real credentials or personal data.
- Users can delete imported Markdown and rebuild SQLite indexes from the vault.

Before release, review Tauri capabilities, macOS signing/notarization settings, dependency
licenses, and the exact external service terms for every enabled connector.
