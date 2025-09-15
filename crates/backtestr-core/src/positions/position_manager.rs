use anyhow::{anyhow, Result};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::events::{BarEvent, EventHandler, TickEvent};
use crate::indicators::IndicatorValue;
use crate::interfaces::epic3_contracts::{PositionEventHandler, PositionSnapshot, TradeEvent};
use crate::mtf::MTFStateManager;
use crate::positions::pnl_calculator::PnlCalculator;
use crate::positions::position::{Position, PositionSide};
use crate::positions::position_state::{PositionStatistics, StateValidator};
use backtestr_data::{Bar, Tick, Timeframe};

/// Thread-safe position management system
pub struct PositionManager {
    /// All positions indexed by ID - using DashMap for concurrent access
    positions: Arc<DashMap<Uuid, Position>>,
    /// Index by symbol for fast symbol-based lookups
    symbol_index: Arc<DashMap<String, Vec<Uuid>>>,
    /// Parent-child relationships
    hierarchy_index: Arc<DashMap<Uuid, Vec<Uuid>>>,
    /// Reference to MTF state manager for market data
    mtf_state: Option<Arc<MTFStateManager>>,
    /// P&L calculator
    pnl_calculator: Arc<PnlCalculator>,
    /// Position statistics
    statistics: Arc<DashMap<String, PositionStatistics>>,
    /// Integration with Epic 2 checkpoint system
    checkpoint_enabled: bool,
    /// Trade events for logging
    trade_events: Arc<DashMap<String, Vec<TradeEvent>>>,
}

impl PositionManager {
    /// Create a new position manager
    pub fn new() -> Self {
        Self {
            positions: Arc::new(DashMap::new()),
            symbol_index: Arc::new(DashMap::new()),
            hierarchy_index: Arc::new(DashMap::new()),
            mtf_state: None,
            pnl_calculator: Arc::new(PnlCalculator::new()),
            statistics: Arc::new(DashMap::new()),
            checkpoint_enabled: true,
            trade_events: Arc::new(DashMap::new()),
        }
    }

    /// Create with MTF state manager integration
    pub fn with_mtf_state(mtf_state: Arc<MTFStateManager>) -> Self {
        let mut manager = Self::new();
        manager.mtf_state = Some(mtf_state);
        manager
    }

    /// Create from Epic 2 checkpoint during recovery
    pub fn from_checkpoint(
        snapshot: &PositionSnapshot,
        mtf_state: Option<Arc<MTFStateManager>>,
    ) -> Result<Self> {
        let mut manager = Self::new();
        manager.mtf_state = mtf_state;

        // Deserialize positions from checkpoint
        let positions: HashMap<Uuid, Position> = bincode::deserialize(&snapshot.positions_data)?;

        for (id, position) in positions {
            manager.positions.insert(id, position.clone());

            // Rebuild symbol index
            manager
                .symbol_index
                .entry(position.symbol.clone())
                .or_default()
                .push(id);

            // Rebuild hierarchy index
            if let Some(parent_id) = position.parent_id {
                manager
                    .hierarchy_index
                    .entry(parent_id)
                    .or_default()
                    .push(id);
            }
        }

        Ok(manager)
    }

    /// Save position state for Epic 2 checkpoint
    pub fn to_checkpoint(&self) -> Result<PositionSnapshot> {
        let positions: HashMap<Uuid, Position> = self
            .positions
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect();

        let positions_data = bincode::serialize(&positions)?;
        let floating_pnl = self.get_total_floating_pnl();

        Ok(PositionSnapshot {
            timestamp: chrono::Utc::now().timestamp(),
            version: "1.0.0".to_string(),
            positions_data,
            account_balance: 10000.0, // TODO: Get from account manager
            used_margin: self.calculate_total_margin(),
            floating_pnl,
        })
    }

