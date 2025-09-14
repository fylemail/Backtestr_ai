//! Mock Position Manager for Epic 3 integration testing
//!
//! This module provides a mock implementation of the position management
//! interfaces to test integration with the Epic 2 MTF engine.

use crate::events::{EventHandler, TickEvent, BarEvent};
use crate::indicators::IndicatorValue;
use crate::interfaces::{
    ExecutionModel, OrderSide, PositionEventHandler, PositionSnapshot, PositionStateStore,
    TradeEvent, TradeLogger,
};
use backtestr_data::{Bar, Tick, Timeframe};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Performance metrics for event processing
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub tick_count: usize,
    pub bar_count: usize,
    pub indicator_count: usize,
    pub total_tick_time_us: u128,
    pub total_bar_time_us: u128,
    pub total_indicator_time_us: u128,
    pub max_tick_time_us: u128,
    pub max_bar_time_us: u128,
    pub max_indicator_time_us: u128,
}

impl PerformanceMetrics {
    pub fn average_tick_time_us(&self) -> f64 {
        if self.tick_count == 0 {
            0.0
        } else {
            self.total_tick_time_us as f64 / self.tick_count as f64
        }
    }

    pub fn average_bar_time_us(&self) -> f64 {
        if self.bar_count == 0 {
            0.0
        } else {
            self.total_bar_time_us as f64 / self.bar_count as f64
        }
    }

    pub fn average_indicator_time_us(&self) -> f64 {
        if self.indicator_count == 0 {
            0.0
        } else {
            self.total_indicator_time_us as f64 / self.indicator_count as f64
        }
    }
}

/// Mock position manager for testing Epic 2→3 integration
pub struct MockPositionManager {
    /// Event log for verification
    event_log: Arc<Mutex<Vec<String>>>,
    /// Trade events for lifecycle tracking
    trade_events: Arc<Mutex<Vec<TradeEvent>>>,
    /// Performance metrics
    metrics: Arc<Mutex<PerformanceMetrics>>,
    /// Simulated positions (position_id -> details)
    positions: Arc<Mutex<HashMap<String, MockPosition>>>,
    /// Execution model for testing
    execution_model: ExecutionModel,
    /// Enable detailed logging
    verbose: bool,
}

/// Mock position for testing
#[derive(Debug, Clone)]
struct MockPosition {
    id: String,
    symbol: String,
    side: OrderSide,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
    opened_at: i64,
    pnl: f64,
}

impl MockPositionManager {
    /// Create a new mock position manager
    pub fn new(execution_model: ExecutionModel, verbose: bool) -> Self {
        Self {
            event_log: Arc::new(Mutex::new(Vec::new())),
            trade_events: Arc::new(Mutex::new(Vec::new())),
            metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
            positions: Arc::new(Mutex::new(HashMap::new())),
            execution_model,
            verbose,
        }
    }

    /// Get a copy of the event log
    pub fn get_event_log(&self) -> Vec<String> {
        self.event_log.lock().unwrap().clone()
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.lock().unwrap().clone()
    }

    /// Simulate opening a position
    pub fn open_position(
        &mut self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        price: f64,
    ) -> String {
        let position_id = format!("POS_{}", self.positions.lock().unwrap().len() + 1);

        let position = MockPosition {
            id: position_id.clone(),
            symbol: symbol.to_string(),
            side,
            quantity,
            entry_price: price,
            current_price: price,
            stop_loss: None,
            take_profit: None,
            opened_at: Instant::now().elapsed().as_millis() as i64,
            pnl: 0.0,
        };

        self.positions
            .lock()
            .unwrap()
            .insert(position_id.clone(), position);

        // Log trade event
        let event = TradeEvent::OrderPlaced {
            id: position_id.clone(),
            symbol: symbol.to_string(),
            side,
            quantity,
            timestamp: Instant::now().elapsed().as_millis() as i64,
        };
        self.log_event(event);

        position_id
    }

    /// Update position P&L based on current price
    fn update_position_pnl(&mut self, position_id: &str, current_price: f64) {
        let mut positions = self.positions.lock().unwrap();
        if let Some(position) = positions.get_mut(position_id) {
            position.current_price = current_price;
            let price_diff = current_price - position.entry_price;
            position.pnl = match position.side {
                OrderSide::Buy => price_diff * position.quantity,
                OrderSide::Sell => -price_diff * position.quantity,
            };
        }
    }

    /// Check if stop loss or take profit is triggered
    fn check_exit_conditions(&mut self, position: &MockPosition) {
        let should_close = if let Some(sl) = position.stop_loss {
            match position.side {
                OrderSide::Buy => position.current_price <= sl,
                OrderSide::Sell => position.current_price >= sl,
            }
        } else if let Some(tp) = position.take_profit {
            match position.side {
                OrderSide::Buy => position.current_price >= tp,
                OrderSide::Sell => position.current_price <= tp,
            }
        } else {
            false
        };

        if should_close {
            let event = TradeEvent::PositionClosed {
                id: position.id.clone(),
                pnl: position.pnl,
                timestamp: Instant::now().elapsed().as_millis() as i64,
            };
            self.log_event(event);
        }
    }

