use backtestr_data::{Database, Tick};
use chrono::{Duration, Utc};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn setup_test_database(tick_count: usize) -> Database {
    let mut db = Database::new_memory().unwrap();
    let now = Utc::now();
    let mut ticks = Vec::with_capacity(tick_count);

    // Generate ticks spread over 24 hours
    for i in 0..tick_count {
        let timestamp =
            now - Duration::hours(24) + Duration::milliseconds((i * 86400000 / tick_count) as i64);
        let bid = 1.0920 + (i as f64 * 0.0001);
        let ask = bid + 0.0002;

        ticks.push(Tick::new("EURUSD".to_string(), timestamp, bid, ask));
    }

    // Also add some GBPUSD ticks for testing symbol filtering
    for i in 0..(tick_count / 10) {
        let timestamp = now - Duration::hours(24)
            + Duration::milliseconds((i * 86400000 / (tick_count / 10)) as i64);
        let bid = 1.2500 + (i as f64 * 0.0001);
        let ask = bid + 0.0003;

        ticks.push(Tick::new("GBPUSD".to_string(), timestamp, bid, ask));
    }

    db.insert_batch(&ticks).unwrap();
    db
}

fn bench_query_by_symbol(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_by_symbol");

    for &tick_count in &[1000, 10000, 100000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(tick_count),
            &tick_count,
            |b, &tick_count| {
                let db = setup_test_database(tick_count);
                let now = Utc::now();
                let start = now - Duration::hours(25);
                let end = now + Duration::hours(1);

                b.iter(|| {
                    let results = db
                        .query_ticks(black_box("EURUSD"), black_box(start), black_box(end))
                        .unwrap();
                    black_box(results);
                });
            },
        );
    }

    group.finish();
}

fn bench_query_time_ranges(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_time_ranges");
    let db = setup_test_database(100000);
    let now = Utc::now();

    // Test different time range sizes
    let ranges = vec![
        ("1_hour", Duration::hours(1)),
        ("6_hours", Duration::hours(6)),
        ("12_hours", Duration::hours(12)),
        ("24_hours", Duration::hours(24)),
    ];

    for (name, duration) in ranges {
        group.bench_function(name, |b| {
            let start = now - duration;
            let end = now;

            b.iter(|| {
                let results = db
                    .query_ticks(black_box("EURUSD"), black_box(start), black_box(end))
                    .unwrap();
                black_box(results);
            });
        });
    }

    group.finish();
}

fn bench_query_response_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_response_time");

    // Benchmark to validate <100ms requirement for basic queries
    group.bench_function("basic_query_100k_ticks", |b| {
        let db = setup_test_database(100000);
        let now = Utc::now();
        let start = now - Duration::hours(1);
        let end = now;

        b.iter(|| {
            let results = db
                .query_ticks(black_box("EURUSD"), black_box(start), black_box(end))
                .unwrap();
            black_box(results);
        });
    });

    group.finish();
}

fn bench_count_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("count_operations");

    for &tick_count in &[1000, 10000, 100000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(tick_count),
            &tick_count,
            |b, &tick_count| {
                let db = setup_test_database(tick_count);

                b.iter(|| {
                    let count = db.count_ticks().unwrap();
                    black_box(count);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_query_by_symbol,
    bench_query_time_ranges,
    bench_query_response_time,
    bench_count_operations
);
criterion_main!(benches);
