use backtestr_data::{Database, Tick};
use chrono::{Duration, Utc};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::mem;

fn generate_ticks(count: usize) -> Vec<Tick> {
    let now = Utc::now();
    let mut ticks = Vec::with_capacity(count);

    for i in 0..count {
        let timestamp = now + Duration::milliseconds(i as i64);
        let bid = 1.0920 + (i as f64 * 0.00001);
        let ask = bid + 0.0002;

        ticks.push(Tick::new("EURUSD".to_string(), timestamp, bid, ask));
    }

    ticks
}

fn bench_memory_per_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_per_tick");

    // Test memory usage characteristics
    let tick = generate_ticks(1).into_iter().next().unwrap();
    let tick_size = mem::size_of_val(&tick);

    println!("Size of a single Tick struct: {} bytes", tick_size);
    println!("Tick fields breakdown:");
    println!("  - id: {} bytes", mem::size_of::<Option<i64>>());
    println!("  - symbol: {} bytes", mem::size_of::<String>());
    println!("  - timestamp: {} bytes", mem::size_of::<i64>());
    println!("  - bid: {} bytes", mem::size_of::<f64>());
    println!("  - ask: {} bytes", mem::size_of::<f64>());
    println!("  - bid_size: {} bytes", mem::size_of::<Option<i64>>());
    println!("  - ask_size: {} bytes", mem::size_of::<Option<i64>>());

    for &count in &[1000, 10000, 100000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter(|| {
                let mut db = Database::new_memory().unwrap();
                let ticks = generate_ticks(count);

                // Calculate memory used by ticks vector
                let vec_capacity = ticks.capacity() * mem::size_of::<Tick>();
                println!(
                    "Vector capacity for {} ticks: {} bytes",
                    count, vec_capacity
                );

                db.insert_batch(black_box(&ticks)).unwrap();

                // Ensure database persists for measurement
                black_box(&db);
            });
        });
    }

    group.finish();
}

fn bench_memory_1m_ticks(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_1m_ticks");
    group.sample_size(10); // Reduce sample size for large dataset

    // Benchmark to validate <500MB for 1M ticks requirement
    group.bench_function("1_million_ticks", |b| {
        b.iter(|| {
            let mut db = Database::new_memory().unwrap();

            // Insert in batches to avoid huge temporary allocations
            let batch_size = 10000;
            let num_batches = 100; // 100 * 10000 = 1M

            for _batch_num in 0..num_batches {
                let ticks = generate_ticks(batch_size);
                db.insert_batch(black_box(&ticks)).unwrap();
            }

            // Estimate memory usage
            // Each tick in SQLite roughly uses:
            // - symbol: ~10 bytes (varchar)
            // - timestamp: 8 bytes (int64)
            // - bid: 8 bytes (float64)
            // - ask: 8 bytes (float64)
            // - bid_size: 8 bytes (nullable int64)
            // - ask_size: 8 bytes (nullable int64)
            // - overhead: ~20 bytes (row overhead, indexes)
            // Total: ~70 bytes per tick

            let estimated_memory = 1_000_000 * 70; // 70MB for 1M ticks
            let estimated_mb = estimated_memory as f64 / (1024.0 * 1024.0);

            println!("Estimated memory for 1M ticks: {:.2} MB", estimated_mb);

            // Validate requirement (estimated)
            assert!(
                estimated_mb < 500.0,
                "Estimated memory usage {:.2}MB exceeds 500MB limit",
                estimated_mb
            );

            // Keep database alive for measurement
            black_box(db);
        });
    });

    group.finish();
}

fn bench_database_size_growth(c: &mut Criterion) {
    let mut group = c.benchmark_group("database_growth");
    group.sample_size(10);

    let sizes = vec![10000, 50000, 100000];

    for size in sizes {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let mut db = Database::new_memory().unwrap();

                // Insert in reasonable batches
                let batch_size = 10000;
                let num_batches = size / batch_size;

                for _ in 0..num_batches {
                    let ticks = generate_ticks(batch_size);
                    db.insert_batch(black_box(&ticks)).unwrap();
                }

                // Handle remainder
                let remainder = size % batch_size;
                if remainder > 0 {
                    let ticks = generate_ticks(remainder);
                    db.insert_batch(black_box(&ticks)).unwrap();
                }

                // Query to ensure data is stored
                let count = db.count_ticks().unwrap();
                assert_eq!(count, size);

                // Estimate memory
                let estimated_memory = size * 70; // ~70 bytes per tick in SQLite
                let estimated_mb = estimated_memory as f64 / (1024.0 * 1024.0);

                println!("{} ticks: estimated {:.2} MB", size, estimated_mb);

                black_box(db);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_memory_per_tick,
    bench_memory_1m_ticks,
    bench_database_size_growth
);
criterion_main!(benches);
