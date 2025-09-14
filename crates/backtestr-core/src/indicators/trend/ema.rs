use crate::indicators::indicator_trait::{BarData, Indicator};

#[derive(Debug)]
pub struct EMA {
    period: usize,
    multiplier: f64,
    current_value: Option<f64>,
    count: usize,
    sma_sum: f64,
}

impl EMA {
    pub fn new(period: usize) -> Self {
        let multiplier = 2.0 / (period as f64 + 1.0);
        Self {
            period,
            multiplier,
            current_value: None,
            count: 0,
            sma_sum: 0.0,
        }
    }
}

impl Indicator for EMA {
    type Input = BarData;
    type Output = f64;

    fn name(&self) -> &str {
        "EMA"
    }

    fn warm_up_period(&self) -> usize {
        self.period
    }

    fn update(&mut self, input: BarData) -> Option<f64> {
        let value = input.close;
        self.count += 1;

        if self.count < self.period {
            self.sma_sum += value;
            None
        } else if self.count == self.period {
            self.sma_sum += value;
            let initial_ema = self.sma_sum / self.period as f64;
            self.current_value = Some(initial_ema);
            Some(initial_ema)
        } else {
            let prev_ema = self.current_value.unwrap();
            let new_ema = (value - prev_ema) * self.multiplier + prev_ema;
            self.current_value = Some(new_ema);
            Some(new_ema)
        }
    }

    fn current(&self) -> Option<f64> {
        self.current_value
    }

    fn reset(&mut self) {
        self.current_value = None;
        self.count = 0;
        self.sma_sum = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ema_calculation() {
        let mut ema = EMA::new(3);

        let bars = vec![
            BarData { open: 100.0, high: 102.0, low: 99.0, close: 100.0, volume: 1000.0, timestamp: 1 },
            BarData { open: 101.0, high: 103.0, low: 100.0, close: 102.0, volume: 1100.0, timestamp: 2 },
            BarData { open: 102.0, high: 104.0, low: 101.0, close: 103.0, volume: 1200.0, timestamp: 3 },
            BarData { open: 103.0, high: 105.0, low: 102.0, close: 104.0, volume: 1300.0, timestamp: 4 },
        ];

        assert_eq!(ema.update(bars[0].clone()), None);
        assert_eq!(ema.update(bars[1].clone()), None);

        let result = ema.update(bars[2].clone());
        assert!(result.is_some());
        let initial_sma = (100.0 + 102.0 + 103.0) / 3.0;
        assert!((result.unwrap() - initial_sma).abs() < 0.001);

        let result = ema.update(bars[3].clone());
        assert!(result.is_some());
        let multiplier = 2.0 / 4.0; // 2 / (period + 1)
        let expected = (104.0 - initial_sma) * multiplier + initial_sma;
        assert!((result.unwrap() - expected).abs() < 0.001);
    }

    #[test]
    fn test_ema_smoothing() {
        let mut ema = EMA::new(5);

        for i in 1..=10 {
            let bar = BarData {
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0 + i as f64,
                volume: 1000.0,
                timestamp: i,
            };
            ema.update(bar);
        }

        let final_value = ema.current().unwrap();
        assert!(final_value > 105.0 && final_value < 110.0);
    }
}