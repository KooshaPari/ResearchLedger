# Testing strategy

- Static/resource: `npm run verify:resources`, `npm run build`.
- Frontend/capture: `npm test` (65 tests).
- Rust: `CARGO_TARGET_DIR=/tmp/researchledger-target cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml` (57 tests).
- Release: build app-only bundle, verify packaged resources, install, then smoke provider
  menus and vault/export flows.
- Account smoke: run LinkedIn capture and GitHub import only with the user's authenticated
  local profile; record counts/hashes, never tokens or raw snapshots.
