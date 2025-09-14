use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Timeframe {
    M1,  // 1 minute
    M5,  // 5 minutes
    M15, // 15 minutes
    H1,  // 1 hour
    H4,  // 4 hours
    D1,  // 1 day
}

impl Timeframe {
    /// Returns duration in milliseconds
    pub fn duration_ms(&self) -> i64 {
        match self {
            Timeframe::M1 => 60_000,
            Timeframe::M5 => 300_000,
            Timeframe::M15 => 900_000,
            Timeframe::H1 => 3_600_000,
            Timeframe::H4 => 14_400_000,
            Timeframe::D1 => 86_400_000,
        }
    }

    /// Returns duration in seconds
    pub fn duration_secs(&self) -> i64 {
        self.duration_ms() / 1000
    }

    /// Returns human-readable string
    pub fn as_str(&self) -> &str {
        match self {
            Timeframe::M1 => "1m",
            Timeframe::M5 => "5m",
            Timeframe::M15 => "15m",
            Timeframe::H1 => "1h",
            Timeframe::H4 => "4h",
            Timeframe::D1 => "1d",
        }
    }

    /// Returns all available timeframes
    pub fn all() -> Vec<Timeframe> {
        vec![
            Timeframe::M1,
            Timeframe::M5,
            Timeframe::M15,
            Timeframe::H1,
            Timeframe::H4,
            Timeframe::D1,
        ]
    }

    /// Calculate the start timestamp for a bar given a tick timestamp
    pub fn bar_start_timestamp(&self, tick_timestamp: i64) -> i64 {
        let duration = self.duration_ms();
        (tick_timestamp / duration) * duration
    }

    /// Calculate the end timestamp for a bar given its start timestamp
    pub fn bar_end_timestamp(&self, bar_start: i64) -> i64 {
        bar_start + self.duration_ms()
    }

    /// Check if a timestamp is at a bar boundary
    pub fn is_bar_boundary(&self, timestamp: i64) -> bool {
        timestamp % self.duration_ms() == 0
    }
}

impl fmt::Display for Timeframe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Timeframe {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "1m" | "m1" => Ok(Timeframe::M1),
            "5m" | "m5" => Ok(Timeframe::M5),
            "15m" | "m15" => Ok(Timeframe::M15),
            "1h" | "h1" | "60m" => Ok(Timeframe::H1),
            "4h" | "h4" | "240m" => Ok(Timeframe::H4),
            "1d" | "d1" | "daily" => Ok(Timeframe::D1),
            _ => Err(format!("Invalid timeframe: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeframe_duration() {
        assert_eq!(Timeframe::M1.duration_ms(), 60_000);
        assert_eq!(Timeframe::M5.duration_ms(), 300_000);
        assert_eq!(Timeframe::M15.duration_ms(), 900_000);
        assert_eq!(Timeframe::H1.duration_ms(), 3_600_000);
        assert_eq!(Timeframe::H4.duration_ms(), 14_400_000);
        assert_eq!(Timeframe::D1.duration_ms(), 86_400_000);
    }

    #[test]
    fn test_timeframe_duration_secs() {
        assert_eq!(Timeframe::M1.duration_secs(), 60);
        assert_eq!(Timeframe::M5.duration_secs(), 300);
        assert_eq!(Timeframe::H1.duration_secs(), 3600);
    }

    #[test]
    fn test_timeframe_as_str() {
        assert_eq!(Timeframe::M1.as_str(), "1m");
        assert_eq!(Timeframe::M5.as_str(), "5m");
        assert_eq!(Timeframe::M15.as_str(), "15m");
        assert_eq!(Timeframe::H1.as_str(), "1h");
        assert_eq!(Timeframe::H4.as_str(), "4h");
        assert_eq!(Timeframe::D1.as_str(), "1d");
    }

    #[test]
    fn test_timeframe_from_str() {
        assert_eq!(Timeframe::from_str("1m").unwrap(), Timeframe::M1);
        assert_eq!(Timeframe::from_str("M1").unwrap(), Timeframe::M1);
        assert_eq!(Timeframe::from_str("5m").unwrap(), Timeframe::M5);
        assert_eq!(Timeframe::from_str("1h").unwrap(), Timeframe::H1);
        assert_eq!(Timeframe::from_str("H1").unwrap(), Timeframe::H1);
        assert_eq!(Timeframe::from_str("60m").unwrap(), Timeframe::H1);
        assert_eq!(Timeframe::from_str("daily").unwrap(), Timeframe::D1);
        assert!(Timeframe::from_str("invalid").is_err());
    }

    #[test]
    fn test_bar_start_timestamp() {
        let tf = Timeframe::M1;
        // 2024-01-01 00:00:30.500 (30.5 seconds into the minute)
        let tick_timestamp = 1704067230500;
        // Should round down to 2024-01-01 00:00:00
        let expected_start = 1704067200000;
        assert_eq!(tf.bar_start_timestamp(tick_timestamp), expected_start);

        // Test with 5-minute timeframe
        let tf = Timeframe::M5;
        // 2024-01-01 00:03:30 (3.5 minutes)
        let tick_timestamp = 1704067410000;
        // Should round down to 2024-01-01 00:00:00
        let expected_start = 1704067200000;
        assert_eq!(tf.bar_start_timestamp(tick_timestamp), expected_start);
    }

    #[test]
    fn test_bar_end_timestamp() {
        let tf = Timeframe::M1;
        let bar_start = 1704067200000; // 2024-01-01 00:00:00
        let expected_end = 1704067260000; // 2024-01-01 00:01:00
        assert_eq!(tf.bar_end_timestamp(bar_start), expected_end);
    }

    #[test]
    fn test_is_bar_boundary() {
        let tf = Timeframe::M1;
        assert!(tf.is_bar_boundary(1704067200000)); // Exactly on minute
        assert!(!tf.is_bar_boundary(1704067230000)); // 30 seconds into minute
    }
}
