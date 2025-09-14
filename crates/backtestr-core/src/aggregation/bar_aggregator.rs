use crate::events::{BarCompletionEvent, EventBus};
use backtestr_data::models::Bar;
use backtestr_data::timeframe::Timeframe;
use std::collections::HashMap;

use super::{GapDetector, SessionManager, VolumeAggregator};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AggregationMethod {
    Standard,
    VolumeWeighted,
}

#[derive(Debug, Clone)]
pub struct AggregationRule {
    pub source_timeframe: Timeframe,
    pub target_timeframe: Timeframe,
    pub bars_per_aggregation: usize,
    pub aggregation_method: AggregationMethod,
}

impl AggregationRule {
    pub fn new(
        source_timeframe: Timeframe,
        target_timeframe: Timeframe,
        bars_per_aggregation: usize,
    ) -> Self {
        Self {
            source_timeframe,
            target_timeframe,
            bars_per_aggregation,
            aggregation_method: AggregationMethod::Standard,
        }
    }

    pub fn with_method(mut self, method: AggregationMethod) -> Self {
        self.aggregation_method = method;
        self
    }
}

pub struct BarAggregator {
    aggregation_rules: HashMap<Timeframe, AggregationRule>,
    session_manager: SessionManager,
    gap_detector: GapDetector,
    volume_aggregator: VolumeAggregator,
    event_bus: EventBus,
    pending_bars: HashMap<Timeframe, Vec<Bar>>,
}

