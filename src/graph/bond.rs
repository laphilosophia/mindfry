//! Bond - Living connections between lineages
//!
//! Unlike traditional graph databases where edges are static,
//! Bonds in MindFry are living entities that strengthen with use
//! and weaken (decay) without reinforcement.

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::arena::LineageId;
use crate::setun::Trit;

/// Default polarity for backward compatibility (excitatory)
fn default_polarity() -> Trit {
    Trit::True
}

/// Unique identifier for a bond
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct BondId(pub u32);

impl BondId {
    /// Invalid/null bond ID
    pub const NULL: Self = Self(u32::MAX);

    /// Check if this is a valid bond ID
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

impl From<u32> for BondId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<usize> for BondId {
    fn from(v: usize) -> Self {
        Self(v as u32)
    }
}

bitflags! {
    /// Flags for bond state
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct BondFlags: u32 {
        /// Bond is active (not deleted)
        const ACTIVE = 1 << 0;
        /// Bond was learned (Hebbian) vs explicit
        const LEARNED = 1 << 1;
        /// Bond is bidirectional
        const BIDIRECTIONAL = 1 << 2;
        /// Bond is protected from decay
        const PROTECTED = 1 << 3;
    }
}

impl Default for BondFlags {
    fn default() -> Self {
        Self::ACTIVE
    }
}

/// A living connection between two lineages
///
/// Memory layout: 32 bytes
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub struct Bond {
    /// Source lineage
    pub source: LineageId,

    /// Target lineage
    pub target: LineageId,

    /// Connection strength (0.0 - 1.0)
    /// Decays over time, strengthens with reinforcement
    pub strength: f32,

    /// Traversal cost (0.0 - 1.0)
    /// Energy consumed when traversing this bond
    pub cost: f32,

    /// Decay rate (per second)
    pub decay_rate: f32,

    /// Last access/reinforcement timestamp
    pub last_access: u64,

    /// State flags
    pub flags: BondFlags,

    /// Bond polarity: +1 (Synergy), 0 (Neutral), -1 (Antagonism)
    /// Determines how energy propagates through this bond
    #[serde(default = "default_polarity")]
    pub polarity: Trit,
}

impl Default for Bond {
    fn default() -> Self {
        Self {
            source: LineageId::NULL,
            target: LineageId::NULL,
            strength: 1.0,
            cost: 0.1,
            decay_rate: 0.0005,
            last_access: now_nanos(),
            flags: BondFlags::ACTIVE,
            polarity: Trit::True,
        }
    }
}

impl Bond {
    /// Create a new bond between two lineages
    pub fn new(source: LineageId, target: LineageId, strength: f32) -> Self {
        Self {
            source,
            target,
            strength,
            last_access: now_nanos(),
            ..Default::default()
        }
    }

    /// Create a learned (Hebbian) bond
    pub fn learned(source: LineageId, target: LineageId, strength: f32) -> Self {
        let mut bond = Self::new(source, target, strength);
        bond.flags.insert(BondFlags::LEARNED);
        bond
    }

    /// Check if bond is active
    #[inline]
    pub fn is_active(&self) -> bool {
        self.flags.contains(BondFlags::ACTIVE)
    }

    /// Check if bond is learned (Hebbian)
    #[inline]
    pub fn is_learned(&self) -> bool {
        self.flags.contains(BondFlags::LEARNED)
    }

    /// Compute current strength with decay applied
    pub fn current_strength(&self) -> f32 {
        if self.flags.contains(BondFlags::PROTECTED) {
            return self.strength;
        }

        let elapsed_secs = elapsed_seconds(self.last_access);
        self.strength * (-self.decay_rate * elapsed_secs).exp()
    }

    /// Reinforce the bond (strengthen connection)
    pub fn reinforce(&mut self, delta: f32) {
        self.strength = (self.current_strength() + delta).clamp(0.0, 1.0);
        self.last_access = now_nanos();
    }

