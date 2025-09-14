use backtestr_core::mtf::{MTFConfig, MTFStateManager};
use backtestr_core::persistence::{
    CheckpointData, CheckpointManager, MTFStateSnapshot, PersistenceConfig, StateRecovery,
};
use backtestr_data::Tick;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;
use tempfile::tempdir;

fn bench_checkpoint_creation(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let dir = tempdir().unwrap();

    c.bench_function("checkpoint_creation", |b| {
        b.to_async(&runtime).iter(|| async {
            let mut manager = CheckpointManager::new(dir.path().to_path_buf(), 60, 6, 5).unwrap();
            let state = MTFStateManager::with_default_config();

            // Create state with some data
            for i in 0..100 {
                let tick = Tick::new_with_millis(
                    "EURUSD".to_string(),
                    1704067200000 + i * 1000,
                    1.0920 + (i as f64) * 0.0001,
                    1.0922 + (i as f64) * 0.0001,
                );
                state.process_tick(&tick).unwrap();
            }

            black_box(manager.create_checkpoint(&state, 1000).await.unwrap())
        });
    });
}

fn bench_state_recovery(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let dir = tempdir().unwrap();

    // Create a checkpoint first
    runtime.block_on(async {
        let mut manager = CheckpointManager::new(dir.path().to_path_buf(), 60, 6, 5).unwrap();
        let state = MTFStateManager::with_default_config();

        for i in 0..1000 {
            let tick = Tick::new_with_millis(
                "EURUSD".to_string(),
                1704067200000 + i * 1000,
                1.0920 + (i as f64) * 0.0001,
                1.0922 + (i as f64) * 0.0001,
            );
            state.process_tick(&tick).unwrap();
        }

        manager.create_checkpoint(&state, 10000).await.unwrap();
    });

    c.bench_function("state_recovery", |b| {
        b.to_async(&runtime).iter(|| async {
            let recovery = StateRecovery::new(dir.path());
            black_box(recovery.recover_state().await.unwrap())
        });
    });
}

fn bench_serialization(c: &mut Criterion) {
    let checkpoint = CheckpointData {
        version: 1,
        timestamp: 1704067200000,
        tick_count: 10000,
        mtf_state: MTFStateSnapshot {
            current_tick: Some(Tick::new_with_millis(
                "EURUSD".to_string(),
                1704067200000,
                1.0920,
                1.0922,
            )),
            symbol_states: Default::default(),
            partial_bars: Default::default(),
            completed_bar_ids: Default::default(),
            last_processed_timestamp: 1704067200000,
        },
        indicator_states: Default::default(),
        metadata: backtestr_core::persistence::serialization::CheckpointMetadata {
            created_at: 1704067200000,
            backtest_id: "bench-123".to_string(),
            symbol_count: 10,
            total_bars: 1000,
            engine_version: "1.0.0".to_string(),
        },
        checksum: 0,
    };

    c.bench_function("checkpoint_serialization", |b| {
        b.iter(|| {
            let serialized = bincode::serialize(&checkpoint).unwrap();
            black_box(serialized)
        });
    });
}

fn bench_compression(c: &mut Criterion) {
    use backtestr_core::persistence::compression::compress_data;

    // Create realistic checkpoint data
    let data = vec![0u8; 100_000]; // 100KB of data

    c.bench_function("checkpoint_compression", |b| {
        b.iter(|| {
            let compressed = compress_data(&data, 6).unwrap();
            black_box(compressed)
        });
    });
}

fn bench_checksum(c: &mut Criterion) {
    use backtestr_core::persistence::validation::calculate_checksum;

    let data = vec![0u8; 10_000]; // 10KB of data

    c.bench_function("checksum_calculation", |b| {
        b.iter(|| {
            let checksum = calculate_checksum(&data);
            black_box(checksum)
        });
    });
}

fn bench_checkpoint_overhead(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let dir = tempdir().unwrap();

    let state = MTFStateManager::with_default_config();

    // Benchmark without checkpointing
    c.bench_function("tick_processing_without_checkpoint", |b| {
        b.iter(|| {
            let tick = Tick::new_with_millis("EURUSD".to_string(), 1704067200000, 1.0920, 1.0922);
            black_box(state.process_tick(&tick).unwrap())
        });
    });

    // Benchmark with checkpointing overhead
    c.bench_function("tick_processing_with_checkpoint_check", |b| {
        b.to_async(&runtime).iter(|| async {
            let manager = CheckpointManager::new(dir.path().to_path_buf(), 60, 6, 5).unwrap();

            let tick = Tick::new_with_millis("EURUSD".to_string(), 1704067200000, 1.0920, 1.0922);
            let result = state.process_tick(&tick).unwrap();

            // Check if checkpoint needed (overhead)
            let _ = manager.should_checkpoint();

            black_box(result)
        });
    });
}

criterion_group! {
    name = persistence_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100);
    targets = bench_checkpoint_creation,
              bench_state_recovery,
              bench_serialization,
              bench_compression,
              bench_checksum,
              bench_checkpoint_overhead
}

criterion_main!(persistence_benches);
