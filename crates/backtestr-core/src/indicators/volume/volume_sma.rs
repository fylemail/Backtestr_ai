use crate::indicators::indicator_trait::{BarData, Indicator};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct VolumeSMA {
    period: usize,
    volumes: VecDeque<f64>,
    sum: f64,
    current_value: Option<f64>,
}

impl VolumeSMA {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            volumes: VecDeque::with_capacity(period),
            sum: 0.0,
            current_value: None,
        }
    }
}

impl Indicator for VolumeSMA {
    type Input = BarData;
    type Output = f64;

    fn name(&self) -> &str {
        "VolumeSMA"
    }

    fn warm_up_period(&self) -> usize {
        self.period
    }

    fn update(&mut self, input: BarData) -> Option<f64> {
        let volume = input.volume;

        self.volumes.push_back(volume);
        self.sum += volume;

        if self.volumes.len() > self.period {
            let old_volume = self.volumes.pop_front().unwrap();
            self.sum -= old_volume;
        }

        if self.volumes.len() == self.period {
            let avg_volume = self.sum / self.period as f64;
            self.current_value = Some(avg_volume);
            Some(avg_volume)
        } else {
            None
        }
    }

    fn current(&self) -> Option<f64> {
        self.current_value
    }

    fn reset(&mut self) {
        self.volumes.clear();
        self.sum = 0.0;
        self.current_value = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_sma() {
        let mut vol_sma = VolumeSMA::new(3);

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
                volume: 1500.0,
                timestamp: 2,
            },
            BarData {
                open: 102.0,
                high: 104.0,
                low: 101.0,
                close: 103.0,
                volume: 2000.0,
                timestamp: 3,
            },
            BarData {
                open: 103.0,
                high: 105.0,
                low: 102.0,
                close: 104.0,
                volume: 1200.0,
                timestamp: 4,
            },
        ];

        assert_eq!(vol_sma.update(bars[0].clone()), None);
        assert_eq!(vol_sma.update(bars[1].clone()), None);

        let result = vol_sma.update(bars[2].clone());
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 1500.0); // (1000 + 1500 + 2000) / 3

        let result = vol_sma.update(bars[3].clone());
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 1566.6666666666667); // (1500 + 2000 + 1200) / 3
    }
}