    /// Get the other end of the bond given one endpoint
    #[inline]
    pub fn other(&self, from: LineageId) -> LineageId {
        if self.source == from {
            self.target
        } else {
            self.source
        }
    }
}

/// Minimum bond strength before pruning
pub const BOND_PRUNE_THRESHOLD: f32 = 0.05;

/// Graph storage for bonds with adjacency tracking
pub struct BondGraph {
    /// Contiguous bond storage
    bonds: Vec<Bond>,
    /// Active bond count
    count: usize,
    /// Free list for recycled slots
    free_list: Vec<BondId>,
    /// Adjacency: lineage -> list of bond indices
    adjacency: Vec<Vec<BondId>>,
    /// Maximum lineages (for adjacency sizing)
    max_lineages: usize,
}

impl BondGraph {
    /// Create a new bond graph
    pub fn with_capacity(max_lineages: usize, max_bonds: usize) -> Self {
        Self {
            bonds: Vec::with_capacity(max_bonds),
            count: 0,
            free_list: Vec::new(),
            adjacency: vec![Vec::new(); max_lineages],
            max_lineages,
        }
    }

    /// Get the number of active bonds
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if graph is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Get the maximum bond capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.bonds.capacity()
    }

    /// Create a new bond
    pub fn connect(&mut self, bond: Bond) -> Option<BondId> {
        // Validate lineage IDs
        if !bond.source.is_valid() || !bond.target.is_valid() {
            return None;
        }
        if bond.source.index() >= self.max_lineages || bond.target.index() >= self.max_lineages {
            return None;
        }

        let id = if let Some(recycled) = self.free_list.pop() {
            self.bonds[recycled.index()] = bond;
            recycled
        } else {
            let id = BondId(self.bonds.len() as u32);
            self.bonds.push(bond);
            id
        };

        // Update adjacency (bidirectional)
        self.adjacency[bond.source.index()].push(id);
        self.adjacency[bond.target.index()].push(id);

        self.count += 1;
        Some(id)
    }

    /// Get a bond by ID
    #[inline]
    pub fn get(&self, id: BondId) -> Option<&Bond> {
        self.bonds.get(id.index()).filter(|b| b.is_active())
    }

    /// Get a mutable bond by ID
    #[inline]
    pub fn get_mut(&mut self, id: BondId) -> Option<&mut Bond> {
        self.bonds.get_mut(id.index()).filter(|b| b.is_active())
    }

    /// Get all bond IDs connected to a lineage
    pub fn neighbors(&self, lineage: LineageId) -> &[BondId] {
        if lineage.index() < self.adjacency.len() {
            &self.adjacency[lineage.index()]
        } else {
            &[]
        }
    }

    /// Get neighbor lineages with their bond strengths
    pub fn neighbors_with_strength(
        &self,
        lineage: LineageId,
    ) -> impl Iterator<Item = (LineageId, f32)> + '_ {
        self.neighbors(lineage).iter().filter_map(move |&bond_id| {
            let bond = self.get(bond_id)?;
            let neighbor = bond.other(lineage);
            Some((neighbor, bond.current_strength()))
        })
    }

    /// Remove a bond
    pub fn disconnect(&mut self, id: BondId) -> bool {
        if let Some(bond) = self.bonds.get_mut(id.index())
            && bond.is_active()
        {
            // Remove from adjacency lists
            if let Some(adj) = self.adjacency.get_mut(bond.source.index()) {
                adj.retain(|&bid| bid != id);
            }
            if let Some(adj) = self.adjacency.get_mut(bond.target.index()) {
                adj.retain(|&bid| bid != id);
            }

            bond.flags.remove(BondFlags::ACTIVE);
            self.free_list.push(id);
            self.count -= 1;
            return true;
        }
        false
    }

    /// Find bond between two lineages
    pub fn find_bond(&self, a: LineageId, b: LineageId) -> Option<BondId> {
        // Search smaller adjacency list
        let a_neighbors = self.neighbors(a);
        let b_neighbors = self.neighbors(b);

        let (search_in, find_target) = if a_neighbors.len() <= b_neighbors.len() {
            (a_neighbors, b)
        } else {
            (b_neighbors, a)
        };

        for &bond_id in search_in {
            if let Some(bond) = self.get(bond_id)
                && bond.other(if find_target == b { a } else { b }) == find_target
            {
                return Some(bond_id);
            }
        }
        None
    }

    /// Prune weak bonds below threshold
    pub fn prune(&mut self, threshold: f32) -> usize {
        let mut pruned = 0;
        let to_prune: Vec<_> = self
            .bonds
            .iter()
            .enumerate()
            .filter_map(|(i, bond)| {
                if bond.is_active() && bond.current_strength() < threshold {
                    Some(BondId(i as u32))
                } else {
                    None
                }
            })
            .collect();

        for id in to_prune {
            if self.disconnect(id) {
                pruned += 1;
            }
        }
        pruned
    }

    /// Iterate over all active bonds
    pub fn iter(&self) -> impl Iterator<Item = (BondId, &Bond)> {
        self.bonds
            .iter()
            .enumerate()
            .filter(|(_, b)| b.is_active())
            .map(|(i, b)| (BondId(i as u32), b))
    }
}

