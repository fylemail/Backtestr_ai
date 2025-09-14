//! Epic 3 Interface Contracts
//!
//! This module defines the interface contracts between Epic 2 (MTF Engine)
//! and Epic 3 (Position Management & Execution) components.
//! These interfaces ensure clean separation of concerns and enable
//! Epic 3 implementation without modifying Epic 2 code.

use crate::events::EventHandler;
use crate::indicators::IndicatorValue;
use backtestr_data::{Bar, Tick, Timeframe};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Extension of existing EventHandler trait for position management
///
/// This trait will be implemented by the position management system
/// to receive events from the MTF engine and indicator pipeline.
pub trait PositionEventHandler: EventHandler {
    /// Called when a bar completes in any timeframe
    ///
    /// # Arguments
    /// * `bar` - The completed bar data
    /// * `timeframe` - The timeframe of the completed bar
    /// * `symbol` - The symbol for which the bar completed
    fn on_bar_complete(&mut self, bar: &Bar, timeframe: Timeframe, symbol: &str);

    /// Called on each tick for real-time position P&L updates
    ///
    /// # Arguments
    /// * `tick` - The incoming tick data
    /// * `symbol` - The symbol for the tick
    fn on_tick_update(&mut self, tick: &Tick, symbol: &str);

    /// Called when an indicator value updates
    ///
    /// # Arguments
    /// * `indicator` - The updated indicator value
    /// * `timeframe` - The timeframe of the indicator
    /// * `symbol` - The symbol for which the indicator was calculated
    fn on_indicator_update(&mut self, indicator: &IndicatorValue, timeframe: Timeframe, symbol: &str);
}

/// Provides market context for order execution
///
/// This trait will be implemented by the MTF engine to provide
/// current market data needed for order execution decisions.
pub trait ExecutionContext {
    /// Get the current bid/ask spread from the latest tick data
    ///
    /// # Arguments
    /// * `symbol` - The symbol to get spread for
    ///
    /// # Returns
    /// * `Option<(bid, ask)>` - Current bid and ask prices, or None if no data
    fn get_current_spread(&self, symbol: &str) -> Option<(f64, f64)>;

    /// Get bar context for slippage calculation
    ///
    /// # Arguments
    /// * `symbol` - The symbol to get bar for
    /// * `timeframe` - The timeframe of the bar
    ///
    /// # Returns
    /// * `Option<&Bar>` - Reference to the latest bar, or None if no data
    fn get_bar_context(&self, symbol: &str, timeframe: Timeframe) -> Option<&Bar>;

    /// Check if the market is open based on session boundaries
    ///
    /// # Arguments
    /// * `symbol` - The symbol to check
    /// * `timestamp` - The timestamp to check
    ///
    /// # Returns
    /// * `bool` - True if market is open, false otherwise
    fn is_market_open(&self, symbol: &str, timestamp: i64) -> bool;

    /// Get the latest tick timestamp for a symbol
    ///
    /// # Arguments
    /// * `symbol` - The symbol to check
    ///
    /// # Returns
    /// * `Option<i64>` - The timestamp of the last tick, or None if no data
    fn get_last_tick_time(&self, symbol: &str) -> Option<i64>;
}

/// Provides data for risk calculations
///
/// This trait will be implemented by the indicator pipeline and MTF engine
/// to provide data needed for risk management decisions.
pub trait RiskContext {
    /// Get an indicator value for risk decisions
    ///
    /// # Arguments
    /// * `symbol` - The symbol to get indicator for
    /// * `timeframe` - The timeframe of the indicator
    /// * `name` - The name of the indicator
    ///
    /// # Returns
    /// * `Option<f64>` - The indicator value, or None if not available
    fn get_indicator(&self, symbol: &str, timeframe: Timeframe, name: &str) -> Option<f64>;

    /// Get volatility measurement for position sizing
    ///
    /// # Arguments
    /// * `symbol` - The symbol to get volatility for
    /// * `timeframe` - The timeframe for volatility calculation
    ///
    /// # Returns
    /// * `Option<f64>` - The volatility value (ATR or similar), or None if not available
    fn get_volatility(&self, symbol: &str, timeframe: Timeframe) -> Option<f64>;

    /// Get margin requirement for a symbol
    ///
    /// # Arguments
    /// * `symbol` - The symbol to get margin for
    ///
    /// # Returns
    /// * `f64` - The margin requirement (default 1.0 for no leverage)
    fn get_margin_requirement(&self, symbol: &str) -> f64;

