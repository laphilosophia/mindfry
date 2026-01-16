# 🧠🔥 MindFry

> **The World's First Ephemeral Graph Database** — A Cognitive DB Engine built with Rust

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## What is MindFry?

MindFry is a **biologically-inspired database** that treats data as living neurons, not static records.

| Feature            | Redis                    | Neo4j           | MindFry                               |
| ------------------ | ------------------------ | --------------- | ------------------------------------- |
| **Data Lifecycle** | TTL (binary: alive/dead) | Permanent       | **DecayRate** (gradient fade)         |
| **Connections**    | None                     | Static RELATION | **Living BOND** (strengthens/weakens) |
| **Memory Model**   | Key-Value                | Graph           | **Ephemeral Graph** with history      |

## Core Concepts

- **Lineage**: A memory unit with energy, decay rate, and history
- **Bond**: A living connection that strengthens with use, weakens without
- **Engram**: Historical snapshot within a lineage's memory
- **Psyche Arena**: Hot storage for active lineages
- **Akashic Records**: Cold persistence layer

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                          MindFry Core (Rust)                        │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐      │
│  │   Psyche Arena  │  │   Bond Graph    │  │  Strata Arena   │      │
│  │   (Lineages)    │  │   (Living)      │  │  (Engrams)      │      │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘      │
│           └───────────────────┼─────────────────────┘               │
│                               ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    Decay Engine (Rayon)                      │    │
│  │    Background: decay computation, bond pruning               │    │
│  └─────────────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────────────┤
│                        Protocol Layer                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │
│  │ TCP (MFBP)  │  │    WASM     │  │     FFI     │                  │
│  └─────────────┘  └─────────────┘  └─────────────┘                  │
└─────────────────────────────────────────────────────────────────────┘
```

## Quick Start

```bash
cd mindfry-core
cargo build --release
cargo test --lib
```

## Project Structure

```
mindfry/
├── mindfry-core/           # Rust core engine
│   ├── src/
│   │   ├── arena/          # Psyche + Strata arenas
│   │   ├── graph/          # Living bond graph
│   │   ├── dynamics/       # Decay engine with LUT
│   │   ├── protocol/       # MFBP (TCP binary protocol)
│   │   └── persistence/    # Akashic Records (sled)
│   └── benches/            # Criterion benchmarks
├── legacy/                 # Original TypeScript implementation
└── README.md
```

## Performance Goals

| Metric                   | Target   |
| ------------------------ | -------- |
| Decay tick (1M lineages) | < 1ms    |
| Bond lookup              | O(1)     |
| Memory per lineage       | 32 bytes |
| Memory per bond          | 24 bytes |

## Roadmap

- [x] **Phase 1**: Core Arenas (Psyche, Strata, Bonds, Decay)
- [ ] **Phase 2**: MFBP Protocol (TCP binary)
- [ ] **Phase 3**: Persistence (sled integration)
- [ ] **Phase 4**: FFI/WASM bindings
- [ ] **Phase 5**: Production hardening

## Why Rust?

| Concern        | TypeScript            | Rust                   |
| -------------- | --------------------- | ---------------------- |
| GC Latency     | 10-100ms spikes       | Zero                   |
| Concurrency    | Single-threaded       | Multi-threaded (Rayon) |
| Memory Control | TypedArray workaround | Native arenas          |
| Embeddability  | Node.js only          | WASM, FFI (Python, Go) |

## License

MIT © [Erdem Arslan](https://github.com/laphilosophia)
