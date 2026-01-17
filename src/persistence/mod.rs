//! Akashic Records - Persistence Layer
//!
//! The eternal memory of MindFry. Even when the process dies,
//! the memories, bonds, and traumas persist in the Akashic Records.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                     Akashic Records                             │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
//! │  │    meta     │  │  snapshots  │  │        wal (future)     │  │
//! │  │  (config)   │  │  (backups)  │  │  (write-ahead log)      │  │
//! │  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
//! │                          │                                      │
//! │                    ┌─────▼─────┐                                │
//! │                    │   sled    │                                │
//! │                    │  (disk)   │                                │
//! │                    └───────────┘                                │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Persistence Strategy
//!
//! - **Snapshots**: Full arena dumps at key moments (manual or scheduled)
//! - **WAL (Phase 3.5)**: Write-ahead log for crash recovery

mod akashic;
mod indexer;
pub mod snapshot;

pub use akashic::{AkashicConfig, AkashicError, AkashicStore};
pub use indexer::LineageIndexer;
pub use snapshot::{PhysicsSnapshot, Snapshot, SnapshotMeta};
