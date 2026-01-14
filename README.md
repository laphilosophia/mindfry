# mindfry 🧠🔥

High-performance consciousness-inspired memory graph with lazy decay and auto-association.

## Features

- **Uint8Array Storage** - 25x memory reduction vs object-based storage
- **Lazy Decay** - Energy computed on-demand, no tick() needed
- **Auto-Association** - Priming-based automatic bond creation
- **Decay Lookup Table** - Pre-computed exp() values for fast decay
- **Bond Density Cap** - Controlled graph growth (max 20 bonds/node)
- **Epoch-bounded Cache** - TopK cache invalidated only on state change

## Installation

```bash
npm install mindfry
```

## Quick Start

```typescript
import { createMind } from 'mindfry'

// Create a mind with auto-association enabled
const mind = createMind<{ text: string }>({
  defaultThreshold: 0.3,
  autoAssociate: true
})

// Remember creates memory and auto-bonds to conscious memories
mind.remember('quantum', { text: 'Quantum Computing' })
mind.remember('neural', { text: 'Neural Networks' })
mind.remember('ai', { text: 'Artificial Intelligence' })

// Stimulate activates memory and propagates priming
mind.stimulate('quantum', 0.4)

// Query conscious memories
const conscious = mind.getConscious()
console.log(conscious.map(m => m.content.text))

// Traverse from a starting point
for (const item of mind.recall('quantum', 2)) {
  console.log(`Depth ${item.depth}: ${mind.getAll()[item.memoryIndex].id}`)
}
```

## Core Concepts

### Conscious vs Subconscious

Memories exist in two states:

| State | Condition | Access Cost |
|-------|-----------|-------------|
| **Conscious** | `energy >= threshold` | Free (O(1)) |
| **Subconscious** | `energy < threshold` | Requires `surface()` |

### Lazy Decay

Energy decays over time using the formula:

```
E(t) = E₀ × e^(-λt)
```

But computed **on-demand**, not eagerly. No CPU cost when idle.

### Priming

When memory A is stimulated, connected memories receive fractional activation:

```
B.energy += A.energyDelta × primingDecay × bondStrength
```

## API Reference

### `createMind<T>(config?)`

```typescript
const mind = createMind<ContentType>({
  // Limits
  maxMemories: 65536,        // 64K max
  maxBonds: 262144,          // 256K max

  // Defaults
  defaultThreshold: 0.5,     // Conscious boundary
  defaultDecayRate: 0.001,   // ~12 min half-life

  // Priming
  primingDecay: 0.3,         // Energy decay per hop
  maxPrimingDepth: 3,        // Max propagation depth

  // Auto-association
  autoAssociate: true,       // Enable automatic bonds
  autoAssociateTopK: 5,      // Max bonds per remember
  autoAssociateMinEnergy: 0.3
})
```

### Methods

| Method | Description |
|--------|-------------|
| `remember(id, content, energy?)` | Create memory with auto-association |
| `associate(id, fromId, toId, strength?)` | Manual bond creation |
| `stimulate(id, delta?, prime?)` | Activate memory + priming |
| `get(id)` | Get conscious memory |
| `surface(id)` | Bring subconscious memory to conscious |
| `recall(startId, maxDepth?, maxCost?)` | Traverse graph |
| `getConscious()` | All conscious memories |
| `getSubconscious()` | All subconscious memories |
| `getBonds()` | All bonds |
| `getStats()` | Memory/bond/energy statistics |

## Performance

| Operation | Complexity | Notes |
|-----------|------------|-------|
| remember | O(K) | K = autoAssociateTopK |
| get | O(1) | Direct index lookup |
| stimulate | O(D×B) | D = depth, B = avg bonds |
| getEnergy | O(1) | LUT lookup |
| topKConscious | O(1)* | Cached, invalidated on change |

*After initial computation

## Build

```bash
npm run build    # ESM + CJS + DTS
npm run test     # Run tests
npm run dev      # Watch mode
```

## License

MIT
