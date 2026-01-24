# MindFry

**A Subjective Biological Memory Substrate**

> _"Databases store data. MindFry_ **_feels_** _it."_

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-83%20passing-brightgreen)]()
[![Version](https://img.shields.io/badge/version-1.7.0-blue)]()

---

> ⚠️ **EXPERIMENTAL:** MindFry is currently in active R&D. The API is volatile. It simulates biological memory processes which may result in data inhibition (data loss from the user's perspective) based on the system's "mood". **Do not use for banking.**

---

## What Makes This Different?

Traditional databases are **objective** — they store exactly what you give them, forever (or until you delete it).

MindFry is **subjective** — it processes data through a simulated cognitive layer that can:

- **Forget** data that isn't accessed (organic decay)
- **Suppress** data it finds antagonistic (mood-based inhibition)
- **Strengthen** frequently accessed data (Hebbian learning)
- **Propagate** stimulation through neural bonds (synaptic chains)

## 🧬 Core Principles

### 🧠 Tri-Cortex Architecture

Decisions are made using **Balanced Ternary Logic** (Setun):

- `+1` = True / Excitation
- `0` = Unknown / Neutral
- `-1` = False / Inhibition

The database has a **Personality Octet** (8 dimensions) and a **Mood** that affects data prioritization.

### ❤️ Mood & Personality

> **Note:** Mood affects which data surfaces first, not whether your requests succeed. All data remains accessible — mood just influences the "attention" priority.

Mood modulates the consciousness threshold:

- **High Mood** → More memories feel "close" and accessible
- **Low Mood** → Only the most important memories surface naturally

**Override anytime:** Use `BYPASS_FILTERS` flag for guaranteed access regardless of mood.

### 🕸️ Synaptic Propagation

When you `stimulate("A")`:

```
A (+1.0) → B (+0.5) → C (+0.25) → ... (damped)
```

Touch one memory, its neighbors tremble.

### 💾 Resurrection

Shutdown and restart. The database remembers:

- Its mood
- Its personality
- All lineages and bonds

## Quick Start

### 🐳 Docker (Recommended)

```bash
docker run -d -p 9527:9527 ghcr.io/laphilosophia/mindfry:latest
```

### From Source

```bash
# Clone
git clone https://github.com/laphilosophia/mindfry.git
cd mindfry

# Run server
cargo run --release --bin mindfry-server

# In another terminal
cargo run --bin mfcli -- ping
cargo run --bin mfcli -- create fire 0.9
cargo run --bin mfcli -- stimulate fire 1.0
cargo run --bin mfcli -- stats
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

| Component            | Status     |
| -------------------- | ---------- |
| Core Engine          | ✅ Working |
| SDK (TypeScript)     | ✅ Working |
| Persistence          | ✅ Working |
| Auto-Propagation     | ✅ Working |
| OQL (Query Language) | 🚧 Planned |
| CEREBRO (GUI)        | 🚧 Planned |
| Documentation Site   | 🚧 Planned |

## License

[Apache-2.0](LICENSE) © [Erdem Arslan](https://github.com/laphilosophia)

---

_"If you're not embarrassed by the first version of your product, you've launched too late."_ — Reid Hoffman
