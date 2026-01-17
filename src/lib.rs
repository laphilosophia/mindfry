//! # MindFry - The World's First Ephemeral Graph Database
//!
//! A Cognitive DB Engine that treats data as living neurons, not static records.
//!
//! ## Core Concepts
//!
//! - **Lineage**: A memory unit with energy, decay, and history
//! - **Bond**: A living connection between lineages that strengthens with use
//! - **Engram**: A historical snapshot within a lineage
//! - **Psyche Arena**: Hot storage for active lineages
//! - **Akashic Records**: Cold persistence layer
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                          MindFry Core                               │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐      │
//! │  │   Psyche Arena  │  │   Bond Graph    │  │  Strata Arena   │      │
//! │  │   (Lineages)    │  │   (CSR/Adj)     │  │  (Engrams)      │      │
//! │  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘      │
//! │           └───────────────────┼─────────────────────┘               │
//! │                               ▼                                      │
//! │  ┌─────────────────────────────────────────────────────────────┐    │
//! │  │                    Decay Engine (Rayon)                      │    │
//! │  └─────────────────────────────────────────────────────────────┘    │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::module_inception)]

pub mod arena;
pub mod dynamics;
pub mod graph;
pub mod setun;

// Feature-gated modules
#[cfg(feature = "server")]
pub mod persistence;
#[cfg(feature = "server")]
pub mod protocol;

pub mod ffi;

// Re-exports
pub use arena::{Engram, Lineage, LineageId, PsycheArena, StrataArena};
pub use dynamics::{DecayConfig, DecayEngine};
pub use graph::{Bond, BondGraph, BondId};
pub use setun::{Octet, Quantizer, Trit};

/// Default maximum lineages in psyche arena
pub const DEFAULT_MAX_LINEAGES: usize = 1 << 20; // 1M lineages

/// Default maximum bonds in graph
pub const DEFAULT_MAX_BONDS: usize = 1 << 22; // 4M bonds

/// Default maximum engrams per lineage (history depth)
pub const DEFAULT_STRATA_DEPTH: usize = 64;

/// MindFry database instance
pub struct MindFry {
    /// Active memory storage
    pub psyche: PsycheArena,
    /// Historical engram storage
    pub strata: StrataArena,
    /// Living bond graph
    pub bonds: BondGraph,
    /// Background decay engine
    pub decay: DecayEngine,
}

impl MindFry {
    /// Create a new MindFry instance with default configuration
    pub fn new() -> Self {
        Self::with_config(MindFryConfig::default())
    }

    /// Create a new MindFry instance with custom configuration
    pub fn with_config(config: MindFryConfig) -> Self {
        let psyche = PsycheArena::with_capacity(config.max_lineages);
        let strata = StrataArena::with_capacity(config.max_lineages, config.strata_depth);
        let bonds = BondGraph::with_capacity(config.max_lineages, config.max_bonds);
        let decay = DecayEngine::new(config.decay);

        Self {
            psyche,
            strata,
            bonds,
            decay,
        }
    }
}

impl Default for MindFry {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for MindFry instance
#[derive(Debug, Clone)]
pub struct MindFryConfig {
    /// Maximum lineages in psyche arena
    pub max_lineages: usize,
    /// Maximum bonds in graph
    pub max_bonds: usize,
    /// History depth per lineage
    pub strata_depth: usize,
    /// Decay engine configuration
    pub decay: DecayConfig,
}

impl Default for MindFryConfig {
    fn default() -> Self {
        Self {
            max_lineages: DEFAULT_MAX_LINEAGES,
            max_bonds: DEFAULT_MAX_BONDS,
            strata_depth: DEFAULT_STRATA_DEPTH,
            decay: DecayConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_mindfry() {
        let db = MindFry::new();
        assert_eq!(db.psyche.len(), 0);
        assert_eq!(db.bonds.len(), 0);
    }
}
