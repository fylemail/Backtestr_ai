use backtestr_data::{Bar, Timeframe};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BarEventType {
    BarOpened,
    BarClosed,
    BarUpdated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarEvent {
    pub event_type: BarEventType,
    pub bar: Bar,
    pub symbol: String,
    pub timeframe: Timeframe,
    pub timestamp: i64,
    pub sequence: u64,
}

impl BarEvent {
    pub fn new(event_type: BarEventType, bar: Bar, sequence: u64) -> Self {
        let symbol = bar.symbol.clone();
        let timeframe = bar.timeframe;
        let timestamp = match event_type {
            BarEventType::BarOpened => bar.timestamp_start,
            BarEventType::BarClosed => bar.timestamp_end,
            BarEventType::BarUpdated => chrono::Utc::now().timestamp_millis(),
        };

        Self {
            event_type,
            bar,
            symbol,
            timeframe,
            timestamp,
            sequence,
        }
    }

    pub fn bar_opened(bar: Bar, sequence: u64) -> Self {
        Self::new(BarEventType::BarOpened, bar, sequence)
    }

    pub fn bar_closed(bar: Bar, sequence: u64) -> Self {
        Self::new(BarEventType::BarClosed, bar, sequence)
    }

    pub fn bar_updated(bar: Bar, sequence: u64) -> Self {
        Self::new(BarEventType::BarUpdated, bar, sequence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bar_event_creation() {
        let bar = Bar::new(
            "EURUSD".to_string(),
            Timeframe::M1,
            1704067200000,
            1704067260000,
            1.0920,
            1.0925,
            1.0918,
            1.0923,
        );

        let event = BarEvent::bar_closed(bar.clone(), 100);

        assert_eq!(event.event_type, BarEventType::BarClosed);
        assert_eq!(event.symbol, "EURUSD");
        assert_eq!(event.timeframe, Timeframe::M1);
        assert_eq!(event.timestamp, 1704067260000);
        assert_eq!(event.sequence, 100);
    }

    #[test]
    fn test_different_event_types() {
        let bar = Bar::new(
            "EURUSD".to_string(),
            Timeframe::M1,
            1704067200000,
            1704067260000,
            1.0920,
            1.0925,
            1.0918,
            1.0923,
        );

        let opened = BarEvent::bar_opened(bar.clone(), 1);
        assert_eq!(opened.event_type, BarEventType::BarOpened);
        assert_eq!(opened.timestamp, 1704067200000);

        let closed = BarEvent::bar_closed(bar.clone(), 2);
        assert_eq!(closed.event_type, BarEventType::BarClosed);
        assert_eq!(closed.timestamp, 1704067260000);

        let updated = BarEvent::bar_updated(bar, 3);
        assert_eq!(updated.event_type, BarEventType::BarUpdated);
        assert!(updated.timestamp > 0);
    }
}
