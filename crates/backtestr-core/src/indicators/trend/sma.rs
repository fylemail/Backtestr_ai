use std::collections::VecDeque;
use crate::indicators::indicator_trait::{BarData, Indicator};

#[derive(Debug)]
pub struct SMA {
    period: usize,
    values: VecDeque<f64>,
    sum: f64,
    current_value: Option<f64>,
}

impl SMA {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            values: VecDeque::with_capacity(period),
            sum: 0.0,
            current_value: None,
        }
    }
}

impl Indicator for SMA {
    type Input = BarData;
    type Output = f64;

    fn name(&self) -> &str {
        "SMA"
    }

    fn warm_up_period(&self) -> usize {
        self.period
    }

    fn update(&mut self, input: BarData) -> Option<f64> {
        let value = input.close;

        self.values.push_back(value);
        self.sum += value;

        if self.values.len() > self.period {
            let old_value = self.values.pop_front().unwrap();
            self.sum -= old_value;
        }

        if self.values.len() == self.period {
            let sma = self.sum / self.period as f64;
            self.current_value = Some(sma);
            Some(sma)
        } else {
            None
        }
    }

    fn current(&self) -> Option<f64> {
        self.current_value
    }

    fn reset(&mut self) {
        self.values.clear();
        self.sum = 0.0;
        self.current_value = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_calculation() {
        let mut sma = SMA::new(3);

        let bars = vec![
            BarData { open: 100.0, high: 102.0, low: 99.0, close: 100.0, volume: 1000.0, timestamp: 1 },
            BarData { open: 101.0, high: 103.0, low: 100.0, close: 102.0, volume: 1100.0, timestamp: 2 },
            BarData { open: 102.0, high: 104.0, low: 101.0, close: 103.0, volume: 1200.0, timestamp: 3 },
            BarData { open: 103.0, high: 105.0, low: 102.0, close: 104.0, volume: 1300.0, timestamp: 4 },
        ];

        assert_eq!(sma.update(bars[0].clone()), None); // Not enough data
        assert_eq!(sma.update(bars[1].clone()), None); // Not enough data

        let result = sma.update(bars[2].clone());
        assert!(result.is_some());
        assert!((result.unwrap() - 101.667).abs() < 0.001); // (100 + 102 + 103) / 3

        let result = sma.update(bars[3].clone());
        assert!(result.is_some());
        assert!((result.unwrap() - 103.0).abs() < 0.001); // (102 + 103 + 104) / 3
    }

    #[test]
    fn test_sma_reset() {
        let mut sma = SMA::new(2);

        let bar = BarData { open: 100.0, high: 102.0, low: 99.0, close: 100.0, volume: 1000.0, timestamp: 1 };
        sma.update(bar.clone());
        sma.update(bar);

        assert!(sma.current().is_some());

        sma.reset();
        assert!(sma.current().is_none());
        assert_eq!(sma.values.len(), 0);
    }
}