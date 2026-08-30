# ResearchLedger GitHub Device Flow and Resumable Starred Import

## Status and decision

This design is the next release-critical ResearchLedger slice after the
preserved-work reconciliation merged at `236cbf0`. It replaces the current
developer-only `gh auth token` dependency with a first-run connection owned by
the installed desktop app.

The default ownership model is a **personal GitHub App** for the local
ResearchLedger workstation. This keeps installation, authorization, and
repository access under the user's personal GitHub account. The model is
deliberately replaceable: an organization-owned App can use the same local
protocol later, but changes installation and repository-selection policy and
is not part of this implementation slice.

GitHub Device Flow is the authorization mechanism. The application needs the
App's public Client ID, but never a client secret. The user authorizes in their
own browser; the desktop app stores the resulting credential only in the OS
credential store.

## Problem statement

Today `import_github_from_gh` launches `gh auth token`, receives a raw token in
the Rust process, fetches all starred repositories into memory, and then fetches
their READMEs. It is not a usable first-run flow, cannot report identity or
connection state, has no persistent app credential, and skips a repository
entirely when a README request fails.

The user needs a local-first research import that produces one deterministic
source entry per starred repository with the name, description, URL, README,
and provenance. An unavailable README must never erase otherwise useful
repository metadata. Interrupted imports must resume without duplicate source
entries or unbounded network work.

## Scope

### In scope

- Native GitHub App Device Flow connection for GitHub.com.
- OS credential-store persistence, identity display, disconnect, and token
  refresh/reconnection handling.
- Paged, resumable import of the authenticated user's starred repositories.
- Repository metadata plus README import as stable local Markdown/SQLite source
  documents with provenance.
- Item-level status and actionable errors for authorization, pagination,
  rate-limits, unavailable repositories, and README failures.
- Contract, unit, component, and installed-app smoke verification.

### Platform boundary

Version one is macOS-only because ResearchLedger's release and Keychain proof
are macOS artifacts. The credential interface is platform-neutral, but Linux
or Windows support is not implied by this design: each requires its own secure
credential-store adapter, packaging test, and release gate before it can be
claimed as supported.

### Out of scope

- Sending vault content, credentials, telemetry, embeddings, or search queries
  to a ResearchLedger service. The feature remains local-first.
- GitHub write operations, repository mutation, issue/PR ingestion, or an
  organization-installation UI.
- LinkedIn browser automation, browser-profile copying, cookie extraction,
  session cloning, CAPTCHA handling, or reaction-feed crawling. LinkedIn stays
  manual permalink/content import under the project policy.
- Recursive external crawling without the existing explicit, bounded reference
  crawl consent flow.

## Authorization and security boundary

```text
React UI                    Rust/Tauri                     GitHub + macOS
--------                    ----------                     ------------
connect button ----start--> DeviceFlowService --POST-----> device-code endpoint
      |                          |                                  |
      |<-- code + URL + expiry ---+                                  |
open browser -------------------------------------------------------> authorize
      |                          |<--- poll response ----------------+
      |<-- connection status ----+                                  |
      |                          +--- write token -----------------> Keychain
import button ------run--------> GitHubImportService ----GET------> API
      |<-- sanitized progress ---+--- write docs/SQLite ----------> local vault
disconnect ----------run--------> CredentialStore ------delete----> Keychain
```

The frontend must never receive an access token, refresh token, client secret,
or raw authorization response. Tauri command responses contain only an opaque
connection state, GitHub login/display name, expiry class, and import status.
Structured logs must use error categories and HTTP status classes only; they
must not record response bodies, authorization codes, headers, or repository
README text.

The Rust credential adapter uses the platform credential store. Its service and
account names are stable, versioned identifiers, not a path inside the vault.
Disconnect deletes the credential first, then clears the non-secret connection
metadata. A failed credential-store deletion is an explicit error and leaves
the UI connected state unchanged.

The credential is a versioned Keychain JSON record containing the access token,
optional refresh token, and server-provided expiry timestamps. It is never a
vault file. Before a GitHub API call, Rust refreshes an expired access token only
when GitHub supplied both a refresh token and refresh expiry; it posts the
refresh grant directly to GitHub and atomically replaces the Keychain record.
If the refresh grant is denied, expired, malformed, or returns HTTP `401`, Rust
deletes no imported knowledge, clears the credential record, and returns
`ActionRequired(Reconnect)`; the user restarts Device Flow. The UI never learns
whether an access or refresh token existed. If GitHub did not return refresh
material, expiry always requires fresh Device Flow.

