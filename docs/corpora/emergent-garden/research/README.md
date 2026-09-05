# Emergent Garden research index

Campaign `eg-nested-corpus-2026-09`. Current unit: **Wave 4 method review and verified artifact recovery**. Campaign completion remains partial.

## Current entry points

- [Wave 4 execution checkpoint](WAVE-4-EXECUTION-CHECKPOINT.md): completed work, actual recovery and unresolved evidence.
- [Method reviews](WAVE-4-METHODS-REVIEW.md) and [machine-readable review scope](../data/wave-4-source-reviews.json): nine newly reviewed direct papers and deeper Instant-NGP methods.
- [Transfer boundaries](WAVE-4-TRANSFER-BOUNDARIES.md): explicit deductions, counterexamples and proposed controls.
- [Direct-paper coverage](../data/primary-paper-coverage-v4.json): 18 selected-method records across Waves 3–4 and one limited-depth exception among 19 direct arXiv works.
- [Transcript route audit](../data/transcript-route-audit-v4.json): attempts and failures; no complete transcript acquired.
- [Comment aggregate revalidation](../data/comment-revalidation-v4.json): 30,572 records and the persistent count discrepancy, without a fresh raw-text semantic review.

## Preserved evidence

The [official census](2026-09-04-official-census-checkpoint.md) reconciled 74 public uploads. It yielded 62 non-empty descriptions, 528 description edges, 300 normalized target URLs and 93 domains. The [description ledger](../data/youtube-description-edges-v1.json) and [direct frontier](../data/direct-link-frontier-v1.json) preserve source locators and retrieved revisions. URL counts do not equal distinct works or full reviews.

[Wave 3 comments and synthesis](WAVE-3-COMMENTS-AND-SYNTHESIS.md), its [primary-source depth](WAVE-3-PRIMARY-SOURCES.md) and [receipt](../data/wave-3-execution-receipt.json) remain available. They reported 203 creator records reviewed and a chapter-parser correction. The current unit distinguishes inherited findings from re-executed checks and flags unrecovered derived graph/private input rather than silently manufacturing it.

The prior [anchor synthesis](WAVE-1-ANCHOR-CORPUS.md), [chronology](CREATOR-CHRONOLOGY-WAVE-1.md), [project identities](PROJECT-GALLERY-WAVE-1.md), [claim ledger](CLAIM-LEDGER-WAVE-1.md), [ontology](CONCEPT-ONTOLOGY-WAVE-1.md), [source graph](NESTED-SOURCE-GRAPH-WAVE-1.md), and [portfolio mapping](PORTFOLIO-APPLICABILITY-WAVE-1.md) remain historical research. Later corrections qualify their claims; a larger census does not automatically validate them.

## Tools and validation

The [v2 census collector](../../../../scripts/research/collect_emergent_garden_youtube_v2.py) checks public-upload reconciliation. [Comment intake](../../../../scripts/research/collect_emergent_garden_comments.py) accounts for incomplete embedded reply sets. [Offline analysis](../../../../scripts/research/analyze_comment_corpus.py) requires the actual private source inputs; missing input is not success. The [bundle verifier](../../../../scripts/research/verify_research_bundle.py) checks a packaged handoff's manifest, required artifacts and bytes without executing source material.

Forty-six inherited offline tests and twelve new package-integrity tests were run. No live coordination or model-provider benchmark is represented by these tests.

## Current limits

Full transcripts remain unacquired. The one-comment discrepancy, full OMNI-EPIC method reading, exhaustive audience review, broad recursive-source closure and historical code lineage remain open. The live Benchora experiment remains proposed, not executed. ResearchLedger stays canonical; RepoLedger stores projection state; project drafts contain bounded proposals only.

The corpus supports a family resemblance across different kinds of generative and adaptive systems, not a universal architecture or a rule that more agents are better. See the Wave 3 correction record and Wave 4 counterexamples before translating a source into an engineering requirement.

Recorded API-data refresh-or-delete deadline: `2026-10-05T05:55:01Z`. No unattended service or completed release is claimed.
