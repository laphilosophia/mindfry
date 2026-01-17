//! Strata Arena - Historical engram storage
//!
//! Stores the historical snapshots (engrams) for each lineage.
//! Uses a ring buffer design for efficient history management.

use serde::{Deserialize, Serialize};

use super::LineageId;

/// A historical snapshot within a lineage
///
/// Memory layout: 24 bytes
/// Represents a single "memory" or event recorded in a lineage's history.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub struct Engram {
    /// Timestamp when this engram was recorded (nanoseconds since epoch)
    pub timestamp: u64,

    /// Stimulation intensity at recording time
    pub stimulation: f32,

    /// Payload ID (index into external payload storage)
    /// u32::MAX means no payload
    pub payload_id: u32,

    /// Source ID (interned string for source identifier)
    pub source_id: u32,

    /// Previous engram index (ring buffer link)
    /// u32::MAX means this is the oldest engram
    pub prev_index: u32,
}

impl Default for Engram {
    fn default() -> Self {
        Self {
            timestamp: 0,
            stimulation: 0.0,
            payload_id: u32::MAX,
            source_id: u32::MAX,
            prev_index: u32::MAX,
        }
    }
}

impl Engram {
    /// Create a new engram
    pub fn new(timestamp: u64, stimulation: f32) -> Self {
        Self {
            timestamp,
            stimulation,
            ..Default::default()
        }
    }

    /// Create engram with payload
    pub fn with_payload(timestamp: u64, stimulation: f32, payload_id: u32) -> Self {
        Self {
            timestamp,
            stimulation,
            payload_id,
            ..Default::default()
        }
    }

    /// Check if this engram has a payload
    #[inline]
    pub fn has_payload(&self) -> bool {
        self.payload_id != u32::MAX
    }

    /// Check if this is the oldest engram in chain
    #[inline]
    pub fn is_oldest(&self) -> bool {
        self.prev_index == u32::MAX
    }
}

/// Arena for storing historical engrams
///
/// Uses a flat buffer with per-lineage ring buffer semantics.
/// Each lineage has a fixed-size window of history.
pub struct StrataArena {
    /// Contiguous storage: [lineage_0_engrams..., lineage_1_engrams..., ...]
    data: Vec<Engram>,
    /// Number of engrams per lineage (depth)
    depth: usize,
    /// Capacity (max lineages)
    _capacity: usize,
}

impl StrataArena {
    /// Create a new strata arena
    ///
    /// # Arguments
    /// * `max_lineages` - Maximum number of lineages
    /// * `depth` - History depth per lineage
    pub fn with_capacity(max_lineages: usize, depth: usize) -> Self {
        let total_size = max_lineages * depth;
        Self {
            data: vec![Engram::default(); total_size],
            depth,
            _capacity: max_lineages,
        }
    }

    /// Get the history depth
    #[inline]
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Get the base index for a lineage's engram window
    #[inline]
    fn base_index(&self, lineage: LineageId) -> usize {
        lineage.index() * self.depth
    }

    /// Record a new engram for a lineage
    ///
    /// Returns the global index of the new engram, and updates
    /// the head_index that should be stored in the Lineage.
    pub fn record(&mut self, lineage: LineageId, current_head: u32, engram: Engram) -> u32 {
        let base = self.base_index(lineage);

        // Find next slot using ring buffer logic
        let slot_offset = if current_head == u32::MAX {
            // First engram for this lineage
            0
        } else {
            // Next slot in ring buffer
            let current_offset = (current_head as usize) - base;
            (current_offset + 1) % self.depth
        };

        let global_index = base + slot_offset;

        // Link to previous
        let mut new_engram = engram;
        if current_head != u32::MAX {
            new_engram.prev_index = current_head;
        }

        self.data[global_index] = new_engram;
        global_index as u32
    }

    /// Get an engram by global index
    #[inline]
    pub fn get(&self, index: u32) -> Option<&Engram> {
        self.data.get(index as usize)
    }

    /// Iterate through a lineage's history (newest to oldest)
    pub fn history(&self, head_index: u32) -> impl Iterator<Item = &Engram> {
        HistoryIter {
            arena: self,
            current: head_index,
            remaining: self.depth,
        }
    }

    /// Get raw data for persistence
    pub fn as_slice(&self) -> &[Engram] {
        &self.data
    }
}

/// Iterator over engram history
struct HistoryIter<'a> {
    arena: &'a StrataArena,
    current: u32,
    remaining: usize,
}

impl<'a> Iterator for HistoryIter<'a> {
    type Item = &'a Engram;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 || self.current == u32::MAX {
            return None;
        }

        let engram = self.arena.get(self.current)?;
        self.current = engram.prev_index;
        self.remaining -= 1;
        Some(engram)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engram_default() {
        let e = Engram::default();
        assert!(!e.has_payload());
        assert!(e.is_oldest());
    }

    #[test]
    fn test_strata_arena_record() {
        let mut arena = StrataArena::with_capacity(10, 4);
        let lineage = LineageId(0);

        // Record first engram
        let head1 = arena.record(lineage, u32::MAX, Engram::new(1000, 0.5));
        assert_eq!(head1, 0);

        // Record second engram
        let head2 = arena.record(lineage, head1, Engram::new(2000, 0.7));
        assert_eq!(head2, 1);

        // Verify chain
        let e2 = arena.get(head2).unwrap();
        assert_eq!(e2.prev_index, head1);
    }

    #[test]
    fn test_history_iteration() {
        let mut arena = StrataArena::with_capacity(10, 4);
        let lineage = LineageId(0);

        let head1 = arena.record(lineage, u32::MAX, Engram::new(1000, 0.1));
        let head2 = arena.record(lineage, head1, Engram::new(2000, 0.2));
        let head3 = arena.record(lineage, head2, Engram::new(3000, 0.3));

        let history: Vec<_> = arena.history(head3).collect();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].stimulation, 0.3); // Newest first
        assert_eq!(history[1].stimulation, 0.2);
        assert_eq!(history[2].stimulation, 0.1);
    }

    #[test]
    fn test_ring_buffer_wrap() {
        let mut arena = StrataArena::with_capacity(10, 3); // Only 3 slots per lineage
        let lineage = LineageId(0);

        let h1 = arena.record(lineage, u32::MAX, Engram::new(1000, 0.1));
        let h2 = arena.record(lineage, h1, Engram::new(2000, 0.2));
        let h3 = arena.record(lineage, h2, Engram::new(3000, 0.3));
        let h4 = arena.record(lineage, h3, Engram::new(4000, 0.4)); // Wraps to slot 0

        // h4 should be at slot 0 (wrapped)
        assert_eq!(h4, 0);

        // h4's prev should point to h3
        assert_eq!(arena.get(h4).unwrap().prev_index, h3);
    }
}
