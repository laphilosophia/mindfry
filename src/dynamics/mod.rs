//! Dynamics module - Background decay and learning engine
//!
//! This module provides the core dynamics:
//! - `DecayEngine`: Background decay computation
//! - `SynapseEngine`: Polarity-aware signal propagation
//! - `Learner`: Hebbian association (future)

mod decay;
mod synapse;

pub use decay::{DecayConfig, DecayEngine, GcResult};
pub use synapse::{SynapseConfig, SynapseEngine};
