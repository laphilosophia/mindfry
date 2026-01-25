# MindFry Roadmap

**Last Updated**: 2026-01-26
**Current Version**: v1.9.0 Stable
**Next Milestone**: v1.10.0 (Production-Ready)

---

> MindFry is not a general-purpose database.
> It is a cognitive memory substrate.
> This roadmap reflects that priority.

---

## Version Strategy

MindFry follows a battle-hardened release path. The "Experimental" status will be removed at v1.10, with API stability guaranteed from v1.15 onwards.

| Version   | Codename     | Focus                                                           |
| :-------- | :----------- | :-------------------------------------------------------------- |
| **v1.10** | Bedrock      | Production-Ready — Stability completion, Cognitive Soak testing |
| **v1.11** | Mnemonic I   | Mnemonic Core Phase 1 — Will system, Temperature tiers          |
| **v1.12** | Mnemonic II  | Mnemonic Core Phase 2 — Protection levels, Cascade refinement   |
| **v1.13** | Mnemonic III | Mnemonic Core Phase 3 — Resonance recall                        |
| **v1.14** | Armor        | Hardening — Refactoring, code coverage, edge case handling      |
| **v1.15** | Mirror       | Polish — Performance optimization, **API Freeze**               |
| **v1.16** | Atlas        | Documentation — Architecture guides, failure mode catalog       |
| **v2.0**  | Cortex       | Organic Query Language (OQL)                                    |

---

## v1.10.0 — Production-Ready (Bedrock)

### Objective

Remove the "Experimental" designation by completing the Stability Layer and passing Cognitive Soak testing.

### Stability Layer Completion

**Completed:**

- [x] Graceful shutdown with connection drain — v1.7.0
- [x] Crash recovery with shutdown marker — v1.8.0
- [x] Warmup enforcement (WarmingUp error code) — v1.8.0
- [x] Exhaustion backpressure — v1.7.0
- [x] Health check endpoints — v1.7.0
- [x] Snapshot export and restore — v1.3.0
- [x] Sparse Snapshot Format v2 with zstd compression — v1.9.0

**Remaining:**

- [ ] Memory corruption protection (arena checksums)
- [ ] Data validation (bond symmetry verification)
- [ ] Point-in-time recovery (named snapshots)
- [ ] Orphan cleanup (dangling bond detection)
- [ ] Anomaly detection fundamentals (drift monitoring)

### Cognitive Soak Testing

Production-readiness for a cognitive substrate requires more than infrastructure stability. The system must demonstrate behavioral consistency over extended operation.

**Cognitive Drift Measurement:**

- [ ] Fresh vs. 30-day instance comparison using identical seeds
- [ ] Epistemic snapshot differential analysis
- [ ] Protection and taboo violation counters

**Recovery Drift Analysis:**

- [ ] Post-crash query distribution analysis (first 100 queries)
- [ ] Latency shape comparison (day 1 vs. day 20)
- [ ] Shock/Coma classification accuracy
- [ ] Warmup duration regression analysis

**Pathological Scenarios:**

- [ ] Burst and decay collision handling
- [ ] Pathological graph growth (extreme bond density)
- [ ] Concurrent interaction during snapshot
- [ ] Weekly stress scenarios

**Exit Criteria:**

| Criterion             | Requirement                                     |
| :-------------------- | :---------------------------------------------- |
| Crash recovery        | Behavioral consistency, not merely zero crashes |
| Cognitive drift       | Less than 5% deviation from fresh instance      |
| Identity preservation | 100% consistency after snapshot/restore cycle   |
| Invariant violations  | Zero protection or taboo violations             |
| Latency shape         | Stable distribution curve                       |
| Soak duration         | 30 days or 7 days with time-acceleration        |

---

## v1.11.0 — Mnemonic Core Phase 1 (Mnemonic I)

### Objective

Introduce the Will system and Temperature tiers as internal behavioral enhancements.

### Will System

The Will system provides antifragile recovery behavior. Unlike systems that model human fragility, MindFry emerges stronger from damage.

