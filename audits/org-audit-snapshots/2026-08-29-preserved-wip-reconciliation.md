# Preserved WIP Reconciliation

**Date:** 2026-08-29 UTC  
**Scope:** `KooshaPari/ResearchLedger` local worktrees and `origin` refs  
**Disposition:** preserve all recovery refs; no bulk merge, delete, force-push, or history rewrite.

## Summary

This reconciliation reviewed the non-ancestral ResearchLedger refs before
starting new feature work.  The normal development lines have already passed
through review and merge.  The remaining `wip/preserve-*` refs are Airlock
recovery evidence, not an integration queue.  Most identified functional
surfaces are represented by current `main` code and regression coverage.  Four
unique patches require isolated file-level comparison before any recovery
proposal; this pass proposes no recovery PR.

## Evidence collected

| Check | Result |
| --- | --- |
| `main` | `04837035ad344bb2abd1ff8de7ed86a884067b90`; clean and aligned with `origin/main` at collection time |
| Open pull requests | None |
| Non-ancestral `origin` refs | 85 |
| Ordinary/dependency/CI refs | 13; each maps to an already merged or closed PR |
| Preserved Airlock WIP refs | 72 refs, 64 distinct Git trees |
| WIP commit subjects | 39 explicit snapshot/preserve, 8 governance/docs/dependency, 25 semantic candidates |

The inventory was produced without ref mutation using:

```zsh
git for-each-ref refs/remotes/origin --no-merged=origin/main
git log origin/main --format='%H%x09%s'
gh pr list --repo KooshaPari/ResearchLedger --state all
```

## Ordinary ref disposition

| Area | Evidence | Disposition |
| --- | --- | --- |
| TypeScript native config | PR #69 | Merged |
| Parallel vault test isolation | PRs #70 and #71 | Merged |
| Mergify commit formatting | PR #68 | Merged |
| CodeQL/scorecard repair | PRs #58 and #62 | Merged |
| Dependency updates | PRs #63, #64, #65, and #66 | Merged |
| Earlier Mergify repairs | PRs #47, #51, and #55 | Merged |

## Semantic WIP reconciliation

| Preserved intent | Current-main evidence | Disposition |
| --- | --- | --- |
| TS7/native typecheck | PR #1 | Integrated |
| Reddit and X provider ingest/capture | PR #3 and current `reddit.rs`, `x.rs`, `provider_html.rs` | Integrated |
| Chunking, hybrid retrieval, and reranking | PR #4 and current `chunking.rs`, `rag.rs`, `embeddings.rs` tests | Integrated |
| Resource verification, CSP, and provider UI hooks | current `verify_resources.mjs`, `verify_csp.mjs`, and `App.tsx` | Integrated |
| Release workflow and signing hardening | PR #15, PR #40, and the notarized 0.1.0 macOS artifact | Integrated |
| Provenance/audit model | PR #36 and current `audits/` structure | Integrated |
| Unicode ranking normalization | `rag::tests::reranker_normalizes_unicode_case_and_character_length` | Integrated |
| Embedding compatibility filtering | PR #48 and `storage::tests::vector_search_excludes_incompatible_model_version_and_dimensions` | Integrated |
| Bounded consented reference traversal | PR #49 and `storage` traversal/lease tests | Integrated |
| Mergify modernization | PRs #41, #51, #55, and #68 | Integrated |
| `c019316` embedding and GitHub delta | Current modules exist, but the historical patch is not patch-equivalent | Isolate before recovery decision |
| `021209e` consented reference-queue delta | Later bounded traversal exists, but the historical patch is not patch-equivalent | Isolate before recovery decision |
| `9d01799` A+ integration snapshot | Later release-gate integration covers portions, not the complete patch | Preserve; isolate before recovery decision |
| `3520179` resolved A+ integration snapshot | Later release-gate integration covers portions, not the complete patch | Preserve; isolate before recovery decision |

## Preservation decision

The `origin/wip/preserve-20260822-researchledger/unreachable-*` refs remain
protected evidence.  A snapshot name, a divergent ancestry graph, or an older
file-level diff is not proof of a recoverable feature; importing any such
snapshot wholesale would regress newer merged work and invalidate provenance.

A future recovery proposal must identify one narrow behavioral delta absent
from `main`, include a focused test, and enter review as a fresh PR.  The four
unresolved unique patches above are the only remaining comparison queue; all
other reviewed semantic lines are integrated or governance-only.  Until that
comparison completes, the preserved refs remain untouched.