impl BarAggregator {
    pub fn new(
        session_manager: SessionManager,
        gap_detector: GapDetector,
        event_bus: EventBus,
    ) -> Self {
        let mut aggregation_rules = HashMap::new();

        // Setup default aggregation rules
        aggregation_rules.insert(
            Timeframe::M5,
            AggregationRule::new(Timeframe::M1, Timeframe::M5, 5),
        );
        aggregation_rules.insert(
            Timeframe::M15,
            AggregationRule::new(Timeframe::M5, Timeframe::M15, 3),
        );
        aggregation_rules.insert(
            Timeframe::H1,
            AggregationRule::new(Timeframe::M15, Timeframe::H1, 4),
        );
        aggregation_rules.insert(
            Timeframe::H4,
            AggregationRule::new(Timeframe::H1, Timeframe::H4, 4),
        );
        aggregation_rules.insert(
            Timeframe::D1,
            AggregationRule::new(Timeframe::H4, Timeframe::D1, 6),
        );

        Self {
            aggregation_rules,
            session_manager,
            gap_detector,
            volume_aggregator: VolumeAggregator::new(),
            event_bus,
            pending_bars: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, timeframe: Timeframe, rule: AggregationRule) {
        self.aggregation_rules.insert(timeframe, rule);
    }

    pub fn process_bar(&mut self, bar: Bar, source_timeframe: Timeframe) -> Vec<Bar> {
        let mut completed_bars = Vec::new();
        let mut events_to_publish = Vec::new();

        // Find all timeframes that aggregate from this source
        let target_timeframes: Vec<Timeframe> = self
            .aggregation_rules
            .iter()
            .filter(|(_, rule)| rule.source_timeframe == source_timeframe)
            .map(|(tf, _)| *tf)
            .collect();

        for target_tf in target_timeframes {
            // Add bar to pending bars for this timeframe
            self.pending_bars
                .entry(target_tf)
                .or_default()
                .push(bar.clone());

            // Try to aggregate with current pending bars
            let pending = self.pending_bars.get(&target_tf).unwrap();
            if let Some(aggregated) = self.try_aggregate_bars(pending, target_tf) {
                completed_bars.push(aggregated.clone());

                // Prepare completion event
                let event = match target_tf {
                    Timeframe::M5 => BarCompletionEvent::FiveMinuteBar(aggregated),
                    Timeframe::M15 => BarCompletionEvent::FifteenMinuteBar(aggregated),
                    Timeframe::H1 => BarCompletionEvent::HourBar(aggregated),
                    Timeframe::H4 => BarCompletionEvent::FourHourBar(aggregated),
                    Timeframe::D1 => BarCompletionEvent::DailyBar(aggregated),
                    _ => BarCompletionEvent::MinuteBar(aggregated),
                };
                events_to_publish.push(event);

                // Clear pending bars after successful aggregation
                self.pending_bars.get_mut(&target_tf).unwrap().clear();
            }
        }

        // Publish events after all processing
        for event in events_to_publish {
            self.event_bus.publish(event);
        }

        completed_bars
    }

    pub fn aggregate_bars(
        &mut self,
        source_bars: &[Bar],
        target_timeframe: Timeframe,
    ) -> Option<Bar> {
        let rule = self.aggregation_rules.get(&target_timeframe)?;

        if source_bars.is_empty() {
            return None;
        }

        // Check for sufficient bars
        if source_bars.len() < rule.bars_per_aggregation {
            // Check if we hit a session boundary
            if let Some(last_bar) = source_bars.last() {
                if self
                    .session_manager
                    .is_session_boundary(target_timeframe, last_bar.timestamp_end)
                {
                    return Some(self.create_session_bar(source_bars, target_timeframe));
                }
            }
            return None;
        }

        // Check for gaps
        if self.gap_detector.has_gap(source_bars) {
            return self.handle_gap_aggregation(source_bars, target_timeframe);
        }

        // Normal aggregation
        self.aggregate_standard(source_bars, target_timeframe)
    }

    fn try_aggregate_bars(&self, pending_bars: &[Bar], target_timeframe: Timeframe) -> Option<Bar> {
        let rule = self.aggregation_rules.get(&target_timeframe)?;

        if pending_bars.len() >= rule.bars_per_aggregation {
            let bars_to_aggregate: Vec<Bar> = pending_bars
                .iter()
                .take(rule.bars_per_aggregation)
                .cloned()
                .collect();
            return self.aggregate_standard(&bars_to_aggregate, target_timeframe);
        }

        // Check for session boundary
        if let Some(last_bar) = pending_bars.last() {
            if self
                .session_manager
                .is_session_boundary(target_timeframe, last_bar.timestamp_end)
            {
                return Some(self.create_session_bar(pending_bars, target_timeframe));
            }
        }

        None
    }

    fn aggregate_standard(&self, source_bars: &[Bar], target_timeframe: Timeframe) -> Option<Bar> {
        if source_bars.is_empty() {
            return None;
        }

        let first_bar = &source_bars[0];
        let last_bar = source_bars.last()?;

        // Calculate OHLC values
        let open = first_bar.open;
        let close = last_bar.close;

        let high = source_bars
            .iter()
            .map(|b| b.high)
            .max_by(|a, b| a.partial_cmp(b).unwrap())?;

        let low = source_bars
            .iter()
            .map(|b| b.low)
            .min_by(|a, b| a.partial_cmp(b).unwrap())?;

        // Aggregate volume
        let volume = self.volume_aggregator.aggregate_volume(source_bars);
        let tick_count = self.volume_aggregator.aggregate_tick_count(source_bars);

        let mut bar = Bar::new(
            first_bar.symbol.clone(),
            target_timeframe,
            first_bar.timestamp_start,
            last_bar.timestamp_end,
            open,
            high,
            low,
            close,
        );

        if let Some(vol) = volume {
            bar = bar.with_volume(vol);
        }

        if let Some(ticks) = tick_count {
            bar = bar.with_tick_count(ticks);
        }

        Some(bar)
    }

    fn create_session_bar(&self, source_bars: &[Bar], target_timeframe: Timeframe) -> Bar {
        // Create a bar that respects session boundaries even with incomplete data
        if source_bars.is_empty() {
            panic!("Cannot create session bar from empty source bars");
        }

        let first_bar = &source_bars[0];
        let last_bar = source_bars.last().unwrap();

        let high = source_bars
            .iter()
            .map(|b| b.high)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        let low = source_bars
            .iter()
            .map(|b| b.low)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        let volume = self.volume_aggregator.aggregate_volume(source_bars);
        let tick_count = self.volume_aggregator.aggregate_tick_count(source_bars);

        let mut bar = Bar::new(
            first_bar.symbol.clone(),
            target_timeframe,
            first_bar.timestamp_start,
            last_bar.timestamp_end,
            first_bar.open,
            high,
            low,
            last_bar.close,
        );

        if let Some(vol) = volume {
            bar = bar.with_volume(vol);
        }

        if let Some(ticks) = tick_count {
            bar = bar.with_tick_count(ticks);
        }

        bar
    }

    fn handle_gap_aggregation(
        &self,
        source_bars: &[Bar],
        target_timeframe: Timeframe,
    ) -> Option<Bar> {
        // Handle aggregation when there's a gap in the data
        // For now, we'll aggregate what we have
        self.aggregate_standard(source_bars, target_timeframe)
    }

    pub fn force_close_bars(&mut self, timestamp: i64) -> Vec<Bar> {
        let mut closed_bars = Vec::new();
        let mut events_to_publish = Vec::new();

        let timeframes: Vec<Timeframe> = self.pending_bars.keys().copied().collect();

        for timeframe in timeframes {
            let pending = self.pending_bars.get(&timeframe).unwrap();
            if !pending.is_empty()
                && self
                    .session_manager
                    .is_session_boundary(timeframe, timestamp)
            {
                if let Some(bar) = self.aggregate_standard(pending, timeframe) {
                    closed_bars.push(bar.clone());

                    // Prepare completion event
                    let event = match timeframe {
                        Timeframe::M5 => BarCompletionEvent::FiveMinuteBar(bar),
                        Timeframe::M15 => BarCompletionEvent::FifteenMinuteBar(bar),
                        Timeframe::H1 => BarCompletionEvent::HourBar(bar),
                        Timeframe::H4 => BarCompletionEvent::FourHourBar(bar),
                        Timeframe::D1 => BarCompletionEvent::DailyBar(bar),
                        _ => BarCompletionEvent::MinuteBar(bar),
                    };
                    events_to_publish.push(event);

                    self.pending_bars.get_mut(&timeframe).unwrap().clear();
                }
            }
        }

        // Publish events after all processing
        for event in events_to_publish {
            self.event_bus.publish(event);
        }

        closed_bars
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn create_test_bar(
        symbol: &str,
        timeframe: Timeframe,
        start: i64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    ) -> Bar {
        Bar::new(
            symbol.to_string(),
            timeframe,
            start,
            start + timeframe.duration_ms(),
            open,
            high,
            low,
            close,
        )
    }

    #[test]
    fn test_aggregation_rule_creation() {
        let rule = AggregationRule::new(Timeframe::M1, Timeframe::M5, 5)
            .with_method(AggregationMethod::VolumeWeighted);

        assert_eq!(rule.source_timeframe, Timeframe::M1);
        assert_eq!(rule.target_timeframe, Timeframe::M5);
        assert_eq!(rule.bars_per_aggregation, 5);
        assert_eq!(rule.aggregation_method, AggregationMethod::VolumeWeighted);
    }

    #[test]
    fn test_basic_aggregation() {
        let session_manager = SessionManager::new();
        let gap_detector = GapDetector::new(Duration::minutes(5));
        let event_bus = EventBus::new();
        let mut aggregator = BarAggregator::new(session_manager, gap_detector, event_bus);

        // Create 5 one-minute bars
        let mut source_bars = Vec::new();
        let base_timestamp = 1704067200000; // 2024-01-01 00:00:00

        for i in 0..5 {
            let bar = create_test_bar(
                "EURUSD",
                Timeframe::M1,
                base_timestamp + i * 60_000,
                1.0920 + i as f64 * 0.0001,
                1.0925 + i as f64 * 0.0001,
                1.0915 + i as f64 * 0.0001,
                1.0922 + i as f64 * 0.0001,
            );
            source_bars.push(bar);
        }

        // Aggregate to 5-minute bar
        let aggregated = aggregator.aggregate_bars(&source_bars, Timeframe::M5);
        assert!(aggregated.is_some());

        let bar = aggregated.unwrap();
        assert_eq!(bar.symbol, "EURUSD");
        assert_eq!(bar.timeframe, Timeframe::M5);
        assert_eq!(bar.open, 1.0920);
        assert_eq!(bar.close, 1.0926);
        assert_eq!(bar.high, 1.0929);
        assert_eq!(bar.low, 1.0915);
    }

    #[test]
    fn test_insufficient_bars() {
        let session_manager = SessionManager::new();
        let gap_detector = GapDetector::new(Duration::minutes(5));
        let event_bus = EventBus::new();
        let mut aggregator = BarAggregator::new(session_manager, gap_detector, event_bus);

        // Create only 3 one-minute bars (need 5 for M5)
        let mut source_bars = Vec::new();
        let base_timestamp = 1704067200000;

        for i in 0..3 {
            let bar = create_test_bar(
                "EURUSD",
                Timeframe::M1,
                base_timestamp + i * 60_000,
                1.0920,
                1.0925,
                1.0915,
                1.0922,
            );
            source_bars.push(bar);
        }

        // Should return None (insufficient bars)
        let aggregated = aggregator.aggregate_bars(&source_bars, Timeframe::M5);
        assert!(aggregated.is_none());
    }
}
