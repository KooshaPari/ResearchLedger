# Retrieval and enrichment pipeline

ResearchLedger treats an imported item as a durable research seed, not the final answer.
Every seed is normalized, fingerprinted, linked to its source, and eligible for bounded
enrichment.

The Markdown interchange target is [Open Knowledge Format (OKF) v0.1](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md): each concept has parseable frontmatter with a non-empty `type`, portable links, and a `# Citations` section where claims depend on external sources. OKF intentionally does not prescribe a database or vector engine, so those remain implementation details of the local-first vault.

## Deterministic stages

1. Capture the source payload and immutable provenance.
2. Normalize Markdown, canonical URLs, titles, and timestamps.
3. Extract outbound references into `document_links`, deduplicated by canonical URL.
4. Schedule bounded reference fetches with per-domain limits, robots/terms checks, and
   content-size/time budgets.
5. Distill each fetched source into claims, definitions, alternatives, and open questions;
   retain the raw source and cite every distilled claim.
6. Index chunks in lexical FTS5/BM25 and, when enabled, a vector backend.
7. Retrieve with reciprocal-rank fusion across lexical and vector results, then rerank with
   a configured local or remote model.
8. Build answer context with stable citation IDs, source URLs, capture timestamps, and
   confidence/coverage metadata.

## Storage choices

SQLite/Markdown is the default local source of truth. SQLite FTS5 provides deterministic
offline BM25-style retrieval and survives rebuilds from Markdown. A vector adapter should
be selected by corpus size and deployment target:

- SQLite vector extension for a single-user desktop corpus;
- LanceDB for a file-backed local vector index;
- Qdrant for a separately hosted service;
- pgvector only when a server database is already justified.

Dragonfly is an optional hot cache, not canonical storage. NATS is an optional durable
enrichment event bus for multi-process or remote workers. Neo4j is an optional graph query
projection; `document_links` remains sufficient for the local default. MinIO is an optional
object store for large raw captures and media, never a prerequisite for the desktop vault.

Model providers are interfaces, not hardcoded dependencies: local embedding/rerank models
are preferred for privacy, while Gemini or another hosted provider can be explicitly enabled
for higher-quality enrichment. Every model call records provider, model, prompt/version,
input fingerprint, output fingerprint, and citation references.
