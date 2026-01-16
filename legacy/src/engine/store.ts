// ═══════════════════════════════════════════════════════════════
// mindfry - Uint8Array Memory Storage
//
// High-performance memory storage using TypedArrays
// 256 levels of precision (Uint8: 0-255 → 0.0-1.0)
// ═══════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────
// CONSTANTS
// ─────────────────────────────────────────────────────────────

export const MAX_MEMORIES = 65536    // 64K memories max
export const MAX_BONDS = 262144      // 256K bonds max
export const SCALE = 255             // Uint8 scale factor

// Memory layout: 4 bytes per node
const BYTES_PER_MEMORY = 4
const MEM_ENERGY = 0
const MEM_THRESHOLD = 1
const MEM_DECAY = 2
const MEM_FLAGS = 3

// Bond layout: 4 bytes per bond (strength, cost, decay, flags)
const BYTES_PER_BOND = 4
const BOND_STRENGTH = 0
const BOND_COST = 1
const BOND_DECAY = 2
const BOND_FLAGS = 3

// Flags
const FLAG_ACTIVE = 1 << 0

// ─────────────────────────────────────────────────────────────
// TYPES
// ─────────────────────────────────────────────────────────────

export interface MemoryHandle {
  readonly index: number
  readonly id: string
}

export interface BondHandle {
  readonly index: number
  readonly id: string
  readonly from: number
  readonly to: number
}

export interface MemoryStoreConfig {
  maxMemories?: number
  maxBonds?: number
  clock?: () => number
}

// ─────────────────────────────────────────────────────────────
// MEMORY STORE
// ─────────────────────────────────────────────────────────────

export class MemoryStore {
  // Numeric data (Uint8 for energy/threshold/decay/flags)
  private readonly memoryData: Uint8Array
  private readonly bondData: Uint8Array

  // Timestamps (Uint32 - relative milliseconds)
  private readonly memoryTimestamps: Uint32Array
  private readonly bondTimestamps: Uint32Array

  // Bond connections (Uint16 indices)
  private readonly bondFrom: Uint16Array
  private readonly bondTo: Uint16Array

  // ID mappings (cold path)
  private readonly memoryIdToIndex: Map<string, number>
  private readonly bondIdToIndex: Map<string, number>
  private readonly memoryIndexToId: string[]
  private readonly bondIndexToId: string[]

  // Adjacency (outbound edges per memory)
  private readonly adjacency: Map<number, Set<number>>

  // Counters
  private memoryCount = 0
  private bondCount = 0
  private readonly baseTime: number

  // Clock
  private readonly clock: () => number

  // ═══════════════════════════════════════════════════════════
  // OPTIMIZATION 1: Decay Lookup Table
  // Pre-computed exp(-rate * elapsed) for common values
  // ═══════════════════════════════════════════════════════════
  private static readonly DECAY_BUCKETS = 32  // Time buckets
  private static readonly RATE_BUCKETS = 256  // Rate values (Uint8)
  private static decayLUT: Float32Array | null = null

  private static initDecayLUT(): Float32Array {
    if (MemoryStore.decayLUT) return MemoryStore.decayLUT

    // Time buckets: 0, 1s, 2s, 5s, 10s, 30s, 1m, 5m, 15m, 1h, 6h, 1d, 1w...
    const timeBuckets = [
      0, 1, 2, 5, 10, 30, 60, 120, 300, 600, 900, 1800, 3600,
      7200, 14400, 21600, 43200, 86400, 172800, 259200, 432000,
      604800, 1209600, 2592000, 5184000, 7776000, 15552000,
      31104000, 62208000, 124416000, 248832000, 500000000
    ]

    const lut = new Float32Array(MemoryStore.DECAY_BUCKETS * MemoryStore.RATE_BUCKETS)

    for (let r = 0; r < MemoryStore.RATE_BUCKETS; r++) {
      const rate = r === 0 ? 0 : Math.pow(10, (r / 255) * 3 - 6)
      for (let t = 0; t < MemoryStore.DECAY_BUCKETS; t++) {
        const elapsed = timeBuckets[t]
        lut[r * MemoryStore.DECAY_BUCKETS + t] = Math.exp(-rate * elapsed)
      }
    }

    MemoryStore.decayLUT = lut
    return lut
  }

  // Time bucket mapping
  private static readonly TIME_THRESHOLDS = [
    0, 1, 2, 5, 10, 30, 60, 120, 300, 600, 900, 1800, 3600,
    7200, 14400, 21600, 43200, 86400, 172800, 259200, 432000,
    604800, 1209600, 2592000, 5184000, 7776000, 15552000,
    31104000, 62208000, 124416000, 248832000, 500000000
  ]

  private getTimeBucket(elapsedSeconds: number): number {
    for (let i = MemoryStore.TIME_THRESHOLDS.length - 1; i >= 0; i--) {
      if (elapsedSeconds >= MemoryStore.TIME_THRESHOLDS[i]) return i
    }
    return 0
  }

