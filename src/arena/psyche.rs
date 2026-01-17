//! Psyche Arena - Active lineage storage
//!
//! High-performance arena for storing active lineages with O(1) access.
//! Uses contiguous memory layout for cache efficiency.

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Unique identifier for a lineage within the arena
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct LineageId(pub u32);

impl LineageId {
    /// Invalid/null lineage ID
    pub const NULL: Self = Self(u32::MAX);

    /// Check if this is a valid lineage ID
    #[inline]
    pub fn is_valid(self) -> bool {
        self != Self::NULL
    }

    /// Get the raw index
    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl From<u32> for LineageId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<usize> for LineageId {
    fn from(v: usize) -> Self {
        Self(v as u32)
    }
}

bitflags! {
    /// Flags for lineage state
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct LineageFlags: u32 {
        /// Lineage is active (not deleted)
        const ACTIVE = 1 << 0;
        /// Lineage is conscious (energy >= threshold)
        const CONSCIOUS = 1 << 1;
        /// Lineage is protected from decay (trauma)
        const PROTECTED = 1 << 2;
        /// Lineage has been modified since last persist
        const DIRTY = 1 << 3;
        /// Lineage is pinned in memory
        const PINNED = 1 << 4;
    }
}

impl Default for LineageFlags {
    fn default() -> Self {
        Self::ACTIVE
    }
}

/// A single lineage in the cognitive memory
///
/// Memory layout: 32 bytes (cache-line aligned)
/// This struct represents a "neuron" in the cognitive database.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C, align(32))]
pub struct Lineage {
    /// Current energy level (0.0 - 1.0)
    /// Decays over time when not accessed
    pub energy: f32,

    /// Consciousness threshold (0.0 - 1.0)
    /// When energy >= threshold, lineage is "conscious"
    pub threshold: f32,

    /// Decay rate (per second)
    /// Energy decays as: E(t) = E0 * exp(-decay_rate * Δt)
    pub decay_rate: f32,

    /// Rigidity - resistance to change (0.0 - 1.0)
    /// Higher rigidity = slower learning but more stable
    pub rigidity: f32,

    /// Last access timestamp (nanoseconds since epoch)
    pub last_access: u64,

    /// State flags
    pub flags: LineageFlags,

    /// Index of the head engram in strata arena
    pub head_index: u32,
}

impl Default for Lineage {
    fn default() -> Self {
        Self {
            energy: 1.0,
            threshold: 0.5,
            decay_rate: 0.001,
            rigidity: 0.5,
            last_access: now_nanos(),
            flags: LineageFlags::ACTIVE,
            head_index: u32::MAX,
        }
    }
}

impl Lineage {
    /// Create a new lineage with specified energy
    pub fn new(energy: f32) -> Self {
        Self {
            energy,
            last_access: now_nanos(),
            ..Default::default()
        }
    }

    /// Create a new lineage with full configuration
    pub fn with_config(energy: f32, threshold: f32, decay_rate: f32) -> Self {
        Self {
            energy,
            threshold,
            decay_rate,
            last_access: now_nanos(),
            ..Default::default()
        }
    }

    /// Check if lineage is conscious (energy >= threshold)
    #[inline]
    pub fn is_conscious(&self) -> bool {
        self.energy >= self.threshold
    }

    /// Check if lineage is active
    #[inline]
    pub fn is_active(&self) -> bool {
        self.flags.contains(LineageFlags::ACTIVE)
    }

    /// Check if lineage is protected from decay
    #[inline]
    pub fn is_protected(&self) -> bool {
        self.flags.contains(LineageFlags::PROTECTED)
    }

    /// Compute current energy with decay applied
    pub fn current_energy(&self) -> f32 {
        if self.is_protected() {
            return self.energy;
        }

        let elapsed_secs = elapsed_seconds(self.last_access);
        self.energy * (-self.decay_rate * elapsed_secs).exp()
    }

    /// Stimulate the lineage with energy delta
    pub fn stimulate(&mut self, delta: f32) {
        self.energy = (self.current_energy() + delta).clamp(0.0, 1.0);
        self.last_access = now_nanos();
        self.flags.insert(LineageFlags::DIRTY);

        // Update consciousness flag
        if self.is_conscious() {
            self.flags.insert(LineageFlags::CONSCIOUS);
        } else {
            self.flags.remove(LineageFlags::CONSCIOUS);
        }
    }

