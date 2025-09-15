use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Position {
    /// Unique position identifier
    pub id: Uuid,
    /// Trading symbol
    pub symbol: String,
    /// Position side (Long/Short)
    pub side: PositionSide,
    /// Position quantity (positive number)
    pub quantity: f64,
    /// Entry price
    pub entry_price: f64,
    /// Current market price
    pub current_price: f64,
    /// Stop loss level (optional)
    pub stop_loss: Option<f64>,
    /// Take profit level (optional)
    pub take_profit: Option<f64>,
    /// Position state
    pub state: PositionState,
    /// Open timestamp
    pub opened_at: i64,
    /// Close timestamp (if closed)
    pub closed_at: Option<i64>,
    /// Parent position ID (for hierarchies)
    pub parent_id: Option<Uuid>,
    /// Child position IDs
    pub child_ids: Vec<Uuid>,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionSide {
    Long,
    Short,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionState {
    Pending,   // Order placed but not filled
    Open,      // Position is active
    Closed,    // Position is closed
    Cancelled, // Order was cancelled
}

impl Position {
    /// Create a new position
    pub fn new(
        symbol: String,
        side: PositionSide,
        quantity: f64,
        entry_price: f64,
        opened_at: i64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            symbol,
            side,
            quantity,
            entry_price,
            current_price: entry_price,
            stop_loss: None,
            take_profit: None,
            state: PositionState::Open,
            opened_at,
            closed_at: None,
            parent_id: None,
            child_ids: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create a new position with full parameters
    pub fn new_with_params(
        symbol: String,
        side: PositionSide,
        quantity: f64,
        entry_price: f64,
        stop_loss: Option<f64>,
        take_profit: Option<f64>,
        parent_id: Option<Uuid>,
        opened_at: i64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            symbol,
            side,
            quantity,
            entry_price,
            current_price: entry_price,
            stop_loss,
            take_profit,
            state: PositionState::Open,
            opened_at,
            closed_at: None,
            parent_id,
            child_ids: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Calculate unrealized P&L
    pub fn unrealized_pnl(&self) -> f64 {
        match self.side {
            PositionSide::Long => (self.current_price - self.entry_price) * self.quantity,
            PositionSide::Short => (self.entry_price - self.current_price) * self.quantity,
        }
    }

    /// Calculate realized P&L (for closed positions)
    pub fn realized_pnl(&self, close_price: f64) -> f64 {
        match self.side {
            PositionSide::Long => (close_price - self.entry_price) * self.quantity,
            PositionSide::Short => (self.entry_price - close_price) * self.quantity,
        }
    }

    /// Update current price and recalculate P&L
    pub fn update_price(&mut self, new_price: f64) {
        self.current_price = new_price;
    }

    /// Check if stop loss is triggered
    pub fn is_stop_loss_triggered(&self) -> bool {
        if let Some(stop_level) = self.stop_loss {
            match self.side {
                PositionSide::Long => self.current_price <= stop_level,
                PositionSide::Short => self.current_price >= stop_level,
            }
        } else {
            false
        }
    }

    /// Check if take profit is triggered
    pub fn is_take_profit_triggered(&self) -> bool {
        if let Some(tp_level) = self.take_profit {
            match self.side {
                PositionSide::Long => self.current_price >= tp_level,
                PositionSide::Short => self.current_price <= tp_level,
            }
        } else {
            false
        }
    }

    /// Close the position
    pub fn close(&mut self, close_price: f64, closed_at: i64) -> f64 {
        self.current_price = close_price;
        self.closed_at = Some(closed_at);
        self.state = PositionState::Closed;
        self.realized_pnl(close_price)
    }

    /// Add a child position
    pub fn add_child(&mut self, child_id: Uuid) {
        if !self.child_ids.contains(&child_id) {
            self.child_ids.push(child_id);
        }
    }

    /// Remove a child position
    pub fn remove_child(&mut self, child_id: &Uuid) {
        self.child_ids.retain(|id| id != child_id);
    }

    /// Check if position is open
    pub fn is_open(&self) -> bool {
        self.state == PositionState::Open
    }

    /// Check if position is closed
    pub fn is_closed(&self) -> bool {
        self.state == PositionState::Closed
    }

    /// Set stop loss
    pub fn set_stop_loss(&mut self, stop_loss: f64) {
        self.stop_loss = Some(stop_loss);
    }

    /// Set take profit
    pub fn set_take_profit(&mut self, take_profit: f64) {
        self.take_profit = Some(take_profit);
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let position = Position::new(
            "EURUSD".to_string(),
            PositionSide::Long,
            10000.0,
            1.1000,
            1234567890,
        );

        assert_eq!(position.symbol, "EURUSD");
        assert_eq!(position.side, PositionSide::Long);
        assert_eq!(position.quantity, 10000.0);
        assert_eq!(position.entry_price, 1.1000);
        assert_eq!(position.current_price, 1.1000);
        assert_eq!(position.state, PositionState::Open);
        assert!(position.is_open());
        assert!(!position.is_closed());
    }

    #[test]
    fn test_long_position_pnl() {
        let mut position = Position::new(
            "EURUSD".to_string(),
            PositionSide::Long,
            10000.0,
            1.1000,
            1234567890,
        );

        // Price goes up - profit
        position.update_price(1.1050);
        let pnl = position.unrealized_pnl();
        assert!((pnl - 50.0).abs() < 0.01);

        // Price goes down - loss
        position.update_price(1.0950);
        let pnl = position.unrealized_pnl();
        assert!((pnl - (-50.0)).abs() < 0.01);
    }

    #[test]
    fn test_short_position_pnl() {
        let mut position = Position::new(
            "EURUSD".to_string(),
            PositionSide::Short,
            10000.0,
            1.1000,
            1234567890,
        );

        // Price goes down - profit
        position.update_price(1.0950);
        let pnl = position.unrealized_pnl();
        assert!((pnl - 50.0).abs() < 0.01);

        // Price goes up - loss
        position.update_price(1.1050);
        let pnl = position.unrealized_pnl();
        assert!((pnl - (-50.0)).abs() < 0.01);
    }

    #[test]
    fn test_stop_loss_trigger() {
        let mut long_position = Position::new_with_params(
            "EURUSD".to_string(),
            PositionSide::Long,
            10000.0,
            1.1000,
            Some(1.0950), // Stop loss at 1.0950
            None,
            None,
            1234567890,
        );

        // Not triggered
        long_position.update_price(1.0960);
        assert!(!long_position.is_stop_loss_triggered());

        // Triggered
        long_position.update_price(1.0950);
        assert!(long_position.is_stop_loss_triggered());

        // Also triggered below stop
        long_position.update_price(1.0940);
        assert!(long_position.is_stop_loss_triggered());
    }

    #[test]
    fn test_take_profit_trigger() {
        let mut long_position = Position::new_with_params(
            "EURUSD".to_string(),
            PositionSide::Long,
            10000.0,
            1.1000,
            None,
            Some(1.1050), // Take profit at 1.1050
            None,
            1234567890,
        );

        // Not triggered
        long_position.update_price(1.1040);
        assert!(!long_position.is_take_profit_triggered());

        // Triggered
        long_position.update_price(1.1050);
        assert!(long_position.is_take_profit_triggered());

        // Also triggered above TP
        long_position.update_price(1.1060);
        assert!(long_position.is_take_profit_triggered());
    }

    #[test]
    fn test_position_close() {
        let mut position = Position::new(
            "EURUSD".to_string(),
            PositionSide::Long,
            10000.0,
            1.1000,
            1234567890,
        );

        let pnl = position.close(1.1050, 1234567900);

        assert!((pnl - 50.0).abs() < 0.01);
        assert_eq!(position.state, PositionState::Closed);
        assert!(position.closed_at.is_some());
        assert_eq!(position.closed_at, Some(1234567900));
        assert!(position.is_closed());
    }

    #[test]
    fn test_parent_child_relationships() {
        let parent_id = Uuid::new_v4();
        let mut parent = Position::new_with_params(
            "EURUSD".to_string(),
            PositionSide::Long,
            10000.0,
            1.1000,
            None,
            None,
            None,
            1234567890,
        );

        let child = Position::new_with_params(
            "EURUSD".to_string(),
            PositionSide::Long,
            5000.0,
            1.1010,
            None,
            None,
            Some(parent_id),
            1234567891,
        );

        parent.add_child(child.id);
        assert_eq!(parent.child_ids.len(), 1);
        assert!(parent.child_ids.contains(&child.id));

        parent.remove_child(&child.id);
        assert_eq!(parent.child_ids.len(), 0);
    }

    #[test]
    fn test_position_metadata() {
        let mut position = Position::new(
            "EURUSD".to_string(),
            PositionSide::Long,
            10000.0,
            1.1000,
            1234567890,
        );

        position.add_metadata("strategy".to_string(), "trend_following".to_string());
        position.add_metadata("signal".to_string(), "ma_cross".to_string());

        assert_eq!(position.metadata.len(), 2);
        assert_eq!(
            position.metadata.get("strategy"),
            Some(&"trend_following".to_string())
        );
        assert_eq!(
            position.metadata.get("signal"),
            Some(&"ma_cross".to_string())
        );
    }
}
