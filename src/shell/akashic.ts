// ═══════════════════════════════════════════════════════════════
// mindfry - Akashic Records (Universal Memory Archive)
//
// Cold storage for subconscious memories
// Persist: payload, identity, state hints
// NOT persist: bonds, current energy (computed)
// ═══════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────
// TYPES
// ─────────────────────────────────────────────────────────────

export interface AkashicRecord<T = unknown> {
  id: string
  payload: T

  // State snapshot (hints for reincarnation)
  lastEnergy: number
  lastThreshold: number
  lastAccess: number

  // Reincarnation scoring
  accessCount: number
  accessScore: number  // Decayed importance

  archivedAt: number
}

export interface AkashicBackend<T = unknown> {
  get(id: string): Promise<AkashicRecord<T> | null>
  put(id: string, data: AkashicRecord<T>): Promise<void>
  delete(id: string): Promise<void>
  list(): Promise<string[]>
  count(): Promise<number>
  clear(): Promise<void>
}

// ─────────────────────────────────────────────────────────────
// ACCESS SCORE
// Decayed importance: recent access matters more
// ─────────────────────────────────────────────────────────────

const ACCESS_SCORE_DECAY = 0.0001  // ~2 day half-life

export function computeAccessScore(
  accessCount: number,
  lastAccess: number,
  now: number
): number {
  const ageSeconds = (now - lastAccess) / 1000
  const decayFactor = Math.exp(-ACCESS_SCORE_DECAY * ageSeconds)
  return accessCount * decayFactor
}

// ─────────────────────────────────────────────────────────────
// MEMORY BACKEND (for testing)
// ─────────────────────────────────────────────────────────────

export class MemoryAkashicBackend<T = unknown> implements AkashicBackend<T> {
  private store = new Map<string, AkashicRecord<T>>()

  async get(id: string): Promise<AkashicRecord<T> | null> {
    return this.store.get(id) ?? null
  }

  async put(id: string, data: AkashicRecord<T>): Promise<void> {
    this.store.set(id, data)
  }

  async delete(id: string): Promise<void> {
    this.store.delete(id)
  }

  async list(): Promise<string[]> {
    return [...this.store.keys()]
  }

  async count(): Promise<number> {
    return this.store.size
  }

  async clear(): Promise<void> {
    this.store.clear()
  }
}

// ─────────────────────────────────────────────────────────────
// AKASHIC RECORDS CLASS
// ─────────────────────────────────────────────────────────────

export interface AkashicRecordsConfig {
  backend?: AkashicBackend
  clock?: () => number
}

export class AkashicRecords<T = unknown> {
  private readonly backend: AkashicBackend<T>
  private readonly clock: () => number

  constructor(config: AkashicRecordsConfig = {}) {
    this.backend = (config.backend ?? new MemoryAkashicBackend()) as AkashicBackend<T>
    this.clock = config.clock ?? Date.now
  }

  // ─────────────────────────────────────────────────────────
  // INSCRIBE: Memory → Akashic Records
  // ─────────────────────────────────────────────────────────

  async inscribe(
    id: string,
    payload: T,
    energy: number,
    threshold: number,
    lastAccess: number,
    accessCount: number = 1
  ): Promise<void> {
    const now = this.clock()

    const record: AkashicRecord<T> = {
      id,
      payload,
      lastEnergy: energy,
      lastThreshold: threshold,
      lastAccess,
      accessCount,
      accessScore: computeAccessScore(accessCount, lastAccess, now),
      archivedAt: now
    }

    await this.backend.put(id, record)
  }

  // ─────────────────────────────────────────────────────────
  // RETRIEVE: Akashic Records → Memory
  // ─────────────────────────────────────────────────────────

  async retrieve(id: string): Promise<AkashicRecord<T> | null> {
    const record = await this.backend.get(id)
    if (!record) return null

    // Recalculate accessScore with current time
    const now = this.clock()
    record.accessScore = computeAccessScore(record.accessCount, record.lastAccess, now)

    return record
  }

  async exists(id: string): Promise<boolean> {
    const record = await this.backend.get(id)
    return record !== null
  }

  async erase(id: string): Promise<void> {
    await this.backend.delete(id)
  }

  // ─────────────────────────────────────────────────────────
  // QUERY
  // ─────────────────────────────────────────────────────────

  async count(): Promise<number> {
    return this.backend.count()
  }

  async list(): Promise<string[]> {
    return this.backend.list()
  }

  // Get lowest accessScore records (for pruning)
  async getLowestScoring(n: number): Promise<AkashicRecord<T>[]> {
    const ids = await this.backend.list()
    const records: AkashicRecord<T>[] = []

    for (const id of ids) {
      const record = await this.retrieve(id)
      if (record) records.push(record)
    }

    // Sort by accessScore ascending
    records.sort((a, b) => a.accessScore - b.accessScore)

    return records.slice(0, n)
  }

  async clear(): Promise<void> {
    await this.backend.clear()
  }
}
