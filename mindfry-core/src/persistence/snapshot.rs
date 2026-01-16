//! Snapshot data structures
//!
//! Defines the format for persisted MindFry state.

use serde::{Deserialize, Serialize};

/// Metadata for a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMeta {
    /// Unique snapshot ID (timestamp in nanoseconds)
    pub id: u64,
    /// Human-readable name (optional)
    pub name: Option<String>,
    /// Creation timestamp (seconds since epoch)
    pub created_at: u64,
    /// Number of lineages in snapshot
    pub lineage_count: u32,
    /// Number of bonds in snapshot
    pub bond_count: u32,
    /// Compressed size in bytes
    pub size_bytes: u64,
    /// MindFry version that created this snapshot
    pub version: String,
}

/// A complete snapshot of MindFry state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Snapshot metadata
    pub meta: SnapshotMeta,
    /// Serialized PsycheArena data
    pub psyche_data: Vec<u8>,
    /// Serialized StrataArena data
    pub strata_data: Vec<u8>,
    /// Serialized BondGraph data
    pub bond_data: Vec<u8>,
    /// Physics configuration at snapshot time
    pub physics_config: PhysicsSnapshot,
}

/// Physics state at snapshot time
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PhysicsSnapshot {
    /// Global decay multiplier
    pub decay_multiplier: f32,
    /// Trauma threshold
    pub trauma_threshold: f32,
    /// Bond prune threshold
    pub bond_prune_threshold: f32,
    /// Whether decay was frozen
    pub is_frozen: bool,
}

impl Snapshot {
    /// Calculate total size
    pub fn total_size(&self) -> usize {
        self.psyche_data.len() + self.strata_data.len() + self.bond_data.len()
    }
}
