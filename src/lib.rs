//! # MindFry - Memory with a Conscience
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
//! │           └──────────────────-─┼────────────────────┘               │
//! │                                ▼                                    │
//! │  ┌─────────────────────────────────────────────────────────────┐    │
//! │  │                    Decay Engine (Rayon)                     │    │
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
#[cfg(feature = "server")]
pub mod stability;

pub mod ffi;

// Re-exports
pub use arena::{Engram, Lineage, LineageId, PsycheArena, StrataArena};
pub use dynamics::{DecayConfig, DecayEngine};
pub use graph::{Bond, BondGraph, BondId};
pub use setun::{Cortex, Octet, Quantizer, RetentionBuffer, Trit};

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
    /// Decision-making brain (Setun ternary logic)
    pub cortex: Cortex,
    /// Signal propagation engine
    pub synapse: dynamics::SynapseEngine,
    /// Persistent storage (optional)
    #[cfg(feature = "server")]
    pub store: Option<std::sync::Arc<persistence::AkashicStore>>,
}

impl MindFry {
    /// Create a new MindFry instance with default configuration
    pub fn new() -> Self {
        Self::with_config(MindFryConfig::default())
    }

    /// Create a new MindFry instance with custom configuration
    pub fn with_config(config: MindFryConfig) -> Self {
        use setun::{dimension, Octet, Trit};

        let psyche = PsycheArena::with_capacity(config.max_lineages);
        let strata = StrataArena::with_capacity(config.max_lineages, config.strata_depth);
        let bonds = BondGraph::with_capacity(config.max_lineages, config.max_bonds);
        let decay = DecayEngine::new(config.decay);

        // Default personality: Curious and Preserving
        let mut personality = Octet::neutral();
        personality.set(dimension::CURIOSITY, Trit::True);
        personality.set(dimension::PRESERVATION, Trit::True);
        let cortex = Cortex::new(personality);

        Self {
            psyche,
            strata,
            bonds,
            decay,
            cortex,
            synapse: dynamics::SynapseEngine::new(),
            #[cfg(feature = "server")]
            store: None,
        }
    }

    /// Attach persistent storage to this MindFry instance
    #[cfg(feature = "server")]
    pub fn with_store(mut self, store: std::sync::Arc<persistence::AkashicStore>) -> Self {
        self.store = Some(store);
        self
    }

    /// Attempt to resurrect from the latest snapshot
    ///
    /// Returns Ok(true) if resurrection succeeded, Ok(false) if no snapshot found.
    /// On corruption, logs error and returns Ok(false) for graceful degradation.
    #[cfg(feature = "server")]
    pub fn resurrect(&mut self) -> Result<bool, persistence::AkashicError> {
        use std::time::Instant;

        let store = match &self.store {
            Some(s) => s,
            None => return Ok(false),
        };

        // Check for latest snapshot
        let t0 = Instant::now();
        let snapshot = match store.latest_snapshot()? {
            Some(s) => s,
            None => return Ok(false),
        };
        tracing::debug!("Snapshot loaded in {:?}", t0.elapsed());

        tracing::info!(
            "Restoring '{}' ({} lineages, {} KB)",
            snapshot.meta.name.as_deref().unwrap_or("unnamed"),
            snapshot.meta.lineage_count,
            snapshot.meta.size_bytes / 1024
        );

        // Restore arenas
        let t1 = Instant::now();
        let (psyche, strata, bonds, _physics) = store.restore_snapshot(
            &snapshot,
            self.psyche.capacity(),
            self.bonds.capacity(),
            64, // strata depth
        )?;
        tracing::debug!("Arenas restored in {:?}", t1.elapsed());

        self.psyche = psyche;
        self.strata = strata;
        self.bonds = bonds;

        // Restore Cortex if available
        if let Some(ref cortex_data) = snapshot.cortex_data {
            match bincode::deserialize::<Cortex>(cortex_data) {
                Ok(restored_cortex) => {
                    tracing::info!("🧠 Cortex restored (mood: {:.2})", restored_cortex.mood());
                    self.cortex = restored_cortex;
                }
                Err(e) => {
                    tracing::warn!("⚠️ Failed to restore Cortex: {}, using default", e);
                }
            }
        }

        // Note: Index rebuild deferred - lineage keys stored in protocol layer
        // For now, index will be rebuilt on-demand through handler interactions
        tracing::info!(
            "📇 Index rebuild deferred (lineages: {})",
            self.psyche.len()
        );

        tracing::info!(
            "✅ Resurrection complete: {} lineages, {} bonds",
            self.psyche.len(),
            self.bonds.len()
        );

        Ok(true)
    }

    /// Sync a newly created lineage to the index
    #[cfg(feature = "server")]
    pub fn sync_index_insert(&self, key: &str, id: LineageId) {
        if let Some(ref store) = self.store {
            if let Err(e) = store.indexer().insert(key, id) {
                tracing::warn!("Failed to index lineage '{}': {}", key, e);
            }
        }
    }

    /// Remove a lineage from the index
    #[cfg(feature = "server")]
    pub fn sync_index_remove(&self, key: &str) {
        if let Some(ref store) = self.store {
            if let Err(e) = store.indexer().remove(key) {
                tracing::warn!("Failed to remove lineage '{}' from index: {}", key, e);
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // STABILITY LAYER - SYSTEM LINEAGES
    // ═══════════════════════════════════════════════════════════════

    /// Bootstrap system lineages for stability tracking
    ///
    /// Creates `_system.*` lineages if they don't exist.
    /// Should be called after resurrection or on fresh start.
    #[cfg(feature = "server")]
    pub fn bootstrap_system_lineages(&mut self) {
        use stability::lineages;

        // Health lineage - self-diagnostic
        self.ensure_lineage(lineages::HEALTH, 1.0);

        // State lineage - exhaustion level
        self.ensure_lineage(lineages::STATE, 1.0);

        // Resistance lineage - built from challenges
        self.ensure_lineage(lineages::RESISTANCE, 0.5);

        tracing::debug!("System lineages bootstrapped");
    }

    /// Ensure a lineage exists, create if not
    #[cfg(feature = "server")]
    fn ensure_lineage(&mut self, key: &str, initial_energy: f32) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        if self.psyche.lookup(hash).is_none() {
            let lineage = Lineage::new(initial_energy);
            let id = self.psyche.alloc_with_key(hash, lineage);
            self.sync_index_insert(key, id);
            tracing::trace!("Created system lineage: {}", key);
        }
    }

    /// Get energy of a system lineage
    #[cfg(feature = "server")]
    pub fn get_system_energy(&self, key: &str) -> Option<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        self.psyche
            .lookup(hash)
            .and_then(|id| self.psyche.get(id))
            .map(|l| l.current_energy())
    }

    /// Stimulate a system lineage
    #[cfg(feature = "server")]
    pub fn stimulate_system(&mut self, key: &str, delta: f32) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        if let Some(id) = self.psyche.lookup(hash) {
            if let Some(lineage) = self.psyche.get_mut(id) {
                lineage.stimulate(delta);
            }
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
