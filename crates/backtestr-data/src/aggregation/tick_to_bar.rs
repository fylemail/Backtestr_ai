use crate::models::{Bar, Tick};
use crate::timeframe::Timeframe;
use std::collections::HashMap;

/// Aggregates ticks into bars for multiple timeframes
pub struct TickToBarAggregator {
    /// Active bar builders indexed by symbol and timeframe
    active_bars: HashMap<(String, Timeframe), BarBuilder>,
    /// Completed bars ready to be persisted
    completed_bars: Vec<Bar>,
}

impl Default for TickToBarAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl TickToBarAggregator {
    pub fn new() -> Self {
        Self {
            active_bars: HashMap::new(),
            completed_bars: Vec::new(),
        }
    }

    /// Process a tick and potentially complete bars
    pub fn process_tick(&mut self, tick: &Tick) -> Vec<Bar> {
        let mut completed = Vec::new();

        // Process for all timeframes
        for timeframe in Timeframe::all() {
            let key = (tick.symbol.clone(), timeframe);
            let bar_start = timeframe.bar_start_timestamp(tick.timestamp);
            let bar_end = timeframe.bar_end_timestamp(bar_start);

            // Get or create bar builder
            let builder = self.active_bars.entry(key.clone()).or_insert_with(|| {
                BarBuilder::new(tick.symbol.clone(), timeframe, bar_start, bar_end)
            });

            // Check if this tick belongs to a new bar period
            if tick.timestamp >= builder.timestamp_end {
                // Complete current bar if it has data
                if builder.tick_count > 0 {
                    if let Some(bar) = builder.build() {
                        completed.push(bar.clone());
                        self.completed_bars.push(bar);
                    }
                }

                // Start new bar
                *builder = BarBuilder::new(tick.symbol.clone(), timeframe, bar_start, bar_end);
            }

            // Add tick to current bar
            builder.add_tick(tick);
        }

        completed
    }

    /// Force completion of all active bars (e.g., at end of data)
    pub fn flush(&mut self) -> Vec<Bar> {
        let mut completed = Vec::new();

        for builder in self.active_bars.values() {
            if builder.tick_count > 0 {
                if let Some(bar) = builder.build() {
                    completed.push(bar.clone());
                    self.completed_bars.push(bar);
                }
            }
        }

        self.active_bars.clear();
        completed
    }

    /// Get all completed bars
    pub fn get_completed_bars(&self) -> &[Bar] {
        &self.completed_bars
    }

    /// Clear completed bars (after persisting to database)
    pub fn clear_completed_bars(&mut self) {
        self.completed_bars.clear();
    }
}

/// Builder for a single bar
#[derive(Debug, Clone)]
struct BarBuilder {
    symbol: String,
    timeframe: Timeframe,
    timestamp_start: i64,
    timestamp_end: i64,
    open: Option<f64>,
    high: Option<f64>,
    low: Option<f64>,
    close: Option<f64>,
    volume: i64,
    tick_count: i32,
}

impl BarBuilder {
    fn new(symbol: String, timeframe: Timeframe, timestamp_start: i64, timestamp_end: i64) -> Self {
        Self {
            symbol,
            timeframe,
            timestamp_start,
            timestamp_end,
            open: None,
            high: None,
            low: None,
            close: None,
            volume: 0,
            tick_count: 0,
        }
    }

    fn add_tick(&mut self, tick: &Tick) {
        let midpoint = (tick.bid + tick.ask) / 2.0;

        // Set open on first tick
        if self.open.is_none() {
            self.open = Some(midpoint);
        }

        // Update high
        self.high = Some(self.high.map_or(midpoint, |h| h.max(midpoint)));

        // Update low
        self.low = Some(self.low.map_or(midpoint, |l| l.min(midpoint)));

        // Always update close with latest tick
        self.close = Some(midpoint);

        // Add volume if available
        if let (Some(bid_size), Some(ask_size)) = (tick.bid_size, tick.ask_size) {
            self.volume += (bid_size + ask_size) / 2;
        }

        self.tick_count += 1;
    }

    fn build(&self) -> Option<Bar> {
        match (self.open, self.high, self.low, self.close) {
            (Some(open), Some(high), Some(low), Some(close)) => {
                let mut bar = Bar::new(
                    self.symbol.clone(),
                    self.timeframe,
                    self.timestamp_start,
                    self.timestamp_end,
                    open,
                    high,
                    low,
                    close,
                );

                if self.volume > 0 {
                    bar = bar.with_volume(self.volume);
                }

                if self.tick_count > 0 {
                    bar = bar.with_tick_count(self.tick_count);
                }

                Some(bar)
            }
            _ => None,
        }
    }
}

/// Trait for bar aggregation strategies
pub trait BarAggregator {
    /// Process a single tick
    fn process_tick(&mut self, tick: &Tick) -> Vec<Bar>;

    /// Flush any pending bars
    fn flush(&mut self) -> Vec<Bar>;
}

impl BarAggregator for TickToBarAggregator {
    fn process_tick(&mut self, tick: &Tick) -> Vec<Bar> {
        self.process_tick(tick)
    }

