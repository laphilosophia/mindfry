import { describe, expect, it } from 'vitest'
import { createMind, MemoryStore } from '../src'

describe('MemoryStore (Uint8Array)', () => {
  it('should create memory with correct energy', () => {
    const store = new MemoryStore()
    const handle = store.createMemory('test', 0.8, 0.5, 0.001)

    expect(handle.id).toBe('test')
    expect(store.getEnergy(handle.index)).toBeCloseTo(0.8, 1)
    expect(store.getThreshold(handle.index)).toBeCloseTo(0.5, 1)
    expect(store.isConscious(handle.index)).toBe(true)
  })

  it('should decay energy lazily over time', () => {
    let now = 0
    const store = new MemoryStore({ clock: () => now })

    // Use default decay rate (0.001) which encodes better in Uint8
    const handle = store.createMemory('test', 1.0, 0.3, 0.001)

    expect(store.getEnergy(handle.index)).toBeCloseTo(1.0, 1)

    // Advance 2000 seconds (~33 minutes)
    now = 2000000

    // Energy should have decayed: e^(-0.001 * 2000) ≈ 0.135
    expect(store.getEnergy(handle.index)).toBeLessThan(0.5)
  })

  it('should stimulate and reset decay', () => {
    let now = 0
    const store = new MemoryStore({ clock: () => now })

    const handle = store.createMemory('test', 0.5, 0.3, 0.1)

    now = 5000 // 5 seconds
    store.stimulate(handle.index, 0.3)

    // Energy should be current + delta
    expect(store.getEnergy(handle.index)).toBeGreaterThan(0.5)
  })

  it('should track bonds with adjacency', () => {
    const store = new MemoryStore()

    const a = store.createMemory('a')
    const b = store.createMemory('b')
    const bond = store.createBond('ab', a.index, b.index, 0.8)

    expect(store.getNeighborBonds(a.index).has(bond.index)).toBe(true)
    expect(store.getNeighborBonds(b.index).has(bond.index)).toBe(true)
    expect(store.getBondStrength(bond.index)).toBeCloseTo(0.8, 1)
  })

  it('should return topK conscious memories', () => {
    const store = new MemoryStore()

    store.createMemory('low', 0.3, 0.5)      // subconscious
    store.createMemory('mid', 0.6, 0.5)      // conscious
    store.createMemory('high', 0.9, 0.5)     // conscious
    store.createMemory('max', 1.0, 0.5)      // conscious

    const topK = [...store.topKConscious(2, 0.5)]
    expect(topK.length).toBe(2)

    // Should be max and high (indices 3 and 2)
    expect(store.getMemoryId(topK[0])).toBe('max')
    expect(store.getMemoryId(topK[1])).toBe('high')
  })
})

describe('Mind (v0.2.0)', () => {
  it('should remember and retrieve memories', () => {
    const mind = createMind<{ text: string }>()

    mind.remember('idea', { text: 'Hello' })

    const view = mind.get('idea')
    expect(view).not.toBeUndefined()
    expect(view!.id).toBe('idea')
    expect((view!.content as any).text).toBe('Hello')
  })

  it('should auto-associate on remember', () => {
    const mind = createMind<{ text: string }>({ autoAssociate: true })

    mind.remember('a', { text: 'A' })
    mind.remember('b', { text: 'B' })
    mind.remember('c', { text: 'C' }) // Should auto-bond to a and b

    const bonds = mind.getBonds()
    expect(bonds.length).toBeGreaterThan(0)
  })

  it('should not auto-associate when disabled', () => {
    const mind = createMind<{}>({ autoAssociate: false })

    mind.remember('a', {})
    mind.remember('b', {})

    expect(mind.getBonds().length).toBe(0)
  })

  it('should propagate priming on stimulate', () => {
    const mind = createMind<{}>({ autoAssociate: false, primingDecay: 0.5 })

    const a = mind.remember('a', {}, 0.5)
    const b = mind.remember('b', {}, 0.5)
    mind.associate('ab', 'a', 'b', 0.8)

    const bEnergyBefore = mind.get('b')!.energy

    mind.stimulate('a', 0.4)

    const bEnergyAfter = mind.get('b')!.energy
    expect(bEnergyAfter).toBeGreaterThan(bEnergyBefore)
  })

  it('should traverse via recall (sync)', () => {
    const mind = createMind<{}>({ autoAssociate: false })

    mind.remember('a', {})
    mind.remember('b', {})
    mind.remember('c', {})
    mind.associate('ab', 'a', 'b')
    mind.associate('bc', 'b', 'c')

    const recalled = [...mind.recall('a', 2)]
    expect(recalled.length).toBe(2) // b and c
  })

  it('should surface subconscious memories', () => {
    let now = 0
    const mind = createMind<{}>({
      clock: () => now,
      autoAssociate: false,
      defaultThreshold: 0.3,
      defaultDecayRate: 0.001
    })

    mind.remember('test', {}, 0.35)  // Just above threshold 0.3

    // Decay to subconscious (2000 seconds)
    now = 2000000

    expect(mind.get('test')).toBeUndefined() // subconscious

    mind.surface('test')

    expect(mind.get('test')).not.toBeUndefined() // conscious again
  })

  it('should provide accurate stats', () => {
    const mind = createMind<{}>({ autoAssociate: false })

    mind.remember('a', {}, 0.8)
    mind.remember('b', {}, 0.3) // subconscious (threshold 0.5)

    const stats = mind.getStats()
    expect(stats.memories).toBe(2)
    expect(stats.conscious).toBe(1)
  })
})
