use backtestr_data::{Database, Tick};
use chrono::{Duration, Utc};
use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};

fn generate_ticks(count: usize) -> Vec<Tick> {
    let now = Utc::now();
    let mut ticks = Vec::with_capacity(count);

    for i in 0..count {
        let timestamp = now + Duration::milliseconds(i as i64);
        let bid = 1.0920 + (i as f64 * 0.0001);
        let ask = bid + 0.0002;

        ticks.push(Tick::new("EURUSD".to_string(), timestamp, bid, ask));
    }

    ticks
}

fn bench_single_tick_insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("tick_insertion");

    group.bench_function("single_tick", |b| {
        b.iter_batched(
            || {
                let db = Database::new_memory().unwrap();
                let tick = generate_ticks(1).into_iter().next().unwrap();
                (db, tick)
            },
            |(db, tick)| {
                db.insert_tick(black_box(&tick)).unwrap();
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_batch_insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_insertion");

    for size in &[100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || {
                    let db = Database::new_memory().unwrap();
                    let ticks = generate_ticks(size);
                    (db, ticks)
                },
                |(mut db, ticks)| {
                    db.insert_batch(black_box(&ticks)).unwrap();
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("insertion_throughput");

    // Benchmark to validate 10K ticks/second requirement
    group.bench_function("10k_ticks", |b| {
        b.iter_batched(
            || {
                let db = Database::new_memory().unwrap();
                let ticks = generate_ticks(10000);
                (db, ticks)
            },
            |(mut db, ticks)| {
                // Use batch insertion for optimal performance
                db.insert_batch(black_box(&ticks)).unwrap();
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_prepared_statement_insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("prepared_statement");

    group.bench_function("1000_ticks_prepared", |b| {
        b.iter_batched(
            || {
                let db = Database::new_memory().unwrap();
                let ticks = generate_ticks(1000);
                (db, ticks)
            },
            |(db, ticks)| {
                db.insert_ticks(black_box(&ticks)).unwrap();
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_single_tick_insertion,
    bench_batch_insertion,
    bench_throughput,
    bench_prepared_statement_insertion
);
criterion_main!(benches);
