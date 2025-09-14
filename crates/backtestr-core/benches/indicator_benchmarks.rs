use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use backtestr_core::indicators::*;
use backtestr_data::Timeframe;
use std::time::Duration;

fn generate_bar_data(count: usize) -> Vec<BarData> {
    let mut bars = Vec::with_capacity(count);
    let mut price = 100.0;

    for i in 0..count {
        // Add some realistic price movement
        price += (i as f64 * 0.01).sin() * 2.0;

        bars.push(BarData {
            open: price,
            high: price + 1.0,
            low: price - 1.0,
            close: price + 0.5,
            volume: 10000.0 + (i as f64 * 100.0),
            timestamp: i as i64,
        });
    }

    bars
}

fn bench_individual_indicators(c: &mut Criterion) {
    let bars = generate_bar_data(1000);

    let mut group = c.benchmark_group("individual_indicators");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark SMA
    group.bench_function("SMA_20", |b| {
        let mut sma = SMA::new(20);
        let mut idx = 0;
        b.iter(|| {
            let result = sma.update(black_box(bars[idx % bars.len()].clone()));
            idx += 1;
            black_box(result)
        });
    });

    // Benchmark EMA
    group.bench_function("EMA_20", |b| {
        let mut ema = EMA::new(20);
        let mut idx = 0;
        b.iter(|| {
            let result = ema.update(black_box(bars[idx % bars.len()].clone()));
            idx += 1;
            black_box(result)
        });
    });

    // Benchmark RSI
    group.bench_function("RSI_14", |b| {
        let mut rsi = RSI::new(14);
        let mut idx = 0;
        b.iter(|| {
            let result = rsi.update(black_box(bars[idx % bars.len()].clone()));
            idx += 1;
            black_box(result)
        });
    });

    // Benchmark MACD
    group.bench_function("MACD", |b| {
        let mut macd = MACD::new(12, 26, 9);
        let mut idx = 0;
        b.iter(|| {
            let result = macd.update(black_box(bars[idx % bars.len()].clone()));
            idx += 1;
            black_box(result)
        });
    });

    // Benchmark Bollinger Bands
    group.bench_function("BollingerBands_20", |b| {
        let mut bb = BollingerBands::new(20, 2.0);
        let mut idx = 0;
        b.iter(|| {
            let result = bb.update(black_box(bars[idx % bars.len()].clone()));
            idx += 1;
            black_box(result)
        });
    });

    // Benchmark ADX (new full implementation)
    group.bench_function("ADX_14", |b| {
        let mut adx = ADX::new(14);
        let mut idx = 0;
        b.iter(|| {
            let result = adx.update(black_box(bars[idx % bars.len()].clone()));
            idx += 1;
            black_box(result)
        });
    });

    // Benchmark Parabolic SAR (new full implementation)
    group.bench_function("ParabolicSAR", |b| {
        let mut sar = ParabolicSAR::new(0.02, 0.2);
        let mut idx = 0;
        b.iter(|| {
            let result = sar.update(black_box(bars[idx % bars.len()].clone()));
            idx += 1;
            black_box(result)
        });
    });

    group.finish();
}

fn bench_pipeline_update_all(c: &mut Criterion) {
    let bars = generate_bar_data(100);

    let mut group = c.benchmark_group("pipeline_update");
    group.measurement_time(Duration::from_secs(10));

    for num_indicators in [5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_indicators),
            num_indicators,
            |b, &num| {
                let pipeline = IndicatorPipeline::new(1000);

                // Register indicators
                for i in 0..num {
                    match i % 7 {
                        0 => pipeline.register_indicator(
                            format!("SMA_{}", i),
                            Box::new(SMA::new(20))
                        ),
                        1 => pipeline.register_indicator(
                            format!("EMA_{}", i),
                            Box::new(EMA::new(20))
                        ),
                        2 => pipeline.register_indicator(
                            format!("RSI_{}", i),
                            Box::new(RSI::new(14))
                        ),
                        3 => pipeline.register_indicator(
                            format!("MACD_{}", i),
                            Box::new(MACD::new(12, 26, 9))
                        ),
                        4 => pipeline.register_indicator(
                            format!("BB_{}", i),
                            Box::new(BollingerBands::new(20, 2.0))
                        ),
                        5 => pipeline.register_indicator(
                            format!("ADX_{}", i),
                            Box::new(ADX::new(14))
                        ),
                        _ => pipeline.register_indicator(
                            format!("SAR_{}", i),
                            Box::new(ParabolicSAR::new(0.02, 0.2))
                        ),
                    }
                }

                let mut idx = 0;
                b.iter(|| {
                    let result = pipeline.update_all(
                        black_box(&bars[idx % bars.len()]),
                        Timeframe::M1
                    );
                    idx += 1;
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

fn bench_cache_retrieval(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_retrieval");

    let pipeline = IndicatorPipeline::new(10000);

    // Populate the cache
    let bars = generate_bar_data(100);
    for i in 0..20 {
        pipeline.register_indicator(
            format!("IND_{}", i),
            Box::new(SMA::new(20))
        );
    }

    // Warm up the indicators
    for bar in &bars {
        pipeline.update_all(bar, Timeframe::M1).unwrap();
    }

    group.bench_function("get_value", |b| {
        b.iter(|| {
            black_box(pipeline.get_value("IND_10", Timeframe::M1))
        });
    });

    group.bench_function("get_history_10", |b| {
        b.iter(|| {
            black_box(pipeline.get_history("IND_10", Timeframe::M1, 10))
        });
    });

    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.sample_size(10);

    group.bench_function("1M_bars_20_indicators", |b| {
        b.iter(|| {
            let pipeline = IndicatorPipeline::new(1000000);

            // Register 20 indicators
            for i in 0..20 {
                match i % 4 {
                    0 => pipeline.register_indicator(
                        format!("SMA_{}", i),
                        Box::new(SMA::new(20))
                    ),
                    1 => pipeline.register_indicator(
                        format!("EMA_{}", i),
                        Box::new(EMA::new(20))
                    ),
                    2 => pipeline.register_indicator(
                        format!("RSI_{}", i),
                        Box::new(RSI::new(14))
                    ),
                    _ => pipeline.register_indicator(
                        format!("MACD_{}", i),
                        Box::new(MACD::new(12, 26, 9))
                    ),
                }
            }

            // Process a smaller sample for memory benchmark
            let bars = generate_bar_data(1000);
            for bar in &bars {
                pipeline.update_all(bar, Timeframe::M1).unwrap();
            }

            black_box(pipeline)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_individual_indicators,
    bench_pipeline_update_all,
    bench_cache_retrieval,
    bench_memory_usage
);
criterion_main!(benches);