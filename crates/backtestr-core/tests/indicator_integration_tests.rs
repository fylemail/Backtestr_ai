use backtestr_core::indicators::*;
use backtestr_data::Timeframe;

fn create_test_bars() -> Vec<BarData> {
    vec![
        BarData { open: 100.0, high: 102.0, low: 99.0, close: 101.0, volume: 1000.0, timestamp: 1 },
        BarData { open: 101.0, high: 103.0, low: 100.0, close: 102.0, volume: 1100.0, timestamp: 2 },
        BarData { open: 102.0, high: 104.0, low: 101.0, close: 103.0, volume: 1200.0, timestamp: 3 },
        BarData { open: 103.0, high: 105.0, low: 102.0, close: 104.0, volume: 1300.0, timestamp: 4 },
        BarData { open: 104.0, high: 106.0, low: 103.0, close: 105.0, volume: 1400.0, timestamp: 5 },
        BarData { open: 105.0, high: 107.0, low: 104.0, close: 106.0, volume: 1500.0, timestamp: 6 },
        BarData { open: 106.0, high: 108.0, low: 105.0, close: 107.0, volume: 1600.0, timestamp: 7 },
        BarData { open: 107.0, high: 109.0, low: 106.0, close: 108.0, volume: 1700.0, timestamp: 8 },
        BarData { open: 108.0, high: 110.0, low: 107.0, close: 109.0, volume: 1800.0, timestamp: 9 },
        BarData { open: 109.0, high: 111.0, low: 108.0, close: 110.0, volume: 1900.0, timestamp: 10 },
    ]
}

#[test]
fn test_pipeline_with_multiple_indicators() {
    let pipeline = IndicatorPipeline::new(100);

    // Register various indicators
    pipeline.register_indicator("SMA_5".to_string(), Box::new(SMA::new(5)));
    pipeline.register_indicator("EMA_5".to_string(), Box::new(EMA::new(5)));
    pipeline.register_indicator("RSI_5".to_string(), Box::new(RSI::new(5)));
    pipeline.register_indicator("MACD".to_string(), Box::new(MACD::new(3, 5, 2)));

    let bars = create_test_bars();

    // Process all bars
    for bar in &bars {
        let result = pipeline.update_all(bar, Timeframe::M1).unwrap();
        assert!(result.duration_micros > 0);
    }

    // Verify we can retrieve values
    assert!(pipeline.get_value("SMA_5", Timeframe::M1).is_some());
    assert!(pipeline.get_value("EMA_5", Timeframe::M1).is_some());
    assert!(pipeline.get_value("RSI_5", Timeframe::M1).is_some());
}

#[test]
fn test_pipeline_parallel_processing() {
    let pipeline = IndicatorPipeline::new(100);

    // Register more than 5 indicators to trigger parallel processing
    for i in 0..10 {
        pipeline.register_indicator(
            format!("SMA_{}", i),
            Box::new(SMA::new(5))
        );
    }

    let bars = create_test_bars();

    for bar in &bars {
        let result = pipeline.update_all(bar, Timeframe::M1).unwrap();
        assert_eq!(result.updated_count + result.failed_count, 10);
    }

    // All indicators should have values after processing enough bars
    let last_bar = &bars[bars.len() - 1];
    pipeline.update_all(last_bar, Timeframe::M1).unwrap();

    for i in 0..10 {
        assert!(pipeline.get_value(&format!("SMA_{}", i), Timeframe::M1).is_some());
    }
}

