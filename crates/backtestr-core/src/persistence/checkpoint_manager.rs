//! Checkpoint management for state persistence

use super::compression::compress_data;
use super::serialization::{
    CheckpointData, CheckpointMetadata, MTFStateSnapshot, CHECKPOINT_VERSION,
};
use super::validation::calculate_checksum;
use crate::mtf::MTFStateManager;
use anyhow::{Context, Result};
use chrono::Utc;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::fs;

#[derive(Debug, Clone)]
pub enum CheckpointTrigger {
    TimeElapsed,
    TickCount,
    Manual,
    Shutdown,
}

pub struct CheckpointManager {
    checkpoint_dir: PathBuf,
    checkpoint_interval: Duration,
    compression_level: i32,
    max_checkpoints: usize,
    last_checkpoint: Instant,
    tick_count_since_checkpoint: u64,
    backtest_id: String,
}

impl CheckpointManager {
    pub fn new(
        checkpoint_dir: PathBuf,
        interval_secs: u64,
        compression_level: i32,
        max_checkpoints: usize,
    ) -> Result<Self> {
        std::fs::create_dir_all(&checkpoint_dir)
            .context("Failed to create checkpoint directory")?;

        Ok(Self {
            checkpoint_dir,
            checkpoint_interval: Duration::from_secs(interval_secs),
            compression_level,
            max_checkpoints,
            last_checkpoint: Instant::now(),
            tick_count_since_checkpoint: 0,
            backtest_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    pub fn should_checkpoint(&self) -> Option<CheckpointTrigger> {
        if self.last_checkpoint.elapsed() >= self.checkpoint_interval {
            return Some(CheckpointTrigger::TimeElapsed);
        }

        if self.tick_count_since_checkpoint >= 1_000_000 {
            return Some(CheckpointTrigger::TickCount);
        }

        None
    }

    pub fn increment_tick_count(&mut self) {
        self.tick_count_since_checkpoint += 1;
    }

    pub async fn create_checkpoint(
        &mut self,
        state: &MTFStateManager,
        tick_count: u64,
    ) -> Result<PathBuf> {
        let snapshot = state.create_snapshot()?;

        let metadata = CheckpointMetadata {
            created_at: Utc::now().timestamp_millis(),
            backtest_id: self.backtest_id.clone(),
            symbol_count: snapshot.symbol_states.len(),
            total_bars: calculate_total_bars(&snapshot),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let checkpoint_data = CheckpointData {
            version: CHECKPOINT_VERSION,
            timestamp: Utc::now().timestamp_millis(),
            tick_count,
            mtf_state: snapshot,
            indicator_states: Default::default(), // TODO: Get from state
            metadata,
            checksum: 0,
        };

        // Serialize with bincode (checksum will be calculated separately)
        let serialized = bincode::serialize(&checkpoint_data)?;

        // Calculate checksum of the data WITHOUT the checksum field
        // The checksum is stored separately and not part of what's checksummed
        let checksum = calculate_checksum(&serialized);

        // Compress the serialized data
        let compressed = compress_data(&serialized, self.compression_level)?;

        // Append checksum to the compressed data (8 bytes at the end)
        let mut final_data = compressed;
        final_data.extend_from_slice(&checksum.to_le_bytes());

        // Generate filename
        let filename = format!(
            "checkpoint_{}_{}.btck",
            self.backtest_id,
            Utc::now().format("%Y%m%d_%H%M%S")
        );
        let checkpoint_path = self.checkpoint_dir.join(&filename);

        // Atomic write with proper permissions
        let temp_path = checkpoint_path.with_extension("tmp");
        fs::write(&temp_path, final_data).await?;

        // Set file permissions (Windows-compatible)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&temp_path).await?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600); // Read/write for owner only
            fs::set_permissions(&temp_path, permissions).await?;
        }

        fs::rename(&temp_path, &checkpoint_path).await?;

        // Cleanup old checkpoints
        self.cleanup_old_checkpoints().await?;

        // Update tracking
        self.last_checkpoint = Instant::now();
        self.tick_count_since_checkpoint = 0;

        Ok(checkpoint_path)
    }

    pub async fn find_latest_checkpoint(&self) -> Result<Option<PathBuf>> {
        let mut entries = fs::read_dir(&self.checkpoint_dir).await?;
        let mut checkpoints = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("btck") {
                let metadata = entry.metadata().await?;
                checkpoints.push((path, metadata.modified()?));
            }
        }

        checkpoints.sort_by_key(|&(_, time)| time);
        Ok(checkpoints.last().map(|(path, _)| path.clone()))
    }

    async fn cleanup_old_checkpoints(&self) -> Result<()> {
        let mut entries = fs::read_dir(&self.checkpoint_dir).await?;
        let mut checkpoints = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("btck") {
                let metadata = entry.metadata().await?;
                checkpoints.push((path, metadata.modified()?));
            }
        }

        if checkpoints.len() <= self.max_checkpoints {
            return Ok(());
        }

        checkpoints.sort_by_key(|&(_, time)| time);
        let to_remove = checkpoints.len() - self.max_checkpoints;

        for (path, _) in checkpoints.iter().take(to_remove) {
            fs::remove_file(path).await?;
        }

        Ok(())
    }

    pub async fn list_checkpoints(&self) -> Result<Vec<(PathBuf, u64)>> {
        let mut entries = fs::read_dir(&self.checkpoint_dir).await?;
        let mut checkpoints = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("btck") {
                let metadata = entry.metadata().await?;
                checkpoints.push((path, metadata.len()));
            }
        }

        Ok(checkpoints)
    }
}

fn calculate_total_bars(snapshot: &MTFStateSnapshot) -> usize {
    snapshot
        .symbol_states
        .values()
        .flat_map(|state| state.bar_counts.values())
        .sum()
}
