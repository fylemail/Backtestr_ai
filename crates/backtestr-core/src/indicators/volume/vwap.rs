use crate::indicators::indicator_trait::{BarData, Indicator};

#[derive(Debug)]
pub struct VWAP {
    cumulative_volume: f64,
    cumulative_pv: f64,
    current_value: Option<f64>,
    session_start: Option<i64>,
    reset_on_session: bool,
}

impl VWAP {
    pub fn new(reset_on_session: bool) -> Self {
        Self {
            cumulative_volume: 0.0,
            cumulative_pv: 0.0,
            current_value: None,
            session_start: None,
            reset_on_session,
        }
    }

    fn is_new_session(&self, timestamp: i64) -> bool {
        if !self.reset_on_session {
            return false;
        }

        if let Some(start) = self.session_start {
            // Simple day boundary check (assumes timestamps are in seconds)
            let day_seconds = 86400;
            timestamp / day_seconds != start / day_seconds
        } else {
            true
        }
    }
}

impl Default for VWAP {
    fn default() -> Self {
        Self::new(true)
    }
}

impl Indicator for VWAP {
    type Input = BarData;
    type Output = f64;

    fn name(&self) -> &str {
        "VWAP"
    }

    fn warm_up_period(&self) -> usize {
        1
    }

    fn update(&mut self, input: BarData) -> Option<f64> {
        if self.is_new_session(input.timestamp) {
            self.cumulative_volume = 0.0;
            self.cumulative_pv = 0.0;
            self.session_start = Some(input.timestamp);
        }

        let typical_price = (input.high + input.low + input.close) / 3.0;
        self.cumulative_pv += typical_price * input.volume;
        self.cumulative_volume += input.volume;

        if self.cumulative_volume > 0.0 {
            let vwap = self.cumulative_pv / self.cumulative_volume;
            self.current_value = Some(vwap);
            Some(vwap)
        } else {
            None
        }
    }

    fn current(&self) -> Option<f64> {
        self.current_value
    }

    fn reset(&mut self) {
        self.cumulative_volume = 0.0;
        self.cumulative_pv = 0.0;
        self.current_value = None;
        self.session_start = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vwap_calculation() {
        let mut vwap = VWAP::new(false);

        let bars = vec![
            BarData { open: 100.0, high: 102.0, low: 99.0, close: 101.0, volume: 1000.0, timestamp: 1 },
            BarData { open: 101.0, high: 103.0, low: 100.0, close: 102.0, volume: 1500.0, timestamp: 2 },
            BarData { open: 102.0, high: 104.0, low: 101.0, close: 103.0, volume: 2000.0, timestamp: 3 },
        ];

        let result1 = vwap.update(bars[0].clone());
        assert!(result1.is_some());
        let tp1 = (102.0 + 99.0 + 101.0) / 3.0;
        assert!((result1.unwrap() - tp1).abs() < 0.001);

        let result2 = vwap.update(bars[1].clone());
        assert!(result2.is_some());

        let result3 = vwap.update(bars[2].clone());
        assert!(result3.is_some());

        // VWAP should be weighted average of typical prices
        let vwap_value = result3.unwrap();
        assert!(vwap_value > 100.0 && vwap_value < 103.0);
    }
}