#[test]
fn test_indicator_reset() {
    let pipeline = IndicatorPipeline::new(100);

    pipeline.register_indicator("RSI".to_string(), Box::new(RSI::new(5)));

    let bars = create_test_bars();

    // Process enough bars for RSI warm-up (period + 1 = 6)
    for bar in &bars[..6] {
        pipeline.update_all(bar, Timeframe::M1).unwrap();
    }

    assert!(pipeline.get_value("RSI", Timeframe::M1).is_some());

    // Reset the indicator
    pipeline.reset_indicator("RSI");

    // Value should be None after reset
    assert!(pipeline.get_value("RSI", Timeframe::M1).is_none());

    // Process bars again after reset (need 6 for RSI warm-up)
    for bar in &bars[..6] {
        pipeline.update_all(bar, Timeframe::M1).unwrap();
    }

    // Should have a value again after reprocessing
    assert!(pipeline.get_value("RSI", Timeframe::M1).is_some());
}

#[test]
fn test_cache_history_retrieval() {
    let pipeline = IndicatorPipeline::new(100);

    pipeline.register_indicator("SMA".to_string(), Box::new(SMA::new(3)));

    let bars = create_test_bars();

    // Process all bars
    for bar in &bars {
        pipeline.update_all(bar, Timeframe::M1).unwrap();
    }

    // Get history
    let history = pipeline.get_history("SMA", Timeframe::M1, 5);

    // Should have at least some history
    assert!(!history.is_empty());
    assert!(history.len() <= 5);

    // Values should be in chronological order
    for i in 1..history.len() {
        assert!(history[i].timestamp > history[i - 1].timestamp);
    }
}

#[test]
fn test_multiple_timeframes() {
    let pipeline = IndicatorPipeline::new(100);

    pipeline.register_indicator("EMA".to_string(), Box::new(EMA::new(3)));

    let bars = create_test_bars();

    // Process same indicator for different timeframes
    for bar in &bars {
        pipeline.update_all(bar, Timeframe::M1).unwrap();
        pipeline.update_all(bar, Timeframe::M5).unwrap();
        pipeline.update_all(bar, Timeframe::M15).unwrap();
    }

    // Each timeframe should have its own cached values
    let m1_value = pipeline.get_value("EMA", Timeframe::M1);
    let m5_value = pipeline.get_value("EMA", Timeframe::M5);
    let m15_value = pipeline.get_value("EMA", Timeframe::M15);

    assert!(m1_value.is_some());
    assert!(m5_value.is_some());
    assert!(m15_value.is_some());
}

#[test]
fn test_indicator_warm_up_periods() {
    let pipeline = IndicatorPipeline::new(100);

    // Register indicators with different warm-up periods
    pipeline.register_indicator("SMA_3".to_string(), Box::new(SMA::new(3)));
    pipeline.register_indicator("SMA_5".to_string(), Box::new(SMA::new(5)));
    pipeline.register_indicator("RSI_5".to_string(), Box::new(RSI::new(5)));

    let bars = create_test_bars();

    // After 3 bars, SMA_3 should have values but not SMA_5
    for bar in &bars[..3] {
        pipeline.update_all(bar, Timeframe::M1).unwrap();
    }

    assert!(pipeline.get_value("SMA_3", Timeframe::M1).is_some());
    assert!(pipeline.get_value("SMA_5", Timeframe::M1).is_none());

    // After 5 bars, both should have values
    for bar in &bars[3..5] {
        pipeline.update_all(bar, Timeframe::M1).unwrap();
    }

    assert!(pipeline.get_value("SMA_3", Timeframe::M1).is_some());
    assert!(pipeline.get_value("SMA_5", Timeframe::M1).is_some());
}

#[test]
fn test_new_adx_implementation() {
    let mut adx = ADX::new(14);

    // Generate enough bars for ADX to produce values
    let mut bars = Vec::new();
    for i in 0..50 {
        let base = 100.0 + i as f64 * 0.5;
        bars.push(BarData {
            open: base,
            high: base + 2.0,
            low: base - 1.0,
            close: base + 1.0,
            volume: 1000.0,
            timestamp: i,
        });
    }

    let mut has_value = false;
    for bar in bars {
        if let Some(value) = adx.update(bar) {
            has_value = true;
            // ADX should be between 0 and 100
            assert!(value >= 0.0 && value <= 100.0);
        }
    }

    assert!(has_value, "ADX should produce values after warm-up period");
}