  // ═══════════════════════════════════════════════════════════
  // OPTIMIZATION 2: TopK Cache (Epoch-bounded)
  // Only recompute when epoch changes
  // ═══════════════════════════════════════════════════════════
  private topKCache: number[] = []
  private topKCacheEpoch = 0
  private currentEpoch = 0
  private readonly topKCacheSize: number

  // ═══════════════════════════════════════════════════════════
  // OPTIMIZATION 3: Bond Density Cap
  // Max bonds per memory node
  // ═══════════════════════════════════════════════════════════
  private readonly maxBondsPerNode: number

  constructor(config: MemoryStoreConfig = {}) {
    const maxMem = config.maxMemories ?? MAX_MEMORIES
    const maxBond = config.maxBonds ?? MAX_BONDS
    this.clock = config.clock ?? Date.now
    this.baseTime = this.clock()
    this.topKCacheSize = 10
    this.maxBondsPerNode = 20  // Hard cap

    // Initialize decay LUT
    MemoryStore.initDecayLUT()

    // Allocate buffers
    this.memoryData = new Uint8Array(maxMem * BYTES_PER_MEMORY)
    this.bondData = new Uint8Array(maxBond * BYTES_PER_BOND)
    this.memoryTimestamps = new Uint32Array(maxMem)
    this.bondTimestamps = new Uint32Array(maxBond)
    this.bondFrom = new Uint16Array(maxBond)
    this.bondTo = new Uint16Array(maxBond)

    // ID maps
    this.memoryIdToIndex = new Map()
    this.bondIdToIndex = new Map()
    this.memoryIndexToId = []
    this.bondIndexToId = []
    this.adjacency = new Map()
  }

  // ─────────────────────────────────────────────────────────
  // MEMORY OPERATIONS
  // ─────────────────────────────────────────────────────────

  createMemory(
    id: string,
    energy: number = 1.0,
    threshold: number = 0.5,
    decayRate: number = 0.001
  ): MemoryHandle {
    if (this.memoryIdToIndex.has(id)) {
      throw new Error(`Memory "${id}" already exists`)
    }

    const index = this.memoryCount++
    const offset = index * BYTES_PER_MEMORY
    const now = this.relativeTime()

    // Store data
    this.memoryData[offset + MEM_ENERGY] = this.toUint8(energy)
    this.memoryData[offset + MEM_THRESHOLD] = this.toUint8(threshold)
    this.memoryData[offset + MEM_DECAY] = this.encodeDecayRate(decayRate)
    this.memoryData[offset + MEM_FLAGS] = FLAG_ACTIVE
    this.memoryTimestamps[index] = now

    // ID mapping
    this.memoryIdToIndex.set(id, index)
    this.memoryIndexToId[index] = id
    this.adjacency.set(index, new Set())

    // Invalidate topK cache
    this.currentEpoch++

    return { index, id }
  }

  getEnergy(index: number): number {
    const offset = index * BYTES_PER_MEMORY
    const baseEnergy = this.fromUint8(this.memoryData[offset + MEM_ENERGY])
    const rateIndex = this.memoryData[offset + MEM_DECAY]
    const elapsed = (this.relativeTime() - this.memoryTimestamps[index]) / 1000

    // Use LUT for fast decay lookup
    const timeBucket = this.getTimeBucket(elapsed)
    const decayFactor = MemoryStore.decayLUT![rateIndex * MemoryStore.DECAY_BUCKETS + timeBucket]

    return baseEnergy * decayFactor
  }

  getThreshold(index: number): number {
    return this.fromUint8(this.memoryData[index * BYTES_PER_MEMORY + MEM_THRESHOLD])
  }

  isConscious(index: number): boolean {
    return this.getEnergy(index) >= this.getThreshold(index)
  }

  stimulate(index: number, energyDelta: number): void {
    const offset = index * BYTES_PER_MEMORY
    const currentEnergy = this.getEnergy(index)
    const newEnergy = Math.min(1.0, currentEnergy + energyDelta)

    this.memoryData[offset + MEM_ENERGY] = this.toUint8(newEnergy)
    this.memoryTimestamps[index] = this.relativeTime()

    // Invalidate topK cache
    this.currentEpoch++
  }

  // ─────────────────────────────────────────────────────────
  // BOND OPERATIONS
  // ─────────────────────────────────────────────────────────

