// ═══════════════════════════════════════════════════════════════
// mindfry - Psyche (Soul/Consciousness)
//
// High-performance consciousness container
// Uint8Array storage + Priming-based auto-association
// ═══════════════════════════════════════════════════════════════

import { MemoryStore, type BondHandle, type MemoryHandle, type MemoryStoreConfig } from '../engine/store'

// ─────────────────────────────────────────────────────────────
// TYPES
// ─────────────────────────────────────────────────────────────

export interface PsycheConfig extends MemoryStoreConfig {
  defaultThreshold?: number
  defaultDecayRate?: number
  primingDecay?: number
  maxPrimingDepth?: number
  autoAssociate?: boolean
  autoAssociateTopK?: number
  autoAssociateMinEnergy?: number
}

export interface MemoryView {
  readonly index: number
  readonly id: string
  readonly energy: number
  readonly threshold: number
  readonly isConscious: boolean
  content: unknown
}

export interface BondView {
  readonly index: number
  readonly id: string
  readonly from: string
  readonly to: string
  readonly strength: number
  readonly cost: number
}

export interface RecallItem {
  memoryIndex: number
  depth: number
  cost: number
}

// ─────────────────────────────────────────────────────────────
// PSYCHE CLASS (SOUL/CONSCIOUSNESS)
// ─────────────────────────────────────────────────────────────

export class Psyche<T = unknown> {
  private readonly store: MemoryStore
  private readonly contents: Map<number, T>
  private readonly config: Required<PsycheConfig>

  constructor(config: PsycheConfig = {}) {
    this.config = {
      maxMemories: config.maxMemories ?? 65536,
      maxBonds: config.maxBonds ?? 262144,
      clock: config.clock ?? Date.now,
      defaultThreshold: config.defaultThreshold ?? 0.5,
      defaultDecayRate: config.defaultDecayRate ?? 0.001,
      primingDecay: config.primingDecay ?? 0.3,
      maxPrimingDepth: config.maxPrimingDepth ?? 3,
      autoAssociate: config.autoAssociate ?? true,
      autoAssociateTopK: config.autoAssociateTopK ?? 5,
      autoAssociateMinEnergy: config.autoAssociateMinEnergy ?? 0.3
    }

    this.store = new MemoryStore({
      maxMemories: this.config.maxMemories,
      maxBonds: this.config.maxBonds,
      clock: this.config.clock
    })

    this.contents = new Map()
  }

  // ─────────────────────────────────────────────────────────
  // CORE OPERATIONS
  // ─────────────────────────────────────────────────────────

  remember(id: string, content: T, energy: number = 1.0): MemoryHandle {
    const handle = this.store.createMemory(
      id,
      energy,
      this.config.defaultThreshold,
      this.config.defaultDecayRate
    )

    this.contents.set(handle.index, content)

    // Priming-based auto-association
    if (this.config.autoAssociate) {
      this.autoAssociate(handle)
    }

    return handle
  }

  associate(id: string, fromId: string, toId: string, strength?: number): BondHandle | null {
    const fromIndex = this.store.getMemoryIndex(fromId)
    const toIndex = this.store.getMemoryIndex(toId)

    if (fromIndex === undefined) throw new Error(`Memory "${fromId}" not found`)
    if (toIndex === undefined) throw new Error(`Memory "${toId}" not found`)

    return this.store.createBond(id, fromIndex, toIndex, strength)
  }

  stimulate(id: string, energyDelta: number = 0.3, prime: boolean = true): void {
    const index = this.store.getMemoryIndex(id)
    if (index === undefined) throw new Error(`Memory "${id}" not found`)

    this.store.stimulate(index, energyDelta)

    if (prime) {
      this.propagatePriming(index, energyDelta, 0, new Set([index]))
    }
  }

  // ─────────────────────────────────────────────────────────
  // AUTO-ASSOCIATION (Priming-based)
  // ─────────────────────────────────────────────────────────

  private autoAssociate(handle: MemoryHandle): void {
    let bondCount = 0

    for (const targetIndex of this.store.topKConscious(
      this.config.autoAssociateTopK,
      this.config.autoAssociateMinEnergy
    )) {
      if (targetIndex === handle.index) continue

      const targetEnergy = this.store.getEnergy(targetIndex)

      // Bond strength = target energy
      const strength = targetEnergy

      const bondId = `auto_${handle.id}_${this.store.getMemoryId(targetIndex)}_${bondCount++}`
      this.store.createBond(bondId, handle.index, targetIndex, strength)
    }
  }

  // ─────────────────────────────────────────────────────────
  // PRIMING PROPAGATION
  // ─────────────────────────────────────────────────────────

