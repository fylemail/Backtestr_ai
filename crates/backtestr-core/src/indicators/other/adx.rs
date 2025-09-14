//! Average Directional Index (ADX) implementation.
//!
//! ADX measures the strength of a trend, regardless of direction.
//! Values above 25 typically indicate a strong trend.

use crate::indicators::indicator_trait::{BarData, Indicator};
use std::collections::VecDeque;

/// Average Directional Index (ADX) indicator.
///
/// ADX is a trend strength indicator that ranges from 0 to 100.
/// It doesn't indicate trend direction, only strength:
/// - 0-25: Weak or no trend
/// - 25-50: Strong trend
/// - 50-75: Very strong trend
/// - 75-100: Extremely strong trend
#[derive(Debug)]
pub struct ADX {
    period: usize,
    plus_dm_ema: EMA,
    minus_dm_ema: EMA,
    atr_ema: EMA,
    dx_ema: EMA,
    previous_high: Option<f64>,
    previous_low: Option<f64>,
    dx_values: VecDeque<f64>,
    current_value: Option<f64>,
}

/// Simple EMA implementation for internal use
#[derive(Debug)]
struct EMA {
    period: usize,
    multiplier: f64,
    value: Option<f64>,
}

impl EMA {
    fn new(period: usize) -> Self {
        Self {
            period,
            multiplier: 1.0 / period as f64, // Using Wilder's smoothing
            value: None,
        }
    }

    fn update(&mut self, input: f64) -> Option<f64> {
        self.value = Some(match self.value {
            None => input,
            Some(prev) => prev + self.multiplier * (input - prev),
        });
        self.value
    }

    fn current(&self) -> Option<f64> {
        self.value
    }

    fn reset(&mut self) {
        self.value = None;
    }
}

impl ADX {
    /// Creates a new ADX indicator with the specified period.
    ///
    /// Standard period is 14.
    pub fn new(period: usize) -> Self {
        Self {
            period,
            plus_dm_ema: EMA::new(period),
            minus_dm_ema: EMA::new(period),
            atr_ema: EMA::new(period),
            dx_ema: EMA::new(period),
            previous_high: None,
            previous_low: None,
            dx_values: VecDeque::with_capacity(period),
            current_value: None,
        }
    }

    fn calculate_directional_movement(&self, bar: &BarData) -> (f64, f64, f64) {
        if let (Some(prev_high), Some(prev_low)) = (self.previous_high, self.previous_low) {
            // Calculate directional movements
            let up_move = bar.high - prev_high;
            let down_move = prev_low - bar.low;

            let plus_dm = if up_move > down_move && up_move > 0.0 {
                up_move
            } else {
                0.0
            };

            let minus_dm = if down_move > up_move && down_move > 0.0 {
                down_move
            } else {
                0.0
            };

            // Calculate true range
            let high_low = bar.high - bar.low;
            let high_close = (bar.high - prev_low).abs();
            let low_close = (bar.low - prev_high).abs();
            let true_range = high_low.max(high_close).max(low_close);

            (plus_dm, minus_dm, true_range)
        } else {
            (0.0, 0.0, bar.high - bar.low)
        }
    }
}

impl Indicator for ADX {
    type Input = BarData;
    type Output = f64;

    fn name(&self) -> &str {
        "ADX"
    }

    fn warm_up_period(&self) -> usize {
        self.period * 2 + 1
    }

    fn update(&mut self, input: BarData) -> Option<f64> {
        let (plus_dm, minus_dm, true_range) = self.calculate_directional_movement(&input);

        // Update EMAs
        self.plus_dm_ema.update(plus_dm);
        self.minus_dm_ema.update(minus_dm);
        self.atr_ema.update(true_range);

        // Store current high/low for next calculation
        self.previous_high = Some(input.high);
        self.previous_low = Some(input.low);

        // Calculate DI+ and DI-
        if let (Some(smooth_plus_dm), Some(smooth_minus_dm), Some(smooth_tr)) = (
            self.plus_dm_ema.current(),
            self.minus_dm_ema.current(),
            self.atr_ema.current(),
        ) {
            if smooth_tr > 0.0 {
                let plus_di = (smooth_plus_dm / smooth_tr) * 100.0;
                let minus_di = (smooth_minus_dm / smooth_tr) * 100.0;

                // Calculate DX
                let di_sum = plus_di + minus_di;
                let dx = if di_sum > 0.0 {
                    ((plus_di - minus_di).abs() / di_sum) * 100.0
                } else {
                    0.0
                };

                // Calculate ADX as smoothed DX
                if let Some(adx) = self.dx_ema.update(dx) {
                    self.current_value = Some(adx);
                    return Some(adx);
                }
            }
        }

        None
    }

    fn current(&self) -> Option<f64> {
        self.current_value
    }

    fn reset(&mut self) {
        self.plus_dm_ema.reset();
        self.minus_dm_ema.reset();
        self.atr_ema.reset();
        self.dx_ema.reset();
        self.previous_high = None;
        self.previous_low = None;
        self.dx_values.clear();
        self.current_value = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adx_calculation() {
        let mut adx = ADX::new(14);

        // Generate trending market data
        let mut bars = Vec::new();
        for i in 0..50 {
            let base = 100.0 + i as f64 * 0.5; // Uptrend
            bars.push(BarData {
                open: base,
                high: base + 1.0,
                low: base - 0.5,
                close: base + 0.5,
                volume: 1000.0,
                timestamp: i,
            });
        }

        let mut last_value = None;
        for bar in bars {
            if let Some(value) = adx.update(bar) {
                assert!(value >= 0.0 && value <= 100.0);
                last_value = Some(value);
            }
        }

        // In a strong trend, ADX should be above 25
        assert!(last_value.is_some());
        assert!(last_value.unwrap() > 20.0); // Should indicate trend
    }

    #[test]
    fn test_adx_ranging_market() {
        let mut adx = ADX::new(14);

        // Generate ranging/sideways market data
        let mut bars = Vec::new();
        for i in 0..50 {
            let base = 100.0 + (i as f64 * 0.5).sin() * 2.0; // Oscillating
            bars.push(BarData {
                open: base,
                high: base + 0.5,
                low: base - 0.5,
                close: base,
                volume: 1000.0,
                timestamp: i,
            });
        }

        let mut last_value = None;
        for bar in bars {
            if let Some(value) = adx.update(bar) {
                last_value = Some(value);
            }
        }

        // In a ranging market, ADX should be low
        assert!(last_value.is_some());
        assert!(last_value.unwrap() < 30.0); // Should indicate weak/no trend
    }

    #[test]
    fn test_adx_reset() {
        let mut adx = ADX::new(14);

        // Add some data
        for i in 0..20 {
            let bar = BarData {
                open: 100.0 + i as f64,
                high: 101.0 + i as f64,
                low: 99.0 + i as f64,
                close: 100.5 + i as f64,
                volume: 1000.0,
                timestamp: i,
            };
            adx.update(bar);
        }

        assert!(adx.current().is_some());

        adx.reset();
        assert!(adx.current().is_none());
        assert!(adx.previous_high.is_none());
        assert!(adx.previous_low.is_none());
    }

    #[test]
    fn test_adx_warm_up_period() {
        let adx = ADX::new(14);
        assert_eq!(adx.warm_up_period(), 29); // 14 * 2 + 1
    }
}
