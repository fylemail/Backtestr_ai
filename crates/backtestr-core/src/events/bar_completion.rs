use backtestr_data::models::Bar;
use std::fmt;

#[derive(Debug, Clone)]
pub enum BarCompletionEvent {
    MinuteBar(Bar),
    FiveMinuteBar(Bar),
    FifteenMinuteBar(Bar),
    HourBar(Bar),
    FourHourBar(Bar),
    DailyBar(Bar),
}

impl BarCompletionEvent {
    pub fn bar(&self) -> &Bar {
        match self {
            Self::MinuteBar(bar) |
            Self::FiveMinuteBar(bar) |
            Self::FifteenMinuteBar(bar) |
            Self::HourBar(bar) |
            Self::FourHourBar(bar) |
            Self::DailyBar(bar) => bar,
        }
    }

    pub fn timeframe_name(&self) -> &str {
        match self {
            Self::MinuteBar(_) => "1M",
            Self::FiveMinuteBar(_) => "5M",
            Self::FifteenMinuteBar(_) => "15M",
            Self::HourBar(_) => "1H",
            Self::FourHourBar(_) => "4H",
            Self::DailyBar(_) => "D1",
        }
    }

    pub fn timestamp(&self) -> i64 {
        self.bar().timestamp_end
    }
}

impl fmt::Display for BarCompletionEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bar = self.bar();
        write!(
            f,
            "{} Bar Completed: {} O:{:.5} H:{:.5} L:{:.5} C:{:.5}",
            self.timeframe_name(),
            bar.symbol,
            bar.open,
            bar.high,
            bar.low,
            bar.close
        )
    }
}