//! Dynamics module - Background decay and learning engine
//!
//! This module provides the core dynamics:
//! - `DecayEngine`: Background decay computation
//! - `Learner`: Hebbian association (future)

mod decay;

pub use decay::{DecayConfig, DecayEngine, GcResult};
