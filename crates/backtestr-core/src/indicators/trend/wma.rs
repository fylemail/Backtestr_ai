use crate::indicators::indicator_trait::{BarData, Indicator};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct WMA {
    period: usize,
    values: VecDeque<f64>,
    weight_sum: f64,
    current_value: Option<f64>,
}

impl WMA {
    pub fn new(period: usize) -> Self {
        let weight_sum = (period * (period + 1)) as f64 / 2.0;
        Self {
            period,
            values: VecDeque::with_capacity(period),
            weight_sum,
            current_value: None,
        }
    }
}

impl Indicator for WMA {
    type Input = BarData;
    type Output = f64;

    fn name(&self) -> &str {
        "WMA"
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
            let mut weighted_sum = 0.0;
            for (i, &val) in self.values.iter().enumerate() {
                let weight = (i + 1) as f64;
                weighted_sum += val * weight;
            }

            let wma = weighted_sum / self.weight_sum;
            self.current_value = Some(wma);
            Some(wma)
        } else {
            None
        }
    }

    fn current(&self) -> Option<f64> {
        self.current_value
    }

    fn reset(&mut self) {
        self.values.clear();
        self.current_value = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wma_calculation() {
        let mut wma = WMA::new(3);

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
        ];

        assert_eq!(wma.update(bars[0].clone()), None);
        assert_eq!(wma.update(bars[1].clone()), None);

        let result = wma.update(bars[2].clone());
        assert!(result.is_some());
        // WMA = (100*1 + 102*2 + 103*3) / (1+2+3) = (100 + 204 + 309) / 6 = 102.167
        assert!((result.unwrap() - 102.167).abs() < 0.001);
    }

    #[test]
    fn test_wma_weights() {
        let mut wma = WMA::new(4);

        for i in 1..=4 {
            let bar = BarData {
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: i as f64,
                volume: 1000.0,
                timestamp: i,
            };
            wma.update(bar);
        }

        let result = wma.current().unwrap();
        // WMA = (1*1 + 2*2 + 3*3 + 4*4) / (1+2+3+4) = (1 + 4 + 9 + 16) / 10 = 3.0
        assert!((result - 3.0).abs() < 0.001);
    }
}
