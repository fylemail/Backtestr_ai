use backtestr_data::{Database, Tick};
use chrono::{Duration, Utc};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn generate_ticks(count: usize) -> Vec<Tick> {
    let now = Utc::now();
    (0..count)
        .map(|i| {
            Tick::new(
                "EURUSD".to_string(),
                now + Duration::seconds(i as i64),
                1.0920 + (i as f64) * 0.0001,
                1.0922 + (i as f64) * 0.0001,
            )
            .with_sizes(1000000, 1000000)
        })
        .collect()
}

fn bench_insert_ticks(c: &mut Criterion) {
    let ticks_10k = generate_ticks(10_000);

    c.bench_function("insert_10k_ticks", |b| {
        b.iter(|| {
            let db = Database::new_memory().unwrap();
            db.insert_ticks(black_box(&ticks_10k)).unwrap();
        });
    });
}

fn bench_query_ticks(c: &mut Criterion) {
    let db = Database::new_memory().unwrap();
    let ticks = generate_ticks(10_000);
    db.insert_ticks(&ticks).unwrap();

    let now = Utc::now();

    c.bench_function("query_10k_ticks", |b| {
        b.iter(|| {
            let result = db
                .query_ticks(
                    black_box("EURUSD"),
                    black_box(now),
                    black_box(now + Duration::seconds(10_000)),
                )
                .unwrap();
            black_box(result);
        });
    });
}

fn bench_memory_usage(c: &mut Criterion) {
    c.bench_function("memory_100k_ticks", |b| {
        b.iter(|| {
            let db = Database::new_memory().unwrap();
            let ticks = generate_ticks(100_000);
            db.insert_ticks(&ticks).unwrap();

            // Verify we can query them
            let count = db.count_ticks().unwrap();
            assert_eq!(count, 100_000);
            black_box(db);
        });
    });
}

criterion_group!(
    benches,
    bench_insert_ticks,
    bench_query_ticks,
    bench_memory_usage
);
criterion_main!(benches);