    /// Open a new position
    #[allow(clippy::too_many_arguments)]
    pub fn open_position(
        &self,
        symbol: String,
        side: PositionSide,
        quantity: f64,
        entry_price: f64,
        stop_loss: Option<f64>,
        take_profit: Option<f64>,
        parent_id: Option<Uuid>,
        opened_at: i64,
    ) -> Result<Uuid> {
        // Validate inputs
        if quantity <= 0.0 {
            return Err(anyhow!("Quantity must be positive"));
        }
        if entry_price <= 0.0 {
            return Err(anyhow!("Entry price must be positive"));
        }

        // Create new position
        let position = Position::new_with_params(
            symbol.clone(),
            side,
            quantity,
            entry_price,
            stop_loss,
            take_profit,
            parent_id,
            opened_at,
        );

        let position_id = position.id;

        // Update parent if exists
        if let Some(pid) = parent_id {
            if let Some(mut parent) = self.positions.get_mut(&pid) {
                parent.add_child(position_id);
            }
        }

        // Insert position
        self.positions.insert(position_id, position.clone());

        // Update symbol index
        self.symbol_index
            .entry(symbol.clone())
            .or_default()
            .push(position_id);

        // Update hierarchy index if child
        if let Some(pid) = parent_id {
            self.hierarchy_index
                .entry(pid)
                .or_default()
                .push(position_id);
        }

        // Update statistics
        self.statistics
            .entry(symbol.clone())
            .or_default()
            .record_opened();

        // Log trade event
        self.log_trade_event(TradeEvent::OrderPlaced {
            id: position_id.to_string(),
            symbol,
            side: match side {
                PositionSide::Long => crate::interfaces::epic3_contracts::OrderSide::Buy,
                PositionSide::Short => crate::interfaces::epic3_contracts::OrderSide::Sell,
            },
            quantity,
            timestamp: opened_at,
        });

        Ok(position_id)
    }

    /// Get position by ID
    pub fn get_position(&self, id: &Uuid) -> Option<Position> {
        self.positions.get(id).map(|entry| entry.clone())
    }

    /// Update position price (for P&L calculation)
    pub fn update_position_price(&self, id: &Uuid, new_price: f64) -> Result<()> {
        let mut position = self
            .positions
            .get_mut(id)
            .ok_or_else(|| anyhow!("Position not found"))?;

        if !StateValidator::can_modify(position.state) {
            return Err(anyhow!(
                "Cannot modify position in state {:?}",
                position.state
            ));
        }

        position.update_price(new_price);

        // Check stop loss and take profit
        if position.is_stop_loss_triggered() {
            self.log_trade_event(TradeEvent::StopLossTriggered {
                position_id: id.to_string(),
                price: new_price,
                timestamp: chrono::Utc::now().timestamp(),
            });
        }

        if position.is_take_profit_triggered() {
            self.log_trade_event(TradeEvent::TakeProfitTriggered {
                position_id: id.to_string(),
                price: new_price,
                timestamp: chrono::Utc::now().timestamp(),
            });
        }

        Ok(())
    }

    /// Close position
    pub fn close_position(&self, id: &Uuid, close_price: f64, closed_at: i64) -> Result<f64> {
        let mut position = self
            .positions
            .get_mut(id)
            .ok_or_else(|| anyhow!("Position not found"))?;

        if !StateValidator::can_close(position.state) {
            return Err(anyhow!(
                "Cannot close position in state {:?}",
                position.state
            ));
        }

        let pnl = position.close(close_price, closed_at);

        // Update statistics
        self.statistics
            .entry(position.symbol.clone())
            .or_default()
            .update_with_closed_position(pnl);

        // Log trade event
        self.log_trade_event(TradeEvent::PositionClosed {
            id: id.to_string(),
            pnl,
            timestamp: closed_at,
        });

        Ok(pnl)
    }

