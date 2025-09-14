# Story 2.4: MTF State Persistence & Recovery

**Epic:** Epic 2 - Multi-Timeframe Synchronization Engine
**Story ID:** STORY-2.4
**Status:** Blocked by Stories 2.1, 2.2, 2.3
**Branch:** `story/STORY-2.4-state-persistence`

## Story Description

**As a** user,
**I want** MTF state to persist and recover,
**So that** interrupted backtests can resume without reprocessing all historical data.

## Background & Context

Long backtests can take hours or days. If interrupted (crash, power failure, user pause), we need to resume from the exact tick without reprocessing. This story implements state serialization, checkpointing, and recovery mechanisms for the entire MTF engine state including indicators.

## Acceptance Criteria

### Must Have
1. ✅ **State Serialization**
   - [ ] MTF state fully serializable to disk
   - [ ] Indicator states included in serialization
   - [ ] Partial bar states preserved
   - [ ] Event queue state captured

2. ✅ **Automatic Checkpointing**
   - [ ] Checkpoint every 60 seconds by default
   - [ ] Configurable checkpoint interval
   - [ ] Non-blocking checkpoint writes
   - [ ] Checkpoint validation after write

3. ✅ **State Recovery**
   - [ ] Restore exact tick-level position
   - [ ] Indicator values restored without recalculation
   - [ ] Partial bars restored with correct progress
   - [ ] Resume processing from next tick

4. ✅ **Performance Requirements**
   - [ ] Recovery time <1 second for typical state
   - [ ] Checkpoint write <100ms
   - [ ] Minimal impact on tick processing (<5% overhead)
   - [ ] State file size <100MB for typical backtest

5. ✅ **Reliability Features**
   - [ ] Corruption detection with checksums
   - [ ] Fallback to previous valid checkpoint
   - [ ] Atomic writes (no partial states)
   - [ ] Version compatibility checks

6. ✅ **Storage Strategy**
   - [ ] SQLite for historical bars (persistent)
   - [ ] Binary files for MTF state snapshots
   - [ ] Compressed state files to minimize disk usage
   - [ ] Cleanup of old checkpoints

### Nice to Have
- [ ] Multiple checkpoint retention
- [ ] Incremental checkpoints
- [ ] Cloud storage support
- [ ] State diff/comparison tools

## Technical Design

### Persistence Architecture
```
┌─────────────────────────┐
│    MTF State Manager    │
└───────────┬─────────────┘
            │
    ┌───────▼────────┐
    │ Checkpoint     │
    │ Manager        │
    └───────┬────────┘
            │
    ┌───────▼────────┐
    │ Serialization  │
    │ Engine         │
    └───────┬────────┘
            │
    ┌───────▼────────────────┐
    │                         │
    ▼                         ▼
┌────────┐            ┌──────────┐
│SQLite  │            │Binary    │
│(Bars)  │            │Snapshots │
└────────┘            └──────────┘
```

### File Structure
```
crates/backtestr-core/src/persistence/
├── mod.rs
├── checkpoint_manager.rs   # Checkpoint orchestration
├── serialization.rs        # State serialization
├── recovery.rs            # State recovery logic
├── compression.rs         # Compression utilities
└── validation.rs          # Corruption detection

data/checkpoints/
├── checkpoint_20240115_143022.btck  # Checkpoint files
├── checkpoint_20240115_144022.btck
└── checkpoint_metadata.json         # Checkpoint index
```

### Core Components

