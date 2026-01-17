# MindFry 🧠

[![License: BSL](https://img.shields.io/badge/License-BSL--1.1-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-74%20passing-brightgreen)]()
[![Crates.io](https://img.shields.io/crates/v/mindfry.svg)](https://crates.io/crates/mindfry)
[![Architecture](https://img.shields.io/badge/architecture-Tri--Cortex-purple.svg)](docs/architecture.md)
[![Logic](https://img.shields.io/badge/logic-Balanced%20Ternary-orange.svg)](src/setun.rs)

**MindFry is not a database; it is a substrate for machine memory.**

Unlike traditional databases that store data deterministically (`0` or `1`), MindFry uses a **Tri-Cortex Engine** based on **Balanced Ternary Logic** (`-1`, `0`, `+1`) to simulate organic cognitive processes.

## 🧬 Key Features

- **Tri-Cortex Architecture:** Decision engine mimicking biological inhibition and excitation
- **Ternary Bond Polarity:** Synergy (+1), Neutral (0), Antagonism (-1) relationships
- **SynapseEngine:** Damped signal propagation with ~3 hop blast radius
- **Observer Effect:** Reading data stimulates memory (+0.01 energy)
- **Executive Override:** `QueryFlags` for forensic access (`BYPASS`, `NO_SIDE_EFFECTS`)
- **Consciousness States:** `LUCID`, `DREAMING`, `DORMANT`
- **Organic Decay:** Retention Buffer before final pruning
- **Resurrection Protocol:** State persists across server restarts

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
cargo run --release --bin mindfry-server

# In another terminal
cargo run --bin mfcli -- ping
cargo run --bin mfcli -- create fire 0.9
cargo run --bin mfcli -- create ice 0.7
cargo run --bin mfcli -- connect fire ice 0.8 1    # Synergy bond
cargo run --bin mfcli -- connect fear peace 1.0 -1 # Antagonism bond
cargo run --bin mfcli -- get fire 4                # NO_SIDE_EFFECTS flag
cargo run --bin mfcli -- stats
```

## QueryFlags (Executive Override)

| Flag                | Value | Effect                         |
| ------------------- | ----- | ------------------------------ |
| `BYPASS_FILTERS`    | 0x01  | Skip Cortex/Antagonism filters |
| `INCLUDE_REPRESSED` | 0x02  | Show hidden data               |
| `NO_SIDE_EFFECTS`   | 0x04  | Read without observer effect   |
| `FORENSIC`          | 0x07  | All flags (god mode)           |

## License

BSL 1.1 © [Erdem Arslan](https://github.com/laphilosophia)
