//! Technical indicators for backtesting and trading analysis.
//!
//! This module provides a comprehensive suite of technical indicators commonly used
//! in financial markets analysis. All indicators support incremental calculation
//! for efficient real-time processing.
//!
//! # Architecture
//!
//! - **Trait-based design**: All indicators implement the `Indicator` trait
//! - **Thread-safe**: All indicators are `Send + Sync` for concurrent processing
//! - **Incremental calculation**: Optimized for streaming data without full recalculation
//! - **Caching layer**: Automatic history management with configurable depth
//! - **Pipeline processing**: Parallel execution for multiple indicators
//!
//! # Categories
//!
//! ## Trend Indicators
//! - **SMA** - Simple Moving Average
//! - **EMA** - Exponential Moving Average
//! - **WMA** - Weighted Moving Average
//! - **DEMA** - Double Exponential Moving Average
//!
//! ## Momentum Indicators
//! - **RSI** - Relative Strength Index
//! - **MACD** - Moving Average Convergence Divergence
//! - **Stochastic** - Stochastic Oscillator
//! - **CCI** - Commodity Channel Index
//! - **Williams %R** - Williams Percent Range
//!
//! ## Volatility Indicators
//! - **Bollinger Bands** - Price bands based on standard deviation
//! - **ATR** - Average True Range
//! - **Keltner Channels** - ATR-based price channels
//! - **Donchian Channels** - High/Low price channels
//!
//! ## Volume Indicators
//! - **OBV** - On-Balance Volume
//! - **Volume SMA** - Simple Moving Average of Volume
//! - **VWAP** - Volume Weighted Average Price
//!
//! ## Other Indicators
//! - **ADX** - Average Directional Index
//! - **Parabolic SAR** - Stop and Reverse indicator
//! - **Pivot Points** - Support/Resistance levels
//!
//! # Examples
//!
//! ```
//! use backtestr_core::indicators::{IndicatorPipeline, RSI, SMA, BarData};
//! use backtestr_core::Timeframe;
//!
//! // Create pipeline
//! let mut pipeline = IndicatorPipeline::new(100);
//!
//! // Register indicators
//! pipeline.register_indicator("RSI_14".to_string(), Box::new(RSI::new(14)));
//! pipeline.register_indicator("SMA_20".to_string(), Box::new(SMA::new(20)));
//!
//! // Process bar data
//! let bar = BarData {
//!     open: 100.0,
//!     high: 102.0,
//!     low: 99.0,
//!     close: 101.0,
//!     volume: 10000.0,
//!     timestamp: 1234567890,
//! };
//!
//! pipeline.update_all(&bar, Timeframe::M1).unwrap();
//!
//! // Get values
//! if let Some(rsi) = pipeline.get_value("RSI_14", Timeframe::M1) {
//!     println!("RSI: {}", rsi);
//! }
//! ```

pub mod indicator_trait;
pub mod cache;
pub mod pipeline;
pub mod trend;
pub mod momentum;
pub mod volatility;
pub mod volume;
pub mod other;

pub use indicator_trait::{Indicator, IndicatorDefaults, IndicatorValue, BarData};
pub use cache::IndicatorCache;
pub use pipeline::IndicatorPipeline;

// Re-export all indicators
pub use trend::{SMA, EMA, WMA, DEMA};
pub use momentum::{RSI, MACD, Stochastic, CCI, WilliamsR};
pub use volatility::{BollingerBands, ATR, KeltnerChannels, DonchianChannels};
pub use volume::{OBV, VolumeSMA, VWAP};
pub use other::{PivotPoints, SupportResistance, ADX, ParabolicSAR};
