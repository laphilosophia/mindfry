//! Arena module - High-performance memory storage
//!
//! This module provides the core memory arenas for MindFry:
//! - `PsycheArena`: Active lineage storage
//! - `StrataArena`: Historical engram storage
//! - Allocator utilities

mod psyche;
mod strata;

pub use psyche::{Lineage, LineageId, LineageFlags, PsycheArena};
pub use strata::{Engram, StrataArena};
