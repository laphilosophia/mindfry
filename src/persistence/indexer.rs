//! Lineage Indexer - O(1) Key-to-ID Lookup
//!
//! Provides a reverse index from String keys to LineageId for fast lookups.
//! Built on top of Sled's key-value store.
//!
//! # Design
//!
//! This indexer is **Snapshot-Based**: it rebuilds from persisted data on restore.
//! For in-memory operations, the PsycheArena's HashMap provides O(1) access.
//!
//! The disk index is used for:
//! - Server startup (restore from cold storage)
//! - Single-key lookups without loading full arena

use sled::Tree;

use super::AkashicError;
use crate::arena::LineageId;

/// Result type for indexer operations
pub type Result<T> = std::result::Result<T, AkashicError>;

/// Lineage Index - Maps String Keys to LineageIds
///
/// Uses a sled Tree for persistent O(1) lookups.
pub struct LineageIndexer {
    /// The sled tree storing key -> id mappings
    tree: Tree,
}

impl LineageIndexer {
    /// Create a new indexer from a sled tree
    pub fn new(tree: Tree) -> Self {
        Self { tree }
    }

    /// Insert a key-to-id mapping
    pub fn insert(&self, key: &str, id: LineageId) -> Result<()> {
        let id_bytes = id.0.to_le_bytes();
        self.tree.insert(key.as_bytes(), &id_bytes)?;
        Ok(())
    }

    /// Remove a key from the index
    pub fn remove(&self, key: &str) -> Result<bool> {
        let removed = self.tree.remove(key.as_bytes())?;
        Ok(removed.is_some())
    }

    /// Lookup a LineageId by key
    pub fn get(&self, key: &str) -> Result<Option<LineageId>> {
        match self.tree.get(key.as_bytes())? {
            Some(bytes) => {
                if bytes.len() >= 4 {
                    let id = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                    Ok(Some(LineageId(id)))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// Check if a key exists in the index
    pub fn contains(&self, key: &str) -> Result<bool> {
        Ok(self.tree.contains_key(key.as_bytes())?)
    }

    /// Get the number of indexed entries
    pub fn len(&self) -> usize {
        self.tree.len()
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    /// Clear the entire index
    pub fn clear(&self) -> Result<()> {
        self.tree.clear()?;
        Ok(())
    }

    /// Rebuild the index from a list of (key, id) pairs
    ///
    /// Used after restoring from a snapshot to populate the index.
    pub fn rebuild(&self, entries: impl Iterator<Item = (String, LineageId)>) -> Result<usize> {
        // Clear existing index
        self.clear()?;

        let mut count = 0;
        for (key, id) in entries {
            self.insert(&key, id)?;
            count += 1;
        }

        // Flush to disk
        self.tree.flush()?;

        Ok(count)
    }

    /// Iterate over all indexed entries
    pub fn iter(&self) -> impl Iterator<Item = Result<(String, LineageId)>> + '_ {
        self.tree.iter().map(|result| {
            let (key_bytes, value_bytes) = result?;
            let key = String::from_utf8_lossy(&key_bytes).to_string();
            let id = if value_bytes.len() >= 4 {
                LineageId(u32::from_le_bytes([
                    value_bytes[0],
                    value_bytes[1],
                    value_bytes[2],
                    value_bytes[3],
                ]))
            } else {
                LineageId(0)
            };
            Ok((key, id))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn temp_indexer() -> LineageIndexer {
        let dir = tempdir().unwrap();
        let db = sled::open(dir.path()).unwrap();
        let tree = db.open_tree("test_index").unwrap();
        LineageIndexer::new(tree)
    }

    #[test]
    fn test_indexer_insert_and_get() {
        let indexer = temp_indexer();

        indexer.insert("session_123", LineageId(42)).unwrap();
        indexer.insert("user_abc", LineageId(100)).unwrap();

        assert_eq!(indexer.get("session_123").unwrap(), Some(LineageId(42)));
        assert_eq!(indexer.get("user_abc").unwrap(), Some(LineageId(100)));
        assert_eq!(indexer.get("nonexistent").unwrap(), None);
    }

    #[test]
    fn test_indexer_remove() {
        let indexer = temp_indexer();

        indexer.insert("temp_key", LineageId(1)).unwrap();
        assert!(indexer.contains("temp_key").unwrap());

        indexer.remove("temp_key").unwrap();
        assert!(!indexer.contains("temp_key").unwrap());
    }

    #[test]
    fn test_indexer_rebuild() {
        let indexer = temp_indexer();

        // Insert some entries
        indexer.insert("old_key", LineageId(999)).unwrap();

        // Rebuild with new data
        let entries = vec![
            ("key_a".to_string(), LineageId(1)),
            ("key_b".to_string(), LineageId(2)),
            ("key_c".to_string(), LineageId(3)),
        ];

        let count = indexer.rebuild(entries.into_iter()).unwrap();

        assert_eq!(count, 3);
        assert_eq!(indexer.len(), 3);
        assert!(!indexer.contains("old_key").unwrap()); // Old data cleared
        assert!(indexer.contains("key_a").unwrap());
    }

    #[test]
    fn test_indexer_len() {
        let indexer = temp_indexer();

        assert_eq!(indexer.len(), 0);
        assert!(indexer.is_empty());

        indexer.insert("a", LineageId(1)).unwrap();
        indexer.insert("b", LineageId(2)).unwrap();

        assert_eq!(indexer.len(), 2);
        assert!(!indexer.is_empty());
    }
}
