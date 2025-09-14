use crate::indicators::indicator_trait::{BarData, Indicator};

#[derive(Debug)]
pub struct DEMA {
    period: usize,
    ema1: EMA,
    ema2: EMA,
    current_value: Option<f64>,
}

#[derive(Debug)]
struct EMA {
    period: usize,
    multiplier: f64,
    current_value: Option<f64>,
    count: usize,
    sma_sum: f64,
}

impl EMA {
    fn new(period: usize) -> Self {
        let multiplier = 2.0 / (period as f64 + 1.0);
        Self {
            period,
            multiplier,
            current_value: None,
            count: 0,
            sma_sum: 0.0,
        }
    }

    fn update(&mut self, value: f64) -> Option<f64> {
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

    fn reset(&mut self) {
        self.current_value = None;
        self.count = 0;
        self.sma_sum = 0.0;
    }
}

impl DEMA {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            ema1: EMA::new(period),
            ema2: EMA::new(period),
            current_value: None,
        }
    }
}

impl Indicator for DEMA {
    type Input = BarData;
    type Output = f64;

    fn name(&self) -> &str {
        "DEMA"
    }

    fn warm_up_period(&self) -> usize {
        self.period * 2 - 1
    }

    fn update(&mut self, input: BarData) -> Option<f64> {
        let value = input.close;

        if let Some(ema1_value) = self.ema1.update(value) {
            if let Some(ema2_value) = self.ema2.update(ema1_value) {
                let dema = 2.0 * ema1_value - ema2_value;
                self.current_value = Some(dema);
                Some(dema)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn current(&self) -> Option<f64> {
        self.current_value
    }

    fn reset(&mut self) {
        self.ema1.reset();
        self.ema2.reset();
        self.current_value = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dema_calculation() {
        let mut dema = DEMA::new(3);

        let mut has_value = false;
        for i in 1..=10 {
            let bar = BarData {
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0 + i as f64,
                volume: 1000.0,
                timestamp: i,
            };

            if let Some(value) = dema.update(bar) {
                has_value = true;
                assert!(value > 100.0);
            }
        }

        assert!(has_value);
        assert!(dema.current().is_some());
    }

    #[test]
    fn test_dema_responsiveness() {
        let mut dema = DEMA::new(5);
        let mut sma_sum = 0.0;
        let mut count = 0;

        for i in 1..=20 {
            let close = if i <= 10 { 100.0 } else { 110.0 };
            let bar = BarData {
                open: close,
                high: close,
                low: close,
                close,
                volume: 1000.0,
                timestamp: i,
            };

            dema.update(bar);

            if i > 10 {
                sma_sum += close;
                count += 1;
            }
        }

        let dema_value = dema.current().unwrap();
        assert!(dema_value > 105.0);
    }
}