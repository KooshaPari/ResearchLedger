# Emergent Garden Research Index

**Campaign:** `eg-nested-corpus-2026-09`  
**Current completed unit:** Wave 2 recent-window census and high-value direct-link expansion  
**Campaign completion:** incomplete  
**Blocking gate:** `G1_INVENTORY` — ResearchLedger issue #82

## Wave 1 artifacts

1. [`WAVE-1-ANCHOR-CORPUS.md`](WAVE-1-ANCHOR-CORPUS.md) — source-backed synthesis across artificial life, cellular automata, optimization, agent swarms, embodied agents, recursive artifact improvement, and real-game automation.
2. [`CREATOR-CHRONOLOGY-WAVE-1.md`](CREATOR-CHRONOLOGY-WAVE-1.md) — longitudinal map from early evolutionary toys through neural local rules, Mindcraft, coordination experiments, recursive code search, and foreground game automation.
3. [`PROJECT-GALLERY-WAVE-1.md`](PROJECT-GALLERY-WAVE-1.md) — identity and implementation map for the thirteen projects named by the creator gallery, with confirmed, probable, ambiguous, and unresolved states.
4. [`CLAIM-LEDGER-WAVE-1.md`](CLAIM-LEDGER-WAVE-1.md) — 28 claims with evidence class, confidence, competing interpretations, and falsification conditions.
5. [`CONCEPT-ONTOLOGY-WAVE-1.md`](CONCEPT-ONTOLOGY-WAVE-1.md) — 40 normalized concepts, causal relations, anti-conflation rules, and portfolio translations.
6. [`NESTED-SOURCE-GRAPH-WAVE-1.md`](NESTED-SOURCE-GRAPH-WAVE-1.md) — roots, video nodes, 30 creator-owned repositories, expanded implementation lineages, stop decisions, and the Wave 2 frontier.
7. [`PORTFOLIO-APPLICABILITY-WAVE-1.md`](PORTFOLIO-APPLICABILITY-WAVE-1.md) — evidence-gated repository mapping and ten falsifiable portfolio experiments.
8. [`WAVE-1-QUALITY-REPORT.md`](WAVE-1-QUALITY-REPORT.md) — coverage, evidence limitations, gate status, alternative-hypothesis audit, and reproduction status.
9. [`../data/wave-1-inventory.json`](../data/wave-1-inventory.json) — machine-readable Wave 1 inventory and quality state.

## Wave 2 artifacts

1. [`YOUTUBE-CENSUS-WAVE-2.md`](YOUTUBE-CENSUS-WAVE-2.md) — official-source recent-window metadata census, description coverage, outbound-link counts, transcript-route policy, and the explicit G1 blocker.
2. [`DIRECT-LINK-EXPANSION-WAVE-2.md`](DIRECT-LINK-EXPANSION-WAVE-2.md) — revision- and response-pinned expansion of 23 high-value targets discovered in the recent description window.
3. [`WAVE-2-CHECKPOINT.md`](WAVE-2-CHECKPOINT.md) — gate state, completed evidence, new findings, alternatives, blocked work, and fanout boundaries.
4. [`../data/youtube-channel-inventory-v1.json`](../data/youtube-channel-inventory-v1.json) — normalized recent-window video and description metadata.
5. [`../data/youtube-text-coverage-v1.json`](../data/youtube-text-coverage-v1.json) — per-video text and transcript-route coverage matrix.
6. [`../data/youtube-description-edges-v1.json`](../data/youtube-description-edges-v1.json) — normalized outbound links from the returned descriptions.
7. [`../data/direct-link-frontier-v1.json`](../data/direct-link-frontier-v1.json) — machine-readable repository, paper, and creator-surface expansion state.
8. [`../data/youtube-census-status-v1.json`](../data/youtube-census-status-v1.json) — machine-readable G1 state and next action.
9. [`../../../../scripts/research/collect_emergent_garden_youtube.py`](../../../../scripts/research/collect_emergent_garden_youtube.py) — official Data API collector with an explicitly incomplete official Atom fallback.
10. [`../../../../scripts/research/build_emergent_garden_link_frontier.py`](../../../../scripts/research/build_emergent_garden_link_frontier.py) — bounded high-value edge expander.

