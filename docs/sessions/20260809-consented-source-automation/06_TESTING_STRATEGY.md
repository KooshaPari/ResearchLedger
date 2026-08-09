# Testing Strategy

Test consent denial, expiry/revocation, exact/reference URL scope, redirect boundaries,
provenance span integrity, GitHub atomic binding, Device Flow interval/cancel/error handling,
and pagination/rate diagnostics using fakes and fixtures. Add sanitization/redaction tests for
every source payload and log path.

WDIO desktop coverage is opt-in debug-only and non-focus: local fixtures/app only, never a
production account, browser profile, cookie store, or source-acquisition test. Normal CI does
not depend on it.