  createBond(
    id: string,
    fromIndex: number,
    toIndex: number,
    strength: number = 1.0,
    cost: number = 0.1
  ): BondHandle | null {
    if (this.bondIdToIndex.has(id)) {
      throw new Error(`Bond "${id}" already exists`)
    }

    // Bond density cap - reject if either node has too many bonds
    const fromBonds = this.adjacency.get(fromIndex)
    const toBonds = this.adjacency.get(toIndex)
    if ((fromBonds && fromBonds.size >= this.maxBondsPerNode) ||
      (toBonds && toBonds.size >= this.maxBondsPerNode)) {
      return null  // Silently reject, not an error
    }

    const index = this.bondCount++
    const offset = index * BYTES_PER_BOND
    const now = this.relativeTime()

    // Store data
    this.bondData[offset + BOND_STRENGTH] = this.toUint8(strength)
    this.bondData[offset + BOND_COST] = this.toUint8(cost)
    this.bondData[offset + BOND_DECAY] = this.encodeDecayRate(0.0005)
    this.bondData[offset + BOND_FLAGS] = FLAG_ACTIVE
    this.bondTimestamps[index] = now
    this.bondFrom[index] = fromIndex
    this.bondTo[index] = toIndex

    // ID mapping
    this.bondIdToIndex.set(id, index)
    this.bondIndexToId[index] = id

    // Adjacency (bidirectional)
    this.adjacency.get(fromIndex)?.add(index)
    this.adjacency.get(toIndex)?.add(index)

    return { index, id, from: fromIndex, to: toIndex }
  }

  getBondStrength(index: number): number {
    const offset = index * BYTES_PER_BOND
    const baseStrength = this.fromUint8(this.bondData[offset + BOND_STRENGTH])
    const decayRate = this.decodeDecayRate(this.bondData[offset + BOND_DECAY])
    const elapsed = (this.relativeTime() - this.bondTimestamps[index]) / 1000

    return baseStrength * Math.exp(-decayRate * elapsed)
  }

  getBondCost(index: number): number {
    return this.fromUint8(this.bondData[index * BYTES_PER_BOND + BOND_COST])
  }

  getNeighborBonds(memoryIndex: number): Set<number> {
    return this.adjacency.get(memoryIndex) ?? new Set()
  }

  getBondTarget(bondIndex: number, fromIndex: number): number {
    return this.bondFrom[bondIndex] === fromIndex
      ? this.bondTo[bondIndex]
      : this.bondFrom[bondIndex]
  }

  // ─────────────────────────────────────────────────────────
  // QUERY OPERATIONS
  // ─────────────────────────────────────────────────────────

  getMemoryIndex(id: string): number | undefined {
    return this.memoryIdToIndex.get(id)
  }

  getMemoryId(index: number): string {
    return this.memoryIndexToId[index]
  }

  get size(): number {
    return this.memoryCount
  }

  get bondSize(): number {
    return this.bondCount
  }

  // Top K memories by energy (epoch-bounded cache)
  *topKConscious(k: number, minEnergy: number = 0): Generator<number> {
    // Check if cache is valid
    if (this.topKCacheEpoch === this.currentEpoch && this.topKCache.length > 0) {
      for (let i = 0; i < Math.min(k, this.topKCache.length); i++) {
        const idx = this.topKCache[i]
        if (this.getEnergy(idx) >= minEnergy) {
          yield idx
        }
      }
      return
    }

    // Rebuild cache
    const result: { index: number; energy: number }[] = []

    for (let i = 0; i < this.memoryCount; i++) {
      const energy = this.getEnergy(i)
      if (energy < minEnergy || !this.isConscious(i)) continue

      // Insert in sorted position
      let pos = result.length
      while (pos > 0 && result[pos - 1].energy < energy) pos--

      if (pos < this.topKCacheSize) {
        result.splice(pos, 0, { index: i, energy })
        if (result.length > this.topKCacheSize) result.pop()
      }
    }

    // Update cache
    this.topKCache = result.map(r => r.index)
    this.topKCacheEpoch = this.currentEpoch

    for (let i = 0; i < Math.min(k, this.topKCache.length); i++) {
      yield this.topKCache[i]
    }
  }

  // ─────────────────────────────────────────────────────────
  // UTILITIES
  // ─────────────────────────────────────────────────────────

  private relativeTime(): number {
    return this.clock() - this.baseTime
  }

  private toUint8(value: number): number {
    return Math.round(Math.max(0, Math.min(1, value)) * SCALE)
  }

  private fromUint8(value: number): number {
    return value / SCALE
  }

  // Decay rate encoding: 0-255 maps to logarithmic scale
  // 0 = 0 (no decay), 255 = 1.0/sec (very fast)
  private encodeDecayRate(rate: number): number {
    if (rate <= 0) return 0
    // log scale: rate = 10^((value/255) * 3 - 6)
    // inverse: value = (log10(rate) + 6) / 3 * 255
    const log = Math.log10(Math.max(rate, 1e-6))
    return Math.round(((log + 6) / 3) * SCALE)
  }

  private decodeDecayRate(value: number): number {
    if (value === 0) return 0
    return Math.pow(10, (value / SCALE) * 3 - 6)
  }

  // ─────────────────────────────────────────────────────────
  // STATS
  // ─────────────────────────────────────────────────────────

  getStats(): { memories: number; bonds: number; conscious: number; totalEnergy: number } {
    let conscious = 0
    let totalEnergy = 0

    for (let i = 0; i < this.memoryCount; i++) {
      const energy = this.getEnergy(i)
      totalEnergy += energy
      if (this.isConscious(i)) conscious++
    }

    return {
      memories: this.memoryCount,
      bonds: this.bondCount,
      conscious,
      totalEnergy
    }
  }
}
