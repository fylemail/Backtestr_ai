use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Represents a calculated indicator value at a specific timestamp.
///
/// # Examples
///
/// ```
/// use backtestr_core::indicators::IndicatorValue;
///
/// let value = IndicatorValue {
///     value: 50.5,
///     timestamp: 1234567890,
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IndicatorValue {
    /// The calculated indicator value
    pub value: f64,
    /// Unix timestamp when this value was calculated
    pub timestamp: i64,
}

/// Core trait that all technical indicators must implement.
///
/// This trait provides a uniform interface for all indicators, supporting
/// incremental calculation, warm-up periods, and state management.
///
/// # Type Parameters
///
/// * `Input` - The type of data the indicator accepts (typically `BarData`)
/// * `Output` - The type of value the indicator produces (typically `f64`)
///
/// # Thread Safety
///
/// All implementations must be `Send + Sync` to support parallel processing.
///
/// # Examples
///
/// ```
/// use backtestr_core::indicators::{Indicator, BarData};
///
/// #[derive(Debug)]
/// struct MyIndicator {
///     period: usize,
///     values: Vec<f64>,
///     current_value: Option<f64>,
/// }
///
/// impl Indicator for MyIndicator {
///     type Input = BarData;
///     type Output = f64;
///
///     fn name(&self) -> &str { "MyIndicator" }
///     fn warm_up_period(&self) -> usize { self.period }
///     fn update(&mut self, input: BarData) -> Option<f64> {
///         self.values.push(input.close);
///         if self.values.len() >= self.period {
///             let sum: f64 = self.values.iter().take(self.period).sum();
///             self.current_value = Some(sum / self.period as f64);
///         }
///         self.current_value
///     }
///     fn current(&self) -> Option<f64> { self.current_value }
///     fn reset(&mut self) {
///         self.values.clear();
///         self.current_value = None;
///     }
/// }
/// ```
pub trait Indicator: Send + Sync + Debug {
    /// The input type this indicator accepts
    type Input;
    /// The output type this indicator produces
    type Output;

    /// Returns the name of this indicator.
    ///
    /// Used for logging, caching keys, and debugging.
    fn name(&self) -> &str;

    /// Returns the number of data points needed before the indicator can produce valid values.
    ///
    /// For example, a 20-period SMA needs 20 data points before calculating.
    fn warm_up_period(&self) -> usize;

    /// Updates the indicator with new data and returns the calculated value if ready.
    ///
    /// Returns `None` during the warm-up period, `Some(value)` once enough data is available.
    ///
    /// # Performance
    ///
    /// Implementations should use incremental calculation where possible to avoid
    /// recalculating the entire history on each update.
    fn update(&mut self, input: Self::Input) -> Option<Self::Output>;

    /// Returns the current indicator value without updating.
    ///
    /// Returns `None` if the indicator hasn't warmed up yet.
    fn current(&self) -> Option<Self::Output>;

    /// Resets the indicator to its initial state, clearing all internal data.
    fn reset(&mut self);

    /// Checks if the indicator has enough data to produce valid values.
    ///
    /// Default implementation checks if `current()` returns `Some`.
    fn is_ready(&self) -> bool {
        self.current().is_some()
    }
}

/// Default configuration parameters for all indicators.
///
/// This struct centralizes the default periods and parameters used across
/// all technical indicators, making it easy to maintain consistent defaults
/// and customize them when needed.
///
/// # Examples
///
/// ```
/// use backtestr_core::indicators::IndicatorDefaults;
///
/// let defaults = IndicatorDefaults::default();
/// assert_eq!(defaults.rsi_period, 14);
/// assert_eq!(defaults.sma_period, 20);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorDefaults {
    pub sma_period: usize,
    pub ema_period: usize,
    pub rsi_period: usize,
    pub macd_fast: usize,
    pub macd_slow: usize,
    pub macd_signal: usize,
    pub bollinger_period: usize,
    pub bollinger_std_dev: f64,
    pub atr_period: usize,
    pub stochastic_k_period: usize,
    pub stochastic_d_period: usize,
    pub cci_period: usize,
    pub williams_r_period: usize,
    pub adx_period: usize,
    pub wma_period: usize,
    pub dema_period: usize,
    pub vwap_period: usize,
    pub obv_sma_period: usize,
    pub keltner_period: usize,
    pub keltner_multiplier: f64,
    pub donchian_period: usize,
    pub sar_acceleration: f64,
    pub sar_maximum: f64,
}

impl Default for IndicatorDefaults {
    fn default() -> Self {
        Self {
            sma_period: 20,
            ema_period: 20,
            rsi_period: 14,
            macd_fast: 12,
            macd_slow: 26,
            macd_signal: 9,
            bollinger_period: 20,
            bollinger_std_dev: 2.0,
            atr_period: 14,
            stochastic_k_period: 14,
            stochastic_d_period: 3,
            cci_period: 20,
            williams_r_period: 14,
            adx_period: 14,
            wma_period: 20,
            dema_period: 20,
            vwap_period: 0, // 0 means session-based
            obv_sma_period: 20,
            keltner_period: 20,
            keltner_multiplier: 2.0,
            donchian_period: 20,
            sar_acceleration: 0.02,
            sar_maximum: 0.2,
        }
    }
}

/// Factory trait for creating indicator instances with default parameters.
pub trait IndicatorFactory: Send + Sync {
    /// Creates a new indicator instance with the given default parameters.
    fn create(
        &self,
        params: &IndicatorDefaults,
    ) -> Box<dyn Indicator<Input = BarData, Output = f64>>;

    /// Returns the name of the indicator this factory creates.
    fn name(&self) -> &str;
}

/// Standard OHLCV bar data used as input for most indicators.
///
/// This struct represents a single price bar with Open, High, Low, Close prices
/// and Volume, along with a timestamp. It's the primary input type for technical indicators.
///
/// # Examples
///
/// ```
/// use backtestr_core::indicators::BarData;
///
/// let bar = BarData {
///     open: 100.0,
///     high: 102.0,
///     low: 99.0,
///     close: 101.5,
///     volume: 10000.0,
///     timestamp: 1234567890,
/// };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct BarData {
    /// Opening price of the bar
    pub open: f64,
    /// Highest price during the bar period
    pub high: f64,
    /// Lowest price during the bar period
    pub low: f64,
    /// Closing price of the bar
    pub close: f64,
    /// Volume traded during the bar period
    pub volume: f64,
    /// Unix timestamp of the bar
    pub timestamp: i64,
}