The implementation must verify the exact least-privilege GitHub App user
permissions required by the current REST endpoints during implementation. It
must not substitute a broad personal-access-token scope or expand to GitHub
write permissions.

## Tauri contract

Commands are Rust-owned and use typed request/response structs.

| Command | Input | Safe response | Behavior |
| --- | --- | --- | --- |
| `github_connection_status` | none | `Disconnected`, `Pending`, `Connected`, or `ActionRequired` plus non-secret identity | Reads local metadata and validates no token value is exposed. |
| `github_device_start` | none | verification URL, user code, expiry timestamp, poll interval | Requests a device code using the configured public Client ID. It does not start duplicate concurrent sessions. |
| `github_device_poll` | opaque session id | pending, slow-down, connected identity, denied, expired, or failed category | Polls no faster than GitHub's returned interval, applies slow-down, and expires locally. |
| `github_device_cancel` | opaque session id | cancelled | Deletes in-memory pending state; no credential has been stored before success. |
| `github_disconnect` | none | disconnected | Deletes the Keychain credential and local non-secret metadata. |
| `import_github_starred` | vault path, optional resume policy | progress summary and per-item status counts | Uses only the Rust credential adapter and checkpoints after each durable item. |

The existing `import_github_from_gh` command remains an explicitly labeled
Advanced compatibility fallback for the first native-connection release. It
must never be the default button path and must never disclose `gh` output to
the UI. Existing `github:<owner>/<repo>` documents need no data migration: the
native importer uses the same identity and atomically upserts only after a
successful repository metadata write. It must not modify historic `gh`-imported
documents until that first successful native write. Removal is considered only
in a later, separately reviewed release after native connection dogfooding.

## Connection UX

The GitHub source card has these mutually exclusive states:

1. **Disconnected** - explains the read-only data categories and offers
   `Connect GitHub` and an Advanced `Use existing GitHub CLI session` option.
2. **Awaiting authorization** - shows the GitHub-hosted verification URL,
   copyable user code, expiry countdown, `Open GitHub`, and `Cancel`. It never
   embeds a password form.
3. **Connected** - shows the authorized GitHub identity, authorization time,
   and `Import starred repositories` / `Disconnect` actions.
4. **Importing** - reports pages scanned, metadata saved, README unavailable,
   retryable failures, and the next retry time when rate-limited. It supports
   cancellation only between repository units, preserving the durable
   checkpoint.
5. **Action required** - distinguishes revoked/expired authorization,
   missing App configuration, rate limiting, inaccessible repositories, and
   local credential-store failure. Each state has one recovery action.

The source action is disabled until a vault is selected. A successful import
message reports created, updated, unchanged, README-unavailable, and failed
counts separately. It must not report an import as wholly successful when any
source item is unresolved.

## Import model and durability

The importer calls the authenticated-stars endpoint one page at a time, with
the API's maximum documented page size. It validates the authenticated identity
before the first page. For each repository it writes the deterministic
repository source entry before attempting the README, then updates that same
entry with README status/content and a timestamped provenance record.

The source identity remains `github:<owner>/<repo>`. Its Markdown front matter
includes the canonical repository URL, full name, description, language/topics
when available, default branch, import run id, and capture timestamp. README
provenance records the GitHub endpoint and result class, never a credential.

An import-run table stores: run id, identity, started/updated time, requested
vault, last completed page, last completed repository identity, immutable
configuration version, state, and categorized failures. It stores no access
token, authorization code, or README body. The importer commits a checkpoint
only after the document and SQLite index update have both succeeded. Resuming a
run repeats at most the current repository and relies on idempotent upsert.

Result handling is explicit:

| Result | Document outcome | Run outcome |
| --- | --- | --- |
| README returned | Metadata and README persisted | item complete |
| README not found | Metadata persisted with `README unavailable` provenance | item complete with warning |
| Repository inaccessible | Existing entry retained; new metadata record carries classified failure | retry only if category is retryable |
| Rate limited | Current item is not marked complete | run pauses with reset/retry guidance |
| Network/transient 5xx | Current item remains pending | retry at 1, 2, then 4 seconds; pause after three failures |
| Bad payload/decode | Metadata retained; payload issue recorded without body | item failure, no blind retry loop |

