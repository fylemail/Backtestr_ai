use backtestr_data::{CsvImporter, Database};
use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_csv(rows: usize, dir: &TempDir) -> PathBuf {
    let file_path = dir.path().join(format!("test_{}.csv", rows));
    let mut file = File::create(&file_path).unwrap();

    // Write header
    writeln!(file, "symbol,timestamp,bid,ask,bid_size,ask_size").unwrap();

    // Write data rows
    let base_time = Utc::now();
    for i in 0..rows {
        let timestamp = base_time.timestamp_millis() + (i as i64 * 1000);
        let bid = 1.0920 + (i as f64 * 0.0001);
        let ask = bid + 0.0002;
        writeln!(file, "EURUSD,{},{},{},1000000,1000000", timestamp, bid, ask).unwrap();
    }

    file_path
}

fn bench_csv_import_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("csv_import_sizes");
    group.sample_size(10); // Reduce sample size for larger files

    for &rows in &[1000, 5000, 10000] {
        group.bench_with_input(BenchmarkId::from_parameter(rows), &rows, |b, &rows| {
            b.iter_batched_ref(
                || {
                    let temp_dir = TempDir::new().unwrap();
                    let csv_path = create_test_csv(rows, &temp_dir);
                    let db = Database::new_memory().unwrap();
                    let importer = CsvImporter::new(db);
                    (importer, csv_path, temp_dir)
                },
                |(ref mut importer, csv_path, _temp_dir)| {
                    let summary = importer.import_file(black_box(csv_path)).unwrap();
                    black_box(summary);
                },
                criterion::BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

fn bench_csv_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("csv_import_throughput");
    group.sample_size(10);

    // Benchmark to validate 10K rows/second requirement
    group.bench_function("10k_rows", |b| {
        b.iter_batched_ref(
            || {
                let temp_dir = TempDir::new().unwrap();
                let csv_path = create_test_csv(10000, &temp_dir);
                let db = Database::new_memory().unwrap();
                let importer = CsvImporter::new(db);
                (importer, csv_path, temp_dir)
            },
            |(importer, csv_path, _temp_dir)| {
                let summary = importer.import_file(black_box(csv_path)).unwrap();
                assert_eq!(summary.rows_imported, 10000);
                black_box(summary);
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.finish();
}

fn bench_csv_with_invalid_rows(c: &mut Criterion) {
    let mut group = c.benchmark_group("csv_import_validation");
    group.sample_size(10);

    group.bench_function("5k_rows_with_10pct_invalid", |b| {
        b.iter_batched_ref(
            || {
                let temp_dir = TempDir::new().unwrap();
                let file_path = temp_dir.path().join("mixed_data.csv");
                let mut file = File::create(&file_path).unwrap();

                // Write header
                writeln!(file, "symbol,timestamp,bid,ask,bid_size,ask_size").unwrap();

                // Write mixed valid and invalid data
                let base_time = Utc::now();
                for i in 0..5000 {
                    if i % 10 == 0 {
                        // Invalid row (negative price)
                        writeln!(
                            file,
                            "EURUSD,{},-1.0,1.0923,,",
                            base_time.timestamp_millis()
                        )
                        .unwrap();
                    } else {
                        // Valid row
                        let timestamp = base_time.timestamp_millis() + (i as i64 * 1000);
                        let bid = 1.0920 + (i as f64 * 0.0001);
                        let ask = bid + 0.0002;
                        writeln!(file, "EURUSD,{},{},{},1000000,1000000", timestamp, bid, ask)
                            .unwrap();
                    }
                }

                let db = Database::new_memory().unwrap();
                let importer = CsvImporter::new(db);
                (importer, file_path, temp_dir)
            },
            |(importer, csv_path, _temp_dir)| {
                let summary = importer.import_file(black_box(csv_path)).unwrap();
                black_box(summary);
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_csv_import_sizes,
    bench_csv_throughput,
    bench_csv_with_invalid_rows
);
criterion_main!(benches);