| Property         | Description                              |
| :--------------- | :--------------------------------------- |
| Will strength    | Configurable via genome (default: 1.0)   |
| Recovery rate    | Speed of post-damage recovery            |
| Scars as wisdom  | Damage events increase future resistance |
| Stuck prevention | Prevents persistent degraded states      |

**Scope:**

- [ ] Will strength property in genome configuration
- [ ] Will-driven recovery after crash events
- [ ] Wisdom accumulation from damage events
- [ ] Recovery rate tuning

### Temperature Tiers

Internal state classification for memory accessibility:

| Tier   | Characteristics                                      |
| :----- | :--------------------------------------------------- |
| Hot    | Recently accessed, instant retrieval, minimal decay  |
| Warm   | Moderate access, fast retrieval, normal decay        |
| Cold   | Rarely accessed, slow retrieval, accelerated decay   |
| Frozen | Repressed/archived, requires unlock, suspended decay |

**Scope:**

- [ ] Temperature tier assignment logic
- [ ] Access-based tier transitions
- [ ] Tier-aware query behavior

---

## v1.12.0 — Mnemonic Core Phase 2 (Mnemonic II)

### Objective

Implement Protection levels and refine Cascade behavior.

### Protection Levels

| Level    | Decay Resistance | Modification      | Deletion              |
| :------- | :--------------- | :---------------- | :-------------------- |
| Absolute | Immune           | Blocked           | Blocked               |
| Hardened | 95%              | Requires override | Requires override     |
| Pinned   | 80%              | Allowed           | Requires confirmation |
| Normal   | 0%               | Allowed           | Allowed               |

**Scope:**

- [ ] Protection level assignment API
- [ ] Decay resistance enforcement
- [ ] Cascade exemption for protected nodes

### Cascade Mode Refinement

- [ ] Recall cascade (energy boost to related memories)
- [ ] Forget cascade (energy drain for trauma processing)
- [ ] Lock cascade (freeze related memories)
- [ ] Unlock cascade (awaken dormant clusters)

---

## v1.13.0 — Mnemonic Core Phase 3 (Mnemonic III)

### Objective

Implement Resonance recall for mood-aware memory retrieval.

### Resonance Recall

Mood and context influence which memories surface during queries.

**Scope:**

- [ ] Emotional similarity weighting
- [ ] Contextual similarity weighting
- [ ] Temporal proximity weighting
- [ ] Mood-biased recall behavior

---

## v1.14.0 — Hardening (Armor)

### Objective

Improve code quality and edge case coverage.

**Refactoring:**

- [ ] Dead code elimination
- [ ] Module boundary cleanup
- [ ] Error handling consistency
- [ ] Lint compliance (pedantic level)

**Code Coverage:**

- [ ] Unit test coverage exceeding 80%
- [ ] Integration test suite
- [ ] Edge case tests (empty substrate, maximum capacity, corrupt input)

**Error Messages:**

- [ ] Descriptive message for each error code
- [ ] Debug mode with verbose logging

---

## v1.15.0 — Performance and API Freeze (Mirror)

### Objective

Performance optimization and **API Freeze**. Breaking changes after this version require a major version bump.

**Performance:**

- [ ] Memory scaling profiling (1K, 10K, 100K lineages)
- [ ] Latency percentiles under load (P50, P95, P99)
- [ ] Propagation depth optimization

**API Freeze:**

- [ ] Public API audit
- [ ] Deprecation cleanup
- [ ] Stability guarantee documentation

**Retrospective:**

- [ ] Journey review (v1.0 through v1.15)
- [ ] Technical debt inventory

---

## v1.16.0 — Documentation (Atlas)

### Objective

Comprehensive documentation for adopters and operators.

**Architecture Guide:**

- [ ] Core concepts (Lineage, Bond, Decay, Stimulation)
- [ ] Data flow diagrams
- [ ] Single-node design explanation

**Behavior Documentation:**

- [ ] Decay behavior specification
- [ ] Stimulation and propagation patterns
- [ ] Personality and mood filtering

**Operational Documentation:**

- [ ] Failure mode catalog
- [ ] Recovery procedures
- [ ] Deployment patterns

---

## v2.0.0 — Organic Query Language (Cortex)

### Prerequisite

v1.16 must be complete. OQL is built on a battle-hardened foundation.