    /// Print performance report
    pub fn print_performance_report(&self) {
        let metrics = self.get_metrics();
        println!("\n=== Mock Position Manager Performance Report ===");
        println!("Tick Events:");
        println!("  Count: {}", metrics.tick_count);
        println!("  Average processing: {:.2}μs", metrics.average_tick_time_us());
        println!("  Max processing: {}μs", metrics.max_tick_time_us);

        println!("\nBar Events:");
        println!("  Count: {}", metrics.bar_count);
        println!("  Average processing: {:.2}μs", metrics.average_bar_time_us());
        println!("  Max processing: {}μs", metrics.max_bar_time_us);

        println!("\nIndicator Events:");
        println!("  Count: {}", metrics.indicator_count);
        println!("  Average processing: {:.2}μs", metrics.average_indicator_time_us());
        println!("  Max processing: {}μs", metrics.max_indicator_time_us);

        println!("\nPositions:");
        println!("  Open: {}", self.positions.lock().unwrap().len());
        println!("  Trade events logged: {}", self.trade_events.lock().unwrap().len());
        println!("================================================\n");
    }
}

impl EventHandler for MockPositionManager {
    fn on_tick(&self, event: &TickEvent) {
        if self.verbose {
            let mut log = self.event_log.lock().unwrap();
            log.push(format!("Tick event: {:?}", event));
        }
    }

    fn on_bar(&self, event: &BarEvent) {
        if self.verbose {
            let mut log = self.event_log.lock().unwrap();
            log.push(format!("Bar event: {:?}", event));
        }
    }
}

impl PositionEventHandler for MockPositionManager {
    fn on_bar_complete(&mut self, bar: &Bar, timeframe: Timeframe, symbol: &str) {
        let start = Instant::now();

        // Log event
        if self.verbose {
            let mut log = self.event_log.lock().unwrap();
            log.push(format!(
                "Bar complete: {} @ {:?} | O:{:.5} H:{:.5} L:{:.5} C:{:.5}",
                symbol, timeframe, bar.open, bar.high, bar.low, bar.close
            ));
        }

        // Update metrics
        let elapsed = start.elapsed().as_micros();
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.bar_count += 1;
            metrics.total_bar_time_us += elapsed;
            if elapsed > metrics.max_bar_time_us {
                metrics.max_bar_time_us = elapsed;
            }
        } // Drop metrics lock before calling other methods

        // Simulate position management logic
        let positions = self.positions.lock().unwrap().clone();
        for (id, position) in positions.iter() {
            if position.symbol == symbol {
                self.update_position_pnl(&id, bar.close);
            }
        }
    }

    fn on_tick_update(&mut self, tick: &Tick, symbol: &str) {
        let start = Instant::now();

        // Log every 1000th tick to avoid spam
        if self.verbose && self.metrics.lock().unwrap().tick_count % 1000 == 0 {
            let mut log = self.event_log.lock().unwrap();
            log.push(format!(
                "Tick #{}: {} @ {:.5}/{:.5}",
                self.metrics.lock().unwrap().tick_count,
                symbol,
                tick.bid,
                tick.ask
            ));
        }

        // Update metrics
        let elapsed = start.elapsed().as_micros();
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.tick_count += 1;
            metrics.total_tick_time_us += elapsed;
            if elapsed > metrics.max_tick_time_us {
                metrics.max_tick_time_us = elapsed;
            }
        } // Drop metrics lock before calling other methods

        // Update position P&L with current price
        let mid_price = (tick.bid + tick.ask) / 2.0;
        let positions = self.positions.lock().unwrap().clone();
        for (id, position) in positions.iter() {
            if position.symbol == symbol {
                self.update_position_pnl(&id, mid_price);
                self.check_exit_conditions(&position);
            }
        }
    }

    fn on_indicator_update(&mut self, indicator: &IndicatorValue, timeframe: Timeframe, symbol: &str) {
        let start = Instant::now();

        // Log indicator updates
        if self.verbose {
            let mut log = self.event_log.lock().unwrap();
            log.push(format!(
                "Indicator @ {:?} for {} = {:.5}",
                timeframe, symbol, indicator.value
            ));
        }

        // Update metrics
        let elapsed = start.elapsed().as_micros();
        let mut metrics = self.metrics.lock().unwrap();
        metrics.indicator_count += 1;
        metrics.total_indicator_time_us += elapsed;
        if elapsed > metrics.max_indicator_time_us {
            metrics.max_indicator_time_us = elapsed;
        }

        // Simulate strategy decisions based on indicators
        // (This is where Epic 3 would make trading decisions)
    }
}

