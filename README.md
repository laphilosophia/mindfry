# MindFry 🧠

[![License: BSL](https://img.shields.io/badge/License-BSL--1.1-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-74%20passing-brightgreen)]()
[![Version](https://img.shields.io/badge/version-1.6.0-blue)]()
[![Architecture](https://img.shields.io/badge/architecture-Tri--Cortex-purple.svg)](docs/architecture.md)
[![Logic](https://img.shields.io/badge/logic-Balanced%20Ternary-orange.svg)](src/setun.rs)

**MindFry is not a database; it is a substrate for machine memory.**

Unlike traditional databases that store data deterministically (`0` or `1`), MindFry uses a **Tri-Cortex Engine** based on **Balanced Ternary Logic** (`-1`, `0`, `+1`) to simulate organic cognitive processes.

## 🧬 Key Features

- **Tri-Cortex Architecture:** Decision engine mimicking biological inhibition and excitation
- **Ternary Bond Polarity:** Synergy (+1), Neutral (0), Antagonism (-1) relationships
- **Auto-Propagation:** stimulate(A) cascades through bonds with damped energy
- **SynapseEngine:** ~3 hop blast radius, 50% resistance per hop
- **Observer Effect:** Reading data stimulates memory (+0.01 energy)
- **Executive Override:** `QueryFlags` for forensic access
- **Consciousness States:** `LUCID`, `DREAMING`, `DORMANT`
- **Resurrection Protocol:** State persists across server restarts

## 📦 Installation

```bash
# Core Server (Rust)
cargo install mindfry-server

# Client SDK (TypeScript)
npm install mindfry
```

## Quick Start

```bash
# Run server
cargo run --release --bin mindfry-server

# In another terminal
cargo run --bin mfcli -- ping
cargo run --bin mfcli -- create A 0.1
cargo run --bin mfcli -- create B 0.1
cargo run --bin mfcli -- connect A B 1.0 1     # Synergy bond
cargo run --bin mfcli -- stimulate A 1.0       # Auto-propagates to B!
cargo run --bin mfcli -- get B 4               # Check B's energy
```

## Control Flags

### QueryFlags (GET)

| Flag                | Value | Effect                         |
| ------------------- | ----- | ------------------------------ |
| `BYPASS_FILTERS`    | 0x01  | Skip Cortex/Antagonism filters |
| `INCLUDE_REPRESSED` | 0x02  | Show hidden data               |
| `NO_SIDE_EFFECTS`   | 0x04  | Read without observer effect   |
| `FORENSIC`          | 0x07  | All flags (god mode)           |

### StimulateFlags (STIMULATE)

| Flag           | Value | Effect                       |
| -------------- | ----- | ---------------------------- |
| `NO_PROPAGATE` | 0x01  | Surgical mode - no cascading |

## License

BSL 1.1 © [Erdem Arslan](https://github.com/laphilosophia)
