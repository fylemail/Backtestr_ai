use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tick {
    pub id: Option<i64>,
    pub symbol: String,
    pub timestamp: i64, // Milliseconds since Unix epoch
    pub bid: f64,
    pub ask: f64,
    pub bid_size: Option<i64>,
    pub ask_size: Option<i64>,
}

impl Tick {
    pub fn new(symbol: String, timestamp: DateTime<Utc>, bid: f64, ask: f64) -> Self {
        Self {
            id: None,
            symbol,
            timestamp: timestamp.timestamp_millis(),
            bid,
            ask,
            bid_size: None,
            ask_size: None,
        }
    }

    pub fn new_with_millis(symbol: String, timestamp: i64, bid: f64, ask: f64) -> Self {
        Self {
            id: None,
            symbol,
            timestamp,
            bid,
            ask,
            bid_size: None,
            ask_size: None,
        }
    }

    pub fn with_sizes(mut self, bid_size: i64, ask_size: i64) -> Self {
        self.bid_size = Some(bid_size);
        self.ask_size = Some(ask_size);
        self
    }

    pub fn timestamp_as_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp_millis(self.timestamp).unwrap_or_else(Utc::now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_creation() {
        let timestamp = Utc::now();
        let tick = Tick::new("EURUSD".to_string(), timestamp, 1.0921, 1.0923);

        assert_eq!(tick.symbol, "EURUSD");
        assert_eq!(tick.bid, 1.0921);
        assert_eq!(tick.ask, 1.0923);
        assert_eq!(tick.bid_size, None);
        assert_eq!(tick.ask_size, None);
    }

    #[test]
    fn test_tick_with_sizes() {
        let timestamp = Utc::now();
        let tick =
            Tick::new("EURUSD".to_string(), timestamp, 1.0921, 1.0923).with_sizes(1000000, 1000000);

        assert_eq!(tick.bid_size, Some(1000000));
        assert_eq!(tick.ask_size, Some(1000000));
    }
}