impl TradeLogger for MockPositionManager {
    fn log_event(&mut self, event: TradeEvent) {
        self.trade_events.lock().unwrap().push(event.clone());

        if self.verbose {
            let mut log = self.event_log.lock().unwrap();
            log.push(format!("Trade event: {:?}", event));
        }
    }

    fn get_position_events(&self, position_id: &str) -> Vec<TradeEvent> {
        self.trade_events
            .lock()
            .unwrap()
            .iter()
            .filter(|event| match event {
                TradeEvent::OrderPlaced { id, .. } => id == position_id,
                TradeEvent::OrderFilled { id, .. } => id == position_id,
                TradeEvent::StopLossTriggered { position_id: id, .. } => id == position_id,
                TradeEvent::TakeProfitTriggered { position_id: id, .. } => id == position_id,
                TradeEvent::PositionClosed { id, .. } => id == position_id,
                _ => false,
            })
            .cloned()
            .collect()
    }

    fn clear_events(&mut self) {
        self.trade_events.lock().unwrap().clear();
        self.event_log.lock().unwrap().clear();
    }
}

/// Mock implementation of PositionStateStore for testing
pub struct MockPositionStateStore {
    snapshots: Arc<Mutex<Vec<PositionSnapshot>>>,
    mtf_compatible_version: String,
}

impl MockPositionStateStore {
    pub fn new() -> Self {
        Self {
            snapshots: Arc::new(Mutex::new(Vec::new())),
            mtf_compatible_version: "2.0.0".to_string(),
        }
    }
}

impl PositionStateStore for MockPositionStateStore {
    fn save_positions(&self, snapshot: &PositionSnapshot) -> Result<()> {
        let mut snapshots = self.snapshots.lock().unwrap();
        snapshots.push(snapshot.clone());
        Ok(())
    }

    fn restore_positions(&self) -> Result<PositionSnapshot> {
        let snapshots = self.snapshots.lock().unwrap();
        snapshots
            .last()
            .cloned()
            .ok_or_else(|| anyhow!("No position snapshots available"))
    }

    fn is_compatible_with_mtf(&self, mtf_version: &str) -> bool {
        // Simple version check for testing
        mtf_version == self.mtf_compatible_version
    }

    fn clear_position_snapshots(&self) -> Result<()> {
        self.snapshots.lock().unwrap().clear();
        Ok(())
    }

    fn get_latest_snapshot_time(&self) -> Option<i64> {
        self.snapshots
            .lock()
            .unwrap()
            .last()
            .map(|s| s.timestamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_position_manager_creation() {
        let manager = MockPositionManager::new(ExecutionModel::Realistic, false);
        assert_eq!(manager.get_metrics().tick_count, 0);
        assert_eq!(manager.get_metrics().bar_count, 0);
        assert_eq!(manager.get_metrics().indicator_count, 0);
    }

    #[test]
    fn test_position_lifecycle() {
        let mut manager = MockPositionManager::new(ExecutionModel::Perfect, true);

        // Open a position
        let position_id = manager.open_position("EURUSD", OrderSide::Buy, 1.0, 1.1000);
        assert!(!position_id.is_empty());

        // Verify position was created
        let positions = manager.positions.lock().unwrap();
        assert_eq!(positions.len(), 1);
        assert!(positions.contains_key(&position_id));

        // Verify trade event was logged
        let events = manager.trade_events.lock().unwrap();
        assert_eq!(events.len(), 1);
        matches!(&events[0], TradeEvent::OrderPlaced { .. });
    }

    #[test]
    fn test_performance_metrics() {
        let mut manager = MockPositionManager::new(ExecutionModel::Realistic, false);

        // Simulate some events
        let tick = Tick {
            id: Some(1),
            symbol: "EURUSD".to_string(),
            timestamp: 1234567890,
            bid: 1.1000,
            ask: 1.1001,
            bid_size: Some(100),
            ask_size: Some(100),
        };

        for _ in 0..100 {
            manager.on_tick_update(&tick, "EURUSD");
        }

        let metrics = manager.get_metrics();
        assert_eq!(metrics.tick_count, 100);
        // Average time might be 0 on very fast machines, just check it doesn't panic
        let _ = metrics.average_tick_time_us();
    }

    #[test]
    fn test_position_state_store() {
        let store = MockPositionStateStore::new();

        let snapshot = PositionSnapshot {
            timestamp: 1234567890,
            version: "1.0.0".to_string(),
            positions_data: vec![1, 2, 3],
            account_balance: 10000.0,
            used_margin: 1000.0,
            floating_pnl: 50.0,
        };

        // Save and restore
        store.save_positions(&snapshot).unwrap();
        let restored = store.restore_positions().unwrap();

        assert_eq!(restored.timestamp, snapshot.timestamp);
        assert_eq!(restored.account_balance, snapshot.account_balance);

        // Check compatibility
        assert!(store.is_compatible_with_mtf("2.0.0"));
        assert!(!store.is_compatible_with_mtf("1.0.0"));
    }
}