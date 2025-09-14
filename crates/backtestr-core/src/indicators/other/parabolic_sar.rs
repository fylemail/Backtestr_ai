//! Parabolic SAR (Stop and Reverse) implementation.
//!
//! The Parabolic SAR is a trend-following indicator that provides entry/exit points.
//! It appears as dots above or below the price, indicating potential reversal points.

use crate::indicators::indicator_trait::{BarData, Indicator};

/// Parabolic SAR indicator for identifying trend reversals.
///
/// The SAR follows price movements and accelerates when the trend extends.
/// When price crosses the SAR, it signals a potential trend reversal.
///
/// # Parameters
///
/// - `acceleration`: Initial acceleration factor (typically 0.02)
/// - `max_acceleration`: Maximum acceleration factor (typically 0.2)
#[derive(Debug)]
pub struct ParabolicSAR {
    acceleration_factor: f64,
    acceleration_step: f64,
    max_acceleration: f64,
    current_sar: Option<f64>,
    extreme_point: Option<f64>,
    is_long: bool,
    previous_bar: Option<BarData>,
}

impl ParabolicSAR {
    /// Creates a new Parabolic SAR indicator.
    ///
    /// # Arguments
    ///
    /// * `acceleration_step` - The step by which AF increases (typically 0.02)
    /// * `max_acceleration` - Maximum acceleration factor (typically 0.2)
    pub fn new(acceleration_step: f64, max_acceleration: f64) -> Self {
        Self {
            acceleration_factor: acceleration_step,
            acceleration_step,
            max_acceleration,
            current_sar: None,
            extreme_point: None,
            is_long: true,
            previous_bar: None,
        }
    }

    fn initialize(&mut self, bar: &BarData) {
        // Initialize SAR below price for long position
        self.current_sar = Some(bar.low);
        self.extreme_point = Some(bar.high);
        self.is_long = true;
        self.acceleration_factor = self.acceleration_step;
    }

    fn update_sar(&mut self, bar: &BarData) -> f64 {
        let current_sar = self.current_sar.unwrap();
        let extreme_point = self.extreme_point.unwrap();

        // Calculate next SAR
        let mut next_sar = current_sar + self.acceleration_factor * (extreme_point - current_sar);

        if self.is_long {
            // For long positions, SAR cannot be above the lowest low of the last two periods
            if let Some(prev) = &self.previous_bar {
                next_sar = next_sar.min(prev.low).min(bar.low);
            }

            // Check for reversal
            if bar.low <= next_sar {
                // Switch to short
                self.is_long = false;
                self.extreme_point = Some(bar.low);
                self.acceleration_factor = self.acceleration_step;
                next_sar = extreme_point; // Previous extreme becomes new SAR
            } else {
                // Update extreme point if new high
                if bar.high > extreme_point {
                    self.extreme_point = Some(bar.high);
                    // Increase acceleration factor
                    self.acceleration_factor = (self.acceleration_factor + self.acceleration_step)
                        .min(self.max_acceleration);
                }
            }
        } else {
            // For short positions, SAR cannot be below the highest high of the last two periods
            if let Some(prev) = &self.previous_bar {
                next_sar = next_sar.max(prev.high).max(bar.high);
            }

            // Check for reversal
            if bar.high >= next_sar {
                // Switch to long
                self.is_long = true;
                self.extreme_point = Some(bar.high);
                self.acceleration_factor = self.acceleration_step;
                next_sar = extreme_point; // Previous extreme becomes new SAR
            } else {
                // Update extreme point if new low
                if bar.low < extreme_point {
                    self.extreme_point = Some(bar.low);
                    // Increase acceleration factor
                    self.acceleration_factor = (self.acceleration_factor + self.acceleration_step)
                        .min(self.max_acceleration);
                }
            }
        }

        self.current_sar = Some(next_sar);
        next_sar
    }
}

impl Indicator for ParabolicSAR {
    type Input = BarData;
    type Output = f64;

    fn name(&self) -> &str {
        "ParabolicSAR"
    }

    fn warm_up_period(&self) -> usize {
        2 // Needs at least 2 bars to establish trend
    }

    fn update(&mut self, input: BarData) -> Option<f64> {
        if self.current_sar.is_none() {
            if self.previous_bar.is_some() {
                self.initialize(&input);
            }
            self.previous_bar = Some(input);
            self.current_sar
        } else {
            let sar = self.update_sar(&input);
            self.previous_bar = Some(input);
            Some(sar)
        }
    }

    fn current(&self) -> Option<f64> {
        self.current_sar
    }

    fn reset(&mut self) {
        self.acceleration_factor = self.acceleration_step;
        self.current_sar = None;
        self.extreme_point = None;
        self.is_long = true;
        self.previous_bar = None;
    }
}

