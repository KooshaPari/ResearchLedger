# Wave 6: recovered inputs and an executed coordination reference

Campaign `eg-nested-corpus-2026-09`. Tracking: [AgilePlus #1073](https://github.com/KooshaPari/AgilePlus/issues/1073).

This is a continuation checkpoint, not a declaration that the entire corpus or language-agent experiment is complete. It preserves the execution facts established during the interrupted Wave 6 pass and distinguishes local measurements from hosted verification still requiring a readback.

## Comment evidence recovery

The fresh documented-API capture ran in [33956842110](https://github.com/KooshaPari/ResearchLedger/actions/runs/33956842110) at acquisition source `b6ada57f0211c67a66bfbaf3f345ecb08b76b9c0`. Its encrypted artifact, ID `9966649890`, is 5,885,792 bytes with SHA-256 `17a9ebdec97ee19f8315b0744a23143398997a5acba16ca541abd073916d5527`.

The preceding local analysis recovered 30,573 comment/reply records and reproduced the description-marker counts while rebuilding the comment-linked graph. This is a new capture, not decryption or replay of the earlier Wave 3 source. The one additional record does not by itself prove that the previous one-comment discrepancy was resolved. Exact per-video reconciliation must determine that.

Recovered graph data must be packaged as actual machine-readable payload, not replaced by aggregate counts. Do not claim an exhaustive fresh semantic reading of the entire audience corpus merely because every record was parsed. Original text, audience profiles, encryption private keys and credentials must not enter the public Git projection.

## Third-party transcript-mirror evidence

Two bounded public-page inspection runs succeeded:

- [33957536396](https://github.com/KooshaPari/ResearchLedger/actions/runs/33957536396), artifact `9966845218`, SHA-256 `4d172d1164be2c51c1f8a652752216088e143b1cde0b418aebb087957de5a616`;
- [33957878043](https://github.com/KooshaPari/ResearchLedger/actions/runs/33957878043), artifact `9966954481`, SHA-256 `16406a658bdcf0edf37e57d1c2c89445724a3d7a3dd5e7ab22dda1750dab5b53`.

The preceding analysis found three timestamped mirrors, including a narration-spanning mirror of *The Chaos of AI Agents*. The public page for that source is [here](https://lilys.ai/it/notes/ai-agent-20251128/the-chaos-of-ai-agents). These are unverified third-party text representations, not creator-certified caption tracks. A beginning-to-sign-off span does not certify every word, interval or speaker attribution. The source-page bytes, extraction method, timestamps, associated video identity and omitted spans must remain distinguishable.

A model-written summary, partial preview, metadata response and timestamped transcript mirror are separate evidence types. Do not count all four as acquired exact captions. This checkpoint does not close full-channel transcript coverage, and no raw transcript text is reproduced here.

## Actual state-changing reference experiment

[Benchora #107](https://github.com/KooshaPari/Benchora/pull/107) introduces a separate executable finite-state reference on branch `worktrees/emergent-garden-reference-20260905`, source revision `0802d6925463788e81d58d52f8a6d03c45552909`. It is not an implementation or completion of the language-agent benchmark in [#106](https://github.com/KooshaPari/Benchora/pull/106).

The measured design is six dispatch policies, three arithmetic task classes, six fault conditions, three controls and twenty seeds: 6,480 outer trials. Scripted workers change in-memory state under seeded scheduling. No external models, live games, user worktrees or hardware are involved.

The previous local execution recorded the following results:

| Control | Correct final states | False completion claims |
| --- | ---: | ---: |
| Unchecked | 1,178 / 2,160 | 862 |
| Versioned and idempotent | 2,040 / 2,160 | 0 |
| Versioned with recovery | 2,160 / 2,160 | 0 |

Sixteen deterministic tests passed in the local execution, and no calculation-budget violation was recorded. Hosted-job verification is a separate receipt and is not inferred from the PR description.

A development confound was corrected before these measurements: independent copies now share one fault per outer trial rather than each receiving an injected fault. Their total budget is split among copies. Actual operation/selection costs are recorded; equal ceilings do not establish equal consumption.

## What the results support

Within this deliberately simplified simulator, version-aware and idempotent handling prevents false completion claims produced by injected protocol faults. That alone does not recover a task stranded on a failed worker. An explicit recovery condition closes the remaining failures in the measured task set.

This supports separating correctness checks from progress/recovery mechanisms. It does not establish that the recovery policy is universally safe, that a topology is best for LLMs, or that a real process can restart durably. The worker's arithmetic reasoning is correct by construction except where protocol faults intervene. Voting here is not independent model judgment. The selected faults, tasks and schedules determine the result's scope.

## Portfolio decisions still requiring evidence

Agentora's proposed-versus-observed-versus-accepted distinction is relevant, but adding a universal manager is not justified by these results. Tracera should distinguish rejected stale actions, deduplicated events, unknown outcomes, recovery attempts and verified end state. Benchora should preserve result denominators, consumption, failed trials, injected faults and source versions. Recovery from retained in-memory state must not be mislabeled durable crash recovery.

A production decision still requires actual adapters, model behavior, genuine task difficulty, permission checks, cancellation, durable storage and external-side-effect semantics. Physical rollback and compensation remain different from restoring a simulation snapshot.

## Remaining work

Full-channel transcript coverage, exhaustive audience interpretation, closed recursive bibliography, complete historical behavior reconstruction and the real language-agent benchmark remain open. The supported transcription connector suggested to the operator requires an actual connection before use; its existence is not a completed acquisition. No additional paid provider invocation, product-runtime migration, merge or release is implied.
