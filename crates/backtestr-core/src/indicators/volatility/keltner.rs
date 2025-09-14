use crate::indicators::indicator_trait::{BarData, Indicator};

#[derive(Debug)]
pub struct KeltnerChannels {
    period: usize,
    multiplier: f64,
    ema: EMA,
    atr: ATR,
    current_middle: Option<f64>,
    current_upper: Option<f64>,
    current_lower: Option<f64>,
}

#[derive(Debug)]
struct EMA {
    period: usize,
    multiplier: f64,
    current_value: Option<f64>,
    count: usize,
    sma_sum: f64,
}

#[derive(Debug)]
struct ATR {
    period: usize,
    current_atr: Option<f64>,
    previous_close: Option<f64>,
    count: usize,
    tr_sum: f64,
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

impl ATR {
    fn new(period: usize) -> Self {
        Self {
            period,
            current_atr: None,
            previous_close: None,
            count: 0,
            tr_sum: 0.0,
        }
    }

    fn update(&mut self, bar: &BarData) -> Option<f64> {
        let tr = if let Some(prev_close) = self.previous_close {
            let hl = bar.high - bar.low;
            let hc = (bar.high - prev_close).abs();
            let lc = (bar.low - prev_close).abs();
            hl.max(hc).max(lc)
        } else {
            bar.high - bar.low
        };

        self.count += 1;

        if self.count <= self.period {
            self.tr_sum += tr;
            if self.count == self.period {
                let initial_atr = self.tr_sum / self.period as f64;
                self.current_atr = Some(initial_atr);
                self.previous_close = Some(bar.close);
                return Some(initial_atr);
            }
        } else if let Some(prev_atr) = self.current_atr {
            let new_atr = ((prev_atr * (self.period - 1) as f64) + tr) / self.period as f64;
            self.current_atr = Some(new_atr);
            self.previous_close = Some(bar.close);
            return Some(new_atr);
        }

        self.previous_close = Some(bar.close);
        None
    }

    fn reset(&mut self) {
        self.current_atr = None;
        self.previous_close = None;
        self.count = 0;
        self.tr_sum = 0.0;
    }
}

#[derive(Debug, Clone)]
pub struct KeltnerOutput {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

impl KeltnerChannels {
    pub fn new(period: usize, multiplier: f64) -> Self {
        Self {
            period,
            multiplier,
            ema: EMA::new(period),
            atr: ATR::new(period),
            current_middle: None,
            current_upper: None,
            current_lower: None,
        }
    }

    pub fn get_channels(&self) -> Option<KeltnerOutput> {
        if let (Some(upper), Some(middle), Some(lower)) =
            (self.current_upper, self.current_middle, self.current_lower) {
            Some(KeltnerOutput { upper, middle, lower })
        } else {
            None
        }
    }
}

impl Indicator for KeltnerChannels {
    type Input = BarData;
    type Output = f64;

    fn name(&self) -> &str {
        "KeltnerChannels"
    }

    fn warm_up_period(&self) -> usize {
        self.period
    }

    fn update(&mut self, input: BarData) -> Option<f64> {
        let typical_price = (input.high + input.low + input.close) / 3.0;
        let ema_value = self.ema.update(typical_price);
        let atr_value = self.atr.update(&input);

        if let (Some(ema), Some(atr)) = (ema_value, atr_value) {
            let upper = ema + (self.multiplier * atr);
            let lower = ema - (self.multiplier * atr);

            self.current_middle = Some(ema);
            self.current_upper = Some(upper);
            self.current_lower = Some(lower);

            Some(ema)
        } else {
            None
        }
    }

    fn current(&self) -> Option<f64> {
        self.current_middle
    }

    fn reset(&mut self) {
        self.ema.reset();
        self.atr.reset();
        self.current_middle = None;
        self.current_upper = None;
        self.current_lower = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keltner_channels() {
        let mut kc = KeltnerChannels::new(5, 2.0);

        let bars = vec![
            BarData { open: 100.0, high: 102.0, low: 99.0, close: 101.0, volume: 1000.0, timestamp: 1 },
            BarData { open: 101.0, high: 103.0, low: 100.0, close: 102.0, volume: 1100.0, timestamp: 2 },
            BarData { open: 102.0, high: 104.0, low: 101.0, close: 103.0, volume: 1200.0, timestamp: 3 },
            BarData { open: 103.0, high: 105.0, low: 102.0, close: 104.0, volume: 1300.0, timestamp: 4 },
            BarData { open: 104.0, high: 106.0, low: 103.0, close: 105.0, volume: 1400.0, timestamp: 5 },
            BarData { open: 105.0, high: 107.0, low: 104.0, close: 106.0, volume: 1500.0, timestamp: 6 },
        ];

        for bar in bars {
            kc.update(bar);
        }

        let channels = kc.get_channels();
        assert!(channels.is_some());

        let result = channels.unwrap();
        assert!(result.upper > result.middle);
        assert!(result.middle > result.lower);
    }
}