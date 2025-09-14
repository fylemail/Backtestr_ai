use backtestr_core::{MTFConfig, MTFStateManager, StateQuery};
use backtestr_data::{Tick, Timeframe};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

fn generate_ticks(symbol: &str, count: usize) -> Vec<Tick> {
    let base_time = 1704067200000i64; // 2024-01-01 00:00:00
    (0..count)
        .map(|i| {
            let timestamp = base_time + (i as i64 * 100); // 100ms intervals
            let price = 1.0920 + (i as f64 * 0.00001);
            Tick::new_with_millis(symbol.to_string(), timestamp, price, price + 0.0002)
                .with_sizes(1000000, 1000000)
        })
        .collect()
}

fn bench_tick_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("tick_processing");
    group.measurement_time(Duration::from_secs(10));

    for timeframe_count in [1, 3, 6] {
        let timeframes = Timeframe::all().into_iter().take(timeframe_count).collect();
        let config = MTFConfig {
            enabled_timeframes: timeframes,
            ..Default::default()
        };

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("timeframes", timeframe_count),
            &timeframe_count,
            |b, _| {
                let manager = MTFStateManager::new(config.clone());
                let tick =
                    Tick::new_with_millis("EURUSD".to_string(), 1704067230000, 1.0920, 1.0922);

                b.iter(|| manager.process_tick(black_box(&tick)).unwrap());
            },
        );
    }

    group.finish();
}

fn bench_state_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_query");

    // Setup: Create manager with some data
    let manager = MTFStateManager::with_default_config();
    let ticks = generate_ticks("EURUSD", 1000);
    for tick in &ticks {
        manager.process_tick(tick).unwrap();
    }

    group.bench_function("snapshot", |b| {
        let query = StateQuery::new(&manager);
        b.iter(|| query.get_snapshot(black_box("EURUSD")));
    });

    group.bench_function("timeframe_snapshot", |b| {
        let query = StateQuery::new(&manager);
        b.iter(|| query.get_timeframe_snapshot(black_box("EURUSD"), black_box(Timeframe::M1)));
    });

    group.bench_function("partial_bars", |b| {
        let query = StateQuery::new(&manager);
        b.iter(|| query.get_all_partial_bars(black_box("EURUSD")));
    });

    group.finish();
}

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    group.measurement_time(Duration::from_secs(10));

    for tick_count in [100, 1000, 10000] {
        group.throughput(Throughput::Elements(tick_count as u64));
        group.bench_with_input(
            BenchmarkId::new("ticks", tick_count),
            &tick_count,
            |b, &count| {
                let ticks = generate_ticks("EURUSD", count);

                b.iter(|| {
                    let manager = MTFStateManager::with_default_config();
                    for tick in &ticks {
                        manager.process_tick(black_box(tick)).unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");

    group.bench_function("1m_ticks", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::ZERO;

            for _ in 0..iters {
                let manager = MTFStateManager::with_default_config();
                let ticks = generate_ticks("EURUSD", 1_000_000);

                let start = std::time::Instant::now();
                for tick in &ticks {
                    manager.process_tick(tick).unwrap();
                }
                let memory = manager.get_memory_usage_estimate();
                total_duration += start.elapsed();

                // Assert memory usage is under 1GB
                assert!(
                    memory < 1_000_000_000,
                    "Memory usage {} exceeds 1GB",
                    memory
                );
            }

            total_duration
        });
    });

    group.finish();
}

fn bench_multi_symbol(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_symbol");

    let symbols = vec!["EURUSD", "GBPUSD", "USDJPY", "AUDUSD", "USDCAD"];

    for symbol_count in [1, 3, 5] {
        group.bench_with_input(
            BenchmarkId::new("symbols", symbol_count),
            &symbol_count,
            |b, &count| {
                let manager = MTFStateManager::with_default_config();
                let mut all_ticks = Vec::new();

                for i in 0..count {
                    let ticks = generate_ticks(symbols[i], 100);
                    all_ticks.extend(ticks);
                }

                b.iter(|| {
                    for tick in &all_ticks {
                        manager.process_tick(black_box(tick)).unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_tick_processing,
    bench_state_query,
    bench_throughput,
    bench_memory_usage,
    bench_multi_symbol
);
criterion_main!(benches);