### OQL Overview

OQL expresses cognitive intentions rather than procedural queries. The canonical format uses declarative intent specifications.

**Design Principle:**

Fluent chaining (`.query().conscious().execute()`) implies procedural execution. OQL expresses **simultaneous intent declarations**, which are ontologically distinct.

**Canonical Format (SCIL):**

```
{
  mode { epistemic }
  intent [conscious, forensic]
  traversal { from "trauma", depth 2 }
  ranking { topK 10 }
}
```

**Scope:**

- [ ] SCIL parser (canonical format)
- [ ] Query Spec to MFBP compiler
- [ ] Validation layer with cognitive axioms
- [ ] Epistemic pipeline (zero side-effects)
- [ ] JSON representation support

**v2.x Optional:**

- [ ] Fluent Builder (convenience wrapper for REPL and testing)

---

## v2.x and Beyond

### Self-Healing Engine (v2.1)

- [ ] Autonomous healing mode
- [ ] Self-repair mechanisms
- [ ] Proactive anomaly response

### Full Mnemonic Core (v3.0)

- [ ] Memory clusters
- [ ] Linked memory with semantic types
- [ ] Inheritance bonds
- [ ] Taboo and Dogma system
- [ ] Core Identity protection

### C127 Protocol Integration (v3.x)

- [ ] Local vs Remote locality semantics
- [ ] Token bucket backpressure
- [ ] SWIM-based peer discovery

---

## Distribution

| Target        | Current      | v1.10      | v2.0        |
| :------------ | :----------- | :--------- | :---------- |
| crates.io     | v1.9.0       | v1.10.0    | v2.0.0      |
| Docker (GHCR) | Available    | Multi-arch | —           |
| NPM           | @mindfry/sdk | Stability  | OQL support |

---

## Success Metrics

| Metric              | Current      | v1.10        | v1.15 | v2.0 |
| :------------------ | :----------- | :----------- | :---- | :--- |
| Unit Tests          | 90           | 100+         | 120+  | 150+ |
| Experimental Status | Yes          | Removed      | —     | —    |
| API Freeze          | No           | No           | Yes   | —    |
| Cognitive Drift     | Not measured | Less than 5% | —     | —    |
| OQL                 | No           | No           | No    | Yes  |

---

## Release History

### v1.9.0 — Sparse Memory Footprint (2026-01-25)

- Sparse Snapshot Format v2: Only non-empty engrams serialized
- zstd compression at level 3
- Footprint reduction from 1.5GB to under 1MB for typical substrates
- SDK consolidation: @mindfry/client and @mindfry/protocol merged into @mindfry/sdk

### v1.8.0 — Trauma-Aware Restart (2026-01-25)

- Crash recovery with Shock/Coma detection at startup
- ShutdownMarker persistence before graceful exit
- Warmup enforcement during resurrection phase

### v1.7.0 — Stability Layer (2026-01-24)

- Stability module: exhaustion, health, shutdown, warmup
- System lineages reserved namespace
- Background snapshot loading
- Graceful shutdown with Ctrl+C handling

### v1.6.x — Launch Era

- v1.6.3: Apache-2.0 license, multi-platform distribution
- v1.6.0 "Genesis": Auto-propagation with LINEAGE.STIMULATE

### v1.5.0 — Executive Override

- Query flags: BYPASS_FILTERS, INCLUDE_REPRESSED, NO_SIDE_EFFECTS, FORENSIC
- Observer effect: +0.01 reinforcement on read

### v1.4.x — Synaptic Evolution

- Ternary polarity: Synergy, Neutral, Antagonism
- SynapseEngine with damped propagation
- 3-hop blast radius limit

### v1.3.0 "Akasha" — Persistence

- Cortex serialization: Mood, DNA, Subconscious Buffer
- Resurrection protocol with identity preservation

### v1.1.x — Neural Gateway

- MFBP v1.0: TCP server with 22 OpCodes
- TypeScript SDK with pipelining support

### v1.0.x — Rust Foundation

- Engine core: 100% Rust, zero-GC, cache-aligned DOD
- Balanced ternary logic implementation

---

Apache-2.0 © Erdem Arslan
