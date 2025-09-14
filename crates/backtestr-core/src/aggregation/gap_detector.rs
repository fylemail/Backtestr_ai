use backtestr_data::models::Bar;
use chrono::{Datelike, DateTime, Duration};
#[cfg(test)]
use chrono::NaiveDateTime;

use super::MarketSchedule;

pub struct GapDetector {
    max_gap_duration: Duration,
    market_schedule: MarketSchedule,
}

impl GapDetector {
    pub fn new(max_gap_duration: Duration) -> Self {
        Self {
            max_gap_duration,
            market_schedule: MarketSchedule::new(),
        }
    }

    pub fn with_schedule(mut self, schedule: MarketSchedule) -> Self {
        self.market_schedule = schedule;
        self
    }

    pub fn has_gap(&self, bars: &[Bar]) -> bool {
        if bars.len() < 2 {
            return false;
        }

        for i in 1..bars.len() {
            if self.is_gap(&bars[i - 1], &bars[i]) {
                return true;
            }
        }

        false
    }

    pub fn is_gap(&self, prev_bar: &Bar, next_bar: &Bar) -> bool {
        let gap_duration_ms = next_bar.timestamp_start - prev_bar.timestamp_end;

        // Convert to Duration
        let gap_duration = Duration::milliseconds(gap_duration_ms);

        // Check if gap exceeds maximum allowed
        if gap_duration > self.max_gap_duration {
            // Check if it's a weekend or holiday gap
            if !self.is_expected_gap(prev_bar.timestamp_end, next_bar.timestamp_start) {
                return true;
            }
        }

        false
    }

    pub fn is_expected_gap(&self, prev_end_ms: i64, next_start_ms: i64) -> bool {
        let prev_end = DateTime::from_timestamp_millis(prev_end_ms).map(|dt| dt.naive_utc());
        let next_start = DateTime::from_timestamp_millis(next_start_ms).map(|dt| dt.naive_utc());

        if prev_end.is_none() || next_start.is_none() {
            return false;
        }

        let prev_dt = prev_end.unwrap();
        let next_dt = next_start.unwrap();

        // Check for weekend gap (Friday to Sunday/Monday)
        if prev_dt.weekday() == chrono::Weekday::Fri &&
           (next_dt.weekday() == chrono::Weekday::Sun || next_dt.weekday() == chrono::Weekday::Mon) {
            return true;
        }

        // Check for holiday gaps
        let mut current_date = prev_dt.date();
        while current_date < next_dt.date() {
            if self.market_schedule.is_holiday(current_date) {
                return true;
            }
            current_date = current_date.succ_opt().unwrap_or(current_date);
        }

        false
    }

    pub fn find_gaps(&self, bars: &[Bar]) -> Vec<GapInfo> {
        let mut gaps = Vec::new();

        for i in 1..bars.len() {
            let prev_bar = &bars[i - 1];
            let next_bar = &bars[i];

            if self.is_gap(prev_bar, next_bar) {
                let gap_type = self.classify_gap(prev_bar, next_bar);
                gaps.push(GapInfo {
                    start_timestamp: prev_bar.timestamp_end,
                    end_timestamp: next_bar.timestamp_start,
                    duration_ms: next_bar.timestamp_start - prev_bar.timestamp_end,
                    gap_type,
                    prev_bar_index: i - 1,
                    next_bar_index: i,
                });
            }
        }

        gaps
    }

    fn classify_gap(&self, prev_bar: &Bar, next_bar: &Bar) -> GapType {
        let prev_end = DateTime::from_timestamp_millis(prev_bar.timestamp_end).map(|dt| dt.naive_utc());
        let next_start = DateTime::from_timestamp_millis(next_bar.timestamp_start).map(|dt| dt.naive_utc());

        if prev_end.is_none() || next_start.is_none() {
            return GapType::Unknown;
        }

        let prev_dt = prev_end.unwrap();
        let next_dt = next_start.unwrap();

        // Weekend gap
        if prev_dt.weekday() == chrono::Weekday::Fri &&
           (next_dt.weekday() == chrono::Weekday::Sun || next_dt.weekday() == chrono::Weekday::Mon) {
            return GapType::Weekend;
        }

        // Check for holiday
        if self.market_schedule.is_holiday(prev_dt.date()) ||
           self.market_schedule.is_holiday(next_dt.date()) {
            return GapType::Holiday;
        }

        // Price gap (significant price movement)
        let price_gap = (next_bar.open - prev_bar.close).abs();
        let avg_price = (next_bar.open + prev_bar.close) / 2.0;
        let gap_percentage = (price_gap / avg_price) * 100.0;

        if gap_percentage > 0.5 {
            return GapType::Price;
        }

        GapType::Data
    }

