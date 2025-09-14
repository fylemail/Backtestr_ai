use crate::indicators::indicator_trait::{BarData, Indicator};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct WilliamsR {
    period: usize,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
    current_value: Option<f64>,
}

impl WilliamsR {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            highs: VecDeque::with_capacity(period),
            lows: VecDeque::with_capacity(period),
            current_value: None,
        }
    }
}

impl Indicator for WilliamsR {
    type Input = BarData;
    type Output = f64;

    fn name(&self) -> &str {
        "Williams%R"
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
            let highest = self.highs.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let lowest = self.lows.iter().fold(f64::INFINITY, |a, &b| a.min(b));

            let williams_r = if highest > lowest {
                ((highest - input.close) / (highest - lowest)) * -100.0
            } else {
                -50.0
            };

            self.current_value = Some(williams_r);
            Some(williams_r)
        } else {
            None
        }
    }

    fn current(&self) -> Option<f64> {
        self.current_value
    }

    fn reset(&mut self) {
        self.highs.clear();
        self.lows.clear();
        self.current_value = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_williams_r_calculation() {
        let mut williams = WilliamsR::new(5);

        let bars = vec![
            BarData {
                open: 100.0,
                high: 102.0,
                low: 99.0,
                close: 101.0,
                volume: 1000.0,
                timestamp: 1,
            },
            BarData {
                open: 101.0,
                high: 103.0,
                low: 100.0,
                close: 102.0,
                volume: 1100.0,
                timestamp: 2,
            },
            BarData {
                open: 102.0,
                high: 104.0,
                low: 101.0,
                close: 103.0,
                volume: 1200.0,
                timestamp: 3,
            },
            BarData {
                open: 103.0,
                high: 105.0,
                low: 102.0,
                close: 104.0,
                volume: 1300.0,
                timestamp: 4,
            },
            BarData {
                open: 104.0,
                high: 106.0,
                low: 103.0,
                close: 105.0,
                volume: 1400.0,
                timestamp: 5,
            },
        ];

        for (i, bar) in bars.iter().enumerate() {
            let result = williams.update(bar.clone());
            if i < 4 {
                assert!(result.is_none());
            } else {
                assert!(result.is_some());
                let value = result.unwrap();
                assert!(value >= -100.0 && value <= 0.0);
            }
        }
    }

    #[test]
    fn test_williams_r_extremes() {
        let mut williams = WilliamsR::new(3);

        for i in 0..5 {
            let bar = BarData {
                open: 100.0 + i as f64,
                high: 100.0 + i as f64 + 1.0,
                low: 100.0 + i as f64,
                close: 100.0 + i as f64 + 0.9,
                volume: 1000.0,
                timestamp: i as i64,
            };
            williams.update(bar);
        }

        let value = williams.current().unwrap();
        assert!(value > -20.0); // Should be near -0 (overbought)

        williams.reset();

        for i in 0..5 {
            let bar = BarData {
                open: 100.0,
                high: 100.0 + 0.1,
                low: 100.0 - i as f64,   // Price going lower
                close: 100.0 - i as f64, // Close at the low
                volume: 1000.0,
                timestamp: i as i64,
            };
            williams.update(bar);
        }

        let value = williams.current().unwrap();
        // When closing at the low, Williams %R should be near -100
        assert!(value < -90.0); // Should be deeply oversold
    }
}
