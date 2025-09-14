use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartialBar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub tick_count: u32,
    pub completion_percentage: f32,
    pub milliseconds_elapsed: i64,
    pub milliseconds_remaining: i64,
}

impl PartialBar {
    pub fn new(price: f64, volume: i64, current_time: i64, bar_start: i64, bar_end: i64) -> Self {
        let duration = bar_end - bar_start;
        let elapsed = current_time - bar_start;
        let remaining = bar_end - current_time;
        let completion = if duration > 0 {
            (elapsed as f32 / duration as f32) * 100.0
        } else {
            0.0
        };

        Self {
            open: price,
            high: price,
            low: price,
            close: price,
            volume,
            tick_count: 1,
            completion_percentage: completion.clamp(0.0, 100.0),
            milliseconds_elapsed: elapsed,
            milliseconds_remaining: remaining.max(0),
        }
    }

    pub fn update(
        &mut self,
        price: f64,
        volume: i64,
        current_time: i64,
        bar_start: i64,
        bar_end: i64,
    ) {
        self.high = self.high.max(price);
        self.low = self.low.min(price);
        self.close = price;
        self.volume += volume;
        self.tick_count += 1;

        let duration = bar_end - bar_start;
        let elapsed = current_time - bar_start;
        let remaining = bar_end - current_time;

        self.milliseconds_elapsed = elapsed;
        self.milliseconds_remaining = remaining.max(0);
        self.completion_percentage = (if duration > 0 {
            (elapsed as f32 / duration as f32) * 100.0
        } else {
            0.0
        })
        .clamp(0.0, 100.0);
    }

    pub fn is_complete(&self) -> bool {
        self.milliseconds_remaining <= 0 || self.completion_percentage >= 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_bar_creation() {
        let bar_start = 1704067200000; // 2024-01-01 00:00:00
        let bar_end = 1704067260000; // 2024-01-01 00:01:00
        let current = 1704067230000; // 30 seconds in

        let bar = PartialBar::new(1.0920, 1000, current, bar_start, bar_end);

        assert_eq!(bar.open, 1.0920);
        assert_eq!(bar.high, 1.0920);
        assert_eq!(bar.low, 1.0920);
        assert_eq!(bar.close, 1.0920);
        assert_eq!(bar.volume, 1000);
        assert_eq!(bar.tick_count, 1);
        assert_eq!(bar.completion_percentage, 50.0);
        assert_eq!(bar.milliseconds_elapsed, 30000);
        assert_eq!(bar.milliseconds_remaining, 30000);
    }

    #[test]
    fn test_partial_bar_update() {
        let bar_start = 1704067200000;
        let bar_end = 1704067260000;
        let current = 1704067230000;

        let mut bar = PartialBar::new(1.0920, 1000, current, bar_start, bar_end);

        // Update with new tick 15 seconds later
        let new_current = 1704067245000;
        bar.update(1.0925, 500, new_current, bar_start, bar_end);

        assert_eq!(bar.open, 1.0920);
        assert_eq!(bar.high, 1.0925);
        assert_eq!(bar.low, 1.0920);
        assert_eq!(bar.close, 1.0925);
        assert_eq!(bar.volume, 1500);
        assert_eq!(bar.tick_count, 2);
        assert_eq!(bar.completion_percentage, 75.0);
        assert_eq!(bar.milliseconds_elapsed, 45000);
        assert_eq!(bar.milliseconds_remaining, 15000);
    }

    #[test]
    fn test_partial_bar_completion() {
        let bar_start = 1704067200000;
        let bar_end = 1704067260000;

        // Bar at 50%
        let current = 1704067230000;
        let bar = PartialBar::new(1.0920, 1000, current, bar_start, bar_end);
        assert!(!bar.is_complete());

        // Bar at 100%
        let current = 1704067260000;
        let bar = PartialBar::new(1.0920, 1000, current, bar_start, bar_end);
        assert!(bar.is_complete());
    }
}
