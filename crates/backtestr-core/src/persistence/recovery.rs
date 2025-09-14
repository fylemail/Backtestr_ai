//! State recovery from checkpoints

use super::compression::decompress_data;
use super::serialization::{CheckpointData, CHECKPOINT_VERSION};
use super::validation::calculate_checksum;
use crate::mtf::MTFStateManager;
use anyhow::{bail, Context, Result};
use std::path::Path;
use tokio::fs;

pub struct StateRecovery {
    checkpoint_dir: std::path::PathBuf,
}

impl StateRecovery {
    pub fn new(checkpoint_dir: impl AsRef<Path>) -> Self {
        Self {
            checkpoint_dir: checkpoint_dir.as_ref().to_path_buf(),
        }
    }

    pub async fn recover_state(&self) -> Result<Option<(MTFStateManager, u64)>> {
        let checkpoint_path = match self.find_latest_valid_checkpoint().await? {
            Some(path) => path,
            None => return Ok(None),
        };

        let (state, tick_count) = self.load_checkpoint(&checkpoint_path).await?;
        Ok(Some((state, tick_count)))
    }

    pub async fn recover_from_specific(
        &self,
        checkpoint_file: &Path,
    ) -> Result<(MTFStateManager, u64)> {
        self.load_checkpoint(checkpoint_file).await
    }

    async fn load_checkpoint(&self, path: &Path) -> Result<(MTFStateManager, u64)> {
        // Read checkpoint file with checksum
        let file_data = fs::read(path)
            .await
            .context("Failed to read checkpoint file")?;

        // Extract checksum from the last 8 bytes
        if file_data.len() < 8 {
            bail!("Checkpoint file too small to contain checksum");
        }
        let (compressed, checksum_bytes) = file_data.split_at(file_data.len() - 8);
        let stored_checksum = u64::from_le_bytes(
            checksum_bytes
                .try_into()
                .context("Failed to read checksum")?,
        );

        // Decompress
        let decompressed =
            decompress_data(compressed).context("Failed to decompress checkpoint")?;

        // Validate checksum
        let calculated_checksum = calculate_checksum(&decompressed);
        if calculated_checksum != stored_checksum {
            bail!(
                "Checkpoint checksum validation failed: expected {}, got {}",
                stored_checksum,
                calculated_checksum
            );
        }

        // Deserialize
        let checkpoint: CheckpointData =
            bincode::deserialize(&decompressed).context("Failed to deserialize checkpoint")?;

        // Validate version
        if checkpoint.version != CHECKPOINT_VERSION {
            bail!(
                "Incompatible checkpoint version: expected {}, got {}",
                CHECKPOINT_VERSION,
                checkpoint.version
            );
        }

        // Reconstruct state
        let mut state = MTFStateManager::with_default_config();
        state
            .restore_from_snapshot(checkpoint.mtf_state)
            .context("Failed to restore MTF state")?;
        state
            .restore_indicators(checkpoint.indicator_states)
            .context("Failed to restore indicator states")?;

        Ok((state, checkpoint.tick_count))
    }

    async fn find_latest_valid_checkpoint(&self) -> Result<Option<std::path::PathBuf>> {
        let mut entries = fs::read_dir(&self.checkpoint_dir).await?;
        let mut checkpoints = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("btck") {
                let metadata = entry.metadata().await?;
                checkpoints.push((path, metadata.modified()?));
            }
        }

        // Sort by modification time (newest first)
        checkpoints.sort_by(|a, b| b.1.cmp(&a.1));

        // Try each checkpoint until we find a valid one
        for (path, _) in checkpoints {
            if self.validate_checkpoint(&path).await.is_ok() {
                return Ok(Some(path));
            }
        }

        Ok(None)
    }

    async fn validate_checkpoint(&self, path: &Path) -> Result<()> {
        let file_data = fs::read(path).await?;

        // Extract checksum from the last 8 bytes
        if file_data.len() < 8 {
            bail!("Checkpoint file too small");
        }
        let (compressed, checksum_bytes) = file_data.split_at(file_data.len() - 8);
        let stored_checksum = u64::from_le_bytes(checksum_bytes.try_into()?);

        // Decompress
        let decompressed = decompress_data(compressed)?;

        // Validate checksum
        let calculated_checksum = calculate_checksum(&decompressed);
        if calculated_checksum != stored_checksum {
            bail!("Invalid checksum");
        }

        // Deserialize and validate version
        let checkpoint: CheckpointData = bincode::deserialize(&decompressed)?;
        if checkpoint.version != CHECKPOINT_VERSION {
            bail!("Invalid version");
        }

        Ok(())
    }

    pub async fn list_available_checkpoints(&self) -> Result<Vec<CheckpointInfo>> {
        let mut entries = fs::read_dir(&self.checkpoint_dir).await?;
        let mut checkpoints = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("btck") {
                if let Ok(info) = self.get_checkpoint_info(&path).await {
                    checkpoints.push(info);
                }
            }
        }

        checkpoints.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(checkpoints)
    }

    async fn get_checkpoint_info(&self, path: &Path) -> Result<CheckpointInfo> {
        let file_data = fs::read(path).await?;

        // Extract checksum from the last 8 bytes
        if file_data.len() < 8 {
            bail!("Checkpoint file too small");
        }
        let (compressed, _) = file_data.split_at(file_data.len() - 8);

        let decompressed = decompress_data(compressed)?;
        let checkpoint: CheckpointData = bincode::deserialize(&decompressed)?;

        Ok(CheckpointInfo {
            path: path.to_path_buf(),
            created_at: checkpoint.metadata.created_at,
            tick_count: checkpoint.tick_count,
            symbol_count: checkpoint.metadata.symbol_count,
            total_bars: checkpoint.metadata.total_bars,
            file_size: file_data.len(),
            backtest_id: checkpoint.metadata.backtest_id,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CheckpointInfo {
    pub path: std::path::PathBuf,
    pub created_at: i64,
    pub tick_count: u64,
    pub symbol_count: usize,
    pub total_bars: usize,
    pub file_size: usize,
    pub backtest_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_recovery_no_checkpoints() {
        let dir = tempdir().unwrap();
        let recovery = StateRecovery::new(dir.path());

        let result = recovery.recover_state().await.unwrap();
        assert!(result.is_none());
    }
}
