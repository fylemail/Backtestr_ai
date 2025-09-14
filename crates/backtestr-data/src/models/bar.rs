use crate::timeframe::Timeframe;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bar {
    pub id: Option<i64>,
    pub symbol: String,
    pub timeframe: Timeframe,
    pub timestamp_start: i64,
    pub timestamp_end: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: Option<i64>,
    pub tick_count: Option<i32>,
}

impl Bar {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        symbol: String,
        timeframe: Timeframe,
        timestamp_start: i64,
        timestamp_end: i64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    ) -> Self {
        Self {
            id: None,
            symbol,
            timeframe,
            timestamp_start,
            timestamp_end,
            open,
            high,
            low,
            close,
            volume: None,
            tick_count: None,
        }
    }

    pub fn with_volume(mut self, volume: i64) -> Self {
        self.volume = Some(volume);
        self
    }

    pub fn with_tick_count(mut self, tick_count: i32) -> Self {
        self.tick_count = Some(tick_count);
        self
    }

    pub fn midpoint(&self) -> f64 {
        (self.high + self.low) / 2.0
    }

    pub fn range(&self) -> f64 {
        self.high - self.low
    }

    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bar_creation() {
        let bar = Bar::new(
            "EURUSD".to_string(),
            Timeframe::M1,
            1704067200000, // 2024-01-01 00:00:00
            1704067260000, // 2024-01-01 00:01:00
            1.0920,
            1.0925,
            1.0918,
            1.0923,
        );

        assert_eq!(bar.symbol, "EURUSD");
        assert_eq!(bar.timeframe, Timeframe::M1);
        assert_eq!(bar.open, 1.0920);
        assert_eq!(bar.high, 1.0925);
        assert_eq!(bar.low, 1.0918);
        assert_eq!(bar.close, 1.0923);
        assert_eq!(bar.volume, None);
        assert_eq!(bar.tick_count, None);
    }

    #[test]
    fn test_bar_with_volume() {
        let bar = Bar::new(
            "EURUSD".to_string(),
            Timeframe::M1,
            1704067200000,
            1704067260000,
            1.0920,
            1.0925,
            1.0918,
            1.0923,
        )
        .with_volume(1000000)
        .with_tick_count(50);

        assert_eq!(bar.volume, Some(1000000));
        assert_eq!(bar.tick_count, Some(50));
    }

    #[test]
    fn test_bar_calculations() {
        let bar = Bar::new(
            "EURUSD".to_string(),
            Timeframe::M1,
            1704067200000,
            1704067260000,
            1.0920,
            1.0930,
            1.0910,
            1.0925,
        );

        assert_eq!(bar.midpoint(), 1.0920);
        // Use epsilon comparison for floating point
        assert!((bar.range() - 0.0020).abs() < 1e-10);
        assert!(bar.is_bullish());
        assert!(!bar.is_bearish());
    }
}