    /// Get the current account balance
    ///
    /// # Returns
    /// * `f64` - The current account balance
    fn get_account_balance(&self) -> f64;

    /// Get the current used margin
    ///
    /// # Returns
    /// * `f64` - The currently used margin
    fn get_used_margin(&self) -> f64;
}

/// Position snapshot for state persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionSnapshot {
    /// Timestamp of the snapshot
    pub timestamp: i64,
    /// Version identifier for compatibility checking
    pub version: String,
    /// Serialized position data
    pub positions_data: Vec<u8>,
    /// Current account balance
    pub account_balance: f64,
    /// Total used margin
    pub used_margin: f64,
    /// Total floating P&L
    pub floating_pnl: f64,
}

/// Extension for position state persistence
///
/// This trait extends the existing persistence system to handle
/// position-specific state storage and recovery.
pub trait PositionStateStore {
    /// Save a position snapshot
    ///
    /// # Arguments
    /// * `snapshot` - The position snapshot to save
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    fn save_positions(&self, snapshot: &PositionSnapshot) -> Result<()>;

    /// Restore positions from the last checkpoint
    ///
    /// # Returns
    /// * `Result<PositionSnapshot>` - The restored snapshot or error
    fn restore_positions(&self) -> Result<PositionSnapshot>;

    /// Check compatibility with MTF state version
    ///
    /// # Arguments
    /// * `mtf_version` - The MTF state version to check against
    ///
    /// # Returns
    /// * `bool` - True if compatible, false otherwise
    fn is_compatible_with_mtf(&self, mtf_version: &str) -> bool;

    /// Clear all position snapshots
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    fn clear_position_snapshots(&self) -> Result<()>;

    /// Get the latest snapshot timestamp
    ///
    /// # Returns
    /// * `Option<i64>` - The timestamp of the latest snapshot, or None if no snapshots
    fn get_latest_snapshot_time(&self) -> Option<i64>;
}

/// Order execution models for Epic 3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionModel {
    /// Perfect execution at exact prices (for ideal testing)
    Perfect,
    /// Realistic execution with spread and slippage
    Realistic,
    /// Worst-case execution for stress testing
    WorstCase,
}

/// Trade lifecycle events for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeEvent {
    /// Order placed
    OrderPlaced {
        id: String,
        symbol: String,
        side: OrderSide,
        quantity: f64,
        timestamp: i64,
    },
    /// Order filled
    OrderFilled {
        id: String,
        price: f64,
        slippage: f64,
        commission: f64,
        timestamp: i64,
    },
    /// Stop loss triggered
    StopLossTriggered {
        position_id: String,
        price: f64,
        timestamp: i64,
    },
    /// Take profit triggered
    TakeProfitTriggered {
        position_id: String,
        price: f64,
        timestamp: i64,
    },
    /// Position closed
    PositionClosed {
        id: String,
        pnl: f64,
        timestamp: i64,
    },
    /// Margin call
    MarginCall {
        required_margin: f64,
        available_margin: f64,
        timestamp: i64,
    },
}

/// Order side for trade events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Trade lifecycle logger interface
pub trait TradeLogger {
    /// Log a trade event
    ///
    /// # Arguments
    /// * `event` - The trade event to log
    fn log_event(&mut self, event: TradeEvent);

    /// Get events for a specific position
    ///
    /// # Arguments
    /// * `position_id` - The position ID to get events for
    ///
    /// # Returns
    /// * `Vec<TradeEvent>` - All events for the position
    fn get_position_events(&self, position_id: &str) -> Vec<TradeEvent>;

    /// Clear all logged events
    fn clear_events(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_snapshot_serialization() {
        let snapshot = PositionSnapshot {
            timestamp: 1234567890,
            version: "1.0.0".to_string(),
            positions_data: vec![1, 2, 3, 4],
            account_balance: 10000.0,
            used_margin: 1000.0,
            floating_pnl: 50.0,
        };

        // Test serialization
        let serialized = serde_json::to_string(&snapshot).unwrap();
        let deserialized: PositionSnapshot = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.timestamp, snapshot.timestamp);
        assert_eq!(deserialized.version, snapshot.version);
        assert_eq!(deserialized.account_balance, snapshot.account_balance);
    }

    #[test]
    fn test_execution_model() {
        assert_eq!(ExecutionModel::Perfect as i32, 0);
        assert_eq!(ExecutionModel::Realistic as i32, 1);
        assert_eq!(ExecutionModel::WorstCase as i32, 2);
    }
}