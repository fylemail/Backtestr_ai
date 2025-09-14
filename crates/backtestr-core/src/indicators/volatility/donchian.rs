use std::collections::VecDeque;
use crate::indicators::indicator_trait::{BarData, Indicator};

#[derive(Debug)]
pub struct DonchianChannels {
    period: usize,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
    current_upper: Option<f64>,
    current_middle: Option<f64>,
    current_lower: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct DonchianOutput {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

impl DonchianChannels {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            highs: VecDeque::with_capacity(period),
            lows: VecDeque::with_capacity(period),
            current_upper: None,
            current_middle: None,
            current_lower: None,
        }
    }

    pub fn get_channels(&self) -> Option<DonchianOutput> {
        if let (Some(upper), Some(middle), Some(lower)) =
            (self.current_upper, self.current_middle, self.current_lower) {
            Some(DonchianOutput { upper, middle, lower })
        } else {
            None
        }
    }
}

impl Indicator for DonchianChannels {
    type Input = BarData;
    type Output = f64;

    fn name(&self) -> &str {
        "DonchianChannels"
    }

    fn warm_up_period(&self) -> usize {
        self.period
    }

    fn update(&mut self, input: BarData) -> Option<f64> {
        self.highs.push_back(input.high);
        self.lows.push_back(input.low);

        if self.highs.len() > self.period {
            self.highs.pop_front();
            self.lows.pop_front();
        }

        if self.highs.len() == self.period {
            let upper = self.highs.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let lower = self.lows.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let middle = (upper + lower) / 2.0;

            self.current_upper = Some(upper);
            self.current_middle = Some(middle);
            self.current_lower = Some(lower);

            Some(middle)
        } else {
            None
        }
    }

    fn current(&self) -> Option<f64> {
        self.current_middle
    }

    fn reset(&mut self) {
        self.highs.clear();
        self.lows.clear();
        self.current_upper = None;
        self.current_middle = None;
        self.current_lower = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_donchian_channels() {
        let mut dc = DonchianChannels::new(5);

        let bars = vec![
            BarData { open: 100.0, high: 102.0, low: 99.0, close: 101.0, volume: 1000.0, timestamp: 1 },
            BarData { open: 101.0, high: 103.0, low: 100.0, close: 102.0, volume: 1100.0, timestamp: 2 },
            BarData { open: 102.0, high: 104.0, low: 101.0, close: 103.0, volume: 1200.0, timestamp: 3 },
            BarData { open: 103.0, high: 105.0, low: 102.0, close: 104.0, volume: 1300.0, timestamp: 4 },
            BarData { open: 104.0, high: 106.0, low: 103.0, close: 105.0, volume: 1400.0, timestamp: 5 },
        ];

        for (i, bar) in bars.iter().enumerate() {
            let result = dc.update(bar.clone());
            if i < 4 {
                assert!(result.is_none());
            } else {
                assert!(result.is_some());
                let channels = dc.get_channels().unwrap();
                assert_eq!(channels.upper, 106.0); // Highest high
                assert_eq!(channels.lower, 99.0); // Lowest low
                assert_eq!(channels.middle, 102.5); // (106 + 99) / 2
            }
        }
    }

    #[test]
    fn test_donchian_breakout() {
        let mut dc = DonchianChannels::new(3);

        let bars = vec![
            BarData { open: 100.0, high: 101.0, low: 99.0, close: 100.0, volume: 1000.0, timestamp: 1 },
            BarData { open: 100.0, high: 101.0, low: 99.0, close: 100.0, volume: 1000.0, timestamp: 2 },
            BarData { open: 100.0, high: 101.0, low: 99.0, close: 100.0, volume: 1000.0, timestamp: 3 },
            BarData { open: 100.0, high: 110.0, low: 100.0, close: 109.0, volume: 2000.0, timestamp: 4 },
        ];

        for bar in &bars[..3] {
            dc.update(bar.clone());
        }

        let initial_channels = dc.get_channels().unwrap();
        assert_eq!(initial_channels.upper, 101.0);

        dc.update(bars[3].clone());
        let breakout_channels = dc.get_channels().unwrap();
        assert_eq!(breakout_channels.upper, 110.0); // New high
    }
}