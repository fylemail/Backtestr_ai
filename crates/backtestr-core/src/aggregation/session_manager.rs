use backtestr_data::timeframe::Timeframe;
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Weekday};
use chrono_tz::Tz;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct MarketHours {
    pub symbol: String,
    pub timezone: Tz,
    pub open_time: NaiveTime,
    pub close_time: NaiveTime,
    pub trading_days: Vec<Weekday>,
    pub session_break: Option<(NaiveTime, NaiveTime)>,
}

impl Default for MarketHours {
    fn default() -> Self {
        // Forex default (24/5) - Sunday 5pm ET to Friday 5pm ET
        MarketHours {
            symbol: "DEFAULT".to_string(),
            timezone: chrono_tz::US::Eastern,
            open_time: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            close_time: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            trading_days: vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ],
            session_break: None,
        }
    }
}

impl MarketHours {
    pub fn forex(symbol: &str) -> Self {
        MarketHours {
            symbol: symbol.to_string(),
            ..Default::default()
        }
    }

    pub fn stock_market(symbol: &str) -> Self {
        MarketHours {
            symbol: symbol.to_string(),
            timezone: chrono_tz::US::Eastern,
            open_time: NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
            close_time: NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            trading_days: vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ],
            session_break: None,
        }
    }

    pub fn futures(symbol: &str) -> Self {
        // CME futures (most common)
        MarketHours {
            symbol: symbol.to_string(),
            timezone: chrono_tz::US::Central,
            open_time: NaiveTime::from_hms_opt(17, 0, 0).unwrap(), // Sunday 5pm CT
            close_time: NaiveTime::from_hms_opt(16, 0, 0).unwrap(), // Friday 4pm CT
            trading_days: vec![
                Weekday::Sun,
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ],
            session_break: Some((
                NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            )),
        }
    }

    pub fn is_trading_time(&self, datetime: NaiveDateTime) -> bool {
        let weekday = datetime.weekday();

        // Check if it's a trading day
        if !self.trading_days.contains(&weekday) {
            return false;
        }

        let time = datetime.time();

        // Check session break
        if let Some((break_start, break_end)) = self.session_break {
            if time >= break_start && time < break_end {
                return false;
            }
        }

        // For 24-hour markets (forex), handle week boundaries
        if self.symbol.contains("USD") || self.symbol.contains("EUR") || self.symbol.contains("GBP")
        {
            // Sunday: only after open_time
            if weekday == Weekday::Sun {
                return time >= self.open_time;
            }
            // Friday: only before close_time
            if weekday == Weekday::Fri {
                return time < self.close_time;
            }
            // Monday-Thursday: all day
            return true;
        }

        // For regular hours markets
        time >= self.open_time && time < self.close_time
    }
}

#[derive(Debug, Clone, Default)]
pub struct MarketSchedule {
    pub holidays: HashSet<NaiveDate>,
    pub early_closes: HashMap<NaiveDate, NaiveTime>,
}

impl MarketSchedule {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_holiday(&mut self, date: NaiveDate) {
        self.holidays.insert(date);
    }

    pub fn add_early_close(&mut self, date: NaiveDate, close_time: NaiveTime) {
        self.early_closes.insert(date, close_time);
    }

    pub fn is_holiday(&self, date: NaiveDate) -> bool {
        self.holidays.contains(&date)
    }

    pub fn get_close_time(&self, date: NaiveDate) -> Option<NaiveTime> {
        self.early_closes.get(&date).copied()
    }
}

pub struct SessionManager {
    market_hours: HashMap<String, MarketHours>,
    market_schedule: MarketSchedule,
    session_close_times: HashMap<Timeframe, NaiveTime>,
}

