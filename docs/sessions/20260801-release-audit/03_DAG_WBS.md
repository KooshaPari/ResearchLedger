# DAG and work breakdown

```text
P0 resource parity ------> P1 auth/export ------> P1 provenance ------> P2 enrichment
                                      \                               \
                                       -> installed-app smoke ----------> A+ gate
P2 retrieval hardening -----------------------------------------------> A+ gate
```

- P0/P1: complete and verified.
- Provenance: complete in source; package rebuild/smoke remains after this patch.
- P2 enrichment: pending bounded fetcher and structured distillation.
- P2 retrieval: pending chunking, model-versioned embeddings, reranking.
- A+: blocked by the two P2 lanes and authenticated installed-app evidence.
