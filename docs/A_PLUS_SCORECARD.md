# ResearchLedger release scorecard

Status as of 2026-08-12: **release candidate, not A+ yet**. This document records
implementation evidence, not a release or installed-app claim. Only consented bounded
reference fetching, persisted claims/provenance, and versioned hybrid retrieval with a
local cross-encoder contract are evidenced in the current integration.

| Requirement | Evidence | Status |
|---|---|---|
| Local-first Tauri desktop | Tauri desktop source, SQLite metadata, and Markdown vault paths are present in the integration | IMPLEMENTED / package and installed-app validation pending |
| GitHub import | Rust-owned local GitHub CLI integration and importer code are present | IMPLEMENTED / authenticated installed-app invocation pending |
| Provider boundary | Official export/manual permalink path is the intended LinkedIn boundary; no release claim is supported by this merge | IMPLEMENTED boundary / live policy and installed-app validation pending |
| Consented bounded fetching | Scoped, expiring, revocable consent grants; hashed consent audit targets; bounded public-reference fetch guards and persisted fetched sources | IMPLEMENTED / validation and security review pending |
| Claims and provenance | Persisted provenance and structured claims with source citations and reproducible spans | IMPLEMENTED / validation pending |
| Search/RAG | FTS5 lexical search, deterministic chunking, model/version/input-hashed vectors, hybrid fusion including vector-only candidates, and a loopback local cross-encoder contract with deterministic fallback | IMPLEMENTED / local-model quality and restart validation pending |
| Frontend and interoperability | Workspace workflows and OKF-style Markdown/JSON interchange are implemented | IMPLEMENTED / validation and packaged-app parity pending |
| Privacy and security | Renderer credential boundary, URL/path guards, consent and audit code are implemented | IMPLEMENTED / security review and installed-app verification pending |
| Verification | No current full validation, package, installed-binary, or authenticated-provider evidence is claimed by this scorecard | OPEN |

## Remaining A+ gates

1. Run the complete validation suite and record reproducible results for this merged integration.
2. Build and validate the release package, then prove packaged and installed-binary parity.
3. Complete security review, including the consent, credential, provider-boundary, URL/path, and
   dependency surfaces; resolve or explicitly govern findings.
4. Run an authenticated GitHub import in the installed application, recording repository and README
   counts plus non-secret hashes; the renderer must never receive a credential.
5. Restart the installed application and verify that consent state, fetched-source provenance,
   claims, vector/index state, and cited retrieval results survive the restart.
6. Validate the configured local cross-encoder/model on the installed path and retain the
   deterministic fallback as the offline behavior.

Operational prerequisite: optional local retrieval services and any permitted packaged connector
dependencies must be installed and started by the operator. No credential, cookie, or raw sensitive
target is retained as release evidence.
