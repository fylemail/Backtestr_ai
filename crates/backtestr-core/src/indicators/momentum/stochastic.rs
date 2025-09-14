use crate::indicators::indicator_trait::{BarData, Indicator};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Stochastic {
    k_period: usize,
    d_period: usize,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
    k_values: VecDeque<f64>,
    current_k: Option<f64>,
    current_d: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct StochasticOutput {
    pub k: f64,
    pub d: f64,
}

impl Stochastic {
    pub fn new(k_period: usize, d_period: usize) -> Self {
        Self {
            k_period,
            d_period,
            highs: VecDeque::with_capacity(k_period),
            lows: VecDeque::with_capacity(k_period),
            k_values: VecDeque::with_capacity(d_period),
            current_k: None,
            current_d: None,
        }
    }

    pub fn get_output(&self) -> Option<StochasticOutput> {
        if let (Some(k), Some(d)) = (self.current_k, self.current_d) {
            Some(StochasticOutput { k, d })
        } else {
            None
        }
    }
}

impl Indicator for Stochastic {
    type Input = BarData;
    type Output = f64;

    fn name(&self) -> &str {
        "Stochastic"
    }

    fn warm_up_period(&self) -> usize {
        self.k_period + self.d_period - 1
    }

    fn update(&mut self, input: BarData) -> Option<f64> {
        self.highs.push_back(input.high);
        self.lows.push_back(input.low);

        if self.highs.len() > self.k_period {
            self.highs.pop_front();
            self.lows.pop_front();
        }

        if self.highs.len() == self.k_period {
            let highest = self.highs.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let lowest = self.lows.iter().fold(f64::INFINITY, |a, &b| a.min(b));

            let k = if highest > lowest {
                ((input.close - lowest) / (highest - lowest)) * 100.0
            } else {
                50.0
            };

            self.current_k = Some(k);
            self.k_values.push_back(k);

            if self.k_values.len() > self.d_period {
                self.k_values.pop_front();
            }

            if self.k_values.len() == self.d_period {
                let d = self.k_values.iter().sum::<f64>() / self.d_period as f64;
                self.current_d = Some(d);
                return Some(k);
            }
        }

        None
    }

    fn current(&self) -> Option<f64> {
        self.current_k
    }

    fn reset(&mut self) {
        self.highs.clear();
        self.lows.clear();
        self.k_values.clear();
        self.current_k = None;
        self.current_d = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stochastic_calculation() {
        let mut stoch = Stochastic::new(5, 3);

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
            BarData {
                open: 105.0,
                high: 107.0,
                low: 104.0,
                close: 106.0,
                volume: 1500.0,
                timestamp: 6,
            },
            BarData {
                open: 106.0,
                high: 108.0,
                low: 105.0,
                close: 107.0,
                volume: 1600.0,
                timestamp: 7,
            },
            BarData {
                open: 107.0,
                high: 109.0,
                low: 106.0,
                close: 108.0,
                volume: 1700.0,
                timestamp: 8,
            },
        ];

        for bar in bars {
            stoch.update(bar);
        }

        let output = stoch.get_output();
        assert!(output.is_some());

        let result = output.unwrap();
        assert!(result.k >= 0.0 && result.k <= 100.0);
        assert!(result.d >= 0.0 && result.d <= 100.0);
    }

    #[test]
    fn test_stochastic_extremes() {
        let mut stoch = Stochastic::new(3, 3);

        for i in 0..10 {
            let price = 100.0 + i as f64;
            let bar = BarData {
                open: price,
                high: price + 1.0,
                low: price,
                close: price + 0.9,
                volume: 1000.0,
                timestamp: i as i64,
            };
            stoch.update(bar);
        }

        let k = stoch.current().unwrap();
        assert!(k > 80.0); // Should be near top of range
    }
}
