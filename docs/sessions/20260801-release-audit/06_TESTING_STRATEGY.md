# Testing strategy

- Static/resource: `npm run verify:resources`, `npm run build`.
- Frontend/capture: `npm test` (65 tests).
- Rust: `CARGO_TARGET_DIR=/tmp/researchledger-target cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml` (69 tests).
- Release: build app-only bundle, verify packaged resources, install, then smoke provider
  menus and vault/export flows. The current installed executable is SHA-256
  `adf1facbfc860afda22fa9151627c8c81519e4fe42466bb500ea1d566ae95895`; packaged Playwright
  module import and resource parity both pass.
- Account smoke: run LinkedIn capture and GitHub import only with the user's authenticated
  local profile; record counts/hashes, never tokens or raw snapshots.
