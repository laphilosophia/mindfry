# MindFry

**A Subjective Biological Memory Substrate**

> _"Databases store data. MindFry_ **_feels_** _it."_

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-90%20passing-brightgreen)]()
[![Version](https://img.shields.io/badge/version-1.9.0-blue)]()
[![crates.io](https://img.shields.io/crates/v/mindfry)](https://crates.io/crates/mindfry)

---

> ⚠️ **EXPERIMENTAL:** MindFry is in active R&D. The API is volatile. It simulates biological memory processes which may result in data inhibition based on the system's "mood". **Do not use for banking.**

---

## What Makes This Different?

Traditional databases are **objective** — they store exactly what you give them.

MindFry is **subjective** — it processes data through a cognitive layer that can:

- **Forget** data that isn't accessed (organic decay)
- **Suppress** data it finds antagonistic (mood-based inhibition)
- **Strengthen** frequently accessed data (Hebbian learning)
- **Propagate** stimulation through neural bonds (synaptic chains)
- **Remember** how it died and adapt accordingly (crash recovery)

## Key Features

### Tri-Cortex Architecture

Decisions use **Balanced Ternary Logic** (Setun):

| Value | Meaning            |
| :---- | :----------------- |
| `+1`  | True / Excitation  |
| `0`   | Unknown / Neutral  |
| `-1`  | False / Inhibition |

### Mood & Personality

The database has a **Personality Octet** (8 dimensions) and a **Mood** that affects data prioritization.

- **High Mood** → More memories feel accessible
- **Low Mood** → Only important memories surface
- **Override:** Use `BYPASS_FILTERS` flag for guaranteed access

### Synaptic Propagation

```
A (+1.0) → B (+0.5) → C (+0.25) → ... (damped)
```

Touch one memory, its neighbors tremble.

### Stability Layer (v1.7+)

Production-grade resilience:

- **Crash Recovery** — Detects shock (unclean shutdown) and coma (prolonged downtime)
- **Warmup Enforcement** — Rejects operations during resurrection (Ping/Stats exempt)
- **Exhaustion Backpressure** — Circuit breaker under high load
- **Graceful Shutdown** — Pre-shutdown snapshot with marker

### Sparse Snapshots (v1.9+)

Portable memory with minimal overhead:

- **Sparse Serialization** — Only non-empty engrams saved
- **zstd Compression** — 10-20x size reduction
- **1.5GB → ~100 bytes** for empty arenas

## Quick Start

### Docker (Recommended)

```bash
docker run -d -p 9527:9527 ghcr.io/cluster-127/mindfry:latest
```

### From Source

```bash
git clone https://github.com/cluster-127/mindfry.git
cd mindfry
cargo run --release --bin mindfry-server

# In another terminal
cargo run --bin mfcli -- ping
cargo run --bin mfcli -- create fire 0.9
cargo run --bin mfcli -- stimulate fire 1.0
cargo run --bin mfcli -- stats
```

### Cargo

```bash
cargo add mindfry
```

## SDK

```bash
npm install mindfry
```

```typescript
import { MindFry } from 'mindfry'

const brain = new MindFry({ host: 'localhost', port: 9527 })
await brain.connect()

// Touch one memory...
await brain.lineage.stimulate({ key: 'trauma', delta: 1.0 })

// ...and its neighbors tremble
const associated = await brain.lineage.get('fear')
console.log(associated.energy) // Increased by propagation
```

## Status

| Component                                        | Status     |
| :----------------------------------------------- | :--------- |
| Core Engine                                      | ✅ Stable  |
| SDK (TypeScript)                                 | ✅ v0.4.0  |
| Persistence (sled)                               | ✅ Stable  |
| Auto-Propagation                                 | ✅ Stable  |
| Stability Layer                                  | ✅ v1.7+   |
| Sparse Snapshots                                 | ✅ v1.9+   |
| [Documentation](https://mindfry-docs.vercel.app) | ✅ Live    |
| OQL (Query Language)                             | 🚧 v2.0    |
| CEREBRO (GUI)                                    | 💭 Planned |

## License

[Apache-2.0](LICENSE) © [Erdem Arslan](https://github.com/cluster-127)

---

_"If you're not embarrassed by the first version of your product, you've launched too late."_ — Reid Hoffman