  private propagatePriming(
    sourceIndex: number,
    energy: number,
    depth: number,
    visited: Set<number>
  ): void {
    if (depth >= this.config.maxPrimingDepth || energy < 0.01) return

    for (const bondIndex of this.store.getNeighborBonds(sourceIndex)) {
      const bondStrength = this.store.getBondStrength(bondIndex)
      if (bondStrength < 0.01) continue

      const targetIndex = this.store.getBondTarget(bondIndex, sourceIndex)
      if (visited.has(targetIndex)) continue

      visited.add(targetIndex)

      const primedEnergy = energy * this.config.primingDecay * bondStrength
      this.store.stimulate(targetIndex, primedEnergy)

      this.propagatePriming(targetIndex, primedEnergy, depth + 1, visited)
    }
  }

  // ─────────────────────────────────────────────────────────
  // QUERY
  // ─────────────────────────────────────────────────────────

  get(id: string): MemoryView | undefined {
    const index = this.store.getMemoryIndex(id)
    if (index === undefined) return undefined
    if (!this.store.isConscious(index)) return undefined

    return this.makeMemoryView(index)
  }

  getAll(): MemoryView[] {
    const result: MemoryView[] = []
    for (let i = 0; i < this.store.size; i++) {
      result.push(this.makeMemoryView(i))
    }
    return result
  }

  getConscious(): MemoryView[] {
    const result: MemoryView[] = []
    for (let i = 0; i < this.store.size; i++) {
      if (this.store.isConscious(i)) {
        result.push(this.makeMemoryView(i))
      }
    }
    return result
  }

  getSubconscious(): MemoryView[] {
    const result: MemoryView[] = []
    for (let i = 0; i < this.store.size; i++) {
      if (!this.store.isConscious(i)) {
        result.push(this.makeMemoryView(i))
      }
    }
    return result
  }

  getBonds(): BondView[] {
    const result: BondView[] = []
    for (let i = 0; i < this.store.bondSize; i++) {
      result.push(this.makeBondView(i))
    }
    return result
  }

  // ─────────────────────────────────────────────────────────
  // TRAVERSAL (Sync iterator)
  // ─────────────────────────────────────────────────────────

  *recall(startId: string, maxDepth: number = Infinity, maxCost: number = Infinity): Generator<RecallItem> {
    const startIndex = this.store.getMemoryIndex(startId)
    if (startIndex === undefined) return

    const visited = new Set<number>()
    const queue: RecallItem[] = [{ memoryIndex: startIndex, depth: 0, cost: 0 }]

    while (queue.length > 0) {
      const current = queue.shift()!
      if (visited.has(current.memoryIndex)) continue
      visited.add(current.memoryIndex)

      if (!this.store.isConscious(current.memoryIndex)) continue

      if (current.depth > 0) {
        yield current
      }

      if (current.depth >= maxDepth) continue

      for (const bondIndex of this.store.getNeighborBonds(current.memoryIndex)) {
        const bondStrength = this.store.getBondStrength(bondIndex)
        if (bondStrength < 0.01) continue

        const targetIndex = this.store.getBondTarget(bondIndex, current.memoryIndex)
        if (visited.has(targetIndex)) continue

        const bondCost = this.store.getBondCost(bondIndex)
        const nextCost = current.cost + bondCost
        if (nextCost > maxCost) continue

        queue.push({
          memoryIndex: targetIndex,
          depth: current.depth + 1,
          cost: nextCost
        })
      }
    }
  }

  // ─────────────────────────────────────────────────────────
  // SURFACE (Subconscious → Conscious)
  // ─────────────────────────────────────────────────────────

  surface(id: string): boolean {
    const index = this.store.getMemoryIndex(id)
    if (index === undefined) return false

    if (this.store.isConscious(index)) return true

    const threshold = this.store.getThreshold(index)
    const energy = this.store.getEnergy(index)
    const needed = threshold - energy + 0.1

    this.store.stimulate(index, needed)
    return true
  }

  // ─────────────────────────────────────────────────────────
  // STATS
  // ─────────────────────────────────────────────────────────

  get size(): number {
    return this.store.size
  }

  getStats() {
    return this.store.getStats()
  }

  // ─────────────────────────────────────────────────────────
  // VIEW HELPERS
  // ─────────────────────────────────────────────────────────

  private makeMemoryView(index: number): MemoryView {
    return {
      index,
      id: this.store.getMemoryId(index),
      energy: this.store.getEnergy(index),
      threshold: this.store.getThreshold(index),
      isConscious: this.store.isConscious(index),
      content: this.contents.get(index)
    }
  }

  private makeBondView(index: number): BondView {
    const fromIndex = (this.store as any).bondFrom[index]
    const toIndex = (this.store as any).bondTo[index]

    return {
      index,
      id: (this.store as any).bondIndexToId[index],
      from: this.store.getMemoryId(fromIndex),
      to: this.store.getMemoryId(toIndex),
      strength: this.store.getBondStrength(index),
      cost: this.store.getBondCost(index)
    }
  }
}

// ─────────────────────────────────────────────────────────────
// FACTORY (backwards compat alias)
// ─────────────────────────────────────────────────────────────

export function createPsyche<T = unknown>(config?: PsycheConfig): Psyche<T> {
  return new Psyche<T>(config)
}

// Backwards compatibility
export { createPsyche as createMind, Psyche as Mind }
export type { PsycheConfig as MindConfig }