    /// Touch the lineage (update last access without changing energy)
    pub fn touch(&mut self) {
        // First apply decay, then reset timer
        self.energy = self.current_energy();
        self.last_access = now_nanos();
    }
}

/// Arena for storing active lineages
pub struct PsycheArena {
    /// Contiguous storage for lineages
    data: Vec<Lineage>,
    /// Current count of active lineages
    count: usize,
    /// Free list for recycled slots
    free_list: Vec<LineageId>,
    /// String ID to LineageId mapping
    id_map: rustc_hash::FxHashMap<u64, LineageId>,
}

impl PsycheArena {
    /// Create a new arena with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            count: 0,
            free_list: Vec::new(),
            id_map: rustc_hash::FxHashMap::default(),
        }
    }

    /// Get the number of active lineages
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if arena is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Get the capacity of the arena
    #[inline]
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Allocate a new lineage
    pub fn alloc(&mut self, lineage: Lineage) -> LineageId {
        let id = if let Some(recycled) = self.free_list.pop() {
            // Reuse a recycled slot
            self.data[recycled.index()] = lineage;
            recycled
        } else {
            // Allocate new slot
            let id = LineageId(self.data.len() as u32);
            self.data.push(lineage);
            id
        };

        self.count += 1;
        id
    }

    /// Allocate with a string key for lookup
    pub fn alloc_with_key(&mut self, key: u64, lineage: Lineage) -> LineageId {
        let id = self.alloc(lineage);
        self.id_map.insert(key, id);
        id
    }

    /// Get a lineage by ID
    #[inline]
    pub fn get(&self, id: LineageId) -> Option<&Lineage> {
        self.data.get(id.index()).filter(|l| l.is_active())
    }

    /// Get a mutable lineage by ID
    #[inline]
    pub fn get_mut(&mut self, id: LineageId) -> Option<&mut Lineage> {
        self.data.get_mut(id.index()).filter(|l| l.is_active())
    }

    /// Look up lineage by string key hash
    pub fn lookup(&self, key: u64) -> Option<LineageId> {
        self.id_map.get(&key).copied()
    }

    /// Delete a lineage
    pub fn free(&mut self, id: LineageId) -> bool {
        if let Some(lineage) = self.data.get_mut(id.index()) {
            if lineage.is_active() {
                lineage.flags.remove(LineageFlags::ACTIVE);
                self.free_list.push(id);
                self.count -= 1;
                return true;
            }
        }
        false
    }

    /// Iterate over all active lineages
    pub fn iter(&self) -> impl Iterator<Item = (LineageId, &Lineage)> {
        self.data
            .iter()
            .enumerate()
            .filter(|(_, l)| l.is_active())
            .map(|(i, l)| (LineageId(i as u32), l))
    }

    /// Iterate mutably over all active lineages
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (LineageId, &mut Lineage)> {
        self.data
            .iter_mut()
            .enumerate()
            .filter(|(_, l)| l.is_active())
            .map(|(i, l)| (LineageId(i as u32), l))
    }

    /// Get raw data slice for persistence
    pub fn as_slice(&self) -> &[Lineage] {
        &self.data
    }
}

// ═══════════════════════════════════════════════════════════════
// TIME UTILITIES
// ═══════════════════════════════════════════════════════════════

/// Get current time in nanoseconds since epoch
#[inline]
fn now_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

/// Calculate elapsed seconds since a timestamp
#[inline]
fn elapsed_seconds(timestamp: u64) -> f32 {
    let now = now_nanos();
    if now > timestamp {
        ((now - timestamp) as f64 / 1_000_000_000.0) as f32
    } else {
        0.0
    }
}

// ═══════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lineage_default() {
        let l = Lineage::default();
        assert_eq!(l.energy, 1.0);
        assert_eq!(l.threshold, 0.5);
        assert!(l.is_conscious());
        assert!(l.is_active());
    }

    #[test]
    fn test_lineage_stimulate() {
        let mut l = Lineage::new(0.3);
        l.threshold = 0.5;
        assert!(!l.is_conscious());

        l.stimulate(0.3);
        assert!(l.is_conscious());
    }

    #[test]
    fn test_psyche_arena_alloc() {
        let mut arena = PsycheArena::with_capacity(100);
        let id = arena.alloc(Lineage::new(0.8));

        assert_eq!(arena.len(), 1);
        assert!(arena.get(id).is_some());
        assert_eq!(arena.get(id).unwrap().energy, 0.8);
    }

    #[test]
    fn test_psyche_arena_free_and_recycle() {
        let mut arena = PsycheArena::with_capacity(100);

        let id1 = arena.alloc(Lineage::new(0.5));
        let _id2 = arena.alloc(Lineage::new(0.7));
        assert_eq!(arena.len(), 2);

        // Free first lineage
        arena.free(id1);
        assert_eq!(arena.len(), 1);
        assert!(arena.get(id1).is_none());

        // Allocate new - should reuse id1's slot
        let id3 = arena.alloc(Lineage::new(0.9));
        assert_eq!(id3, id1); // Recycled slot
        assert_eq!(arena.len(), 2);
    }

    #[test]
    fn test_lineage_id_null() {
        assert!(!LineageId::NULL.is_valid());
        assert!(LineageId(0).is_valid());
    }
}