    pub fn fill_gap(&self, prev_bar: &Bar, next_bar: &Bar, timeframe: backtestr_data::timeframe::Timeframe) -> Vec<Bar> {
        let mut filled_bars = Vec::new();
        let gap_duration_ms = next_bar.timestamp_start - prev_bar.timestamp_end;
        let bar_duration_ms = timeframe.duration_ms();

        if gap_duration_ms <= bar_duration_ms {
            return filled_bars;
        }

        // Create synthetic bars to fill the gap
        let num_bars = (gap_duration_ms / bar_duration_ms) as usize;
        let mut current_timestamp = prev_bar.timestamp_end;

        for _ in 0..num_bars {
            let bar = Bar::new(
                prev_bar.symbol.clone(),
                timeframe,
                current_timestamp,
                current_timestamp + bar_duration_ms,
                prev_bar.close, // Use previous close as OHLC
                prev_bar.close,
                prev_bar.close,
                prev_bar.close,
            );

            filled_bars.push(bar);
            current_timestamp += bar_duration_ms;

            if current_timestamp >= next_bar.timestamp_start {
                break;
            }
        }

        filled_bars
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GapType {
    Weekend,
    Holiday,
    Price,
    Data,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct GapInfo {
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub duration_ms: i64,
    pub gap_type: GapType,
    pub prev_bar_index: usize,
    pub next_bar_index: usize,
}

impl GapInfo {
    pub fn duration_hours(&self) -> f64 {
        self.duration_ms as f64 / (1000.0 * 60.0 * 60.0)
    }

    pub fn is_significant(&self) -> bool {
        matches!(self.gap_type, GapType::Price | GapType::Data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use backtestr_data::timeframe::Timeframe;

    fn create_test_bar(symbol: &str, start: i64, end: i64) -> Bar {
        Bar::new(
            symbol.to_string(),
            Timeframe::M1,
            start,
            end,
            1.0920,
            1.0925,
            1.0915,
            1.0922,
        )
    }

    #[test]
    fn test_gap_detection() {
        let detector = GapDetector::new(Duration::minutes(5));

        // Create bars with a gap
        let bar1 = create_test_bar("EURUSD", 1704067200000, 1704067260000); // 00:00 - 00:01
        let bar2 = create_test_bar("EURUSD", 1704067800000, 1704067860000); // 00:10 - 00:11

        assert!(detector.is_gap(&bar1, &bar2));
    }

    #[test]
    fn test_no_gap() {
        let detector = GapDetector::new(Duration::minutes(5));

        // Create consecutive bars
        let bar1 = create_test_bar("EURUSD", 1704067200000, 1704067260000); // 00:00 - 00:01
        let bar2 = create_test_bar("EURUSD", 1704067260000, 1704067320000); // 00:01 - 00:02

        assert!(!detector.is_gap(&bar1, &bar2));
    }

    #[test]
    fn test_weekend_gap() {
        let detector = GapDetector::new(Duration::hours(48));

        // Friday 5pm to Sunday 5pm (weekend gap)
        let friday_close = NaiveDateTime::parse_from_str("2024-01-05 17:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc()
            .timestamp_millis();
        let sunday_open = NaiveDateTime::parse_from_str("2024-01-07 17:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc()
            .timestamp_millis();

        let bar1 = create_test_bar("EURUSD", friday_close - 60000, friday_close);
        let bar2 = create_test_bar("EURUSD", sunday_open, sunday_open + 60000);

        assert!(detector.is_expected_gap(bar1.timestamp_end, bar2.timestamp_start));
    }

    #[test]
    fn test_find_multiple_gaps() {
        let detector = GapDetector::new(Duration::minutes(5));

        let bars = vec![
            create_test_bar("EURUSD", 1704067200000, 1704067260000), // 00:00 - 00:01
            create_test_bar("EURUSD", 1704067260000, 1704067320000), // 00:01 - 00:02
            create_test_bar("EURUSD", 1704067800000, 1704067860000), // 00:10 - 00:11 (gap)
            create_test_bar("EURUSD", 1704067860000, 1704067920000), // 00:11 - 00:12
            create_test_bar("EURUSD", 1704068400000, 1704068460000), // 00:20 - 00:21 (gap)
        ];

        let gaps = detector.find_gaps(&bars);
        assert_eq!(gaps.len(), 2);
        assert_eq!(gaps[0].prev_bar_index, 1);
        assert_eq!(gaps[0].next_bar_index, 2);
        assert_eq!(gaps[1].prev_bar_index, 3);
        assert_eq!(gaps[1].next_bar_index, 4);
    }

    #[test]
    fn test_gap_filling() {
        let detector = GapDetector::new(Duration::minutes(5));

        let bar1 = create_test_bar("EURUSD", 1704067200000, 1704067260000); // 00:00 - 00:01
        let bar2 = create_test_bar("EURUSD", 1704067500000, 1704067560000); // 00:05 - 00:06

        let filled = detector.fill_gap(&bar1, &bar2, Timeframe::M1);
        assert_eq!(filled.len(), 4); // Should create 4 bars to fill the gap

        // Check that filled bars have correct timestamps
        assert_eq!(filled[0].timestamp_start, 1704067260000);
        assert_eq!(filled[0].timestamp_end, 1704067320000);
        assert_eq!(filled[3].timestamp_start, 1704067440000);
        assert_eq!(filled[3].timestamp_end, 1704067500000);
    }
}