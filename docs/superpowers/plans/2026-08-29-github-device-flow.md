# GitHub Device Flow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let the installed local ResearchLedger app securely connect a personal GitHub App through Device Flow and resumably import starred repositories and README-backed provenance entries.

**Architecture:** Rust owns the public Client ID lookup, device-flow exchange, pending-session state, and macOS Keychain credential. The versioned Keychain record holds access/optional refresh material and expiry timestamps; React receives only typed safe states and invokes commands. The importer pages stars one page at a time, persists source metadata before README work, and records an SQLite checkpoint after each item so a cancellation or rate limit never loses corpus progress. v1 supports macOS only.

**Tech Stack:** Tauri 2, Rust, reqwest, keyring, rusqlite/SQLite, React 19, TypeScript, Vitest, Bun.

---

## File structure

| File | Responsibility |
| --- | --- |
| `apps/desktop/src-tauri/Cargo.toml` | Add the OS credential-store and HTTP-mock test dependencies. |
| `apps/desktop/src-tauri/src/github_auth.rs` | Device-code request/poll protocol, response classification, and rate-limit metadata parsing. |
| `apps/desktop/src-tauri/src/github_credentials.rs` | Injectable Keychain adapter; no UI or HTTP behavior. |
| `apps/desktop/src-tauri/src/github.rs` | GitHub REST client, one stars page at a time, README result classification. |
| `apps/desktop/src-tauri/src/storage.rs` | Durable GitHub import run/checkpoint rows and test helpers. |
| `apps/desktop/src-tauri/src/lib.rs` | Tauri state, typed commands, import orchestration, command registration. |
| `src/App.tsx` | Connection card state machine and progress UI. |
| `src/App.test.tsx` | UI command-contract and no-secret assertions. |
| `docs/github-connection.md` | User setup/recovery instructions using the public Client ID only. |

## Task 1: Add credential and deterministic HTTP-test foundations

**Files:**
- Modify: `apps/desktop/src-tauri/Cargo.toml`
- Create: `apps/desktop/src-tauri/src/github_credentials.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing credential-contract tests in `github_credentials.rs`.**

```rust
#[test]
fn credential_store_round_trip_and_delete_are_scoped_to_one_account() {
    let store = MemoryCredentialStore::default();
    store.set("octocat", "access-token").unwrap();
    assert_eq!(store.get("octocat").unwrap(), Some("access-token".into()));
    assert_eq!(store.get("other").unwrap(), None);
    store.delete("octocat").unwrap();
    assert_eq!(store.get("octocat").unwrap(), None);
}

#[test]
fn credential_errors_never_include_the_secret_value() {
    let error = CredentialError::Unavailable("access-token".into()).to_string();
    assert!(!error.contains("access-token"));
}
```

- [ ] **Step 2: Run the credential tests and verify they fail.**

Run: `cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml github_credentials`

Expected: FAIL because `github_credentials` does not yet exist.

- [ ] **Step 3: Add the minimal dependency and adapter.**

Add these entries to `Cargo.toml`:

```toml
[dependencies]
keyring = "3"

[dev-dependencies]
httpmock = "0.7"
```

Define `CredentialStore` with `get`, `set`, and `delete`; implement it with
`keyring::Entry` using service `com.kooshapari.researchledger.github.v1` and
the GitHub login as account. Its stored JSON is private `GithubCredential {
access_token, refresh_token: Option<String>, expires_at, refresh_expires_at }`.
Convert all OS errors to fixed user-safe strings. Keep `MemoryCredentialStore`
behind `#[cfg(test)]`. Add `mod github_credentials;` at the top of `lib.rs`.

- [ ] **Step 4: Run the credential tests and formatting.**

Run: `cargo fmt --manifest-path apps/desktop/src-tauri/Cargo.toml --check && cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml github_credentials`

Expected: PASS.

- [ ] **Step 5: Commit the isolated foundation.**

```bash
git add apps/desktop/src-tauri/Cargo.toml apps/desktop/src-tauri/Cargo.lock apps/desktop/src-tauri/src/github_credentials.rs apps/desktop/src-tauri/src/lib.rs
git commit -m "feat(github): add credential store boundary"
```

## Task 2: Implement and test GitHub Device Flow protocol classification

