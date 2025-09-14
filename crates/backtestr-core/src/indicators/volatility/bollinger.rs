use crate::indicators::indicator_trait::{BarData, Indicator};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct BollingerBands {
    period: usize,
    std_dev: f64,
    values: VecDeque<f64>,
    current_middle: Option<f64>,
    current_upper: Option<f64>,
    current_lower: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct BollingerOutput {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

impl BollingerBands {
    pub fn new(period: usize, std_dev: f64) -> Self {
        Self {
            period,
            std_dev,
            values: VecDeque::with_capacity(period),
            current_middle: None,
            current_upper: None,
            current_lower: None,
        }
    }

    pub fn get_bands(&self) -> Option<BollingerOutput> {
        if let (Some(upper), Some(middle), Some(lower)) =
            (self.current_upper, self.current_middle, self.current_lower)
        {
            Some(BollingerOutput {
                upper,
                middle,
                lower,
            })
        } else {
            None
        }
    }

    fn calculate_std_dev(&self, mean: f64) -> f64 {
        let variance = self
            .values
            .iter()
            .map(|&v| {
                let diff = v - mean;
                diff * diff
            })
            .sum::<f64>()
            / self.period as f64;

        variance.sqrt()
    }
}

impl Indicator for BollingerBands {
    type Input = BarData;
    type Output = f64;

    fn name(&self) -> &str {
        "BollingerBands"
    }

    fn warm_up_period(&self) -> usize {
        self.period
    }

    fn update(&mut self, input: BarData) -> Option<f64> {
        let value = input.close;
        self.values.push_back(value);

        if self.values.len() > self.period {
            self.values.pop_front();
        }

        if self.values.len() == self.period {
            let middle = self.values.iter().sum::<f64>() / self.period as f64;
            let std_deviation = self.calculate_std_dev(middle);

            let upper = middle + (self.std_dev * std_deviation);
            let lower = middle - (self.std_dev * std_deviation);

            self.current_middle = Some(middle);
            self.current_upper = Some(upper);
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
        self.values.clear();
        self.current_middle = None;
        self.current_upper = None;
        self.current_lower = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bollinger_bands_calculation() {
        let mut bb = BollingerBands::new(5, 2.0);

        let bars = vec![
            BarData {
                open: 100.0,
                high: 102.0,
                low: 99.0,
                close: 100.0,
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
            let result = bb.update(bar.clone());
            if i < 4 {
                assert!(result.is_none());
            } else {
                assert!(result.is_some());
                let bands = bb.get_bands().unwrap();
                assert!(bands.upper > bands.middle);
                assert!(bands.middle > bands.lower);
            }
        }
    }

    #[test]
    fn test_bollinger_squeeze() {
        let mut bb = BollingerBands::new(3, 2.0);

        for i in 0..5 {
            let bar = BarData {
                open: 100.0,
                high: 100.1,
                low: 99.9,
                close: 100.0,
                volume: 1000.0,
                timestamp: i as i64,
            };
            bb.update(bar);
        }

        let bands = bb.get_bands().unwrap();
        let bandwidth = bands.upper - bands.lower;
        assert!(bandwidth < 1.0); // Low volatility should create narrow bands
    }
}