```rust
use serde::{Serialize, Deserialize};
use bincode;
use flate2::Compression;

#[derive(Serialize, Deserialize)]
pub struct CheckpointData {
    version: u32,
    timestamp: i64,
    tick_count: u64,
    mtf_state: MTFStateSnapshot,
    indicator_states: HashMap<String, IndicatorSnapshot>,
    metadata: CheckpointMetadata,
    checksum: u64,
}

#[derive(Serialize, Deserialize)]
pub struct MTFStateSnapshot {
    current_tick: Option<Tick>,
    symbol_states: HashMap<String, SymbolMTFState>,
    partial_bars: HashMap<(String, Timeframe), PartialBar>,
    completed_bar_ids: HashMap<(String, Timeframe), Vec<i64>>,
}

pub struct CheckpointManager {
    checkpoint_interval: Duration,
    checkpoint_path: PathBuf,
    compression_level: Compression,
    max_checkpoints: usize,
    last_checkpoint: Instant,
    tick_count_since_checkpoint: u64,
}

// Checkpoint triggers
pub enum CheckpointTrigger {
    TimeElapsed,       // 60 seconds (configurable)
    TickCount,         // 1M ticks processed
    Manual,            // User requested
    Shutdown,          // Graceful shutdown
}

impl CheckpointManager {
    pub fn should_checkpoint(&self) -> Option<CheckpointTrigger> {
        // Check time trigger
        if self.last_checkpoint.elapsed() >= self.checkpoint_interval {
            return Some(CheckpointTrigger::TimeElapsed);
        }

        // Check tick count trigger
        if self.tick_count_since_checkpoint >= 1_000_000 {
            return Some(CheckpointTrigger::TickCount);
        }

        None
    }
}

impl CheckpointManager {
    pub async fn create_checkpoint(&self, state: &MTFStateManager) -> Result<()> {
        // Serialize state
        let snapshot = state.create_snapshot()?;
        let data = CheckpointData {
            version: CHECKPOINT_VERSION,
            timestamp: Utc::now().timestamp_millis(),
            tick_count: state.tick_count(),
            mtf_state: snapshot.mtf_state,
            indicator_states: snapshot.indicators,
            metadata: self.create_metadata(),
            checksum: 0, // Calculate after serialization
        };

        // Serialize with bincode
        let serialized = bincode::serialize(&data)?;

        // Calculate checksum
        let checksum = calculate_checksum(&serialized);

        // Compress
        let compressed = compress_data(&serialized, self.compression_level)?;

        // Atomic write
        let temp_path = self.checkpoint_path.with_extension(".tmp");
        tokio::fs::write(&temp_path, compressed).await?;
        tokio::fs::rename(temp_path, self.checkpoint_path).await?;

        // Cleanup old checkpoints
        self.cleanup_old_checkpoints().await?;

        Ok(())
    }

    pub async fn recover_state(&self) -> Result<MTFStateManager> {
        // Find latest valid checkpoint
        let checkpoint_file = self.find_latest_checkpoint()?;

        // Read and decompress
        let compressed = tokio::fs::read(&checkpoint_file).await?;
        let decompressed = decompress_data(&compressed)?;

        // Deserialize
        let checkpoint: CheckpointData = bincode::deserialize(&decompressed)?;

        // Validate checksum
        if !validate_checksum(&decompressed, checkpoint.checksum) {
            return Err(Error::CorruptedCheckpoint);
        }

        // Validate version
        if checkpoint.version != CHECKPOINT_VERSION {
            return Err(Error::IncompatibleVersion);
        }

        // Reconstruct state
        let mut state = MTFStateManager::new();
        state.restore_from_snapshot(checkpoint.mtf_state)?;
        state.restore_indicators(checkpoint.indicator_states)?;

        // Load bars from SQLite that were created after checkpoint
        let bars = load_bars_after(checkpoint.timestamp)?;
        state.replay_bars(bars)?;

        Ok(state)
    }
}
```

### Checkpoint File Format
```
BTCK Header (16 bytes):
- Magic: "BTCK" (4 bytes)
- Version: u32 (4 bytes)
- Timestamp: i64 (8 bytes)

Compressed Data:
- Serialized CheckpointData (bincode format)
- ZSTD compressed

Footer (8 bytes):
- Checksum: u64 (xxHash)
```

## Dependencies

- **Story 2.1:** MTF State (needs complete state to persist)
- **Story 2.2:** Indicators (needs indicator state)
- **Story 2.3:** Bar Formation (needs bar state)
- **Blocks:** None (final story in Epic 2)

## Implementation Steps

1. **Phase 1: Serialization**
   - Implement state snapshots
   - Add serde derives
   - Test serialization round-trip

2. **Phase 2: Checkpoint Manager**
   - Implement checkpoint creation
   - Add compression
   - Atomic file operations

3. **Phase 3: Recovery**
   - Implement state recovery
   - Add validation checks
   - Test corruption scenarios

4. **Phase 4: Integration**
   - Integrate with MTF engine
   - Add automatic checkpointing
   - Performance optimization

## Definition of Done

- [ ] State serialization complete
- [ ] Checkpoint manager functional
- [ ] Recovery mechanism tested
- [ ] Corruption detection working
- [ ] Performance targets met
- [ ] Integration tests passing
- [ ] Failure recovery tested
- [ ] Documentation complete
- [ ] Code reviewed
- [ ] CI/CD passing

## Performance Benchmarks

```rust
// Checkpoint creation time
// Target: <100ms
bench_checkpoint_creation()

// State recovery time
// Target: <1 second
bench_state_recovery()

// Checkpoint file size
// Target: <100MB
bench_checkpoint_size()

// Processing overhead
// Target: <5% performance impact
bench_checkpoint_overhead()
```

## Risk Assessment

1. **Risk:** State corruption leading to data loss
   - **Mitigation:** Checksums, atomic writes, multiple checkpoints
   - **Testing:** Corruption injection tests

2. **Risk:** Recovery failure after version upgrade
   - **Mitigation:** Version compatibility checks, migration tools
   - **Testing:** Version upgrade scenarios

3. **Risk:** Disk space exhaustion
   - **Mitigation:** Automatic cleanup, compression, limits
   - **Testing:** Long-running space tests

## Testing Strategy

### Serialization Tests
- Round-trip all state types
- Test with large states
- Verify data integrity

### Recovery Tests
- Interrupt at various points
- Recover and continue
- Verify state consistency

### Failure Tests
- Corrupt checkpoint files
- Missing checkpoints
- Disk full scenarios

### Performance Tests
- Checkpoint creation time
- Recovery time
- Impact on tick processing

## Notes

- Consider using ZSTD for better compression
- SQLite remains source of truth for bars
- Checkpoints are for MTF state only
- Future: Consider distributed checkpointing