## Current bounded coverage

### Wave 1 synthesis

- 16 identified high-value video or release nodes;
- 13 analyzed anchor videos;
- 30 public creator-owned GitHub repositories enumerated;
- 13 creator-gallery projects classified;
- 28 falsifiable claims;
- 40 normalized concepts;
- 10 portfolio experiment contracts.

### Wave 2 recent-window evidence

- 15 records returned by the official YouTube Atom channel feed;
- 15 non-empty description records represented by hashes, lengths, excerpts, and normalized links;
- 186 description edges and 116 unique targets across 49 domains;
- 23 unique high-value targets expanded: 11 repositories, 3 primary papers, and 9 creator-controlled pages;
- 23 successful expansions and 0 expansion failures;
- 0 transcript records acquired;
- 1 explicit credential blocker, tracked in [issue #82](https://github.com/KooshaPari/ResearchLedger/issues/82).

These are not complete-channel counts. The Atom feed exposes a recent window and cannot establish the exhaustive uploads boundary.

## Gate state

| Gate                   | State                                                                                                |
| ---------------------- | ---------------------------------------------------------------------------------------------------- |
| `G1_INVENTORY`         | **BLOCKED** — no `YOUTUBE_API_KEY` in repository secrets or the isolated Infisical `dev` environment |
| `G2_TEXT_COVERAGE`     | **PARTIAL** — recent descriptions covered; transcripts unacquired                                    |
| `G3_DIRECT_GRAPH`      | **PARTIAL / RECENT WINDOW EXPANDED** — all discovered high-value targets expanded                    |
| `G4_PROJECT_RELEVANCE` | **PARTIAL** — Wave 1 mappings retained; no broad new fanout clearance                                |
| `G5_EXPERIMENT`        | **SPECIFIED, NOT RUN** — Benchora PR #106                                                            |
| `G6_PROJECT_PR`        | **ONE DOCUMENTATION PILOT OPEN** — no product-code implementation                                    |
| `G7_RELEASE`           | **NOT APPLICABLE**                                                                                   |

## Current verdict

The strongest supported unifier is not “emergence is good.” It is iterative rule-space design:

```text
choose a compact substrate
→ execute it in an environment
→ observe and preserve consequences
→ evaluate useful and harmful behavior
→ change rules, representation, tools, topology, or pressure
→ retain rollback and repeat
```

The chronology and direct graph support continuity across artificial organisms, neural cellular automata, mutable function trees, Minecraft agents, recursive code modification, and Age of Empires tournaments. The common object is an executable search space, not one model architecture.

The direct graph adds three practical distinctions:

1. planner, observation, actuator, environment, and evaluator are separate interfaces;
2. foreground GUI execution and mutable local files are first-class operational constraints;
3. shared-artifact swarms need ownership, merge, rollback, and orphan-prevention controls.

## Strongest correction to the initial thesis

The channel is not uniformly pro-decentralization or pro-open-ended autonomy. Later Mindcraft work adds structured task files, blocked actions, controlled evaluations, safety switches, and action-level collaboration. Directly linked open-ended-search papers also retain learned or benchmark-grounded evaluators, archives, and selection procedures.

The defensible portfolio rule is therefore:

```text
enable bounded exploration
+ preserve evidence and rollback
+ keep global invariants explicit
+ evaluate in the real environment
+ select coordination topology from controlled comparisons
```

## Evidence restrictions

The campaign does not claim:

- that the 15 Atom-feed records are the complete official upload inventory;
- complete transcript, caption, chapter, or description-history coverage;
- reproduction of creator experiments or paper results;
- that every outbound link proves authorship, endorsement, or repository lineage;
- one formal architecture across every project;
- that Patreon posts map one-to-one to public videos;
- authorization for broad project-code changes;
- clearance for further individual repository PR fanout based only on thematic similarity.

## Next admissible transition

`G1_INVENTORY` can advance only after a key restricted to YouTube Data API v3 is stored under the exact name `YOUTUBE_API_KEY` in repository Actions secrets or the repository's Infisical `dev` environment and the census workflow reconciles the complete uploads playlist. Unsupported scraping is not an acceptable substitute.
