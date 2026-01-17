//! Synapse Engine - Damped Signal Propagation
//!
//! Propagates energy through bonds with polarity-aware damping.
//! Prevents "epileptic seizures" (infinite propagation) via resistance and cutoff.
//!
//! ## The Damping Law
//!
//! ```text
//! E_target = E_source × W_bond × P_bond × (1.0 - Resistance)
//! ```
//!
//! With default settings (resistance=0.5, cutoff=0.1):
//! - Hop 1: 0.5
//! - Hop 2: 0.25
//! - Hop 3: 0.125
//! - Hop 4: 0.0625 < 0.1 → CUTOFF
//!
//! Maximum propagation depth: ~3 hops.

use std::collections::HashSet;

use crate::arena::{LineageId, PsycheArena};
use crate::graph::BondGraph;
use crate::setun::Trit;

/// Default resistance (50% energy loss per hop)
pub const DEFAULT_RESISTANCE: f32 = 0.5;

/// Default noise floor (signals below this are ignored)
pub const DEFAULT_CUTOFF: f32 = 0.1;

/// Synapse Engine configuration
#[derive(Debug, Clone)]
pub struct SynapseConfig {
    /// Energy loss per hop (0.0 - 1.0)
    pub resistance: f32,
    /// Minimum signal strength (noise floor)
    pub cutoff: f32,
    /// Maximum propagation depth (safety limit)
    pub max_depth: usize,
}

impl Default for SynapseConfig {
    fn default() -> Self {
        Self {
            resistance: DEFAULT_RESISTANCE,
            cutoff: DEFAULT_CUTOFF,
            max_depth: 10, // Absolute safety limit
        }
    }
}

/// Synapse Engine - Handles signal propagation through bonds
pub struct SynapseEngine {
    config: SynapseConfig,
}

impl SynapseEngine {
    /// Create a new synapse engine with default configuration
    pub fn new() -> Self {
        Self::with_config(SynapseConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: SynapseConfig) -> Self {
        Self { config }
    }

    /// Propagate energy from a source lineage through its bonds
    ///
    /// Returns the number of nodes affected by propagation.
    pub fn propagate(
        &self,
        psyche: &mut PsycheArena,
        bonds: &BondGraph,
        source: LineageId,
        input_energy: f32,
    ) -> usize {
        let mut visited = HashSet::new();
        self.propagate_recursive(psyche, bonds, source, input_energy, &mut visited, 0)
    }

    fn propagate_recursive(
        &self,
        psyche: &mut PsycheArena,
        bonds: &BondGraph,
        source: LineageId,
        input_energy: f32,
        visited: &mut HashSet<LineageId>,
        depth: usize,
    ) -> usize {
        // Cutoff: Signal too weak
        if input_energy.abs() < self.config.cutoff {
            return 0;
        }

        // Depth limit: Safety valve
        if depth >= self.config.max_depth {
            return 0;
        }

        // Loop protection: Already processed
        if visited.contains(&source) {
            return 0;
        }
        visited.insert(source);

        let mut affected = 0;

        // Get neighbors
        let neighbor_ids: Vec<_> = bonds.neighbors(source).to_vec();

        for bond_id in neighbor_ids {
            if let Some(bond) = bonds.get(bond_id) {
                // Skip inactive bonds
                if !bond.is_active() {
                    continue;
                }

                // Calculate transfer energy
                let polarity_weight = match bond.polarity {
                    Trit::True => 1.0,    // Synergy: +1
                    Trit::Unknown => 0.0, // Neutral: insulator
                    Trit::False => -1.0,  // Antagonism: inhibition
                };

                // Neutral bonds are insulators - no propagation
                if polarity_weight == 0.0 {
                    continue;
                }

                let transfer = input_energy * bond.strength * polarity_weight;
                let decayed = transfer * (1.0 - self.config.resistance);

                // Apply to target
                let target = bond.other(source);
                if let Some(lineage) = psyche.get_mut(target) {
                    lineage.stimulate(decayed);
                    affected += 1;

                    // Recursive propagation
                    affected += self.propagate_recursive(
                        psyche,
                        bonds,
                        target,
                        decayed,
                        visited,
                        depth + 1,
                    );
                }
            }
        }

        affected
    }
}

impl Default for SynapseEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::Lineage;
    use crate::graph::Bond;

    fn setup_chain() -> (PsycheArena, BondGraph) {
        let mut psyche = PsycheArena::with_capacity(10);
        let mut bonds = BondGraph::with_capacity(10, 100);

        // Create A -> B -> C chain
        let a = psyche.alloc(Lineage::new(0.5));
        let b = psyche.alloc(Lineage::new(0.1));
        let c = psyche.alloc(Lineage::new(0.1));

        // Synergy bonds
        let mut bond_ab = Bond::new(a, b, 1.0);
        bond_ab.polarity = Trit::True;
        bonds.connect(bond_ab);

        let mut bond_bc = Bond::new(b, c, 1.0);
        bond_bc.polarity = Trit::True;
        bonds.connect(bond_bc);

        (psyche, bonds)
    }

    #[test]
    fn test_synergy_propagation() {
        let (mut psyche, bonds) = setup_chain();
        let engine = SynapseEngine::new();

        let a = LineageId(0);
        let affected = engine.propagate(&mut psyche, &bonds, a, 1.0);

        // Should affect B and C
        assert!(affected >= 1);

        // B should be stimulated (0.5 from hop 1)
        let b = psyche.get(LineageId(1)).unwrap();
        assert!(b.energy > 0.1);

        // C should also be stimulated (less energy due to damping)
        let c = psyche.get(LineageId(2)).unwrap();
        assert!(c.energy > 0.1);
    }

    #[test]
    fn test_neutral_insulation() {
        let mut psyche = PsycheArena::with_capacity(10);
        let mut bonds = BondGraph::with_capacity(10, 100);

        let a = psyche.alloc(Lineage::new(0.5));
        let b = psyche.alloc(Lineage::new(0.1));

        // Neutral bond (insulator)
        let mut bond = Bond::new(a, b, 1.0);
        bond.polarity = Trit::Unknown;
        bonds.connect(bond);

        let engine = SynapseEngine::new();
        let affected = engine.propagate(&mut psyche, &bonds, a, 1.0);

        // Neutral bonds don't propagate
        assert_eq!(affected, 0);
    }

    #[test]
    fn test_antagonism_inhibition() {
        let mut psyche = PsycheArena::with_capacity(10);
        let mut bonds = BondGraph::with_capacity(10, 100);

        let a = psyche.alloc(Lineage::new(0.5));
        let b = psyche.alloc(Lineage::new(0.8)); // High energy

        // Antagonistic bond
        let mut bond = Bond::new(a, b, 1.0);
        bond.polarity = Trit::False;
        bonds.connect(bond);

        let engine = SynapseEngine::new();
        let b_energy_before = psyche.get(LineageId(1)).unwrap().energy;

        engine.propagate(&mut psyche, &bonds, a, 1.0);

        let b_energy_after = psyche.get(LineageId(1)).unwrap().energy;

        // B should be inhibited (energy decreased)
        assert!(b_energy_after < b_energy_before);
    }

    #[test]
    fn test_cutoff_stops_propagation() {
        let (mut psyche, bonds) = setup_chain();

        // Very low energy input
        let engine = SynapseEngine::new();
        let affected = engine.propagate(&mut psyche, &bonds, LineageId(0), 0.05);

        // Should be cut off immediately
        assert_eq!(affected, 0);
    }
}
