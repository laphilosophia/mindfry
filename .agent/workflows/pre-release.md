---
description: Production readiness checklist to run before every version bump
---

# Pre-Release Production Readiness Checklist

Run this checklist before every version bump (`/release`).

## 1. Documentation Check

- [ ] **Behavioral Determinism Contract** exists and is current
  - Epistemic snapshot determinism documented
  - Interactive query variance documented
  - Crash recovery guarantees documented
- [ ] **Failure Taxonomy** documented
  - Hard failures (crash, disk, corruption)
  - Cognitive failures (cascade, saturation)
  - Semantic failures (taboo, identity)
- [ ] **API Discipline Contract** for all public APIs
  - Side-effects documented
  - Epistemic vs interactive classification
  - Warmup behavior documented

## 2. Runtime Guards

- [ ] Warmup enforcement active
  - Warmup state observable (`/health` endpoint)
  - Returns cognitive status, not error
- [ ] Circuit breakers configured
  - Propagation depth limit
  - Energy delta rate limit
  - Memory pressure → read-only mode
  - Snapshot freeze during interaction

## 3. Tests

// turbo

- [ ] Run full test suite: `cargo test`
      // turbo
- [ ] Run benchmarks: `cargo bench`
- [ ] Scenario tests (stimulate → crash → restart → observe)
- [ ] Soak tests (if major release): 24-72h cycle
- [ ] Adversarial tests (extreme bond density, taboo violation)

## 4. Invariant Checks

- [ ] Bond symmetry verified
- [ ] Cluster membership consistent
- [ ] Protection level intact
- [ ] Snapshot restore → graph integrity check

## 5. Profiling (if performance-related changes)

- [ ] Propagation cost curve (bond density → latency)
- [ ] Decay amplification (queries → decay calculations)
- [ ] Snapshot restore time
- [ ] Warmup rejection rate

## 6. Snapshot Format Migration (if format changed)

- [ ] Old format readable by new version
- [ ] Auto-migration on save works
- [ ] Rollback plan documented
- [ ] CHANGELOG has breaking change warning

## 7. Observability (if new ops added)

- [ ] Telemetry / metrics exposed
- [ ] `/health` and `/health/deep` endpoints work
- [ ] Tracing spans for new operations

## 8. Graceful Degradation (if exhaustion logic changed)

- [ ] Exhaustion level → operation matrix documented
  - Normal: all ops
  - Elevated: monitoring
  - Exhausted: reads only
  - Emergency: epistemic only

## 9. Final Checks

// turbo

- [ ] Version bumped in `Cargo.toml`
      // turbo
- [ ] CHANGELOG updated
      // turbo
- [ ] README badges updated (version, test count)
- [ ] Breaking changes documented
- [ ] Migration notes written (if applicable)

---

## Exit Criteria

MindFry is "production ready" for this release if:

1. ✅ Behavior boundaries documented
2. ✅ Failure types classified
3. ✅ Warmup + circuit breakers active
4. ✅ Snapshot restore reliable
5. ✅ Invariant checks pass
6. ✅ All tests green

**None of these require high performance. All require discipline.**