    fn flush(&mut self) -> Vec<Bar> {
        self.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tick(symbol: &str, timestamp_ms: i64, bid: f64, ask: f64) -> Tick {
        Tick::new_with_millis(symbol.to_string(), timestamp_ms, bid, ask)
    }

    #[test]
    fn test_single_bar_aggregation() {
        let mut aggregator = TickToBarAggregator::new();

        // Create ticks within a single minute
        let base_time = 1704067200000; // 2024-01-01 00:00:00
        let ticks = vec![
            create_test_tick("EURUSD", base_time + 10000, 1.0920, 1.0922), // 10 seconds
            create_test_tick("EURUSD", base_time + 20000, 1.0921, 1.0923), // 20 seconds
            create_test_tick("EURUSD", base_time + 30000, 1.0919, 1.0921), // 30 seconds
            create_test_tick("EURUSD", base_time + 40000, 1.0922, 1.0924), // 40 seconds
        ];

        // Process ticks - should not complete any bars yet
        for tick in &ticks {
            let completed = aggregator.process_tick(tick);
            assert_eq!(completed.len(), 0);
        }

        // Process a tick from next minute - should complete the M1 bar
        let next_minute_tick = create_test_tick("EURUSD", base_time + 60000, 1.0923, 1.0925);
        let completed = aggregator.process_tick(&next_minute_tick);

        // Should have completed 1 bar (M1)
        assert!(completed.iter().any(|b| b.timeframe == Timeframe::M1));

        let m1_bar = completed
            .iter()
            .find(|b| b.timeframe == Timeframe::M1)
            .unwrap();
        assert_eq!(m1_bar.symbol, "EURUSD");
        assert_eq!(m1_bar.timestamp_start, base_time);
        assert_eq!(m1_bar.timestamp_end, base_time + 60000);
        assert_eq!(m1_bar.open, 1.0921); // First tick midpoint
        assert_eq!(m1_bar.high, 1.0923); // Highest midpoint
        assert_eq!(m1_bar.low, 1.0920); // Lowest midpoint
        assert_eq!(m1_bar.close, 1.0923); // Last tick midpoint
        assert_eq!(m1_bar.tick_count, Some(4));
    }

    #[test]
    fn test_multiple_timeframe_aggregation() {
        let mut aggregator = TickToBarAggregator::new();
        let base_time = 1704067200000; // 2024-01-01 00:00:00

        // Process ticks for 5 minutes
        for minute in 0..5 {
            let timestamp = base_time + (minute * 60000);
            let tick = create_test_tick(
                "EURUSD",
                timestamp + 30000,
                1.0920 + minute as f64 * 0.0001,
                1.0922 + minute as f64 * 0.0001,
            );
            aggregator.process_tick(&tick);
        }

        // Process tick at 5 minute mark - should complete M1 and M5 bars
        let five_minute_tick = create_test_tick("EURUSD", base_time + 300000, 1.0925, 1.0927);
        let completed = aggregator.process_tick(&five_minute_tick);

        // Should have completed bars for M1 and M5
        assert!(completed.iter().any(|b| b.timeframe == Timeframe::M1));
        assert!(completed.iter().any(|b| b.timeframe == Timeframe::M5));
    }

    #[test]
    fn test_gap_handling() {
        let mut aggregator = TickToBarAggregator::new();
        let base_time = 1704067200000;

        // First tick
        let tick1 = create_test_tick("EURUSD", base_time + 10000, 1.0920, 1.0922);
        aggregator.process_tick(&tick1);

        // Gap - jump to 3 minutes later
        let tick2 = create_test_tick("EURUSD", base_time + 190000, 1.0925, 1.0927);
        let completed = aggregator.process_tick(&tick2);

        // Should have completed the first minute bar
        assert!(completed.iter().any(|b| b.timeframe == Timeframe::M1));
    }

    #[test]
    fn test_flush() {
        let mut aggregator = TickToBarAggregator::new();
        let base_time = 1704067200000;

        // Add some ticks
        for i in 0..3 {
            let tick = create_test_tick("EURUSD", base_time + i * 10000, 1.0920, 1.0922);
            aggregator.process_tick(&tick);
        }

        // Flush should complete all active bars
        let completed = aggregator.flush();

        // Should have bars for all timeframes
        assert!(completed.len() > 0);
        assert!(completed.iter().any(|b| b.timeframe == Timeframe::M1));
    }

    #[test]
    fn test_volume_aggregation() {
        let mut aggregator = TickToBarAggregator::new();
        let base_time = 1704067200000;

        // Create ticks with volume
        let mut tick1 = create_test_tick("EURUSD", base_time + 10000, 1.0920, 1.0922);
        tick1.bid_size = Some(1000000);
        tick1.ask_size = Some(1000000);

        let mut tick2 = create_test_tick("EURUSD", base_time + 20000, 1.0921, 1.0923);
        tick2.bid_size = Some(500000);
        tick2.ask_size = Some(500000);

        aggregator.process_tick(&tick1);
        aggregator.process_tick(&tick2);

        // Flush and check volume
        let completed = aggregator.flush();
        let m1_bar = completed
            .iter()
            .find(|b| b.timeframe == Timeframe::M1)
            .unwrap();

        assert_eq!(m1_bar.volume, Some(1500000)); // (1000000 + 1000000)/2 + (500000 + 500000)/2
    }
}
