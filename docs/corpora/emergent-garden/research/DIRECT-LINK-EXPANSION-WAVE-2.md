# Emergent Garden Direct-Link Expansion — Wave 2

**Campaign:** `eg-nested-corpus-2026-09`  
**Generated:** 2026-09-04T23:21:39Z  
**Input ledger SHA-256:** `284497d4d3f265244784db0e830e0536bf605517c453a58cdb7a526386fd15b6`  
**Input inventory complete:** **no**  
**G3 direct-graph scope:** recent-window high-value links only

## Verdict

The official Atom-feed window exposes a useful direct graph even though the complete channel inventory is blocked. This pass resolves unique implementation candidates through the GitHub REST API, primary papers through arXiv, and creator-controlled pages through bounded HTTP captures. It does not upgrade the recent-window sample into a complete channel graph.

## Coverage

| Measure                            | Count |
| ---------------------------------- | ----: |
| Description edges in input ledger  |   186 |
| Unique targets in input ledger     |   116 |
| Unique high-value targets          |    23 |
| Unique implementation repositories |    11 |
| Unique primary papers              |     3 |
| Unique creator-controlled pages    |     9 |
| Expanded successfully              |    23 |
| Expansion failures                 |     0 |

## Direct implementation repositories

| Repository                                                                                    | Source video(s)                                                | Head           | README evidence        | State    |
| --------------------------------------------------------------------------------------------- | -------------------------------------------------------------- | -------------- | ---------------------- | -------- |
| [MaxRobinsonTheGreat/AgentsOfEmpires](https://github.com/MaxRobinsonTheGreat/AgentsOfEmpires) | AI plays Age of Empires II                                     | `b21286bd39a7` | `6020eb6fa21366b9b608` | EXPANDED |
| [MaxRobinsonTheGreat/agent_prompts](https://github.com/MaxRobinsonTheGreat/agent_prompts)     | The Chaos of AI Agents                                         | `f4896e0d5ead` | `—`                    | EXPANDED |
| [MaxRobinsonTheGreat/fractalsearch](https://github.com/MaxRobinsonTheGreat/fractalsearch)     | Recursive Self-Improvement                                     | `68bf34365aab` | `7e7a0dca401eb4b98b67` | EXPANDED |
| [MaxRobinsonTheGreat/hyperdimensions](https://github.com/MaxRobinsonTheGreat/hyperdimensions) | Evolution in Higher Dimensions; Creatures in Higher Dimensions | `f65a12c5d023` | `—`                    | EXPANDED |
| [MaxRobinsonTheGreat/mandelbrotnn](https://github.com/MaxRobinsonTheGreat/mandelbrotnn)       | Recursive Self-Improvement                                     | `b03a9fc50d27` | `15203163a3dca30f2f86` | EXPANDED |
| [MaxRobinsonTheGreat/slopcity](https://github.com/MaxRobinsonTheGreat/slopcity)               | Unleashing AI Slop Swarms                                      | `965c8a42956c` | `d0f74c4d293fc121a453` | EXPANDED |
| [cabaletta/baritone](https://github.com/cabaletta/baritone)                                   | Can AI (actually) beat Minecraft?                              | `64333af99a07` | `dd44d26257187588c5dc` | EXPANDED |
| [karpathy/autoresearch](https://github.com/karpathy/autoresearch)                             | Recursive Self-Improvement                                     | `228791fb499a` | `3958fd4195ac2f98ed35` | EXPANDED |
| [mboop127/AutoDE](https://github.com/mboop127/AutoDE)                                         | AI plays Age of Empires II                                     | `10a7d75b2aba` | `f31f0604c8e249f2d5c2` | EXPANDED |
| [mindcraft-bots/mindcraft](https://github.com/mindcraft-bots/mindcraft)                       | AI for War (in minecraft); Can AI (actually) beat Minecraft?   | `5f3acc87b479` | `cc58f52ba9c1ffa0a20c` | EXPANDED |
| [mindcraft-ce/mindcraft-ce](https://github.com/mindcraft-ce/mindcraft-ce)                     | Can AI (actually) beat Minecraft?                              | `cc9b6a3bc149` | `41c60975583b8ce2f1cc` | EXPANDED |

## Primary papers

| Paper                                                                                                       | Source video(s)            | Version        | Primary category | State    |
| ----------------------------------------------------------------------------------------------------------- | -------------------------- | -------------- | ---------------- | -------- |
| [Instant Neural Graphics Primitives with a Multiresolution Hash Encoding](https://arxiv.org/abs/2201.05989) | Recursive Self-Improvement | `2201.05989v2` | cs.CV            | EXPANDED |
| [Automating the Search for Artificial Life with Foundation Models](https://arxiv.org/abs/2412.17799)        | Artificial Life            | `2412.17799v2` | cs.AI            | EXPANDED |
| [Darwin Godel Machine: Open-Ended Evolution of Self-Improving Agents](https://arxiv.org/abs/2505.22954)     | Recursive Self-Improvement | `2505.22954v3` | cs.AI            | EXPANDED |

## Creator-controlled surfaces

| Surface                                                                   | Source video(s)                                                | HTTP | Response hash          | State    |
| ------------------------------------------------------------------------- | -------------------------------------------------------------- | ---: | ---------------------- | -------- |
| [Aquarium GA](https://evolvecode.io/alife/aquarium.html)                  | Artificial Life                                                |  200 | `f805934e686c8fb88bdf` | EXPANDED |
| [Strange Loops](https://evolvecode.io/alife/evoloop.html)                 | Artificial Life                                                |  200 | `2cbcabf4667ae16b3f36` | EXPANDED |
| [Game of Life](https://evolvecode.io/alife/gol.html)                      | Artificial Life                                                |  200 | `bb19b16195f729b7132c` | EXPANDED |
| [Lenia](https://evolvecode.io/alife/lenia.html)                           | Artificial Life                                                |  200 | `e7c6a16c7d31561fa655` | EXPANDED |
| [Biomorphs](https://evolvecode.io/hyperspace/biomorphs.html)              | Evolution in Higher Dimensions                                 |  200 | `0a269f101e3d032f4f17` | EXPANDED |
| [Hyperdimensional Functions](https://evolvecode.io/hyperspace/index.html) | Evolution in Higher Dimensions; Creatures in Higher Dimensions |  200 | `4a92e34ee47ac414abf1` | EXPANDED |
| [Picbreeder](https://evolvecode.io/hyperspace/picbreeder.html)            | Evolution in Higher Dimensions                                 |  200 | `703a75bea0d634af9dce` | EXPANDED |
| [Langton's Ant](https://evolvecode.io/turmites/index.html)                | Emergent Complexity                                            |  200 | `a638c470465a663f1041` | EXPANDED |
| [neuralpatterns](https://neuralpatterns.io/)                              | Artificial Life                                                |  200 | `1e0c6d176de94929bdfa` | EXPANDED |

## Mechanism-level implications

1. **Foreground execution is a distinct system constraint.** The Age of Empires chain contains both the strategy-generating repository and a screen-capture game runner. A benchmark that tests only strategy text misses resolution, focus, timing, UI-state, and destructive file-write hazards.
2. **Shared-artifact swarms need ownership and merge controls.** The Slopcity surface explicitly gives multiple agents responsibility for a shared hub while also asking them to render, inspect, critique, and revise their own work. That is a useful adversarial case for same-file contention and orphan prevention, not evidence that unbounded parallelism wins.
3. **Embodied planning depends on a separate actuator substrate.** Baritone is a Minecraft pathfinder, while Mindcraft supplies language-agent planning and task structure. Conflating the planner with the actuator hides stale-state, path-execution, and recovery failures.
4. **Open-ended search still requires an evaluator.** The directly linked ASAL and Darwin Gödel Machine papers use learned or benchmark-grounded evaluation rather than treating novelty alone as success.
5. **Fast representations can change the feasible search regime.** The instant neural graphics primitive paper is implementation context for accelerating repeated evaluation; it does not by itself validate recursive self-improvement.

## Competing interpretations

- A linked repository may be historical context, an external dependency, a runner, or the creator's implementation; the graph preserves those possibilities instead of collapsing every GitHub edge into authorship.
- A reachable creator page can identify an artifact while still failing to establish its source repository or revision history.
- README claims and paper abstracts describe intended systems and reported results; they are not independent reproductions.
- Failure to capture a page can reflect anti-bot controls, TLS, redirects, or transient network errors rather than absence.

## Gate transition

This bounded pass advances the recent-window portion of `G3_DIRECT_GRAPH` to a reproducible expanded state. The campaign-level gate remains partial because `G1_INVENTORY` is blocked and older descriptions have not been enumerated. No additional product-repository fanout is authorized by this report alone.
