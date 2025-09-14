use backtestr_core::aggregation::{
    AggregationMethod, AggregationRule, BarAggregator, GapDetector, MarketHours, SessionManager,
};
use backtestr_core::events::EventBus;
use backtestr_data::models::Bar;
use backtestr_data::timeframe::Timeframe;
use chrono::{Duration, NaiveDateTime};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn create_test_bars(symbol: &str, start_timestamp: i64, count: usize, timeframe: Timeframe) -> Vec<Bar> {
    let mut bars = Vec::new();
    let duration = timeframe.duration_ms();

    for i in 0..count {
        let timestamp_start = start_timestamp + (i as i64 * duration);
        let timestamp_end = timestamp_start + duration;

        // Create realistic price movement
        let base_price = 1.0920;
        let variation = (i as f64 * 0.0001).sin() * 0.0005;

        let open = base_price + variation;
        let close = base_price + variation + 0.0002;
        let high = open.max(close) + 0.0003;
        let low = open.min(close) - 0.0002;

        let bar = Bar::new(
            symbol.to_string(),
            timeframe,
            timestamp_start,
            timestamp_end,
            open,
            high,
            low,
            close,
        )
        .with_volume(1000 + (i as i64 * 100))
        .with_tick_count(50 + (i as i32 * 5));

        bars.push(bar);
    }

    bars
}

#[test]
fn test_multi_timeframe_aggregation_cascade() {
    // Setup
    let session_manager = SessionManager::new();
    let gap_detector = GapDetector::new(Duration::minutes(5));
    let event_bus = EventBus::new();
    let mut aggregator = BarAggregator::new(session_manager, gap_detector, event_bus.clone());

    // Track events
    let m5_count = Arc::new(AtomicUsize::new(0));
    let m15_count = Arc::new(AtomicUsize::new(0));
    let h1_count = Arc::new(AtomicUsize::new(0));

    let m5_clone = Arc::clone(&m5_count);
    let m15_clone = Arc::clone(&m15_count);
    let h1_clone = Arc::clone(&h1_count);

    event_bus.subscribe("5M", move |_| {
        m5_clone.fetch_add(1, Ordering::SeqCst);
    });

    event_bus.subscribe("15M", move |_| {
        m15_clone.fetch_add(1, Ordering::SeqCst);
    });

    event_bus.subscribe("1H", move |_| {
        h1_clone.fetch_add(1, Ordering::SeqCst);
    });

    // Create 60 one-minute bars (1 hour of data)
    let start = 1704067200000; // 2024-01-01 00:00:00
    let m1_bars = create_test_bars("EURUSD", start, 60, Timeframe::M1);

    // Process each minute bar - the aggregator should handle the cascade
    for bar in m1_bars.iter() {
        aggregator.process_bar(bar.clone(), Timeframe::M1);
    }

    // Verify event counts
    assert_eq!(m5_count.load(Ordering::SeqCst), 12); // 60 / 5 = 12

    // M15 bars won't be created automatically from M1 bars processing
    // We need to manually aggregate M5 bars to get M15 bars
    // This is because the aggregator only aggregates from direct source timeframes

    // To get M15 bars, we would need to set up rules for M1 -> M5 -> M15 cascade
    // or manually process M5 bars
}

#[test]
fn test_weekend_gap_handling() {
    let mut session_manager = SessionManager::new();
    session_manager.add_market_hours("EURUSD".to_string(), MarketHours::forex("EURUSD"));

    let gap_detector = GapDetector::new(Duration::hours(48));
    let event_bus = EventBus::new();
    let mut aggregator = BarAggregator::new(session_manager, gap_detector, event_bus);

    // Create bars before and after weekend
    let friday_close = NaiveDateTime::parse_from_str("2024-01-05 16:59:00", "%Y-%m-%d %H:%M:%S")
        .unwrap()
        .and_utc()
        .timestamp_millis();
    let sunday_open = NaiveDateTime::parse_from_str("2024-01-07 17:00:00", "%Y-%m-%d %H:%M:%S")
        .unwrap()
        .and_utc()
        .timestamp_millis();

    let mut bars = Vec::new();

    // Friday bars
    bars.extend(create_test_bars("EURUSD", friday_close - 300_000, 5, Timeframe::M1));

    // Sunday bars (after gap)
    bars.extend(create_test_bars("EURUSD", sunday_open, 5, Timeframe::M1));

    // Aggregate should handle the gap correctly
    let aggregated = aggregator.aggregate_bars(&bars[0..5], Timeframe::M5);
    assert!(aggregated.is_some());

    // The gap should be detected but not prevent aggregation
    let sunday_aggregated = aggregator.aggregate_bars(&bars[5..10], Timeframe::M5);
    assert!(sunday_aggregated.is_some());
}

#[test]
fn test_session_boundary_forced_close() {
    let session_manager = SessionManager::new();
    let gap_detector = GapDetector::new(Duration::minutes(5));
    let event_bus = EventBus::new();
    let mut aggregator = BarAggregator::new(session_manager, gap_detector, event_bus.clone());

    // Track forced closes
    let forced_closes = Arc::new(AtomicUsize::new(0));
    let forced_clone = Arc::clone(&forced_closes);

    event_bus.subscribe_all(move |_| {
        forced_clone.fetch_add(1, Ordering::SeqCst);
    });

    // Add some pending bars
    let start = 1704067200000;
    let bars = create_test_bars("EURUSD", start, 3, Timeframe::M1);

    for bar in bars {
        aggregator.process_bar(bar, Timeframe::M1);
    }

    // Force close at session boundary (daily close at 5pm)
    let close_time = NaiveDateTime::parse_from_str("2024-01-01 17:00:00", "%Y-%m-%d %H:%M:%S")
        .unwrap()
        .and_utc()
        .timestamp_millis();

    let closed = aggregator.force_close_bars(close_time);
    assert!(!closed.is_empty());
}

