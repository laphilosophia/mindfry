# Changelog

All notable changes to this project will be documented in this file.

## [1.5.0] - 2026-01-17

### ⚠️ Breaking Change: Response Framing

`LINEAGE.GET` response now includes status header for unambiguous client parsing.

### Changed

- **Response Format**: `[status:u8] + [payload?]`
  - `0x00` (Found): Full `LineageInfo` payload follows
  - `0x01` (NotFound): No payload, lineage doesn't exist
  - `0x02` (Repressed): No payload, hidden by Antagonism
  - `0x03` (Dormant): No payload, in retention buffer
- **`ResponseData::Lineage`** → **`ResponseData::LineageResult`**
- **`LineageResult`** struct: `{ status: LineageStatus, info: Option<LineageInfo> }`

### Why Breaking?

Previously, NotFound returned `Response::Error`. Now returns `Response::Ok(LineageResult { status: NotFound })`.
This enables richer client-side handling (distinguish "doesn't exist" from "hidden by system").

---

## [1.4.1] - 2026-01-17

### 👑 Executive Override

User authority layer above cognitive filters for reliable data access.

### Added

- **`QueryFlags`**: Bitmask for access control
  - `BYPASS_FILTERS` (0x01): Skip Cortex/Antagonism filters
  - `INCLUDE_REPRESSED` (0x02): Reveal hidden data with REPRESSED status
  - `NO_SIDE_EFFECTS` (0x04): Read without observer effect
  - `FORENSIC` (0x07): All flags combined (god mode)
- **`LineageStatus`**: Rich result states
  - `Found` (0): Normal success
  - `NotFound` (1): Doesn't exist
  - `Repressed` (2): Hidden by Antagonism
  - `Dormant` (3): In retention buffer
- **Observer Effect**: `LINEAGE.GET` now stimulates memory on read (+0.01)

### Protocol

- `LINEAGE.GET` now accepts optional `flags:u8` (backward compat: default 0)

---

## [1.4.0] - 2026-01-17

### 🧠 Synaptic Evolution

Bonds now have polarity for biologically-inspired signal propagation.

### Added

- **`Bond.polarity`**: Ternary bond type (+1/0/-1)
  - `Trit::True` (+1): Synergy (excitatory) - both stimulate
  - `Trit::Unknown` (0): Neutral (insulator) - no propagation
  - `Trit::False` (-1): Antagonism (inhibitory) - suppression
- **`SynapseEngine`**: Damped signal propagation
  - Resistance: 0.5 (50% energy loss per hop)
  - Cutoff: 0.1 (noise floor)
  - Max depth: ~3 hops (mathematically bounded)
  - Loop protection via visited set

### Internal

- 74/74 tests passing (+4 synapse tests)
- Backward compatible: old bonds default to `polarity: Trit::True`

---

## [1.3.0] - 2026-01-17

### 🧬 Cortex Persistence

Nabu's identity (mood, personality) now survives server restarts.

### Added

- **Serde derives**: `Trit`, `Octet`, `Quantizer`, `Cortex`, `RetentionBuffer` now serializable
- **`cortex_data` in Snapshot**: Optional Cortex state preserved in snapshots
- **Mood-Coupled Sensitivity**: Consciousness sensitivity now varies with mood (0.5x-1.5x)
- **`LineageIndexer`**: O(1) key-to-id lookup via sled tree
  - `insert()`, `get()`, `remove()`, `rebuild()` methods
  - Integrated into `AkashicStore` with `indexer()` accessor
- **Resurrection Protocol**: Server boot restores from latest snapshot
  - `MindFry::resurrect()` - restore Cortex + Lineages + Bonds
  - `MindFry::with_store()` - attach persistent storage
  - Graceful degradation on corrupted snapshots

### Changed

- `take_snapshot()` now accepts optional `Cortex` parameter
- Base consciousness sensitivity reduced from 10.0 to 5.0 for mood modulation

### Internal

- 70/70 tests passing
- Backward compatible: old snapshots load with `cortex_data: None`

---

## [1.2.1] - 2026-01-17

### 🔧 Bug Fix: Consciousness Threshold Tuning

Fixed edge case where energy 0.95 with threshold 0.8 was classified as "Dreaming" instead of "Lucid".

### Added

- **`SysMoodSet` OpCode (0x46)**: External mood injection from NABU
  - Payload: `[mood: f32]` (-1.0 to +1.0)
  - Enables real-time emotional state control

### Fixed

- **Consciousness amplification**: Delta now multiplied by 10x for clearer Lucid/Dreaming/Dormant decisions
  - `>3% above threshold` → Lucid (+1)
  - `<3% above threshold` → Dreaming (0)
  - `Below threshold` → Dormant (-1)

### Internal

- 66/66 tests passing
- E2E verified: 0.95 energy → LUCID

---

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

- **`Cortex` Struct**: The decision-making brain
  - Encapsulates system personality (Octet DNA)
  - Mood state with internal shift and external override
  - `consciousness_state()`: Ternary consciousness evaluation (Lucid/Dreaming/Dormant)
  - `decide()`: Mood-modulated analog-to-Trit conversion
  - `evaluate()`: Event resonance calculation
  - Integrated `RetentionBuffer` for safe garbage collection

- **`RetentionBuffer` Struct**: TTL-based data lifecycle management
  - Prevents mass extinction by debouncing deletion decisions
  - `mark_or_tick(id)`: Add to buffer or decrement TTL
  - `restore(id)`: Remove from buffer (data recovered)
  - Configurable TTL (default: 3 ticks)

- **`DecayEngine::process_gc()`**: Cortex-aware garbage collection
  - Ternary viability assessment (Stable/Unstable/Obsolete)
  - Personality-influenced preservation bias
  - Safe deletion via RetentionBuffer TTL

### Fixed

- Clippy warnings in `psyche.rs`, `decay.rs`, `bond.rs`, `handler.rs`

### Internal

- 27 new unit tests (Trit, Octet, Quantizer, Cortex, RetentionBuffer, GC)
- 66/66 total tests passing

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
