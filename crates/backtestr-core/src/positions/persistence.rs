use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::interfaces::epic3_contracts::{PositionSnapshot, PositionStateStore};

/// Position state persistence implementation
pub struct PositionPersistence {
    /// Base directory for position snapshots
    base_dir: PathBuf,
    /// Maximum number of snapshots to keep
    max_snapshots: usize,
}

impl PositionPersistence {
    /// Create a new position persistence manager
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();

        // Ensure directory exists
        fs::create_dir_all(&base_dir)?;

        Ok(Self {
            base_dir,
            max_snapshots: 10,
        })
    }

    /// Set maximum number of snapshots to keep
    pub fn set_max_snapshots(&mut self, max: usize) {
        self.max_snapshots = max;
    }

    /// Get snapshot file path
    fn get_snapshot_path(&self, timestamp: i64) -> PathBuf {
        self.base_dir
            .join(format!("position_snapshot_{}.bin", timestamp))
    }

    /// Get all snapshot files sorted by timestamp
    fn get_snapshot_files(&self) -> Result<Vec<(i64, PathBuf)>> {
        let mut snapshots = Vec::new();

        for entry in fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(filename) = path.file_name() {
                if let Some(name_str) = filename.to_str() {
                    if name_str.starts_with("position_snapshot_") && name_str.ends_with(".bin") {
                        // Extract timestamp from filename
                        let timestamp_str = name_str
                            .trim_start_matches("position_snapshot_")
                            .trim_end_matches(".bin");

                        if let Ok(timestamp) = timestamp_str.parse::<i64>() {
                            snapshots.push((timestamp, path));
                        }
                    }
                }
            }
        }

        snapshots.sort_by_key(|&(ts, _)| ts);
        Ok(snapshots)
    }

    /// Clean up old snapshots
    fn cleanup_old_snapshots(&self) -> Result<()> {
        let snapshots = self.get_snapshot_files()?;

        if snapshots.len() > self.max_snapshots {
            let to_remove = snapshots.len() - self.max_snapshots;
            for (_, path) in snapshots.iter().take(to_remove) {
                fs::remove_file(path)?;
            }
        }

        Ok(())
    }
}

impl PositionStateStore for PositionPersistence {
    /// Save a position snapshot
    fn save_positions(&self, snapshot: &PositionSnapshot) -> Result<()> {
        let path = self.get_snapshot_path(snapshot.timestamp);

        // Serialize with compression
        let data = bincode::serialize(snapshot)?;
        let compressed = zstd::encode_all(&data[..], 3)?;

        // Write to file
        fs::write(&path, compressed)?;

        // Clean up old snapshots
        self.cleanup_old_snapshots()?;

        Ok(())
    }

    /// Restore positions from the last checkpoint
    fn restore_positions(&self) -> Result<PositionSnapshot> {
        let snapshots = self.get_snapshot_files()?;

        if snapshots.is_empty() {
            return Err(anyhow!("No position snapshots found"));
        }

        // Get the latest snapshot
        let (_, path) = snapshots.last().unwrap();

        // Read and decompress
        let compressed = fs::read(path)?;
        let data = zstd::decode_all(&compressed[..])?;

        // Deserialize
        let snapshot: PositionSnapshot = bincode::deserialize(&data)?;

        Ok(snapshot)
    }

    /// Check compatibility with MTF state version
    fn is_compatible_with_mtf(&self, mtf_version: &str) -> bool {
        // Simple version compatibility check
        // In production, this would be more sophisticated
        match mtf_version {
            "1.0.0" | "1.0.1" | "1.1.0" => true,
            _ => false,
        }
    }

    /// Clear all position snapshots
    fn clear_position_snapshots(&self) -> Result<()> {
        let snapshots = self.get_snapshot_files()?;

        for (_, path) in snapshots {
            fs::remove_file(path)?;
        }

        Ok(())
    }

    /// Get the latest snapshot timestamp
    fn get_latest_snapshot_time(&self) -> Option<i64> {
        self.get_snapshot_files()
            .ok()
            .and_then(|snapshots| snapshots.last().map(|(ts, _)| *ts))
    }
}

/// In-memory position state store for testing
#[cfg(test)]
pub struct InMemoryPositionStore {
    snapshots: std::sync::Arc<std::sync::RwLock<Vec<PositionSnapshot>>>,
}

#[cfg(test)]
impl InMemoryPositionStore {
    pub fn new() -> Self {
        Self {
            snapshots: std::sync::Arc::new(std::sync::RwLock::new(Vec::new())),
        }
    }
}

#[cfg(test)]
impl PositionStateStore for InMemoryPositionStore {
    fn save_positions(&self, snapshot: &PositionSnapshot) -> Result<()> {
        let mut snapshots = self.snapshots.write().unwrap();
        snapshots.push(snapshot.clone());
        Ok(())
    }

    fn restore_positions(&self) -> Result<PositionSnapshot> {
        let snapshots = self.snapshots.read().unwrap();
        snapshots
            .last()
            .cloned()
            .ok_or_else(|| anyhow!("No snapshots available"))
    }

