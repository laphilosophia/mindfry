# Changelog

All notable changes to this project will be documented in this file.

## [1.0.0] - 2026-01-17

### 🦀 MAJOR: Rust Pivot

MindFry has been rewritten in Rust as a standalone Cognitive Database Engine.

### Added

- **mindfry-core**: Complete Rust implementation
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
  - Snapshot/Restore: Full arena dumps with bincode serialization
  - Metadata indexing for fast listing
  - Named snapshots for Time Travel
- **Benchmarks**: Criterion benchmarks for decay and graph operations
- **Server binary**: TCP server entry point (placeholder for MFBP)
- **CEREBRO Design Doc**: GUI visualization specification

### Changed

- **Technology**: TypeScript → Rust
- **Architecture**: Library → Database Engine
- **Positioning**: "Memory Graph" → "Ephemeral Graph Database"

### Moved

- Original TypeScript implementation moved to `legacy/` directory

### Technical Decisions (Cluster 127)

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
