# Emergent Garden Research Index

**Campaign:** `eg-nested-corpus-2026-09`

**Current unit:** reconciled official API inventory and bounded direct-source metadata expansion.

**Inventory gate:** `G1_INVENTORY` PASS for public uploads visible at capture.

**Campaign completion:** incomplete; source-text acquisition and recursive research remain partial.

## Start here

Read the [official census checkpoint](2026-09-04-official-census-checkpoint.md) for the current evidence, limitations, corrected counting units, and next research unit. The successful [acquisition run](https://github.com/KooshaPari/ResearchLedger/actions/runs/33948427726) committed its outputs at `6b41757eb9bb85eb1842a41920770758557b3def`.

The missing-credential blocker in issue #82 is resolved by actual API execution, not merely by the operator's installation confirmation. Earlier Atom-window statements describe prior checkpoints.

## Current observed coverage

- 74 public uploads, with two identical ordered enumerations and matching before/after channel counts.
- All 13 reconciliation checks pass; no missing details, duplicate IDs, invalid rows, or wrong-channel records.
- 62 non-empty descriptions; 12 empty descriptions; 58 videos with extracted links.
- 528 video-to-URL edges, 300 normalized target URLs, and 93 domains.
- 54 selected priority target URLs: 22 GitHub, 19 arXiv, and 13 creator-page targets.
- 53 metadata retrieval successes and one failed root-page fetch at `https://evolvecode.io/`.
- Zero transcript texts acquired. Zero chapter records parsed; that parsing result still needs verification.
- 21 offline reconciliation and redaction tests passed locally and in the acquisition workflow.

Target URLs are not interchangeable with distinct intellectual works or repository identities. The 22 GitHub targets contain alias duplicates. The remaining 246 URLs are not automatically irrelevant. Retrieved metadata is not full source review or independent reproduction.

## Current source records

1. [Official census report](YOUTUBE-CENSUS-WAVE-2.md) — current public inventory, description metadata, coarse caption flags, and reconciliation.
2. [Curated direct-link report](DIRECT-LINK-EXPANSION-WAVE-2.md) — actual acquisition scope, URL-level outcomes, known failure, and remaining evidence gaps.
3. [Normalized video inventory](../data/youtube-channel-inventory-v1.json) — video identity, dates, durations, description hashes, and links.
4. [Text-coverage matrix](../data/youtube-text-coverage-v1.json) — per-video metadata and unacquired transcript state.
5. [Description-edge ledger](../data/youtube-description-edges-v1.json) — source videos, original and normalized URLs, and provisional edge classes.
6. [Direct-source frontier](../data/direct-link-frontier-v1.json) — target metadata, revisions or response hashes, and retrieval outcomes.
7. [Census gate status](../data/youtube-census-status-v1.json) — machine-checked G1 state, reconciliation, and counts.
8. [Exact-output manifest](../data/youtube-census-manifest-v2.json) — acquisition revision, run, file hashes, and subsequent narrative correction.
9. [Current checkpoint](2026-09-04-official-census-checkpoint.md) — scope, defects, next work, and publication limits.

## Collectors and tests

The [v2 collector](../../../../scripts/research/collect_emergent_garden_youtube_v2.py) is the credentialed acquisition entry point. Its [21-test suite](../../../../scripts/research/test_emergent_garden_youtube_v2.py) covers count reconciliation, pagination, ownership, missing records, duplicate records, and credential-safe failures.

The [original collector](../../../../scripts/research/collect_emergent_garden_youtube.py) remains for pure parsing and export helpers plus the historical Atom route; its older completeness decision is not used by v2. The [direct-link expander](../../../../scripts/research/build_emergent_garden_link_frontier.py) retrieves bounded metadata. Its inherited narrative generator still needs the current curated scope/counting corrections incorporated before another refresh.

## Preserved Wave 1 research

1. [Anchor synthesis](WAVE-1-ANCHOR-CORPUS.md).
2. [Creator chronology](CREATOR-CHRONOLOGY-WAVE-1.md).
3. [Project-gallery identities](PROJECT-GALLERY-WAVE-1.md).
4. [28-claim ledger](CLAIM-LEDGER-WAVE-1.md).
5. [40-concept ontology](CONCEPT-ONTOLOGY-WAVE-1.md).
6. [Nested source graph](NESTED-SOURCE-GRAPH-WAVE-1.md).
7. [Portfolio applicability and ten experiment contracts](PORTFOLIO-APPLICABILITY-WAVE-1.md).
8. [Wave 1 quality report](WAVE-1-QUALITY-REPORT.md).
9. [Wave 1 machine inventory](../data/wave-1-inventory.json).
10. [Historical Wave 2 checkpoint](WAVE-2-CHECKPOINT.md).

These artifacts preserve prior evidence and interpretations. A larger channel census does not independently revalidate every prior synthesis claim.

## Working hypothesis, not an adopted universal architecture

The earlier synthesis proposed a recurring loop: choose a representation, execute operators in an environment, observe and retain consequences, evaluate, alter the representation or rules, and repeat with recovery available.

Continue testing that interpretation against alternative explanations: genuine technical continuity, a useful metaphor, selective sampling, playful aesthetic exploration, independent prior work, and cases where centralized control is better. Neither open-endedness nor more agents is assumed to improve results.

## Current gates

- G1: PASS for the current public API inventory, not inaccessible history.
- G2: PARTIAL; description metadata acquired, exact transcript text still unacquired.
- G3: PARTIAL; selected metadata expanded, with one failure, unresolved aliases, omitted domains, and unreviewed source contents.
- G4: PARTIAL; portfolio relevance still requires current repository evidence.
- G5: SPECIFIED, NOT RUN; Benchora #106 remains documentation only.
- G6: bounded existing draft PRs; no general product-code clearance.
- G7: no release or merge.

## Next evidence-producing work

Validate chapter extraction and the caption signal, reconcile URL aliases without discarding provenance, review the failed root, and promote omitted first-party or primary-source domains. Then follow older description chains through their actual implementations and papers, retaining locators and contrary evidence.

The recorded metadata refresh-or-delete deadline is `2026-10-05T05:55:01Z`; no unattended refresh service is claimed. ResearchLedger owns the corpus, RepoLedger receives append-only fleet projections, and individual repositories receive only justified derived findings.
