# Changelog

All notable changes to this project will be documented in this file.

## [1.2.0] - 2026-01-17

### 🧠 NEW: Tri-Cortex Decision Engine (Setun Module)

Introduces balanced ternary logic primitives for organic decision-making.

### Added

- **`Trit` Primitive**: Balanced ternary digit (`-1 / 0 / +1`)
  - Biological semantics: Inhibition / Latent / Excitation
  - Operations: `consensus()` (AND), `invert()`, `weight()` (as i8)
  - Full conversions: `From<i8>`, `From<bool>`

- **`Octet` Vector**: 8-dimensional personality/event vector
  - Fixed dimensions: Curiosity, Preservation, Efficiency, Empathy, Rigidity, Volatility, Aggression, Latency
  - `resonance(&other)` → f64: Compatibility score (-1.0 to +1.0)
  - `dissonance(&other)` → f64: Conflict intensity (0.0 to 1.0)
  - `pack() / unpack()`: 8-bit binary serialization

- **`Quantizer` Struct**: Analog-to-Trit converter
  - Dynamic thresholds via `mood_modifier` (-1.0 to +1.0)
  - Positive mood → easier True; Negative mood → easier True=harder
  - `set_mood()` for external modulation (future NABU integration)

### Fixed

- Clippy warnings in `psyche.rs`, `decay.rs`, `bond.rs`, `handler.rs`

### Internal

- 15 new unit tests for Setun module
- 53/53 total tests passing

---

## [1.0.0] - 2026-01-17

### 🦀 MAJOR: Rust Pivot

MindFry has been rewritten in Rust as a standalone Cognitive Database Engine.

### Added

- **Core Engine**
  - `PsycheArena`: 32-byte cache-aligned Lineage storage with O(1) access
  - `StrataArena`: Ring buffer Engram history per lineage
  - `BondGraph`: Living bonds with Hebbian learning and decay
  - `DecayEngine`: Pre-computed LUT (256×32) for O(1) decay calculation
- **MFBP Protocol v1.0**: Binary wire protocol (22 OpCodes)
  - Lineage: `CREATE`, `GET`, `STIMULATE`, `FORGET`, `TOUCH`
  - Bond: `CONNECT`, `REINFORCE`, `SEVER`, `NEIGHBORS`
  - Query: `CONSCIOUS`, `TOP_K`, `TRAUMA`, `PATTERN`
  - System: `PING`, `STATS`, `SNAPSHOT`, `RESTORE`, `FREEZE`, `PHYSICS_TUNE`
  - Stream: `SUBSCRIBE`, `UNSUBSCRIBE`
- **Akashic Records**: sled-based persistence layer
  - Snapshot/Restore with bincode serialization
  - Named snapshots for Time Travel
- **TCP Server** (`mindfry-server`): Async Tokio server
- **CLI Client** (`mfcli`): Command-line interface for testing
- **Benchmarks**: Criterion benchmarks for decay and graph operations
- **CEREBRO Design Doc**: GUI visualization specification (commercial - separate repo)

### Changed

- **Technology**: TypeScript → Rust
- **Architecture**: Library → Database Engine
- **Structure**: Flattened repo (no more `mindfry-core/` subfolder)
- **Package**: Renamed `mindfry-core` → `mindfry`

### Removed

- Legacy TypeScript implementation

### Technical Decisions

| Decision    | Choice                   |
| ----------- | ------------------------ |
| Protocol    | Custom TCP Binary (MFBP) |
| Persistence | sled (Rust-native)       |
| Target      | Dual (Server + WASM)     |
| Integration | Network (TCP)            |

---

## [0.3.0] - 2026-01-14 [Legacy TypeScript]

### Added

- **Morpheus** (God of Dreams): Background maintenance layer
  - Event-hinted execution (not clock-driven)
  - Self-managed queue (no external enqueue API)
  - Atomic tasks: prune, transfer, consolidate (stub)
- **AkashicRecords**: Cold storage persistence
  - `inscribe()`: Memory → Archive
  - `retrieve()`: Archive → Memory (with accessScore decay)
  - MemoryAkashicBackend for testing
- Shell layer stats in demo (Morpheus state, Akashic count, Pressure)

### Changed

- **Mind → Psyche**: Main class renamed (backwards compat alias kept)
- Demo updated with mythological branding

## [0.2.1] - 2026-01-14 [Legacy TypeScript]

### Added

- **Decay Lookup Table**: Pre-computed exp() values (32 time buckets × 256 rates)
- **Epoch-bounded TopK Cache**: Only recomputes when state changes
- **Bond Density Cap**: Max 20 bonds per node
- **Performance metrics** in demo (JS Heap, Store Memory, Render Time)

### Changed

- `getEnergy()` now uses LUT instead of `Math.exp()`
- `topKConscious()` cached and invalidated on epoch change
- `createBond()` returns `null` if density cap exceeded

## [0.2.0] - 2026-01-14 [Legacy TypeScript]

### Added

- **Uint8Array Storage**: 4 bytes per memory (25x reduction)
- **Priming-based Auto-Association**: Automatic bonds on `remember()`
- **MemoryStore class**: Low-level typed array backend

### Changed

- Lazy decay model: energy computed on-demand
- `recall()` switched from async to sync iterator
- API: `remember(id, content, energy)` instead of object config

### Removed

- `tick()` method (no longer needed)
- Object-based Memory/Bond primitives

## [0.1.0] - 2026-01-14 [Legacy TypeScript]

### Added

- Initial implementation
- Memory/Bond primitives with object storage
- Mind core with conscious/subconscious layers
- Priming propagation
- Time skip demo
