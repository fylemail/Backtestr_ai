use crate::indicators::indicator_trait::{BarData, Indicator};

#[derive(Debug)]
pub struct OBV {
    current_obv: f64,
    previous_close: Option<f64>,
}

impl OBV {
    pub fn new() -> Self {
        Self {
            current_obv: 0.0,
            previous_close: None,
        }
    }
}

impl Default for OBV {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for OBV {
    type Input = BarData;
    type Output = f64;

    fn name(&self) -> &str {
        "OBV"
    }

    fn warm_up_period(&self) -> usize {
        1
    }

    fn update(&mut self, input: BarData) -> Option<f64> {
        if let Some(prev_close) = self.previous_close {
            if input.close > prev_close {
                self.current_obv += input.volume;
            } else if input.close < prev_close {
                self.current_obv -= input.volume;
            }
            // If close == prev_close, OBV stays the same
        } else {
            self.current_obv = input.volume;
        }

        self.previous_close = Some(input.close);
        Some(self.current_obv)
    }

    fn current(&self) -> Option<f64> {
        if self.previous_close.is_some() {
            Some(self.current_obv)
        } else {
            None
        }
    }

    fn reset(&mut self) {
        self.current_obv = 0.0;
        self.previous_close = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obv_calculation() {
        let mut obv = OBV::new();

        let bars = vec![
            BarData { open: 100.0, high: 102.0, low: 99.0, close: 101.0, volume: 1000.0, timestamp: 1 },
            BarData { open: 101.0, high: 103.0, low: 100.0, close: 102.0, volume: 1100.0, timestamp: 2 },
            BarData { open: 102.0, high: 104.0, low: 101.0, close: 101.0, volume: 1200.0, timestamp: 3 },
            BarData { open: 101.0, high: 103.0, low: 100.0, close: 103.0, volume: 1300.0, timestamp: 4 },
        ];

        let result1 = obv.update(bars[0].clone());
        assert_eq!(result1, Some(1000.0));

        let result2 = obv.update(bars[1].clone());
        assert_eq!(result2, Some(2100.0)); // Price up, add volume

        let result3 = obv.update(bars[2].clone());
        assert_eq!(result3, Some(900.0)); // Price down, subtract volume

        let result4 = obv.update(bars[3].clone());
        assert_eq!(result4, Some(2200.0)); // Price up, add volume
    }
}