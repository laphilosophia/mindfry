# 🧠🔥 mindfry

> Consciousness-inspired memory graph with lazy decay, auto-association, and mythological architecture

[![npm version](https://img.shields.io/npm/v/mindfry.svg)](https://www.npmjs.com/package/mindfry)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **Psyche** 🦋 - High-performance consciousness container with Uint8Array storage
- **Morpheus** 💤 - Event-driven background maintenance (God of Dreams)
- **AkashicRecords** 📜 - Cold storage persistence layer (Eternal Memory)
- **Lazy Decay** - Energy computed on-demand, zero idle CPU
- **Auto-Association** - Priming-based automatic bonds
- **25x Memory Reduction** - 4 bytes per memory vs ~100 bytes with objects

## Installation

```bash
npm install mindfry
```

## Quick Start

```typescript
import { createPsyche } from 'mindfry'

// Create a consciousness container
const psyche = createPsyche<{ text: string }>({
  defaultThreshold: 0.3,
  autoAssociate: true
})

// Remember something
psyche.remember('idea-1', { text: 'Hello World' }, 1.0)

// Stimulate with priming propagation
psyche.stimulate('idea-1', 0.3)

// Recall associated memories
for (const item of psyche.recall('idea-1', 3)) {
  console.log(item)
}
```

## Architecture

```
┌─────────────────────────────────────────────────┐
│                    mindfry                       │
├─────────────────────────────────────────────────┤
│  Psyche (Consciousness Container)               │
│  ├─ Conscious (energy > threshold)              │
│  └─ Subconscious (energy < threshold)           │
├─────────────────────────────────────────────────┤
│  Morpheus (Background Maintenance)              │
│  ├─ Prune (dead bonds)                          │
│  └─ Transfer (to AkashicRecords)                │
├─────────────────────────────────────────────────┤
│  AkashicRecords (Cold Storage)                  │
│  ├─ inscribe() → persist                        │
│  └─ retrieve() → reincarnate                    │
└─────────────────────────────────────────────────┘
```

## API Reference

### Psyche

```typescript
const psyche = createPsyche<T>(config?: PsycheConfig)

// Core operations
psyche.remember(id, content, energy?)     // Create memory
psyche.associate(id, fromId, toId, strength?)  // Create bond
psyche.stimulate(id, energyDelta, prime?) // Stimulate memory

// Query
psyche.get(id)              // Get conscious memory
psyche.getAll()             // All memories
psyche.getConscious()       // Conscious only
psyche.getSubconscious()    // Subconscious only
psyche.recall(id, depth?)   // Traverse graph

// Surface
psyche.surface(id)          // Bring to consciousness
```

### Morpheus

```typescript
const morpheus = new Morpheus(deps, config?)

morpheus.notify('pressure-drop')  // Hint maintenance opportunity
morpheus.notify('idle')           // System is idle
morpheus.awaken()                 // Interrupt dreaming
```

### AkashicRecords

```typescript
const akashic = new AkashicRecords(config?)

await akashic.inscribe(id, payload, energy, threshold, lastAccess)
await akashic.retrieve(id)         // Returns AkashicRecord | null
await akashic.exists(id)
await akashic.erase(id)
await akashic.count()
```

## Performance

| Metric | Value |
|--------|-------|
| Memory per node | 4 bytes |
| getEnergy() | O(1) LUT lookup |
| topKConscious() | O(1) cached |
| Idle CPU | 0% |
| Bundle (ESM) | ~25 KB |

## Build

```bash
npm run build    # Build ESM/CJS/DTS
npm run test     # Run tests
npm run dev      # Watch mode
```

## License

MIT © [Erdem Arslan](https://github.com/erdemdev)