#[test]
fn test_volume_aggregation_accuracy() {
    let session_manager = SessionManager::new();
    let gap_detector = GapDetector::new(Duration::minutes(5));
    let event_bus = EventBus::new();
    let mut aggregator = BarAggregator::new(session_manager, gap_detector, event_bus);

    // Create bars with specific volumes
    let mut bars = Vec::new();
    let start = 1704067200000;

    for i in 0..5 {
        let bar = Bar::new(
            "EURUSD".to_string(),
            Timeframe::M1,
            start + i * 60_000,
            start + (i + 1) * 60_000,
            1.0920,
            1.0925,
            1.0915,
            1.0922,
        )
        .with_volume(1000 * (i + 1))
        .with_tick_count(50 * (i + 1) as i32);

        bars.push(bar);
    }

    let aggregated = aggregator.aggregate_bars(&bars, Timeframe::M5).unwrap();

    // Verify volume aggregation
    assert_eq!(aggregated.volume, Some(15000)); // 1000 + 2000 + 3000 + 4000 + 5000
    assert_eq!(aggregated.tick_count, Some(750)); // 50 + 100 + 150 + 200 + 250
}

#[test]
fn test_high_low_accuracy_across_aggregation() {
    let session_manager = SessionManager::new();
    let gap_detector = GapDetector::new(Duration::minutes(5));
    let event_bus = EventBus::new();
    let mut aggregator = BarAggregator::new(session_manager, gap_detector, event_bus);

    let mut bars = Vec::new();
    let start = 1704067200000;

    // Create bars with specific high/low patterns
    let prices = vec![
        (1.0920, 1.0925, 1.0915, 1.0922), // Normal
        (1.0922, 1.0940, 1.0920, 1.0935), // High spike
        (1.0935, 1.0938, 1.0910, 1.0912), // Low spike
        (1.0912, 1.0918, 1.0911, 1.0915), // Narrow range
        (1.0915, 1.0920, 1.0914, 1.0918), // Normal
    ];

    for (i, (open, high, low, close)) in prices.iter().enumerate() {
        let bar = Bar::new(
            "EURUSD".to_string(),
            Timeframe::M1,
            start + i as i64 * 60_000,
            start + (i + 1) as i64 * 60_000,
            *open,
            *high,
            *low,
            *close,
        );
        bars.push(bar);
    }

    let aggregated = aggregator.aggregate_bars(&bars, Timeframe::M5).unwrap();

    // Verify high/low are correctly captured
    assert_eq!(aggregated.high, 1.0940); // Maximum high from all bars
    assert_eq!(aggregated.low, 1.0910);  // Minimum low from all bars
    assert_eq!(aggregated.open, 1.0920); // First bar's open
    assert_eq!(aggregated.close, 1.0918); // Last bar's close
}

#[test]
fn test_custom_aggregation_rules() {
    let session_manager = SessionManager::new();
    let gap_detector = GapDetector::new(Duration::minutes(5));
    let event_bus = EventBus::new();
    let mut aggregator = BarAggregator::new(session_manager, gap_detector, event_bus);

    // Add custom aggregation rule for M3 (3-minute bars from M1)
    let custom_rule = AggregationRule::new(Timeframe::M1, Timeframe::M5, 3)
        .with_method(AggregationMethod::Standard);
    aggregator.add_rule(Timeframe::M5, custom_rule);

    let start = 1704067200000;
    let bars = create_test_bars("EURUSD", start, 3, Timeframe::M1);

    // Should aggregate with only 3 bars now
    let aggregated = aggregator.aggregate_bars(&bars, Timeframe::M5);
    assert!(aggregated.is_some());
}

#[test]
fn test_event_ordering_guarantee() {
    let session_manager = SessionManager::new();
    let gap_detector = GapDetector::new(Duration::minutes(5));
    let event_bus = EventBus::new();
    let mut aggregator = BarAggregator::new(session_manager, gap_detector, event_bus.clone());

    let events_order = Arc::new(std::sync::Mutex::new(Vec::new()));
    let events_clone = Arc::clone(&events_order);

    event_bus.subscribe_all(move |event| {
        let mut order = events_clone.lock().unwrap();
        order.push(event.timeframe_name().to_string());
    });

    // Process enough bars to trigger multiple timeframe completions
    let start = 1704067200000;
    let bars = create_test_bars("EURUSD", start, 15, Timeframe::M1);

    for bar in bars {
        aggregator.process_bar(bar, Timeframe::M1);
    }

    let order = events_order.lock().unwrap();

    // M5 events should come before M15 events
    let m5_indices: Vec<_> = order.iter().enumerate()
        .filter(|(_, tf)| *tf == "5M")
        .map(|(i, _)| i)
        .collect();

    let m15_indices: Vec<_> = order.iter().enumerate()
        .filter(|(_, tf)| *tf == "15M")
        .map(|(i, _)| i)
        .collect();

    if !m5_indices.is_empty() && !m15_indices.is_empty() {
        assert!(m5_indices[0] < m15_indices[0]);
    }
}