impl Default for SessionManager {
    fn default() -> Self {
        let mut session_close_times = HashMap::new();

        // Default session close times (Eastern Time)
        session_close_times.insert(Timeframe::D1, NaiveTime::from_hms_opt(17, 0, 0).unwrap());
        session_close_times.insert(Timeframe::H4, NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        session_close_times.insert(Timeframe::H1, NaiveTime::from_hms_opt(0, 0, 0).unwrap());

        SessionManager {
            market_hours: HashMap::new(),
            market_schedule: MarketSchedule::new(),
            session_close_times,
        }
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_market_hours(&mut self, symbol: String, hours: MarketHours) {
        self.market_hours.insert(symbol, hours);
    }

    pub fn get_market_hours(&self, symbol: &str) -> MarketHours {
        self.market_hours
            .get(symbol)
            .cloned()
            .unwrap_or_else(|| MarketHours::forex(symbol))
    }

    pub fn set_session_close_time(&mut self, timeframe: Timeframe, close_time: NaiveTime) {
        self.session_close_times.insert(timeframe, close_time);
    }

    pub fn is_session_boundary(&self, timeframe: Timeframe, timestamp_ms: i64) -> bool {
        let datetime = DateTime::from_timestamp_millis(timestamp_ms).map(|dt| dt.naive_utc());
        if datetime.is_none() {
            return false;
        }
        let dt = datetime.unwrap();

        match timeframe {
            Timeframe::D1 => {
                // Daily bars close at configured time (default 5pm ET)
                if let Some(close_time) = self.session_close_times.get(&Timeframe::D1) {
                    return dt.time() == *close_time;
                }
                false
            }
            Timeframe::H4 => {
                // 4-hour bars align with specific times
                let hour = dt.hour();
                hour % 4 == 0 && dt.minute() == 0 && dt.second() == 0
            }
            Timeframe::H1 => {
                // Hourly bars close at the top of each hour
                dt.minute() == 0 && dt.second() == 0
            }
            Timeframe::M15 => {
                // 15-minute bars
                dt.minute() % 15 == 0 && dt.second() == 0
            }
            Timeframe::M5 => {
                // 5-minute bars
                dt.minute() % 5 == 0 && dt.second() == 0
            }
            Timeframe::M1 => {
                // 1-minute bars
                dt.second() == 0
            }
        }
    }

    pub fn is_market_open(&self, symbol: &str, timestamp_ms: i64) -> bool {
        let datetime = DateTime::from_timestamp_millis(timestamp_ms).map(|dt| dt.naive_utc());
        if datetime.is_none() {
            return false;
        }
        let dt = datetime.unwrap();

        // Check if it's a holiday
        if self.market_schedule.is_holiday(dt.date()) {
            return false;
        }

        // Get market hours for this symbol
        let hours = self.get_market_hours(symbol);
        hours.is_trading_time(dt)
    }

    pub fn get_next_session_open(&self, symbol: &str, timestamp_ms: i64) -> Option<i64> {
        let datetime = DateTime::from_timestamp_millis(timestamp_ms)?.naive_utc();
        let hours = self.get_market_hours(symbol);

        // Find next trading day
        let mut current_date = datetime.date();
        for _ in 0..7 {
            current_date = current_date.succ_opt()?;
            let weekday = current_date.weekday();

            if hours.trading_days.contains(&weekday)
                && !self.market_schedule.is_holiday(current_date)
            {
                let open_datetime = NaiveDateTime::new(current_date, hours.open_time);
                return Some(open_datetime.and_utc().timestamp_millis());
            }
        }

        None
    }

    pub fn get_session_close(&self, symbol: &str, timestamp_ms: i64) -> Option<i64> {
        let datetime = DateTime::from_timestamp_millis(timestamp_ms)?.naive_utc();
        let hours = self.get_market_hours(symbol);

        // Check for early close
        if let Some(early_close) = self.market_schedule.get_close_time(datetime.date()) {
            let close_datetime = NaiveDateTime::new(datetime.date(), early_close);
            return Some(close_datetime.and_utc().timestamp_millis());
        }

        // Regular close
        let close_datetime = NaiveDateTime::new(datetime.date(), hours.close_time);
        Some(close_datetime.and_utc().timestamp_millis())
    }

    pub fn is_weekly_boundary(&self, timestamp_ms: i64) -> bool {
        let datetime = DateTime::from_timestamp_millis(timestamp_ms).map(|dt| dt.naive_utc());
        if datetime.is_none() {
            return false;
        }
        let dt = datetime.unwrap();

        // Weekly bars close on Friday at market close
        dt.weekday() == Weekday::Fri && dt.hour() == 17 && dt.minute() == 0
    }

    pub fn is_monthly_boundary(&self, timestamp_ms: i64) -> bool {
        let datetime = DateTime::from_timestamp_millis(timestamp_ms).map(|dt| dt.naive_utc());
        if datetime.is_none() {
            return false;
        }
        let dt = datetime.unwrap();

        // Check if it's the last trading day of the month
        let next_day = dt.date().succ_opt();
        if let Some(next) = next_day {
            if next.month() != dt.month() {
                // It's the last day of the month
                return dt.hour() == 17 && dt.minute() == 0;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_market_hours() {
        let hours = MarketHours::default();
        assert_eq!(hours.symbol, "DEFAULT");
        assert_eq!(hours.open_time, NaiveTime::from_hms_opt(17, 0, 0).unwrap());
        assert_eq!(hours.close_time, NaiveTime::from_hms_opt(17, 0, 0).unwrap());
        assert_eq!(hours.trading_days.len(), 5);
    }

    #[test]
    fn test_forex_market_hours() {
        let hours = MarketHours::forex("EURUSD");
        assert_eq!(hours.symbol, "EURUSD");
        assert_eq!(hours.trading_days.len(), 5);
    }

    #[test]
    fn test_stock_market_hours() {
        let hours = MarketHours::stock_market("AAPL");
        assert_eq!(hours.symbol, "AAPL");
        assert_eq!(hours.open_time, NaiveTime::from_hms_opt(9, 30, 0).unwrap());
        assert_eq!(hours.close_time, NaiveTime::from_hms_opt(16, 0, 0).unwrap());
    }

    #[test]
    fn test_session_boundary_detection() {
        let manager = SessionManager::new();

        // Test daily boundary (5pm)
        let timestamp = NaiveDateTime::parse_from_str("2024-01-01 17:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc()
            .timestamp_millis();
        assert!(manager.is_session_boundary(Timeframe::D1, timestamp));

        // Test hourly boundary
        let timestamp = NaiveDateTime::parse_from_str("2024-01-01 14:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc()
            .timestamp_millis();
        assert!(manager.is_session_boundary(Timeframe::H1, timestamp));

        // Test non-boundary
        let timestamp = NaiveDateTime::parse_from_str("2024-01-01 14:30:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc()
            .timestamp_millis();
        assert!(!manager.is_session_boundary(Timeframe::H1, timestamp));
    }

    #[test]
    fn test_market_schedule() {
        let mut schedule = MarketSchedule::new();
        let holiday = NaiveDate::from_ymd_opt(2024, 12, 25).unwrap();

        schedule.add_holiday(holiday);
        assert!(schedule.is_holiday(holiday));

        let early_close_date = NaiveDate::from_ymd_opt(2024, 12, 24).unwrap();
        let early_close_time = NaiveTime::from_hms_opt(13, 0, 0).unwrap();

        schedule.add_early_close(early_close_date, early_close_time);
        assert_eq!(
            schedule.get_close_time(early_close_date),
            Some(early_close_time)
        );
    }
}
