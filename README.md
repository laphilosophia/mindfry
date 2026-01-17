# MindFry 🧠

[![License: BSL](https://img.shields.io/badge/License-BSL-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-38%20passing-brightgreen)]()
[![Crates.io](https://img.shields.io/crates/v/mindfry-core.svg)](https://crates.io/crates/mindfry-core)
[![Architecture](https://img.shields.io/badge/architecture-Tri--Cortex-purple.svg)](docs/architecture.md)
[![Logic](https://img.shields.io/badge/logic-Balanced%20Ternary-orange.svg)](src/setun.rs)

**MindFry is not a database; it is a substrate for machine memory.**

Unlike traditional databases that store data deterministically (`0` or `1`), MindFry uses a **Tri-Cortex Engine** based on **Balanced Ternary Logic** (`-1`, `0`, `+1`) to simulate organic cognitive processes. It doesn't just store information; it develops a "feeling" about it based on internal moods and external stimuli.

## 🧬 Key Features

- **Tri-Cortex Architecture:** A decision engine that mimics biological inhibition and excitation.
- **Subjective Reality:** Data availability is filtered through the system's current "Mood" (Depressive, Euphoric, Rigid).
- **Organic Decay:** Unused memories don't just vanish; they enter a `Purgatory` state (Retention Buffer) before final pruning.
- **Consciousness States:** Queries return data based on its energy state: `LUCID` (Active), `DREAMING` (Latent), or `DORMANT` (Subconscious).
- **High Performance:** Written in **Rust** for bare-metal efficiency, bypassing standard FPU overhead for integer-based Ternary operations.

## 📦 Installation

```bash
# Core Server (Rust)
cargo install mindfry-server

# Client SDK (TypeScript)
npm install @mindfry/client
```

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

## License

BSL 1.1 © [Erdem Arslan](https://github.com/laphilosophia)
