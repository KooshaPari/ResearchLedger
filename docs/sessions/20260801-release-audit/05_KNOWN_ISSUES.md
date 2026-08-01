# Known issues

- The prior scorecard overstated enrichment and retrieval as complete; it is now corrected.
- `cargo fmt --check` currently reports pre-existing formatting drift across provider files;
  the provenance patch itself compiles and its focused/full tests pass.
- Frontend tests emit React `act(...)` warnings; they do not fail the suite.
- Installed app must be rebuilt after the provenance patch before it is treated as current.
- Reference fetch, structured claims, reranking, and persisted graph/collection views remain
  unimplemented.