HTTP `403` must not automatically mean rate limiting. The client categorizes it
using GitHub rate-limit response information; permission and abuse/secondary
limit responses get their own user-facing recovery guidance.

HTTP `401` is an authorization failure, never a retry candidate. The importer
checks whether its stored credential can be refreshed once; if not, it checkpoints
the item as pending and returns `ActionRequired(Reconnect)` without deleting the
existing local source documents.

## Knowledge-pipeline integration

Each completed repository source is an ordinary provenance-bearing document.
The existing SQLite/FTS index and Markdown vault become durable before optional
embedding or distillation work. Any later embedding, hybrid retrieval, rerank,
reference-fetch, or claim-distillation run must retain the repository document
id and source URL so a user can recover the original repository/README.

URLs found in a README may enter the existing reference queue only after the
user grants an explicit bounded crawl consent. The GitHub importer itself does
not recursively fetch arbitrary README links.

## Error taxonomy

| Category | User message | Recovery |
| --- | --- | --- |
| App misconfigured | `GitHub connection is not configured on this device.` | Configure a valid public Client ID and enable Device Flow for the personal GitHub App. |
| Authorization pending | `Finish authorization in GitHub before this code expires.` | Open GitHub or wait; polling honors the server interval. |
| Denied/expired/revoked | `GitHub authorization needs to be restarted.` | Start a fresh connection. |
| Unauthorized | `GitHub authorization expired or was revoked.` | Refresh once when available; otherwise reconnect without altering local knowledge. |
| Credential store | `ResearchLedger could not securely save the GitHub connection.` | Review macOS Keychain access and retry; do not fall back to plaintext. |
| Permission | `GitHub did not authorize enough read access for this item.` | Re-authorize after the App's least-privilege configuration is corrected. |
| Rate limit | `GitHub asked ResearchLedger to pause until <time>.` | Resume automatically/with one explicit retry after reset. |
| README unavailable | `Repository imported; its README was unavailable.` | Keep metadata and allow a targeted retry. |

## Verification and release evidence

### Automated tests

- Rust tests with mocked HTTP prove device-code request headers/body, strict
  poll interval, slow-down, expiry, denial, cancellation, refresh replacement,
  refresh failure, and no token-bearing command response/log field.
- Credential-adapter tests use a fake store; CI never accesses the real
  Keychain or a live GitHub account.
- Import tests cover multi-page stars, empty pages, duplicate/resume behavior,
  unchanged historic `gh` imports before a successful native write, missing
  README, private/inaccessible repositories, API/secondary rate limits, HTTP
  `401`, malformed base64, the exact three-attempt 1/2/4-second retry policy,
  and checkpoint crash recovery.
- React component tests prove no token input/display, correct status wording,
  code copy/open behavior, progress rendering, and disconnect state.
- Existing Bun lint/test/build and Rust fmt/clippy/test remain mandatory.

### Manual installed-app smoke

1. Build, sign, notarize, staple, and Gatekeeper-verify the macOS app.
2. In the installed app, choose a new local vault and connect a real GitHub
   account through Device Flow.
3. Verify identity, import actual starred repositories, and inspect at least one
   README-backed entry and one provenance record.
4. Quit/relaunch, confirm the connection and corpus survive, search a repository
   term, and export Markdown.
5. Disconnect, relaunch, prove the app requires reauthorization, and confirm
   imported local knowledge remains unless explicitly removed.

No source-only green test suite is release proof. The final release gate requires
this signed installed-app smoke plus protected-branch CI and review.

## Alternatives rejected

1. **Default to `gh auth token`.** It works only for users who separately
   configure the GitHub CLI, makes first-run failure opaque, and cannot provide
   a polished in-app connection.
2. **Request a pasted personal access token.** This creates secret handling and
   scope-comprehension risk in the UI and makes revocation/disconnect weaker.
3. **OAuth App callback flow.** A callback/deep-link and secret lifecycle add
   deployment complexity without improving the desktop authorization outcome
   over Device Flow.
4. **Browser automation or cookie extraction.** It is inappropriate for GitHub
   authorization and unnecessary when the supported Device Flow exists.

## External configuration prerequisite

Before implementation smoke testing, the owner creates or identifies the
personal GitHub App, enables Device Flow, configures only the required
least-privilege read permissions, and supplies the public Client ID through
local application configuration. No Client Secret, access token, device code,
or Keychain export is requested or accepted by this repository.
