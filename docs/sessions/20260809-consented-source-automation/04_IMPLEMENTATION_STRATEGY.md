# Implementation Strategy

Use a Rust-owned policy service as the single authorization boundary before any acquisition.
Expose only structured, redacted status to the frontend. Store consent receipts, scope decisions,
artifact metadata, and claim-span lineage in the local vault/SQLite model. Keep provider adapters
narrow: LinkedIn has no browser automation surface; GitHub credential access is backend-only.

Do not start implementation from this document alone; first add migrations and tests together in
a dedicated implementation session.