    fn is_compatible_with_mtf(&self, _mtf_version: &str) -> bool {
        true
    }

    fn clear_position_snapshots(&self) -> Result<()> {
        let mut snapshots = self.snapshots.write().unwrap();
        snapshots.clear();
        Ok(())
    }

    fn get_latest_snapshot_time(&self) -> Option<i64> {
        let snapshots = self.snapshots.read().unwrap();
        snapshots.last().map(|s| s.timestamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_position_persistence() {
        let dir = tempdir().unwrap();
        let persistence = PositionPersistence::new(dir.path()).unwrap();

        // Create a snapshot
        let snapshot = PositionSnapshot {
            timestamp: 1234567890,
            version: "1.0.0".to_string(),
            positions_data: vec![1, 2, 3, 4, 5],
            account_balance: 10000.0,
            used_margin: 1000.0,
            floating_pnl: 50.0,
        };

        // Save snapshot
        persistence.save_positions(&snapshot).unwrap();

        // Restore snapshot
        let restored = persistence.restore_positions().unwrap();
        assert_eq!(restored.timestamp, snapshot.timestamp);
        assert_eq!(restored.version, snapshot.version);
        assert_eq!(restored.positions_data, snapshot.positions_data);
        assert_eq!(restored.account_balance, snapshot.account_balance);
    }

    #[test]
    fn test_multiple_snapshots() {
        let dir = tempdir().unwrap();
        let mut persistence = PositionPersistence::new(dir.path()).unwrap();
        persistence.set_max_snapshots(3);

        // Save multiple snapshots
        for i in 1..=5 {
            let snapshot = PositionSnapshot {
                timestamp: 1234567890 + i,
                version: "1.0.0".to_string(),
                positions_data: vec![i as u8],
                account_balance: 10000.0 + i as f64,
                used_margin: 1000.0,
                floating_pnl: 50.0,
            };
            persistence.save_positions(&snapshot).unwrap();
        }

        // Should only keep the last 3 snapshots
        let files = persistence.get_snapshot_files().unwrap();
        assert_eq!(files.len(), 3);

        // Latest should be the 5th snapshot
        let latest = persistence.restore_positions().unwrap();
        assert_eq!(latest.timestamp, 1234567895);
        assert_eq!(latest.account_balance, 10005.0);
    }

    #[test]
    fn test_clear_snapshots() {
        let dir = tempdir().unwrap();
        let persistence = PositionPersistence::new(dir.path()).unwrap();

        // Save a snapshot
        let snapshot = PositionSnapshot {
            timestamp: 1234567890,
            version: "1.0.0".to_string(),
            positions_data: vec![1, 2, 3],
            account_balance: 10000.0,
            used_margin: 1000.0,
            floating_pnl: 50.0,
        };
        persistence.save_positions(&snapshot).unwrap();

        // Clear all snapshots
        persistence.clear_position_snapshots().unwrap();

        // Should have no snapshots
        let result = persistence.restore_positions();
        assert!(result.is_err());
    }

    #[test]
    fn test_latest_snapshot_time() {
        let dir = tempdir().unwrap();
        let persistence = PositionPersistence::new(dir.path()).unwrap();

        // No snapshots initially
        assert_eq!(persistence.get_latest_snapshot_time(), None);

        // Save a snapshot
        let snapshot = PositionSnapshot {
            timestamp: 1234567890,
            version: "1.0.0".to_string(),
            positions_data: vec![1],
            account_balance: 10000.0,
            used_margin: 1000.0,
            floating_pnl: 50.0,
        };
        persistence.save_positions(&snapshot).unwrap();

        // Should return the timestamp
        assert_eq!(persistence.get_latest_snapshot_time(), Some(1234567890));
    }

    #[test]
    fn test_version_compatibility() {
        let dir = tempdir().unwrap();
        let persistence = PositionPersistence::new(dir.path()).unwrap();

        assert!(persistence.is_compatible_with_mtf("1.0.0"));
        assert!(persistence.is_compatible_with_mtf("1.0.1"));
        assert!(persistence.is_compatible_with_mtf("1.1.0"));
        assert!(!persistence.is_compatible_with_mtf("2.0.0"));
        assert!(!persistence.is_compatible_with_mtf("0.9.0"));
    }

    #[test]
    fn test_in_memory_store() {
        let store = InMemoryPositionStore::new();

        let snapshot = PositionSnapshot {
            timestamp: 1234567890,
            version: "1.0.0".to_string(),
            positions_data: vec![1, 2, 3],
            account_balance: 10000.0,
            used_margin: 1000.0,
            floating_pnl: 50.0,
        };

        // Save and restore
        store.save_positions(&snapshot).unwrap();
        let restored = store.restore_positions().unwrap();
        assert_eq!(restored.timestamp, snapshot.timestamp);

        // Get latest time
        assert_eq!(store.get_latest_snapshot_time(), Some(1234567890));

        // Clear
        store.clear_position_snapshots().unwrap();
        assert!(store.restore_positions().is_err());
    }
}