**Files:**
- Create: `apps/desktop/src-tauri/src/github_auth.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing protocol tests against an `httpmock` server.**

```rust
#[tokio::test]
async fn device_start_sends_json_accept_and_public_client_id() {
    let server = MockServer::start_async().await;
    let request = server.mock_async(|when, then| {
        when.method(POST).path("/login/device/code")
            .header("accept", "application/json")
            .body_contains("client_id=public-client-id");
        then.status(200).json_body(json!({
            "device_code": "opaque-device", "user_code": "ABCD-EFGH",
            "verification_uri": "https://github.com/login/device",
            "expires_in": 900, "interval": 5
        }));
    }).await;
    let flow = DeviceFlowClient::new(server.base_url(), "public-client-id").unwrap();
    assert_eq!(flow.start().await.unwrap().poll_interval_seconds, 5);
    request.assert_async().await;
}

#[test]
fn poll_response_never_serializes_access_token_to_ui_state() {
    let state = PollResult::Connected { login: "octocat".into() };
    assert!(!serde_json::to_string(&state).unwrap().contains("access_token"));
}
```

- [ ] **Step 2: Run the tests and verify failure.**

Run: `cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml github_auth`

Expected: FAIL because `DeviceFlowClient` and `PollResult` are undefined.

- [ ] **Step 3: Implement `DeviceFlowClient` and pure response types.**

Use `POST /login/device/code` and `POST /login/oauth/access_token`, form bodies,
and `Accept: application/json`. Define `DeviceStart`, `PollResult`, and private
token-bearing `TokenExchange`. Map GitHub's `authorization_pending`, `slow_down`,
`expired_token`, `access_denied`, and unknown errors into typed variants. Parse
optional `refresh_token`, `expires_in`, and `refresh_token_expires_in` only into
the private token-bearing type. Enforce the server interval and increase it by
five seconds after `slow_down`; reject a poll after local expiry. Keep the access
token in a private return value passed directly to the credential adapter, never
a serializable command value.

- [ ] **Step 4: Add failure cases before moving on.**

Add tests for expiry, cancellation-ready pending state, slow-down interval,
denial, malformed payload, a refresh success that replaces the Keychain record,
a refresh `401` that returns `Reconnect`, and a request error whose text contains
a fake token. Each assertion must prove the error category does not include the
fake token.

- [ ] **Step 5: Verify and commit.**

Run: `cargo fmt --manifest-path apps/desktop/src-tauri/Cargo.toml --check && cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml github_auth`

```bash
git add apps/desktop/src-tauri/src/github_auth.rs apps/desktop/src-tauri/src/lib.rs apps/desktop/src-tauri/Cargo.lock
git commit -m "feat(github): add Device Flow protocol client"
```

## Task 3: Add durable import-run checkpoints and paged REST primitives

**Files:**
- Modify: `apps/desktop/src-tauri/src/storage.rs`
- Modify: `apps/desktop/src-tauri/src/github.rs`

- [ ] **Step 1: Write the failing storage tests.**

```rust
#[test]
fn github_run_checkpoint_survives_reopen_and_keeps_last_completed_item() {
    let root = temp_root();
    let paths = initialize(&root).unwrap();
    let db = open(&paths).unwrap();
    create_github_import_run(&db, "run-1", "octocat", "v1", "2026-08-29T00:00:00Z").unwrap();
    checkpoint_github_import_run(&db, "run-1", 2, "octocat/repo", "running", "2026-08-29T00:01:00Z").unwrap();
    drop(db);
    let reopened = open(&paths).unwrap();
    assert_eq!(github_import_run(&reopened, "run-1").unwrap().unwrap().last_repository, Some("octocat/repo".into()));
}
```

- [ ] **Step 2: Run the test and verify failure.**

Run: `cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml github_run_checkpoint`

Expected: FAIL because the run storage API does not exist.

- [ ] **Step 3: Add the `github_import_runs` schema and typed storage API.**

In `storage::initialize`, execute `CREATE TABLE IF NOT EXISTS github_import_runs`
with `id`, `github_login`, `configuration_version`, `status`, `last_page`,
`last_repository`, `started_at`, `updated_at`, and `error_category`. Do not add
token, authorization-code, README, or raw response columns. Add
`GithubImportRun`, `create_github_import_run`, `checkpoint_github_import_run`,
`github_import_run`, and `finish_github_import_run` functions.

- [ ] **Step 4: Replace aggregate stars with page primitives and classify README results.**

Replace `list_starred() -> Vec<StarredRepository>` with
`list_starred_page(page: u32) -> Result<StarredPage, GithubError>`. Make
`StarredPage` carry items and `is_last_page`. Replace `Option<String>` README
results with `ReadmeResult::{Content(String), Unavailable, RetryAfter(Option<i64>),
PermissionDenied, Unauthorized, InvalidPayload}`. Read `x-ratelimit-remaining` and
`x-ratelimit-reset` before categorizing `403`; only zero remaining is a primary
rate-limit outcome.

- [ ] **Step 5: Add HTTP fixture tests.**

Test a 100-item page followed by an empty page, `404` README becoming
`Unavailable`, a `403` with remaining `0` becoming `RetryAfter`, a `403` with
remaining `10` becoming `PermissionDenied`, malformed base64 becoming
`InvalidPayload` without embedding the payload in an error, and an HTTP `401`
becoming `Unauthorized`, never `RetryAfter`.

- [ ] **Step 6: Verify and commit.**

Run: `cargo fmt --manifest-path apps/desktop/src-tauri/Cargo.toml --check && cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml github storage`

```bash
git add apps/desktop/src-tauri/src/storage.rs apps/desktop/src-tauri/src/github.rs
git commit -m "feat(github): checkpoint paged starred imports"
```

## Task 4: Wire typed Tauri connection commands and import orchestration

**Files:**
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src-tauri/src/github_auth.rs`
- Modify: `apps/desktop/src-tauri/src/github_credentials.rs`

