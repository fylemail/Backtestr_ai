//! State persistence and recovery for MTF engine
//!
//! Provides checkpoint management, serialization, and recovery mechanisms
//! to support resumable backtests with <1 second recovery time.

pub mod checkpoint_manager;
pub mod compression;
pub mod recovery;
pub mod serialization;
pub mod validation;

pub use checkpoint_manager::{CheckpointManager, CheckpointTrigger};
pub use recovery::StateRecovery;
pub use serialization::{CheckpointData, MTFStateSnapshot};
pub use validation::ChecksumValidator;

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    pub checkpoint_dir: PathBuf,
    pub checkpoint_interval_secs: u64,
    pub max_checkpoints: usize,
    pub compression_level: i32,
    pub enable_auto_checkpoint: bool,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            checkpoint_dir: PathBuf::from("data/checkpoints"),
            checkpoint_interval_secs: 60,
            max_checkpoints: 5,
            compression_level: 6,
            enable_auto_checkpoint: true,
        }
    }
}
