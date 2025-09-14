use crate::indicators::indicator_trait::{BarData, Indicator};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct ATR {
    period: usize,
    tr_values: VecDeque<f64>,
    current_atr: Option<f64>,
    previous_close: Option<f64>,
    count: usize,
}

impl ATR {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            tr_values: VecDeque::with_capacity(period),
            current_atr: None,
            previous_close: None,
            count: 0,
        }
    }

    fn calculate_true_range(&self, bar: &BarData) -> f64 {
        if let Some(prev_close) = self.previous_close {
            let hl = bar.high - bar.low;
            let hc = (bar.high - prev_close).abs();
            let lc = (bar.low - prev_close).abs();
            hl.max(hc).max(lc)
        } else {
            bar.high - bar.low
        }
    }
}

impl Indicator for ATR {
    type Input = BarData;
    type Output = f64;

    fn name(&self) -> &str {
        "ATR"
    }

    fn warm_up_period(&self) -> usize {
        self.period + 1
    }

    fn update(&mut self, input: BarData) -> Option<f64> {
        let tr = self.calculate_true_range(&input);
        self.count += 1;

        if self.count <= self.period {
            self.tr_values.push_back(tr);
            if self.count == self.period {
                let initial_atr = self.tr_values.iter().sum::<f64>() / self.period as f64;
                self.current_atr = Some(initial_atr);
                self.previous_close = Some(input.close);
                return Some(initial_atr);
            }
        } else if let Some(prev_atr) = self.current_atr {
            let new_atr = ((prev_atr * (self.period - 1) as f64) + tr) / self.period as f64;
            self.current_atr = Some(new_atr);
            self.previous_close = Some(input.close);
            return Some(new_atr);
        }

        self.previous_close = Some(input.close);
        None
    }

    fn current(&self) -> Option<f64> {
        self.current_atr
    }

    fn reset(&mut self) {
        self.tr_values.clear();
        self.current_atr = None;
        self.previous_close = None;
        self.count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atr_calculation() {
        let mut atr = ATR::new(5);

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
        ];

        for (i, bar) in bars.iter().enumerate() {
            let result = atr.update(bar.clone());
            if i < 4 {
                assert!(result.is_none());
            } else {
                assert!(result.is_some());
                let value = result.unwrap();
                assert!(value > 0.0);
            }
        }
    }

    #[test]
    fn test_atr_with_gaps() {
        let mut atr = ATR::new(3);

        let bars = vec![
            BarData {
                open: 100.0,
                high: 101.0,
                low: 99.0,
                close: 100.0,
                volume: 1000.0,
                timestamp: 1,
            },
            BarData {
                open: 105.0,
                high: 106.0,
                low: 104.0,
                close: 105.0,
                volume: 1000.0,
                timestamp: 2,
            },
            BarData {
                open: 103.0,
                high: 104.0,
                low: 102.0,
                close: 103.0,
                volume: 1000.0,
                timestamp: 3,
            },
            BarData {
                open: 103.0,
                high: 104.0,
                low: 102.0,
                close: 103.0,
                volume: 1000.0,
                timestamp: 4,
            },
        ];

        for bar in bars {
            atr.update(bar);
        }

        let value = atr.current().unwrap();
        assert!(value > 2.0); // Gap should increase ATR
    }
}