- [ ] **Step 1: Write failing command-level tests in `lib.rs`.**

```rust
#[test]
fn connected_status_contains_identity_but_not_credential() {
    let value = GithubConnectionStatus::Connected { login: "octocat".into() };
    let json = serde_json::to_string(&value).unwrap();
    assert!(json.contains("octocat"));
    assert!(!json.contains("token"));
}

#[test]
fn import_summary_separates_readme_unavailable_from_failed() {
    let summary = GithubImportSummary { created: 1, updated: 0, unchanged: 0, readme_unavailable: 1, failed: 0, paused_until: None };
    assert_eq!(summary.readme_unavailable, 1);
    assert_eq!(summary.failed, 0);
}
```

- [ ] **Step 2: Run the command tests and verify failure.**

Run: `cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml connected_status_contains_identity`

Expected: FAIL because the connection types do not exist.

- [ ] **Step 3: Add state and commands.**

Define `GithubDeviceState` as `Mutex<Option<PendingDeviceSession>>`; add it with
`.manage(GithubDeviceState::default())` in `run`. Implement these commands with
serializable safe response types: `github_connection_status`, `github_device_start`,
`github_device_poll`, `github_device_cancel`, `github_disconnect`, and
`import_github_starred`. Register all six in `generate_handler!`.

`github_device_poll` obtains a token privately, fetches `/user` to obtain the
login, writes the versioned credential record to the credential store, then
returns `Connected { login }`. Before import, refresh an expired access token
once only when the record has valid refresh material; a refresh failure clears
the credential record and returns `ActionRequired(Reconnect)`.
`github_disconnect` deletes the credential before returning disconnected. A store
error returns a fixed category and leaves the connection unchanged.

- [ ] **Step 4: Make import atomic at repository boundaries.**

In `import_github_starred`, load the credential through the adapter, create/resume
the run, request one page, then for each repository construct and upsert the
metadata source document before `read_readme`. A README success updates that same
document; `Unavailable` writes `README unavailable.` and increments
`readme_unavailable`; rate limit checkpoints the current item and returns a
paused summary; permission/decode failures preserve metadata and increment
`failed`. Network and 5xx requests retry only at 1, 2, then 4 seconds; after the
third failure the run checkpoints as paused. A `401` checkpoints the item and
returns `ActionRequired(Reconnect)`. Checkpoint only after `upsert_document`
succeeds. Preserve existing `github:<owner>/<repo>` documents from the legacy
`gh` path until the native write succeeds.

- [ ] **Step 5: Add end-to-end in-process tests.**

Use the fake credential store plus mocked REST server to prove: a cancelled run
does not create a credential; restart resumes from the checkpoint; a missing
README creates searchable repository metadata; a legacy `gh`-imported document
is unchanged before native success; an HTTP `401` requires reconnection; and a
fake token never appears in serialized status or error strings.

- [ ] **Step 6: Verify and commit.**

Run: `cargo fmt --manifest-path apps/desktop/src-tauri/Cargo.toml --check && cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --lib`

```bash
git add apps/desktop/src-tauri/src/lib.rs apps/desktop/src-tauri/src/github_auth.rs apps/desktop/src-tauri/src/github_credentials.rs
git commit -m "feat(github): add native connection commands"
```

