//! Akashic Store - The eternal memory
//!
//! Uses sled for persistent key-value storage with multiple trees:
//! - `meta`: Configuration and version info
//! - `snapshots`: Full state backups
//! - `snapshot_meta`: Snapshot metadata index

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sled::{Db, Tree};

use super::snapshot::{PhysicsSnapshot, Snapshot, SnapshotMeta};
use crate::arena::{Engram, Lineage, PsycheArena, StrataArena};
use crate::graph::{Bond, BondGraph};

/// Akashic Store error types
#[derive(Debug)]
pub enum AkashicError {
    /// sled database error
    Sled(sled::Error),
    /// Serialization error
    Serialization(bincode::Error),
    /// Snapshot not found
    SnapshotNotFound(String),
    /// Invalid data format
    InvalidData(String),
    /// IO error
    Io(std::io::Error),
}

impl From<sled::Error> for AkashicError {
    fn from(e: sled::Error) -> Self {
        Self::Sled(e)
    }
}

impl From<bincode::Error> for AkashicError {
    fn from(e: bincode::Error) -> Self {
        Self::Serialization(e)
    }
}

impl From<std::io::Error> for AkashicError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl std::fmt::Display for AkashicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sled(e) => write!(f, "Sled error: {}", e),
            Self::Serialization(e) => write!(f, "Serialization error: {}", e),
            Self::SnapshotNotFound(name) => write!(f, "Snapshot not found: {}", name),
            Self::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            Self::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for AkashicError {}

/// Result type for Akashic operations
pub type Result<T> = std::result::Result<T, AkashicError>;

/// Configuration for Akashic Store
#[derive(Debug, Clone)]
pub struct AkashicConfig {
    /// Path to database directory
    pub path: String,
    /// Whether to flush after every write (slower but safer)
    pub sync_writes: bool,
    /// Cache size in bytes
    pub cache_size: u64,
}

impl Default for AkashicConfig {
    fn default() -> Self {
        Self {
            path: "./mindfry_data".into(),
            sync_writes: false,
            cache_size: 64 * 1024 * 1024, // 64MB cache
        }
    }
}

/// Metadata stored in the meta tree
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoreMeta {
    version: String,
    created_at: u64,
    last_snapshot_id: Option<u64>,
}

/// The Akashic Store - eternal memory for MindFry
pub struct AkashicStore {
    /// The sled database
    db: Db,
    /// Snapshot data tree
    snapshots: Tree,
    /// Snapshot metadata index
    snapshot_meta: Tree,
    /// Lineage key-to-id index for O(1) lookups
    indexer: super::indexer::LineageIndexer,
    /// Configuration
    _config: AkashicConfig,
}

impl AkashicStore {
    /// Open or create an Akashic Store
    pub fn open(config: AkashicConfig) -> Result<Self> {
        let db = sled::Config::new()
            .path(&config.path)
            .cache_capacity(config.cache_size)
            .flush_every_ms(if config.sync_writes { Some(1) } else { None })
            .open()?;

        let snapshots = db.open_tree("snapshots")?;
        let snapshot_meta = db.open_tree("snapshot_meta")?;
        let lineage_index = db.open_tree("lineage_index")?;

        // Initialize meta if first run
        let meta_tree = db.open_tree("meta")?;
        if meta_tree.is_empty() {
            let meta = StoreMeta {
                version: env!("CARGO_PKG_VERSION").to_string(),
                created_at: now_secs(),
                last_snapshot_id: None,
            };
            meta_tree.insert("store_meta", bincode::serialize(&meta)?)?;
        }

        Ok(Self {
            db,
            snapshots,
            snapshot_meta,
            indexer: super::indexer::LineageIndexer::new(lineage_index),
            _config: config,
        })
    }

