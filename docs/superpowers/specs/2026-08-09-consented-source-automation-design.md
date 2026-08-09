# Consented Source Automation Design

**Status:** design gate recovery; implementation deliberately out of scope.

## Decision

ResearchLedger may import only user-consented, policy-supported source material. Each
import is tied to a consent record, an explicit URL scope, immutable acquisition metadata,
and claim-level spans that lead a reader back to the original artifact.

### Source policy

| Source | Allowed path | Explicitly disallowed |
|---|---|---|
| LinkedIn | Capability probe for an approved API; then user-supplied official export or manually supplied permalink/content | Browser DOM reaction/post scraping; automation that reads a signed-in feed; password, session, or cookie extraction |
| GitHub | Existing `gh` credential used only by Rust; otherwise first-party OAuth Device Flow; user-initiated advanced manual import | Copying `gh` tokens to the frontend; scraping GitHub HTML; silent broad-scope authorization |
| Generic web | User-approved seed URLs, constrained reference crawl, or a single manually supplied permalink | Open-ended discovery crawl, login-wall bypass, or collecting unrelated linked pages |

LinkedIn's agreement prohibits scraping/copying service data and use of unauthorized bots;
its API documentation requires member approval and appropriate scopes. A probe therefore
means testing whether the installed approved integration exposes the requested capability,
not attempting browser automation. If unavailable, the UI offers export/permalink/manual
import and records the limitation. Sources: [LinkedIn User Agreement](https://www.linkedin.com/legal/user-agreement),
[LinkedIn authorization overview](https://learn.microsoft.com/en-us/linkedin/shared/authentication/authentication).

## Import contract

`ConsentRegistry` is the gate before acquisition. A record contains: `consent_id`, local
profile, source/provider, acquisition method, declared purpose, data categories, URL scope,
granted time, expiry/revocation state, and a receipt/version. No active matching consent ->
no network request, parsing, persistence, or background refresh.

An import request includes one or more seed URLs plus a scope mode:

| Scope mode | Permit |
|---|---|
| `exact` | Only listed canonical URLs |
| `same-origin` | Seeds and same-origin references that match explicit allow/deny path rules |
| `reference-crawl` | Bounded, declared-depth links from saved artifacts; every candidate is checked before fetch |

Canonicalization removes fragments, normalizes host/scheme and records the pre-normalized URL.
The default is `exact`; crawl depth, page count, byte budget, allowed origins, and path filters
are consent-visible and persisted. External links, redirects leaving scope, robots/login walls,
and ambiguous canonical targets stop with a review-required result.

## Provenance and claims

Acquisition stores source URL, canonical URL, retrieval time, content hash, media type,
import method, consent ID, optional provider object ID, and parser/version. A claim stores its
normalized text plus one or more source spans: `artifact_id`, byte/character offsets, quoted
excerpt hash, locator strategy, and confidence. Rendering must show a backlink and enough
context to verify the span. Transformations (OCR, summary, merge, dedupe) retain parent
artifact IDs and never replace the original evidence.

## Authentication and credentials

GitHub import is atomic: authenticate, immediately call `/user` to bind the identity, then
persist the encrypted credential reference and account binding together, or persist neither.
The Rust backend is the only component that reads `gh` credentials/OS keychain. Its fallback
is GitHub's first-party Device Flow using a registered client ID, `Accept: application/json`,
the returned polling interval, and least privilege. The user may cancel/revoke; poll errors
are surfaced without exposing codes or tokens. Advanced manual import is a separate,
explicitly labelled path for a user-provided export or manually entered repository/permalink.

For starred repositories, request the documented read permission, paginate `/user/starred`,
respect rate/conditional responses, and record each API object and fetched repository artifact
as provenance. GitHub documents Device Flow and the `GET /user/starred` endpoint here:
[OAuth authorization](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps) and
[starred repositories](https://docs.github.com/en/rest/activity/starring?apiVersion=2022-11-28).

## Desktop testing boundary

WebdriverIO desktop testing is a **debug-only, non-focus** lane: opt-in, fixture/local-app
only, no production accounts, no browser profile, no credential access, and no CI requirement.
It may validate a visible consent-state/error surface; it cannot validate LinkedIn acquisition
or act as a source adapter. Primary acceptance evidence is Rust/frontend unit and integration
tests against fakes, contract fixtures, and local vault artifacts.

## Security and privacy requirements

- Data minimization: request and retain only consented categories and scoped artifacts.
- Secret isolation: credentials stay in Rust/keychain; redact tokens, cookies, auth headers,
  device codes, and sensitive URLs from logs, crash reports, fixtures, and provenance displays.
- Local control: show active consents, scope, expiry, imports, and revocation. Revocation stops
  future acquisition immediately and offers local data deletion/export subject to user choice.
- Safe parsing: treat all imported content as untrusted; no execution, remote script loading,
  or HTML rendering without sanitization.
- Auditability: append import decision, consent version, scope decision, and failure reason;
  do not log raw secret material.

## Acceptance tests

1. An import without active matching consent performs zero fetches and records a denial.
2. Exact scope rejects noncanonical variants unless canonicalization resolves to an approved URL.
3. Reference crawl rejects external/over-budget/out-of-depth references before retrieval.
4. Every rendered claim resolves to an immutable artifact plus verified offsets and excerpt hash.
5. LinkedIn capability probe reports approved/unavailable; no browser DOM, cookie, password,
   reaction, or feed access exists in the adapter surface.
6. GitHub `gh` path proves the frontend never receives a token; failure during identity binding
   leaves no usable credential/account state. Device Flow honors server interval and cancellation.
7. GitHub starred import proves pagination, `Accept: application/json`, scope/rate diagnostics,
   and provenance for each saved artifact.
8. WDIO debug lane refuses production endpoints/profiles and is skipped by normal test commands.

## Non-goals

This gate does not authorize implementation, live-account testing, broad web crawling, a
LinkedIn scraper, credential migration, or a release claim.
