use crate::mtf::PartialBar;
use backtestr_data::{Bar, Timeframe};
use std::collections::VecDeque;

const DEFAULT_BAR_HISTORY: usize = 1000;

#[derive(Debug, Clone)]
pub struct TimeframeState {
    pub timeframe: Timeframe,
    pub current_bar: Option<PartialBar>,
    pub completed_bars: VecDeque<Bar>,
    pub bar_start_time: i64,
    pub bar_end_time: i64,
    pub tick_count: u32,
    history_limit: usize,
}

impl TimeframeState {
    pub fn new(timeframe: Timeframe) -> Self {
        Self::with_history_limit(timeframe, DEFAULT_BAR_HISTORY)
    }

    pub fn with_history_limit(timeframe: Timeframe, history_limit: usize) -> Self {
        Self {
            timeframe,
            current_bar: None,
            completed_bars: VecDeque::with_capacity(history_limit),
            bar_start_time: 0,
            bar_end_time: 0,
            tick_count: 0,
            history_limit,
        }
    }

    pub fn process_tick(
        &mut self,
        symbol: &str,
        timestamp: i64,
        price: f64,
        volume: i64,
    ) -> Option<Bar> {
        let bar_start = self.timeframe.bar_start_timestamp(timestamp);
        let bar_end = self.timeframe.bar_end_timestamp(bar_start);

        // Check if we need to complete the current bar and start a new one
        if bar_start != self.bar_start_time && self.current_bar.is_some() {
            // Complete the current bar
            let completed_bar = self.complete_current_bar(symbol);

            // Start new bar
            self.start_new_bar(bar_start, bar_end, price, volume, timestamp);

            return completed_bar;
        }

        // Update or create current bar
        if self.current_bar.is_none() {
            self.start_new_bar(bar_start, bar_end, price, volume, timestamp);
        } else {
            self.update_current_bar(price, volume, timestamp);
        }

        None
    }

    fn start_new_bar(
        &mut self,
        bar_start: i64,
        bar_end: i64,
        price: f64,
        volume: i64,
        timestamp: i64,
    ) {
        self.bar_start_time = bar_start;
        self.bar_end_time = bar_end;
        self.tick_count = 1;
        self.current_bar = Some(PartialBar::new(
            price, volume, timestamp, bar_start, bar_end,
        ));
    }

    fn update_current_bar(&mut self, price: f64, volume: i64, timestamp: i64) {
        if let Some(ref mut bar) = self.current_bar {
            bar.update(
                price,
                volume,
                timestamp,
                self.bar_start_time,
                self.bar_end_time,
            );
            self.tick_count += 1;
        }
    }

    fn complete_current_bar(&mut self, symbol: &str) -> Option<Bar> {
        if let Some(partial) = self.current_bar.take() {
            let completed_bar = Bar::new(
                symbol.to_string(),
                self.timeframe,
                self.bar_start_time,
                self.bar_end_time,
                partial.open,
                partial.high,
                partial.low,
                partial.close,
            )
            .with_volume(partial.volume)
            .with_tick_count(partial.tick_count as i32);

            // Add to history with limit
            self.completed_bars.push_back(completed_bar.clone());
            if self.completed_bars.len() > self.history_limit {
                self.completed_bars.pop_front();
            }

            self.tick_count = 0;
            Some(completed_bar)
        } else {
            None
        }
    }

    pub fn get_latest_bars(&self, count: usize) -> Vec<Bar> {
        let actual_count = count.min(self.completed_bars.len());
        self.completed_bars
            .iter()
            .rev()
            .take(actual_count)
            .rev()
            .cloned()
            .collect()
    }

    pub fn get_completion_percentage(&self) -> f32 {
        self.current_bar
            .as_ref()
            .map(|bar| bar.completion_percentage)
            .unwrap_or(0.0)
    }

    pub fn get_time_remaining_ms(&self) -> i64 {
        self.current_bar
            .as_ref()
            .map(|bar| bar.milliseconds_remaining)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeframe_state_creation() {
        let state = TimeframeState::new(Timeframe::M1);
        assert_eq!(state.timeframe, Timeframe::M1);
        assert!(state.current_bar.is_none());
        assert_eq!(state.completed_bars.len(), 0);
        assert_eq!(state.tick_count, 0);
    }

    #[test]
    fn test_process_first_tick() {
        let mut state = TimeframeState::new(Timeframe::M1);
        let timestamp = 1704067230000; // 30 seconds into minute

        let completed = state.process_tick("EURUSD", timestamp, 1.0920, 1000);

        assert!(completed.is_none());
        assert!(state.current_bar.is_some());
        assert_eq!(state.tick_count, 1);
        assert_eq!(state.bar_start_time, 1704067200000);
        assert_eq!(state.bar_end_time, 1704067260000);
    }

    #[test]
    fn test_process_tick_completes_bar() {
        let mut state = TimeframeState::new(Timeframe::M1);

        // First tick
        state.process_tick("EURUSD", 1704067230000, 1.0920, 1000);

        // Tick in next minute - should complete previous bar
        let completed = state.process_tick("EURUSD", 1704067261000, 1.0925, 500);

        assert!(completed.is_some());
        let bar = completed.unwrap();
        assert_eq!(bar.symbol, "EURUSD");
        assert_eq!(bar.timeframe, Timeframe::M1);
        assert_eq!(bar.open, 1.0920);
        assert_eq!(bar.close, 1.0920);
        assert_eq!(bar.volume, Some(1000));

        // New bar should be started
        assert!(state.current_bar.is_some());
        assert_eq!(state.completed_bars.len(), 1);
    }

    #[test]
    fn test_history_limit() {
        let mut state = TimeframeState::with_history_limit(Timeframe::M1, 2);

        // Generate 3 complete bars
        for i in 0..4 {
            let timestamp = 1704067200000 + (i * 60000); // Each minute
            state.process_tick("EURUSD", timestamp, 1.0920 + (i as f64 * 0.0001), 1000);
        }

        // Should only keep 2 bars in history
        assert_eq!(state.completed_bars.len(), 2);
    }

    #[test]
    fn test_get_latest_bars() {
        let mut state = TimeframeState::new(Timeframe::M1);

        // Generate 5 complete bars
        for i in 0..6 {
            let timestamp = 1704067200000 + (i * 60000);
            state.process_tick("EURUSD", timestamp, 1.0920 + (i as f64 * 0.0001), 1000);
        }

        let latest = state.get_latest_bars(3);
        assert_eq!(latest.len(), 3);

        // Check they're in chronological order
        assert!(latest[0].timestamp_start < latest[1].timestamp_start);
        assert!(latest[1].timestamp_start < latest[2].timestamp_start);
    }
}