/// Returns the position side indicated by the SAR
impl ParabolicSAR {
    /// Returns true if the SAR indicates a long position
    pub fn is_long_signal(&self) -> bool {
        self.is_long
    }

    /// Returns the current extreme point
    pub fn extreme_point(&self) -> Option<f64> {
        self.extreme_point
    }

    /// Returns the current acceleration factor
    pub fn acceleration_factor(&self) -> f64 {
        self.acceleration_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parabolic_sar_uptrend() {
        let mut sar = ParabolicSAR::new(0.02, 0.2);

        // Create uptrend data
        let bars = vec![
            BarData { open: 100.0, high: 102.0, low: 99.0, close: 101.0, volume: 1000.0, timestamp: 1 },
            BarData { open: 101.0, high: 103.0, low: 100.5, close: 102.5, volume: 1000.0, timestamp: 2 },
            BarData { open: 102.5, high: 104.0, low: 101.0, close: 103.5, volume: 1000.0, timestamp: 3 },
            BarData { open: 103.5, high: 105.0, low: 102.0, close: 104.5, volume: 1000.0, timestamp: 4 },
            BarData { open: 104.5, high: 106.0, low: 103.0, close: 105.5, volume: 1000.0, timestamp: 5 },
        ];

        let mut values = Vec::new();
        for bar in bars {
            let high = bar.high; // Store before move
            if let Some(value) = sar.update(bar) {
                values.push(value);
                // In uptrend, SAR should be below price
                assert!(value < high);
            }
        }

        // Should have values after warm-up
        assert!(!values.is_empty());
        // Should remain in long position during uptrend
        assert!(sar.is_long_signal());
    }

    #[test]
    fn test_parabolic_sar_reversal() {
        let mut sar = ParabolicSAR::new(0.02, 0.2);

        // Create data with reversal
        let bars = vec![
            BarData { open: 100.0, high: 102.0, low: 99.0, close: 101.0, volume: 1000.0, timestamp: 1 },
            BarData { open: 101.0, high: 103.0, low: 100.5, close: 102.5, volume: 1000.0, timestamp: 2 },
            BarData { open: 102.5, high: 104.0, low: 101.0, close: 103.5, volume: 1000.0, timestamp: 3 },
            // Reversal bars
            BarData { open: 103.5, high: 104.0, low: 100.0, close: 100.5, volume: 1000.0, timestamp: 4 },
            BarData { open: 100.5, high: 101.0, low: 98.0, close: 98.5, volume: 1000.0, timestamp: 5 },
        ];

        let mut position_changed = false;
        let mut initial_position = None;

        for bar in bars {
            sar.update(bar);

            if initial_position.is_none() && sar.current_sar.is_some() {
                initial_position = Some(sar.is_long_signal());
            } else if let Some(initial) = initial_position {
                if initial != sar.is_long_signal() {
                    position_changed = true;
                }
            }
        }

        // Should detect the reversal
        assert!(position_changed);
    }

    #[test]
    fn test_parabolic_sar_acceleration() {
        let mut sar = ParabolicSAR::new(0.02, 0.2);

        // Create strong uptrend to test acceleration
        let bars = vec![
            BarData { open: 100.0, high: 101.0, low: 99.0, close: 100.5, volume: 1000.0, timestamp: 1 },
            BarData { open: 100.5, high: 103.0, low: 100.0, close: 102.5, volume: 1000.0, timestamp: 2 },
            BarData { open: 102.5, high: 105.0, low: 102.0, close: 104.5, volume: 1000.0, timestamp: 3 },
            BarData { open: 104.5, high: 107.0, low: 104.0, close: 106.5, volume: 1000.0, timestamp: 4 },
            BarData { open: 106.5, high: 109.0, low: 106.0, close: 108.5, volume: 1000.0, timestamp: 5 },
        ];

        let initial_af = sar.acceleration_factor;

        for bar in bars {
            sar.update(bar);
        }

        // Acceleration factor should increase in strong trend
        assert!(sar.acceleration_factor() > initial_af);
        assert!(sar.acceleration_factor() <= 0.2); // Should not exceed max
    }

    #[test]
    fn test_parabolic_sar_reset() {
        let mut sar = ParabolicSAR::new(0.02, 0.2);

        let bar = BarData {
            open: 100.0,
            high: 102.0,
            low: 99.0,
            close: 101.0,
            volume: 1000.0,
            timestamp: 1,
        };

        sar.update(bar.clone());
        sar.update(bar);

        assert!(sar.current().is_some());

        sar.reset();
        assert!(sar.current().is_none());
        assert!(sar.extreme_point().is_none());
        assert_eq!(sar.acceleration_factor(), 0.02);
    }
}