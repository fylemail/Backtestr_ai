use crate::indicators::indicator_trait::{BarData, Indicator};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct RSI {
    period: usize,
    gains: VecDeque<f64>,
    losses: VecDeque<f64>,
    avg_gain: Option<f64>,
    avg_loss: Option<f64>,
    previous_close: Option<f64>,
    current_value: Option<f64>,
}

impl RSI {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            gains: VecDeque::with_capacity(period),
            losses: VecDeque::with_capacity(period),
            avg_gain: None,
            avg_loss: None,
            previous_close: None,
            current_value: None,
        }
    }
}

impl Indicator for RSI {
    type Input = BarData;
    type Output = f64;

    fn name(&self) -> &str {
        "RSI"
    }

    fn warm_up_period(&self) -> usize {
        self.period + 1
    }

    fn update(&mut self, input: BarData) -> Option<f64> {
        let close = input.close;

        if let Some(prev) = self.previous_close {
            let change = close - prev;
            let gain = change.max(0.0);
            let loss = (-change).max(0.0);

            self.gains.push_back(gain);
            self.losses.push_back(loss);

            if self.gains.len() > self.period {
                self.gains.pop_front();
                self.losses.pop_front();
            }

            if self.gains.len() == self.period {
                let avg_gain = if let Some(prev_avg_gain) = self.avg_gain {
                    (prev_avg_gain * (self.period - 1) as f64 + gain) / self.period as f64
                } else {
                    self.gains.iter().sum::<f64>() / self.period as f64
                };

                let avg_loss = if let Some(prev_avg_loss) = self.avg_loss {
                    (prev_avg_loss * (self.period - 1) as f64 + loss) / self.period as f64
                } else {
                    self.losses.iter().sum::<f64>() / self.period as f64
                };

                let rsi = if avg_loss == 0.0 {
                    100.0
                } else if avg_gain == 0.0 {
                    0.0
                } else {
                    let rs = avg_gain / avg_loss;
                    100.0 - (100.0 / (1.0 + rs))
                };

                self.avg_gain = Some(avg_gain);
                self.avg_loss = Some(avg_loss);
                self.previous_close = Some(close);
                self.current_value = Some(rsi);

                return Some(rsi);
            }
        }

        self.previous_close = Some(close);
        None
    }

    fn current(&self) -> Option<f64> {
        self.current_value
    }

    fn reset(&mut self) {
        self.gains.clear();
        self.losses.clear();
        self.avg_gain = None;
        self.avg_loss = None;
        self.previous_close = None;
        self.current_value = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_calculation() {
        let mut rsi = RSI::new(14);

        let prices = vec![
            44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03,
            45.61, 46.28, 46.28, 46.00,
        ];

        for (i, &price) in prices.iter().enumerate() {
            let bar = BarData {
                open: price,
                high: price,
                low: price,
                close: price,
                volume: 1000.0,
                timestamp: i as i64,
            };

            if let Some(value) = rsi.update(bar) {
                assert!(value >= 0.0 && value <= 100.0);
            }
        }

        assert!(rsi.current().is_some());
    }

    #[test]
    fn test_rsi_extremes() {
        let mut rsi = RSI::new(5);

        for i in 0..10 {
            let price = 100.0 + i as f64;
            let bar = BarData {
                open: price,
                high: price,
                low: price,
                close: price,
                volume: 1000.0,
                timestamp: i as i64,
            };
            rsi.update(bar);
        }

        let value = rsi.current().unwrap();
        assert!(value > 70.0); // Should be overbought

        rsi.reset();

        for i in 0..10 {
            let price = 100.0 - i as f64;
            let bar = BarData {
                open: price,
                high: price,
                low: price,
                close: price,
                volume: 1000.0,
                timestamp: i as i64,
            };
            rsi.update(bar);
        }

        let value = rsi.current().unwrap();
        assert!(value < 30.0); // Should be oversold
    }
}
