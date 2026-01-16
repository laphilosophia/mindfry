//! Graph module - Living bond connections
//!
//! This module provides the bond graph implementation:
//! - `Bond`: A living connection between lineages
//! - `BondGraph`: Graph storage with adjacency tracking

mod bond;

pub use bond::{Bond, BondId, BondFlags, BondGraph, BOND_PRUNE_THRESHOLD};
