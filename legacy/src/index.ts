// ═══════════════════════════════════════════════════════════════
// mindfry - Consciousness-Inspired Memory Graph
//
// Psyche     - The soul/consciousness container
// Morpheus   - God of dreams (maintenance)
// Akashic    - Universal memory archive
// ═══════════════════════════════════════════════════════════════

// Engine (low-level)
export { MemoryStore } from './engine'
export type { BondHandle, MemoryHandle, MemoryStoreConfig } from './engine'

// Psyche (high-level consciousness)
export { createMind, createPsyche, Mind, Psyche } from './mind'
export type { BondView, MemoryView, MindConfig, PsycheConfig, RecallItem } from './mind'

// Shell (maintenance layer)
export {
  AkashicRecords, computeAccessScore, MemoryAkashicBackend,
  Morpheus
} from './shell'

export type {
  AkashicBackend, AkashicRecord, AkashicRecordsConfig, DreamTaskType, MorpheusConfig,
  MorpheusDeps
} from './shell'