## Task 5: Replace the GitHub source card with the connection state machine

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/App.test.tsx`

- [ ] **Step 1: Replace the existing GitHub UI test with failing state-machine tests.**

```tsx
it("shows a device code but never a GitHub token field", async () => {
  invokeMock.mockImplementation((command: string) => {
    if (command === "github_connection_status") return Promise.resolve({ state: "disconnected" });
    if (command === "github_device_start") return Promise.resolve({ state: "pending", userCode: "ABCD-EFGH", verificationUri: "https://github.com/login/device", expiresAt: "2026-08-29T00:15:00Z" });
    return Promise.resolve({ selected: true, path: "/tmp/research-vault", documentCount: 0 });
  });
  await renderApp();
  fireEvent.click(await screen.findByRole("button", { name: "Connect GitHub" }));
  expect(await screen.findByText("ABCD-EFGH")).toBeInTheDocument();
  expect(screen.queryByRole("textbox", { name: /token/i })).not.toBeInTheDocument();
});
```

- [ ] **Step 2: Run the UI test and verify failure.**

Run: `bun run test -- src/App.test.tsx`

Expected: FAIL because the current card only invokes `import_github_from_gh`.

- [ ] **Step 3: Implement safe connection and import UI.**

Add typed TypeScript interfaces for all command responses. On startup invoke
`github_connection_status`. Render only one GitHub card state at a time:
Disconnected, Awaiting authorization, Connected, Importing, or Action required.
Use `window.open(verificationUri, "_blank", "noopener,noreferrer")` only after
the user presses `Open GitHub`. Poll using the server-provided interval, stop on
unmount/cancel, and never retain a token in React state. Replace the current
default import action with `import_github_starred`; preserve the CLI path behind
an explicit Advanced disclosure.

- [ ] **Step 4: Add complete behavior tests.**

Cover cancellation, expired/denied authorization, connected identity, disabled
import without a vault, README-unavailable summary, rate-limit retry text, and
disconnect returning to disconnected. Assert no call contains a property named
`token`, `accessToken`, or `refreshToken`.

- [ ] **Step 5: Verify and commit.**

Run: `bun run lint && bun run test -- src/App.test.tsx && bun run build`

```bash
git add src/App.tsx src/App.test.tsx
git commit -m "feat(github): add first-run connection UI"
```

## Task 6: Document setup, verify no regressions, and prepare release proof

**Files:**
- Create: `docs/github-connection.md`
- Modify: `README.md`
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Write the user-facing setup document.**

Document: macOS-only v1 support, personal GitHub App ownership, Device Flow enablement, least-privilege
read-permission verification, where to configure `RESEARCHLEDGER_GITHUB_CLIENT_ID`,
and the rule that the value is a public Client ID. Include recovery steps for
expiry, revocation, Keychain denial, rate limit, missing README, and disconnect.
Explicitly state that no client secret, token, cookie, or browser profile is used.

- [ ] **Step 2: Add a release checklist to the document.**

Include the exact installed-app smoke sequence: new vault, Device Flow connection,
real starred import, provenance inspection, relaunch/search/export, disconnect,
and reauthorization proof. State that a green mock suite is insufficient.

- [ ] **Step 3: Run the complete local quality gate.**

Run:

```bash
bun install --frozen-lockfile
bun run lint
bun run test
bun run build
RESEARCHLEDGER_SKIP_BUNDLE_RESOURCE_VALIDATION=1 cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --lib
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets -- -D warnings
git diff --check
```

Expected: all commands exit `0`; no test fixture, log, documentation, or staged
file contains a real credential.

- [ ] **Step 4: Commit documentation and open the implementation PR.**

```bash
git add docs/github-connection.md README.md CHANGELOG.md
git commit -m "docs(github): explain secure connection setup"
git push --set-upstream origin feat/github-device-flow
gh pr create --base main --title "feat(github): add native starred import" --body "## Summary\n- native Device Flow and Keychain credential boundary\n- resumable starred import with provenance\n\n## Validation\n- full Bun and Rust quality gates"
```

- [ ] **Step 5: Complete hosted and installed-app gates.**

Wait for required PR checks and external review without bypassing protection.
After merge, build/sign/notarize/staple the app, perform the documented real-account
smoke manually, and attach only non-secret evidence (commit, build identifier,
Gatekeeper result, source counts, and assertion outcomes) to the release record.

## Plan self-review

Spec coverage is complete: secure Device Flow, public-Client-ID configuration,
Keychain storage, typed commands, identity/disconnect, paged/resumable star import,
README failure preservation, rate-limit classification, local-first provenance,
bounded follow-on enrichment, automated tests, and installed-app proof each map to
a task above. No plan task expands LinkedIn, GitHub write access, or server-side
data handling.

The implementation must start from a clean feature worktree based on the merged
design specification. Do not merge this plan or implement from a dirty retained
evidence branch.