// ═══════════════════════════════════════════════════════════════
// TIME UTILITIES
// ═══════════════════════════════════════════════════════════════

#[inline]
fn now_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

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
    fn test_bond_default() {
        let b = Bond::default();
        assert_eq!(b.strength, 1.0);
        assert!(b.is_active());
        assert!(!b.is_learned());
    }

    #[test]
    fn test_bond_learned() {
        let b = Bond::learned(LineageId(0), LineageId(1), 0.5);
        assert!(b.is_learned());
    }

    #[test]
    fn test_bond_reinforce() {
        let mut b = Bond::new(LineageId(0), LineageId(1), 0.5);
        b.reinforce(0.2);
        assert!(b.strength > 0.5);
    }

    #[test]
    fn test_bond_graph_connect() {
        let mut graph = BondGraph::with_capacity(100, 1000);
        let bond = Bond::new(LineageId(0), LineageId(1), 0.8);
        let id = graph.connect(bond).unwrap();

        assert_eq!(graph.len(), 1);
        assert!(graph.get(id).is_some());
    }

    #[test]
    fn test_bond_graph_neighbors() {
        let mut graph = BondGraph::with_capacity(100, 1000);

        graph.connect(Bond::new(LineageId(0), LineageId(1), 0.8));
        graph.connect(Bond::new(LineageId(0), LineageId(2), 0.6));
        graph.connect(Bond::new(LineageId(1), LineageId(2), 0.4));

        // Node 0 should have 2 neighbors
        let n0: Vec<_> = graph.neighbors_with_strength(LineageId(0)).collect();
        assert_eq!(n0.len(), 2);

        // Node 2 should also have 2 neighbors
        let n2: Vec<_> = graph.neighbors_with_strength(LineageId(2)).collect();
        assert_eq!(n2.len(), 2);
    }

    #[test]
    fn test_bond_graph_find() {
        let mut graph = BondGraph::with_capacity(100, 1000);
        let id = graph
            .connect(Bond::new(LineageId(5), LineageId(10), 0.7))
            .unwrap();

        assert_eq!(graph.find_bond(LineageId(5), LineageId(10)), Some(id));
        assert_eq!(graph.find_bond(LineageId(10), LineageId(5)), Some(id)); // Bidirectional
        assert_eq!(graph.find_bond(LineageId(0), LineageId(1)), None);
    }

    #[test]
    fn test_bond_graph_disconnect() {
        let mut graph = BondGraph::with_capacity(100, 1000);
        let id = graph
            .connect(Bond::new(LineageId(0), LineageId(1), 0.8))
            .unwrap();

        assert_eq!(graph.len(), 1);
        assert!(graph.disconnect(id));
        assert_eq!(graph.len(), 0);
        assert!(graph.get(id).is_none());
    }
}