    /// Partially close a position
    pub fn partial_close_position(
        &self,
        id: &Uuid,
        close_quantity: f64,
        close_price: f64,
        closed_at: i64,
    ) -> Result<f64> {
        let mut position = self
            .positions
            .get_mut(id)
            .ok_or_else(|| anyhow!("Position not found"))?;

        if !StateValidator::can_close(position.state) {
            return Err(anyhow!(
                "Cannot close position in state {:?}",
                position.state
            ));
        }

        if close_quantity > position.quantity {
            return Err(anyhow!("Close quantity exceeds position quantity"));
        }

        // Calculate partial P&L
        let partial_ratio = close_quantity / position.quantity;
        let partial_pnl = position.realized_pnl(close_price) * partial_ratio;

        // Update position quantity
        position.quantity -= close_quantity;

        if position.quantity <= 0.0 {
            // Fully closed
            position.close(close_price, closed_at);
        }

        Ok(partial_pnl)
    }

    /// Get all positions for a symbol
    pub fn get_positions_by_symbol(&self, symbol: &str) -> Vec<Position> {
        if let Some(ids) = self.symbol_index.get(symbol) {
            ids.iter()
                .filter_map(|id| self.positions.get(id).map(|p| p.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all open positions
    pub fn get_open_positions(&self) -> Vec<Position> {
        self.positions
            .iter()
            .filter(|entry| entry.value().is_open())
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get all child positions of a parent
    pub fn get_child_positions(&self, parent_id: &Uuid) -> Vec<Position> {
        if let Some(child_ids) = self.hierarchy_index.get(parent_id) {
            child_ids
                .iter()
                .filter_map(|id| self.positions.get(id).map(|p| p.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get total floating P&L across all positions
    pub fn get_total_floating_pnl(&self) -> f64 {
        self.positions
            .iter()
            .filter(|entry| entry.value().is_open())
            .map(|entry| entry.value().unrealized_pnl())
            .sum()
    }

    /// Get floating P&L by symbol
    pub fn get_floating_pnl_by_symbol(&self) -> HashMap<String, f64> {
        let positions: Vec<Position> = self.get_open_positions();
        self.pnl_calculator.calculate_pnl_by_symbol(&positions)
    }

    /// Bulk price update for efficiency
    pub fn bulk_update_prices(&self, price_updates: &HashMap<String, f64>) -> Result<()> {
        for (symbol, price) in price_updates {
            if let Some(position_ids) = self.symbol_index.get(symbol) {
                for id in position_ids.iter() {
                    if let Some(mut position) = self.positions.get_mut(id) {
                        if position.is_open() {
                            position.update_price(*price);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Calculate total margin used
    pub fn calculate_total_margin(&self) -> f64 {
        self.positions
            .iter()
            .filter(|entry| entry.value().is_open())
            .map(|entry| {
                let position = entry.value();
                self.pnl_calculator.calculate_margin_required(
                    position.quantity,
                    position.entry_price,
                    100.0, // Default leverage
                )
            })
            .sum()
    }

    /// Get position statistics for a symbol
    pub fn get_statistics(&self, symbol: &str) -> Option<PositionStatistics> {
        self.statistics.get(symbol).map(|s| s.clone())
    }

    /// Clear all closed positions (for memory management)
    pub fn clear_closed_positions(&self) {
        let closed_ids: Vec<Uuid> = self
            .positions
            .iter()
            .filter(|entry| entry.value().is_closed())
            .map(|entry| *entry.key())
            .collect();

        for id in closed_ids {
            self.positions.remove(&id);
        }
    }

    /// Set checkpoint enabled/disabled
    pub fn set_checkpoint_enabled(&mut self, enabled: bool) {
        self.checkpoint_enabled = enabled;
    }

    /// Log a trade event
    fn log_trade_event(&self, event: TradeEvent) {
        let key = match &event {
            TradeEvent::OrderPlaced { id, .. }
            | TradeEvent::OrderFilled { id, .. }
            | TradeEvent::PositionClosed { id, .. } => id.clone(),
            TradeEvent::StopLossTriggered { position_id, .. }
            | TradeEvent::TakeProfitTriggered { position_id, .. } => position_id.clone(),
            TradeEvent::MarginCall { .. } => "margin_calls".to_string(),
        };

        self.trade_events.entry(key).or_default().push(event);
    }

    /// Get trade events for a position
    pub fn get_position_events(&self, position_id: &str) -> Vec<TradeEvent> {
        self.trade_events
            .get(position_id)
            .map(|events| events.clone())
            .unwrap_or_default()
    }

    /// Count open positions
    pub fn count_open_positions(&self) -> usize {
        self.positions
            .iter()
            .filter(|entry| entry.value().is_open())
            .count()
    }

    /// Count total positions
    pub fn count_total_positions(&self) -> usize {
        self.positions.len()
    }
}

impl Default for PositionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_manager_creation() {
        let manager = PositionManager::new();
        assert_eq!(manager.count_total_positions(), 0);
        assert_eq!(manager.count_open_positions(), 0);
    }

    #[test]
    fn test_open_position() {
        let manager = PositionManager::new();
        let result = manager.open_position(
            "EURUSD".to_string(),
            PositionSide::Long,
            100_000.0,
            1.1000,
            Some(1.0950),
            Some(1.1050),
            None,
            1234567890,
        );

        assert!(result.is_ok());
        let position_id = result.unwrap();

        let position = manager.get_position(&position_id);
        assert!(position.is_some());

        let pos = position.unwrap();
        assert_eq!(pos.symbol, "EURUSD");
        assert_eq!(pos.quantity, 100_000.0);
        assert_eq!(pos.entry_price, 1.1000);
    }

    #[test]
    fn test_close_position() {
        let manager = PositionManager::new();
        let position_id = manager
            .open_position(
                "EURUSD".to_string(),
                PositionSide::Long,
                100_000.0,
                1.1000,
                None,
                None,
                None,
                1234567890,
            )
            .unwrap();

        let pnl = manager
            .close_position(&position_id, 1.1050, 1234567900)
            .unwrap();
        assert!((pnl - 500.0).abs() < 0.01);

        let position = manager.get_position(&position_id).unwrap();
        assert!(position.is_closed());
    }

    #[test]
    fn test_bulk_price_update() {
        let manager = PositionManager::new();

        // Open multiple positions
        let _pos1 = manager
            .open_position(
                "EURUSD".to_string(),
                PositionSide::Long,
                100_000.0,
                1.1000,
                None,
                None,
                None,
                1,
            )
            .unwrap();

        let _pos2 = manager
            .open_position(
                "EURUSD".to_string(),
                PositionSide::Short,
                50_000.0,
                1.1010,
                None,
                None,
                None,
                2,
            )
            .unwrap();

        let _pos3 = manager
            .open_position(
                "GBPUSD".to_string(),
                PositionSide::Long,
                100_000.0,
                1.3000,
                None,
                None,
                None,
                3,
            )
            .unwrap();

        // Bulk update prices
        let mut price_updates = HashMap::new();
        price_updates.insert("EURUSD".to_string(), 1.1050);
        price_updates.insert("GBPUSD".to_string(), 1.3050);

        manager.bulk_update_prices(&price_updates).unwrap();

        // Check P&L
        let total_pnl = manager.get_total_floating_pnl();
        // Should be around 800: 500 (long) - 200 (short) + 500 (gbpusd)
        assert!((total_pnl - 800.0).abs() < 1.0);
    }

    #[test]
    fn test_parent_child_relationships() {
        let manager = PositionManager::new();

        let parent_id = manager
            .open_position(
                "EURUSD".to_string(),
                PositionSide::Long,
                100_000.0,
                1.1000,
                None,
                None,
                None,
                1,
            )
            .unwrap();

        let child1_id = manager
            .open_position(
                "EURUSD".to_string(),
                PositionSide::Long,
                50_000.0,
                1.1010,
                None,
                None,
                Some(parent_id),
                2,
            )
            .unwrap();

        let _child2_id = manager
            .open_position(
                "EURUSD".to_string(),
                PositionSide::Long,
                30_000.0,
                1.1020,
                None,
                None,
                Some(parent_id),
                3,
            )
            .unwrap();

        let children = manager.get_child_positions(&parent_id);
        assert_eq!(children.len(), 2);

        let parent = manager.get_position(&parent_id).unwrap();
        assert_eq!(parent.child_ids.len(), 2);
        assert!(parent.child_ids.contains(&child1_id));
    }

    #[test]
    fn test_position_statistics() {
        let manager = PositionManager::new();

        // Open and close positions with various P&L
        let pos1 = manager
            .open_position(
                "EURUSD".to_string(),
                PositionSide::Long,
                100_000.0,
                1.1000,
                None,
                None,
                None,
                1,
            )
            .unwrap();
        manager.close_position(&pos1, 1.1050, 2).unwrap(); // +500

        let pos2 = manager
            .open_position(
                "EURUSD".to_string(),
                PositionSide::Short,
                100_000.0,
                1.1050,
                None,
                None,
                None,
                3,
            )
            .unwrap();
        manager.close_position(&pos2, 1.1100, 4).unwrap(); // -500

        let pos3 = manager
            .open_position(
                "EURUSD".to_string(),
                PositionSide::Long,
                100_000.0,
                1.1000,
                None,
                None,
                None,
                5,
            )
            .unwrap();
        manager.close_position(&pos3, 1.1030, 6).unwrap(); // +300

        let stats = manager.get_statistics("EURUSD").unwrap();
        assert_eq!(stats.total_opened, 3);
        assert_eq!(stats.total_closed, 3);
        assert_eq!(stats.total_wins, 2);
        assert_eq!(stats.total_losses, 1);
        assert!((stats.total_pnl - 300.0).abs() < 0.01);
    }

    #[test]
    fn test_checkpoint_serialization() {
        let manager = PositionManager::new();

        // Add some positions
        manager
            .open_position(
                "EURUSD".to_string(),
                PositionSide::Long,
                100_000.0,
                1.1000,
                None,
                None,
                None,
                1,
            )
            .unwrap();

        manager
            .open_position(
                "GBPUSD".to_string(),
                PositionSide::Short,
                50_000.0,
                1.3000,
                None,
                None,
                None,
                2,
            )
            .unwrap();

        // Create checkpoint
        let snapshot = manager.to_checkpoint().unwrap();
        assert!(!snapshot.positions_data.is_empty());

        // Restore from checkpoint
        let restored = PositionManager::from_checkpoint(&snapshot, None).unwrap();
        assert_eq!(restored.count_total_positions(), 2);
        assert_eq!(restored.count_open_positions(), 2);

        // Verify positions were restored correctly
        let positions = restored.get_positions_by_symbol("EURUSD");
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].quantity, 100_000.0);
    }

    #[test]
    fn test_partial_close() {
        let manager = PositionManager::new();

        let position_id = manager
            .open_position(
                "EURUSD".to_string(),
                PositionSide::Long,
                100_000.0,
                1.1000,
                None,
                None,
                None,
                1,
            )
            .unwrap();

        // Partially close 40%
        let partial_pnl = manager
            .partial_close_position(&position_id, 40_000.0, 1.1050, 2)
            .unwrap();

        assert!((partial_pnl - 200.0).abs() < 0.01); // 40% of 500

        let position = manager.get_position(&position_id).unwrap();
        assert_eq!(position.quantity, 60_000.0);
        assert!(position.is_open());
    }
}

// Implement EventHandler trait (required base trait)
impl EventHandler for PositionManager {
    fn on_tick(&self, event: &TickEvent) {
        // Extract tick data and update positions
        let tick = &event.tick;
        // Note: Using &self instead of &mut self, so we can't modify directly
        // This is handled through interior mutability with DashMap

        // Update positions with new tick price
        if let Some(position_ids) = self.symbol_index.get(&tick.symbol) {
            for id in position_ids.iter() {
                if let Some(mut position) = self.positions.get_mut(id) {
                    if position.is_open() {
                        // Update with appropriate price based on position side
                        let update_price = match position.side {
                            PositionSide::Long => tick.bid,  // Exit at bid for long
                            PositionSide::Short => tick.ask, // Exit at ask for short
                        };
                        position.update_price(update_price);

                        // Check for stop loss or take profit triggers
                        if position.is_stop_loss_triggered() || position.is_take_profit_triggered()
                        {
                            // Mark for closure (can't call close_position directly due to &self)
                            // In a real implementation, we'd queue these for processing
                        }
                    }
                }
            }
        }
    }

    fn on_bar(&self, event: &BarEvent) {
        // Extract bar data and update positions
        let bar = &event.bar;

        // Update positions with bar close price
        if let Some(position_ids) = self.symbol_index.get(&bar.symbol) {
            for id in position_ids.iter() {
                if let Some(mut position) = self.positions.get_mut(id) {
                    if position.is_open() {
                        position.update_price(bar.close);

                        // Check for stop loss or take profit triggers
                        if position.is_stop_loss_triggered() || position.is_take_profit_triggered()
                        {
                            // Mark for closure (can't call close_position directly due to &self)
                            // In a real implementation, we'd queue these for processing
                        }
                    }
                }
            }
        }
    }
}

// Implement PositionEventHandler trait
impl PositionEventHandler for PositionManager {
    fn on_bar_complete(&mut self, bar: &Bar, _timeframe: Timeframe, symbol: &str) {
        // Update positions with bar close price
        if let Some(position_ids) = self.symbol_index.get(symbol) {
            for id in position_ids.iter() {
                if let Some(mut position) = self.positions.get_mut(id) {
                    if position.is_open() {
                        position.update_price(bar.close);

                        // Check for stop loss or take profit triggers
                        if position.is_stop_loss_triggered() || position.is_take_profit_triggered()
                        {
                            let _ = self.close_position(id, bar.close, bar.timestamp_end);
                        }
                    }
                }
            }
        }

        // Access MTF state for additional context if available
        if let Some(_mtf_state) = &self.mtf_state {
            // Additional processing with MTF state can be done here
            // For example, checking spread conditions, session boundaries, etc.
        }
    }

    fn on_tick_update(&mut self, tick: &Tick, symbol: &str) {
        // Update all positions for this symbol with new price
        // Use mid-price for P&L calculation
        let _mid_price = (tick.bid + tick.ask) / 2.0;

        if let Some(position_ids) = self.symbol_index.get(symbol) {
            for id in position_ids.iter() {
                if let Some(mut position) = self.positions.get_mut(id) {
                    if position.is_open() {
                        // Update with appropriate price based on position side
                        let update_price = match position.side {
                            PositionSide::Long => tick.bid,  // Exit at bid for long
                            PositionSide::Short => tick.ask, // Exit at ask for short
                        };
                        position.update_price(update_price);

                        // Check for stop loss or take profit triggers
                        if position.is_stop_loss_triggered() || position.is_take_profit_triggered()
                        {
                            let _ = self.close_position(id, update_price, tick.timestamp);
                        }
                    }
                }
            }
        }

        // Trigger checkpoint if enabled and MTF state is available
        if self.checkpoint_enabled {
            if let Some(_mtf_state) = &self.mtf_state {
                // Request checkpoint through MTF state manager
                // This would be implemented in the MTF state manager
            }
        }
    }

    fn on_indicator_update(
        &mut self,
        _indicator: &IndicatorValue,
        _timeframe: Timeframe,
        _symbol: &str,
    ) {
        // Future: Use indicators for dynamic stop adjustments
        // For now: No-op implementation
        // This could be used for:
        // - Trailing stops based on ATR
        // - Dynamic position sizing based on volatility
        // - Exit signals based on indicator values
    }
}
