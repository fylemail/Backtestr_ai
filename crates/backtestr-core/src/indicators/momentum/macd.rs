use crate::indicators::indicator_trait::{BarData, Indicator};

#[derive(Debug)]
pub struct MACD {
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    fast_ema: EMA,
    slow_ema: EMA,
    signal_ema: EMA,
    current_macd: Option<f64>,
    current_signal: Option<f64>,
    current_histogram: Option<f64>,
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

#[derive(Debug, Clone)]
pub struct MACDOutput {
    pub macd: f64,
    pub signal: f64,
    pub histogram: f64,
}

impl MACD {
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
        Self {
            fast_period,
            slow_period,
            signal_period,
            fast_ema: EMA::new(fast_period),
            slow_ema: EMA::new(slow_period),
            signal_ema: EMA::new(signal_period),
            current_macd: None,
            current_signal: None,
            current_histogram: None,
        }
    }

    pub fn get_output(&self) -> Option<MACDOutput> {
        if let (Some(macd), Some(signal), Some(histogram)) = (
            self.current_macd,
            self.current_signal,
            self.current_histogram,
        ) {
            Some(MACDOutput {
                macd,
                signal,
                histogram,
            })
        } else {
            None
        }
    }
}

impl Indicator for MACD {
    type Input = BarData;
    type Output = f64;

    fn name(&self) -> &str {
        "MACD"
    }

    fn warm_up_period(&self) -> usize {
        self.slow_period + self.signal_period - 1
    }

    fn update(&mut self, input: BarData) -> Option<f64> {
        let close = input.close;

        let fast_value = self.fast_ema.update(close);
        let slow_value = self.slow_ema.update(close);

        if let (Some(fast), Some(slow)) = (fast_value, slow_value) {
            let macd_line = fast - slow;
            self.current_macd = Some(macd_line);

            if let Some(signal) = self.signal_ema.update(macd_line) {
                self.current_signal = Some(signal);
                let histogram = macd_line - signal;
                self.current_histogram = Some(histogram);
                return Some(macd_line);
            }
        }

        None
    }

    fn current(&self) -> Option<f64> {
        self.current_macd
    }

    fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.signal_ema.reset();
        self.current_macd = None;
        self.current_signal = None;
        self.current_histogram = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macd_calculation() {
        let mut macd = MACD::new(12, 26, 9);

        for i in 1..=50 {
            let price = 100.0 + (i as f64 * 0.1).sin() * 5.0;
            let bar = BarData {
                open: price,
                high: price,
                low: price,
                close: price,
                volume: 1000.0,
                timestamp: i as i64,
            };

            macd.update(bar);
        }

        let output = macd.get_output();
        assert!(output.is_some());

        let result = output.unwrap();
        assert!(result.macd.is_finite());
        assert!(result.signal.is_finite());
        assert!(result.histogram.is_finite());
    }

    #[test]
    fn test_macd_trending() {
        let mut macd = MACD::new(3, 6, 3);

        for i in 1..=20 {
            let price = 100.0 + i as f64;
            let bar = BarData {
                open: price,
                high: price,
                low: price,
                close: price,
                volume: 1000.0,
                timestamp: i as i64,
            };

            macd.update(bar);
        }

        let output = macd.get_output();
        assert!(output.is_some());
        let result = output.unwrap();
        assert!(result.macd > 0.0); // Uptrend should have positive MACD
    }
}
