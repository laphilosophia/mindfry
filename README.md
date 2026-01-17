# MindFry

> **The World's First Ephemeral Graph Database** — A Cognitive DB Engine built with Rust

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tests](https://img.shields.io/badge/tests-38%20passing-brightgreen)]()

## What is MindFry?

MindFry is a **biologically-inspired database** that treats data as living neurons, not static records.

| Feature            | Redis                    | Neo4j           | MindFry                               |
| ------------------ | ------------------------ | --------------- | ------------------------------------- |
| **Data Lifecycle** | TTL (binary: alive/dead) | Permanent       | **DecayRate** (gradient fade)         |
| **Connections**    | None                     | Static RELATION | **Living BOND** (strengthens/weakens) |
| **Memory Model**   | Key-Value                | Graph           | **Ephemeral Graph** with history      |

## Quick Start

```bash
# Run server
cargo run --bin mindfry-server

# In another terminal, test with CLI
cargo run --bin mfcli -- ping
cargo run --bin mfcli -- create fire 0.9
cargo run --bin mfcli -- create ice 0.7
cargo run --bin mfcli -- connect fire ice 0.8
cargo run --bin mfcli -- stats
```

## Core Concepts

- **Lineage**: A memory unit with energy, decay rate, and history
- **Bond**: A living connection that strengthens with use, weakens without
- **Engram**: Historical snapshot within a lineage's memory
- **Psyche Arena**: Hot storage for active lineages (O(1) access)
- **Akashic Records**: Cold persistence layer (sled)

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                          MindFry (Rust)                             │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐      │
│  │   Psyche Arena  │  │   Bond Graph    │  │  Strata Arena   │      │
│  │   (Lineages)    │  │   (Living)      │  │  (Engrams)      │      │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘      │
│           └───────────────────┼─────────────────────┘               │
│                               ▼                                     │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    Decay Engine (Rayon)                     │    │
│  └─────────────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────────────┤
│                        Protocol Layer                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │
│  │ TCP (MFBP)  │  │    WASM     │  │     FFI     │                  │
│  └─────────────┘  └─────────────┘  └─────────────┘                  │
└─────────────────────────────────────────────────────────────────────┘
```

## Project Structure

```
mindfry/
├── src/
│   ├── arena/          # Psyche + Strata arenas
│   ├── graph/          # Living bond graph
│   ├── dynamics/       # Decay engine with LUT
│   ├── protocol/       # MFBP (TCP binary protocol)
│   ├── persistence/    # Akashic Records (sled)
│   └── bin/            # Server + CLI binaries
├── benches/            # Criterion benchmarks
├── docs/               # Design documents
└── Cargo.toml
```

## MFBP Protocol

MindFry Binary Protocol - 22 OpCodes over TCP:

| Category    | Commands                               |
| ----------- | -------------------------------------- |
| **Lineage** | CREATE, GET, STIMULATE, FORGET, TOUCH  |
| **Bond**    | CONNECT, REINFORCE, SEVER, NEIGHBORS   |
| **Query**   | CONSCIOUS, TOP_K, TRAUMA, PATTERN      |
| **System**  | PING, STATS, SNAPSHOT, RESTORE, FREEZE |
| **Stream**  | SUBSCRIBE, UNSUBSCRIBE                 |

## Performance

| Metric                   | Target   |
| ------------------------ | -------- |
| Decay tick (1M lineages) | < 1ms    |
| Bond lookup              | O(1)     |
| Memory per lineage       | 32 bytes |
| Memory per bond          | 24 bytes |

## Roadmap

- [x] **Phase 1**: Core Arenas (Psyche, Strata, Bonds, Decay)
- [x] **Phase 2**: MFBP Protocol (22 OpCodes)
- [x] **Phase 3**: Persistence (Akashic Records)
- [x] **Phase 4**: TCP Server + CLI
- [ ] **Phase 5**: CEREBRO GUI (Commercial - Separate Repo)

## Why Rust?

| Concern        | TypeScript            | Rust                   |
| -------------- | --------------------- | ---------------------- |
| GC Latency     | 10-100ms spikes       | Zero                   |
| Concurrency    | Single-threaded       | Multi-threaded (Rayon) |
| Memory Control | TypedArray workaround | Native arenas          |
| Embeddability  | Node.js only          | WASM, FFI (Python, Go) |

## License

BSL 1.1 © [Erdem Arslan](https://github.com/laphilosophia)
