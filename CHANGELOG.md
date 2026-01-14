# Changelog

All notable changes to this project will be documented in this file.

## [0.3.0] - 2026-01-14

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

## [0.2.1] - 2026-01-14

### Added
- **Decay Lookup Table**: Pre-computed exp() values (32 time buckets × 256 rates)
- **Epoch-bounded TopK Cache**: Only recomputes when state changes
- **Bond Density Cap**: Max 20 bonds per node
- **Performance metrics** in demo (JS Heap, Store Memory, Render Time)

### Changed
- `getEnergy()` now uses LUT instead of `Math.exp()`
- `topKConscious()` cached and invalidated on epoch change
- `createBond()` returns `null` if density cap exceeded

## [0.2.0] - 2026-01-14

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

## [0.1.0] - 2026-01-14

### Added
- Initial implementation
- Memory/Bond primitives with object storage
- Mind core with conscious/subconscious layers
- Priming propagation
- Time skip demo