    /// Take a snapshot of the current MindFry state
    pub fn take_snapshot(
        &self,
        name: Option<&str>,
        psyche: &PsycheArena,
        strata: &StrataArena,
        bonds: &BondGraph,
        cortex: Option<&crate::Cortex>,
        physics: PhysicsSnapshot,
    ) -> Result<SnapshotMeta> {
        let snapshot_id = now_nanos();

        // Serialize arena data
        let psyche_data = self.serialize_psyche(psyche)?;
        let strata_data = self.serialize_strata(strata)?;
        let bond_data = self.serialize_bonds(bonds)?;

        // Serialize Cortex if provided
        let cortex_data = match cortex {
            Some(c) => Some(bincode::serialize(c)?),
            None => None,
        };

        let meta = SnapshotMeta {
            id: snapshot_id,
            name: name.map(|s| s.to_string()),
            created_at: now_secs(),
            lineage_count: psyche.len() as u32,
            bond_count: bonds.len() as u32,
            size_bytes: (psyche_data.len() + strata_data.len() + bond_data.len()) as u64,
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let snapshot = Snapshot {
            meta: meta.clone(),
            psyche_data,
            strata_data,
            bond_data,
            cortex_data,
            physics_config: physics,
        };

        // Serialize and store
        let encoded = bincode::serialize(&snapshot)?;

        // Key: snapshot ID as big-endian bytes (for sorted iteration)
        let key = snapshot_id.to_be_bytes();
        self.snapshots.insert(key, encoded)?;

        // Store meta separately for fast listing
        let meta_encoded = bincode::serialize(&meta)?;
        self.snapshot_meta.insert(key, meta_encoded)?;

        // Update last snapshot ID
        self.update_last_snapshot(snapshot_id)?;

        // Flush to disk
        self.db.flush()?;

        Ok(meta)
    }

    /// List all available snapshots (newest first)
    pub fn list_snapshots(&self) -> Result<Vec<SnapshotMeta>> {
        let mut snapshots = Vec::new();

        for result in self.snapshot_meta.iter().rev() {
            let (_, value) = result?;
            let meta: SnapshotMeta = bincode::deserialize(&value)?;
            snapshots.push(meta);
        }

        Ok(snapshots)
    }

    /// Get the latest snapshot
    pub fn latest_snapshot(&self) -> Result<Option<Snapshot>> {
        // Fast path: check if any snapshots exist
        if self.snapshot_meta.is_empty() {
            return Ok(None);
        }

        // Get latest metadata (fast - small data)
        if let Some((key, _)) = self.snapshot_meta.last()? {
            // Then load the full snapshot data using the key
            if let Some(value) = self.snapshots.get(&key)? {
                let snapshot: Snapshot = bincode::deserialize(&value)?;
                return Ok(Some(snapshot));
            }
        }
        Ok(None)
    }

    /// Get a snapshot by ID
    pub fn get_snapshot(&self, id: u64) -> Result<Option<Snapshot>> {
        let key = id.to_be_bytes();
        if let Some(value) = self.snapshots.get(key)? {
            let snapshot: Snapshot = bincode::deserialize(&value)?;
            return Ok(Some(snapshot));
        }
        Ok(None)
    }

    /// Get a snapshot by name
    pub fn get_snapshot_by_name(&self, name: &str) -> Result<Option<Snapshot>> {
        // Search through metadata
        for result in self.snapshot_meta.iter() {
            let (_key, value) = result?;
            let meta: SnapshotMeta = bincode::deserialize(&value)?;
            if meta.name.as_deref() == Some(name) {
                return self.get_snapshot(meta.id);
            }
        }
        Ok(None)
    }

    /// Delete a snapshot by ID
    pub fn delete_snapshot(&self, id: u64) -> Result<bool> {
        let key = id.to_be_bytes();
        let removed_data = self.snapshots.remove(key)?;
        let removed_meta = self.snapshot_meta.remove(key)?;
        self.db.flush()?;
        Ok(removed_data.is_some() || removed_meta.is_some())
    }

    /// Restore MindFry state from a snapshot
    ///
    /// Returns (PsycheArena, StrataArena, BondGraph, PhysicsSnapshot)
    pub fn restore_snapshot(
        &self,
        snapshot: &Snapshot,
        max_lineages: usize,
        max_bonds: usize,
        strata_depth: usize,
    ) -> Result<(PsycheArena, StrataArena, BondGraph, PhysicsSnapshot)> {
        // Deserialize arenas
        let psyche = self.deserialize_psyche(&snapshot.psyche_data, max_lineages)?;
        let strata = self.deserialize_strata(&snapshot.strata_data, max_lineages, strata_depth)?;
        let bonds = self.deserialize_bonds(&snapshot.bond_data, max_lineages, max_bonds)?;

        Ok((psyche, strata, bonds, snapshot.physics_config.clone()))
    }

    /// Get database size on disk
    pub fn disk_size(&self) -> Result<u64> {
        Ok(self.db.size_on_disk()?)
    }

    /// Get a reference to the lineage indexer for O(1) key lookups
    pub fn indexer(&self) -> &super::indexer::LineageIndexer {
        &self.indexer
    }

    // ═══════════════════════════════════════════════════════════════
    // SHUTDOWN MARKER (for crash recovery)
    // ═══════════════════════════════════════════════════════════════

    /// Write a graceful shutdown marker
    pub fn write_shutdown_marker(&self, marker: &crate::stability::ShutdownMarker) -> Result<()> {
        let meta_tree = self.db.open_tree("meta")?;
        let data = bincode::serialize(marker)?;
        meta_tree.insert("shutdown_marker", data)?;
        self.db.flush()?;
        Ok(())
    }

    /// Read and clear the shutdown marker
    /// Returns None if no marker exists (first run or clean startup)
    pub fn read_shutdown_marker(&self) -> Result<Option<crate::stability::ShutdownMarker>> {
        let meta_tree = self.db.open_tree("meta")?;
        match meta_tree.get("shutdown_marker")? {
            Some(data) => {
                let marker: crate::stability::ShutdownMarker = bincode::deserialize(&data)?;
                // Clear the marker so next unclean shutdown is detected
                meta_tree.remove("shutdown_marker")?;
                Ok(Some(marker))
            }
            None => Ok(None),
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // SERIALIZATION HELPERS
    // ═══════════════════════════════════════════════════════════════

    fn serialize_psyche(&self, psyche: &PsycheArena) -> Result<Vec<u8>> {
        // Collect active lineages with their IDs
        let lineages: Vec<(u32, Lineage)> = psyche.iter().map(|(id, l)| (id.0, *l)).collect();

        Ok(bincode::serialize(&lineages)?)
    }

    fn serialize_strata(&self, strata: &StrataArena) -> Result<Vec<u8>> {
        // Just serialize the raw engram slice
        let engrams: Vec<Engram> = strata.as_slice().to_vec();
        Ok(bincode::serialize(&engrams)?)
    }

    fn serialize_bonds(&self, bonds: &BondGraph) -> Result<Vec<u8>> {
        // Collect active bonds
        let bond_list: Vec<Bond> = bonds.iter().map(|(_, b)| *b).collect();

        Ok(bincode::serialize(&bond_list)?)
    }

    fn deserialize_psyche(&self, data: &[u8], capacity: usize) -> Result<PsycheArena> {
        let lineages: Vec<(u32, Lineage)> = bincode::deserialize(data)?;

        let mut arena = PsycheArena::with_capacity(capacity);
        for (_id, lineage) in lineages {
            arena.alloc(lineage);
        }

        Ok(arena)
    }

    fn deserialize_strata(
        &self,
        data: &[u8],
        max_lineages: usize,
        depth: usize,
    ) -> Result<StrataArena> {
        let engrams: Vec<Engram> = bincode::deserialize(data)?;

        // Create arena and populate
        let arena = StrataArena::with_capacity(max_lineages, depth);

        // Note: For now we rebuild from serialized data
        // TODO: Direct memory restore for zero-copy
        let _ = engrams; // Placeholder - full restore needs lineage mapping

        Ok(arena)
    }

    fn deserialize_bonds(
        &self,
        data: &[u8],
        max_lineages: usize,
        max_bonds: usize,
    ) -> Result<BondGraph> {
        let bonds: Vec<Bond> = bincode::deserialize(data)?;

        let mut graph = BondGraph::with_capacity(max_lineages, max_bonds);
        for bond in bonds {
            graph.connect(bond);
        }

        Ok(graph)
    }

    fn update_last_snapshot(&self, id: u64) -> Result<()> {
        let meta_tree = self.db.open_tree("meta")?;
        if let Some(data) = meta_tree.get("store_meta")? {
            let mut meta: StoreMeta = bincode::deserialize(&data)?;
            meta.last_snapshot_id = Some(id);
            meta_tree.insert("store_meta", bincode::serialize(&meta)?)?;
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════
// TIME UTILITIES
// ═══════════════════════════════════════════════════════════════

#[inline]
fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[inline]
fn now_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

// ═══════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::Lineage;
    use crate::arena::LineageId;
    use crate::graph::Bond;
    use tempfile::tempdir;

    fn temp_config() -> AkashicConfig {
        let dir = tempdir().unwrap();
        AkashicConfig {
            path: dir.path().to_string_lossy().to_string(),
            sync_writes: true,
            cache_size: 1024 * 1024,
        }
    }

    #[test]
    fn test_akashic_open() {
        let config = temp_config();
        let store = AkashicStore::open(config);
        assert!(store.is_ok());
    }

    #[test]
    fn test_akashic_snapshot_roundtrip() {
        let config = temp_config();
        let store = AkashicStore::open(config).unwrap();

        // Create test data
        let mut psyche = PsycheArena::with_capacity(100);
        psyche.alloc(Lineage::new(0.9));
        psyche.alloc(Lineage::new(0.7));
        psyche.alloc(Lineage::new(0.5));

        let strata = StrataArena::with_capacity(100, 8);

        let mut bonds = BondGraph::with_capacity(100, 1000);
        bonds.connect(Bond::new(LineageId(0), LineageId(1), 0.8));
        bonds.connect(Bond::new(LineageId(1), LineageId(2), 0.6));

        let physics = PhysicsSnapshot::default();

        // Take snapshot
        let meta = store
            .take_snapshot(
                Some("test_snapshot"),
                &psyche,
                &strata,
                &bonds,
                None,
                physics,
            )
            .unwrap();

        assert_eq!(meta.name, Some("test_snapshot".to_string()));
        assert_eq!(meta.lineage_count, 3);
        assert_eq!(meta.bond_count, 2);

        // Restore
        let snapshot = store.latest_snapshot().unwrap().unwrap();
        let (restored_psyche, _restored_strata, restored_bonds, _physics) =
            store.restore_snapshot(&snapshot, 100, 1000, 8).unwrap();

        assert_eq!(restored_psyche.len(), 3);
        assert_eq!(restored_bonds.len(), 2);
    }

    #[test]
    fn test_akashic_list_snapshots() {
        let config = temp_config();
        let store = AkashicStore::open(config).unwrap();

        let psyche = PsycheArena::with_capacity(10);
        let strata = StrataArena::with_capacity(10, 4);
        let bonds = BondGraph::with_capacity(10, 100);
        let physics = PhysicsSnapshot::default();

        // Take multiple snapshots
        store
            .take_snapshot(
                Some("snap1"),
                &psyche,
                &strata,
                &bonds,
                None,
                physics.clone(),
            )
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        store
            .take_snapshot(
                Some("snap2"),
                &psyche,
                &strata,
                &bonds,
                None,
                physics.clone(),
            )
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        store
            .take_snapshot(Some("snap3"), &psyche, &strata, &bonds, None, physics)
            .unwrap();

        let list = store.list_snapshots().unwrap();
        assert_eq!(list.len(), 3);
        // Should be newest first
        assert_eq!(list[0].name, Some("snap3".to_string()));
        assert_eq!(list[2].name, Some("snap1".to_string()));
    }

    #[test]
    fn test_akashic_get_by_name() {
        let config = temp_config();
        let store = AkashicStore::open(config).unwrap();

        let psyche = PsycheArena::with_capacity(10);
        let strata = StrataArena::with_capacity(10, 4);
        let bonds = BondGraph::with_capacity(10, 100);
        let physics = PhysicsSnapshot::default();

        store
            .take_snapshot(Some("my_save"), &psyche, &strata, &bonds, None, physics)
            .unwrap();

        let snapshot = store.get_snapshot_by_name("my_save").unwrap();
        assert!(snapshot.is_some());
        assert_eq!(snapshot.unwrap().meta.name, Some("my_save".to_string()));

        let not_found = store.get_snapshot_by_name("nonexistent").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_akashic_delete_snapshot() {
        let config = temp_config();
        let store = AkashicStore::open(config).unwrap();

        let psyche = PsycheArena::with_capacity(10);
        let strata = StrataArena::with_capacity(10, 4);
        let bonds = BondGraph::with_capacity(10, 100);
        let physics = PhysicsSnapshot::default();

        let meta = store
            .take_snapshot(Some("to_delete"), &psyche, &strata, &bonds, None, physics)
            .unwrap();

        assert!(store.delete_snapshot(meta.id).unwrap());
        assert!(store.get_snapshot(meta.id).unwrap().is_none());
    }
}
