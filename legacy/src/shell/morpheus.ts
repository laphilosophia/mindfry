// ═══════════════════════════════════════════════════════════════
// mindfry - Morpheus (God of Dreams - Maintenance Membrane)
//
// NOT a background worker - event-hinted maintenance
// Queue is SELF-MANAGED - no external enqueue API
// Each task is ATOMIC - completes fully or not at all
// ═══════════════════════════════════════════════════════════════

import type { MemoryStore } from '../engine/store'
import type { AkashicRecords } from './akashic'

// ─────────────────────────────────────────────────────────────
// TYPES
// ─────────────────────────────────────────────────────────────

export type DreamTaskType = 'prune' | 'transfer' | 'consolidate'

interface DreamTask {
  type: DreamTaskType
  target: number  // Memory or bond index
  estimatedMs: number
}

export interface MorpheusConfig {
  dreamThreshold?: number      // systemPressure below this triggers Dream Mode
  pruneThreshold?: number      // Bond strength below this → prune
  transferThreshold?: number   // Memory energy below this → transfer
  maxTasksPerRun?: number      // Max tasks per notify cycle
}

export interface MorpheusDeps<T = unknown> {
  store: MemoryStore
  akashic: AkashicRecords<T>
  getContent: (index: number) => T | undefined
  removeContent: (index: number) => void
  clock: () => number
}

// ─────────────────────────────────────────────────────────────
// MORPHEUS (GOD OF DREAMS)
// ─────────────────────────────────────────────────────────────

export class Morpheus<T = unknown> {
  private readonly store: MemoryStore
  private readonly akashic: AkashicRecords<T>
  private readonly getContent: (index: number) => T | undefined
  private readonly removeContent: (index: number) => void
  private readonly clock: () => number

  private readonly dreamThreshold: number
  private readonly pruneThreshold: number
  private readonly transferThreshold: number
  private readonly maxTasksPerRun: number

  private queue: DreamTask[] = []
  private running = false
  private interrupted = false

  constructor(deps: MorpheusDeps<T>, config: MorpheusConfig = {}) {
    this.store = deps.store
    this.akashic = deps.akashic
    this.getContent = deps.getContent
    this.removeContent = deps.removeContent
    this.clock = deps.clock

    this.dreamThreshold = config.dreamThreshold ?? 0.3
    this.pruneThreshold = config.pruneThreshold ?? 0.01
    this.transferThreshold = config.transferThreshold ?? 0.05
    this.maxTasksPerRun = config.maxTasksPerRun ?? 5
  }

  // ─────────────────────────────────────────────────────────
  // PUBLIC API (minimal - event-triggered only)
  // ─────────────────────────────────────────────────────────

  /**
   * Notify Morpheus of potential maintenance opportunity.
   * Morpheus decides internally whether to enter Dream Mode.
   * NO external enqueue API - queue is self-managed.
   */
  notify(event: 'pressure-drop' | 'idle'): void {
    if (!this.shouldDream()) return
    this.dream()
  }

  /**
   * Interrupt all pending tasks.
   * Current task completes (atomic guarantee).
   */
  awaken(): void {
    this.interrupted = true
    this.queue = []
  }

  // ─────────────────────────────────────────────────────────
  // INTERNAL: Decision Logic
  // ─────────────────────────────────────────────────────────

  private shouldDream(): boolean {
    if (this.running) return false

    const pressure = this.getSystemPressure()
    return pressure < this.dreamThreshold
  }

  private getSystemPressure(): number {
    const stats = this.store.getStats()
    // Pressure = conscious / total (or 0 if empty)
    return stats.memories === 0 ? 0 : stats.conscious / stats.memories
  }

  // ─────────────────────────────────────────────────────────
  // INTERNAL: Task Discovery (self-managed queue)
  // ─────────────────────────────────────────────────────────

  private discoverTasks(): void {
    this.queue = []

    // Discover prune tasks (dead bonds)
    this.discoverPruneTasks()

    // Discover transfer tasks (low energy memories)
    this.discoverTransferTasks()

    // consolidate is stub for now
  }

  private discoverPruneTasks(): void {
    for (let i = 0; i < this.store.bondSize; i++) {
      const strength = this.store.getBondStrength(i)
      if (strength < this.pruneThreshold) {
        this.queue.push({
          type: 'prune',
          target: i,
          estimatedMs: 1
        })
      }
    }
  }

  private discoverTransferTasks(): void {
    for (let i = 0; i < this.store.size; i++) {
      const energy = this.store.getEnergy(i)
      const threshold = this.store.getThreshold(i)

      // Transfer if energy is very low (below transfer threshold * threshold)
      if (energy < this.transferThreshold * threshold) {
        this.queue.push({
          type: 'transfer',
          target: i,
          estimatedMs: 5
        })
      }
    }
  }

  // ─────────────────────────────────────────────────────────
  // INTERNAL: Dream Execution
  // ─────────────────────────────────────────────────────────

  private dream(): void {
    if (this.running) return
    this.running = true
    this.interrupted = false

    // Discover what needs maintenance
    this.discoverTasks()

    // Process up to maxTasksPerRun
    let processed = 0
    while (
      this.queue.length > 0 &&
      processed < this.maxTasksPerRun &&
      !this.interrupted
    ) {
      // Check pressure before each task
      if (this.getSystemPressure() >= this.dreamThreshold) {
        this.awaken()
        break
      }

      const task = this.queue.shift()!
      this.executeTask(task)
      processed++
    }

    this.running = false
  }

  private executeTask(task: DreamTask): void {
    switch (task.type) {
      case 'prune':
        this.executePrune(task.target)
        break
      case 'transfer':
        this.executeTransfer(task.target)
        break
      case 'consolidate':
        // Stub - no-op for now
        break
    }
  }

  private executePrune(_bondIndex: number): void {
    // Mark bond as inactive (soft delete)
    // TODO: Implement bond deactivation in MemoryStore
  }

  private executeTransfer(memoryIndex: number): void {
    const id = this.store.getMemoryId(memoryIndex)
    const content = this.getContent(memoryIndex)

    if (!id || content === undefined) return

    const energy = this.store.getEnergy(memoryIndex)
    const threshold = this.store.getThreshold(memoryIndex)
    const lastAccess = this.clock()

    // Inscribe to Akashic Records (async, fire-and-forget)
    this.akashic.inscribe(id, content, energy, threshold, lastAccess, 1)
      .then(() => {
        // Remove from active memory after inscribed
        this.removeContent(memoryIndex)
        // TODO: Mark memory as archived in MemoryStore
      })
      .catch(() => {
        // Inscription failed, keep in memory
      })
  }

  // ─────────────────────────────────────────────────────────
  // STATS
  // ─────────────────────────────────────────────────────────

  getStats(): { queueLength: number; dreaming: boolean; pressure: number } {
    return {
      queueLength: this.queue.length,
      dreaming: this.running,
      pressure: this.getSystemPressure()
    }
  }
}