#[test]
fn test_new_parabolic_sar_implementation() {
    let mut sar = ParabolicSAR::new(0.02, 0.2);

    let bars = create_test_bars();

    let mut has_value = false;
    for bar in bars {
        if let Some(value) = sar.update(bar) {
            has_value = true;
            // SAR value should be within reasonable range of price
            assert!(value > 0.0);
            assert!(value < 200.0); // Reasonable upper bound for our test data
        }
    }

    assert!(has_value, "Parabolic SAR should produce values after warm-up");
}

#[test]
fn test_all_indicators_produce_values() {
    let pipeline = IndicatorPipeline::new(1000);

    // Register all 20 indicators
    pipeline.register_indicator("SMA".to_string(), Box::new(SMA::new(5)));
    pipeline.register_indicator("EMA".to_string(), Box::new(EMA::new(5)));
    pipeline.register_indicator("WMA".to_string(), Box::new(WMA::new(5)));
    pipeline.register_indicator("DEMA".to_string(), Box::new(DEMA::new(5)));
    pipeline.register_indicator("RSI".to_string(), Box::new(RSI::new(5)));
    pipeline.register_indicator("MACD".to_string(), Box::new(MACD::new(5, 10, 3)));
    pipeline.register_indicator("Stochastic".to_string(), Box::new(Stochastic::new(5, 3)));
    pipeline.register_indicator("CCI".to_string(), Box::new(CCI::new(5)));
    pipeline.register_indicator("WilliamsR".to_string(), Box::new(WilliamsR::new(5)));
    pipeline.register_indicator("BollingerBands".to_string(), Box::new(BollingerBands::new(5, 2.0)));
    pipeline.register_indicator("ATR".to_string(), Box::new(ATR::new(5)));
    pipeline.register_indicator("KeltnerChannels".to_string(), Box::new(KeltnerChannels::new(5, 2.0)));
    pipeline.register_indicator("DonchianChannels".to_string(), Box::new(DonchianChannels::new(5)));
    pipeline.register_indicator("OBV".to_string(), Box::new(OBV::new()));
    pipeline.register_indicator("VolumeSMA".to_string(), Box::new(VolumeSMA::new(5)));
    pipeline.register_indicator("VWAP".to_string(), Box::new(VWAP::new(false)));
    pipeline.register_indicator("PivotPoints".to_string(), Box::new(PivotPoints::new()));
    pipeline.register_indicator("SupportResistance".to_string(), Box::new(SupportResistance::new(5)));
    pipeline.register_indicator("ADX".to_string(), Box::new(ADX::new(5)));
    pipeline.register_indicator("ParabolicSAR".to_string(), Box::new(ParabolicSAR::new(0.02, 0.2)));

    // Generate enough bars for all indicators
    let mut bars = Vec::new();
    for i in 0..50 {
        let base = 100.0 + (i as f64 * 0.1).sin() * 5.0;
        bars.push(BarData {
            open: base,
            high: base + 2.0,
            low: base - 2.0,
            close: base + 1.0,
            volume: 10000.0 + i as f64 * 100.0,
            timestamp: i,
        });
    }

    // Process all bars
    for bar in &bars {
        pipeline.update_all(bar, Timeframe::M1).unwrap();
    }

    // Check that most indicators have values (some may still be warming up)
    let mut indicators_with_values = 0;
    let indicator_names = pipeline.get_indicator_names();

    for name in &indicator_names {
        if pipeline.get_value(name, Timeframe::M1).is_some() {
            indicators_with_values += 1;
        }
    }

    // At least 18 out of 20 should have values after 50 bars
    assert!(
        indicators_with_values >= 18,
        "Expected at least 18 indicators with values, got {}",
        indicators_with_values
    );
}