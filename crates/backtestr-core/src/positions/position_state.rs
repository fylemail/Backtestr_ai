use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

/// Position state transitions and validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateTransition {
    PendingToOpen,
    PendingToCancelled,
    OpenToClosed,
    OpenToPending, // For partial fills
}

/// State transition validator
pub struct StateValidator;

impl StateValidator {
    /// Validate a state transition
    pub fn validate_transition(
        from: crate::positions::position::PositionState,
        to: crate::positions::position::PositionState,
    ) -> Result<StateTransition> {
        use crate::positions::position::PositionState;

        match (from, to) {
            (PositionState::Pending, PositionState::Open) => Ok(StateTransition::PendingToOpen),
            (PositionState::Pending, PositionState::Cancelled) => {
                Ok(StateTransition::PendingToCancelled)
            }
            (PositionState::Open, PositionState::Closed) => Ok(StateTransition::OpenToClosed),
            (PositionState::Open, PositionState::Pending) => Ok(StateTransition::OpenToPending),
            _ => Err(anyhow!(
                "Invalid state transition from {:?} to {:?}",
                from,
                to
            )),
        }
    }

    /// Check if a state allows modifications
    pub fn can_modify(state: crate::positions::position::PositionState) -> bool {
        use crate::positions::position::PositionState;

        matches!(state, PositionState::Open | PositionState::Pending)
    }

    /// Check if a state allows closing
    pub fn can_close(state: crate::positions::position::PositionState) -> bool {
        use crate::positions::position::PositionState;

        matches!(state, PositionState::Open)
    }

    /// Check if a state allows cancellation
    pub fn can_cancel(state: crate::positions::position::PositionState) -> bool {
        use crate::positions::position::PositionState;

        matches!(state, PositionState::Pending)
    }
}

/// Position statistics for tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PositionStatistics {
    pub total_opened: u64,
    pub total_closed: u64,
    pub total_cancelled: u64,
    pub total_wins: u64,
    pub total_losses: u64,
    pub total_pnl: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub average_win: f64,
    pub average_loss: f64,
    pub win_rate: f64,
    // Track individual wins and losses for accurate calculations
    wins_pnl: Vec<f64>,
    losses_pnl: Vec<f64>,
}

impl PositionStatistics {
    /// Update statistics with a closed position
    pub fn update_with_closed_position(&mut self, pnl: f64) {
        self.total_closed += 1;

        if pnl > 0.0 {
            self.total_wins += 1;
            self.wins_pnl.push(pnl);
            if pnl > self.largest_win {
                self.largest_win = pnl;
            }
        } else if pnl < 0.0 {
            self.total_losses += 1;
            self.losses_pnl.push(pnl);
            if pnl < self.largest_loss {
                self.largest_loss = pnl;
            }
        }

        self.total_pnl += pnl;
        self.calculate_averages();
    }

    /// Calculate average win/loss and win rate
    fn calculate_averages(&mut self) {
        if !self.wins_pnl.is_empty() {
            self.average_win = self.wins_pnl.iter().sum::<f64>() / self.wins_pnl.len() as f64;
        }

        if !self.losses_pnl.is_empty() {
            self.average_loss = self.losses_pnl.iter().sum::<f64>() / self.losses_pnl.len() as f64;
        }

        let total_trades = self.total_wins + self.total_losses;
        if total_trades > 0 {
            self.win_rate = self.total_wins as f64 / total_trades as f64;
        }
    }

    /// Record a new position opening
    pub fn record_opened(&mut self) {
        self.total_opened += 1;
    }

    /// Record a position cancellation
    pub fn record_cancelled(&mut self) {
        self.total_cancelled += 1;
    }

    /// Get profit factor (total wins / total losses)
    pub fn profit_factor(&self) -> f64 {
        if !self.losses_pnl.is_empty() && !self.wins_pnl.is_empty() {
            let total_wins = self.wins_pnl.iter().sum::<f64>();
            let total_losses = self.losses_pnl.iter().map(|l| l.abs()).sum::<f64>();
            if total_losses > 0.0 {
                total_wins / total_losses
            } else {
                f64::INFINITY
            }
        } else if !self.wins_pnl.is_empty() {
            f64::INFINITY
        } else {
            0.0
        }
    }

    /// Get expectancy (average profit per trade)
    pub fn expectancy(&self) -> f64 {
        let total_trades = self.total_closed;
        if total_trades > 0 {
            self.total_pnl / total_trades as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::positions::position::PositionState;

    #[test]
    fn test_valid_state_transitions() {
        assert!(
            StateValidator::validate_transition(PositionState::Pending, PositionState::Open)
                .is_ok()
        );
        assert!(StateValidator::validate_transition(
            PositionState::Pending,
            PositionState::Cancelled
        )
        .is_ok());
        assert!(
            StateValidator::validate_transition(PositionState::Open, PositionState::Closed).is_ok()
        );
    }

    #[test]
    fn test_invalid_state_transitions() {
        assert!(
            StateValidator::validate_transition(PositionState::Closed, PositionState::Open)
                .is_err()
        );
        assert!(
            StateValidator::validate_transition(PositionState::Cancelled, PositionState::Open)
                .is_err()
        );
        assert!(
            StateValidator::validate_transition(PositionState::Closed, PositionState::Pending)
                .is_err()
        );
    }

    #[test]
    fn test_state_permissions() {
        assert!(StateValidator::can_modify(PositionState::Open));
        assert!(StateValidator::can_modify(PositionState::Pending));
        assert!(!StateValidator::can_modify(PositionState::Closed));
        assert!(!StateValidator::can_modify(PositionState::Cancelled));

        assert!(StateValidator::can_close(PositionState::Open));
        assert!(!StateValidator::can_close(PositionState::Pending));
        assert!(!StateValidator::can_close(PositionState::Closed));

        assert!(StateValidator::can_cancel(PositionState::Pending));
        assert!(!StateValidator::can_cancel(PositionState::Open));
        assert!(!StateValidator::can_cancel(PositionState::Closed));
    }

    #[test]
    fn test_position_statistics() {
        let mut stats = PositionStatistics::default();

        stats.record_opened();
        stats.record_opened();
        stats.record_opened();

        stats.update_with_closed_position(100.0);
        stats.update_with_closed_position(-50.0);
        stats.update_with_closed_position(75.0);

        assert_eq!(stats.total_opened, 3);
        assert_eq!(stats.total_closed, 3);
        assert_eq!(stats.total_wins, 2);
        assert_eq!(stats.total_losses, 1);
        assert_eq!(stats.total_pnl, 125.0);
        assert_eq!(stats.largest_win, 100.0);
        assert_eq!(stats.largest_loss, -50.0);
        assert!(stats.win_rate > 0.66 && stats.win_rate < 0.67);
    }

    #[test]
    fn test_profit_factor() {
        let mut stats = PositionStatistics::default();

        stats.update_with_closed_position(100.0);
        stats.update_with_closed_position(50.0);
        stats.update_with_closed_position(-30.0);
        stats.update_with_closed_position(-20.0);

        // Total wins: 150, Total losses: 50
        // Profit factor should be 3.0
        let pf = stats.profit_factor();
        assert!((pf - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_expectancy() {
        let mut stats = PositionStatistics::default();

        stats.update_with_closed_position(100.0);
        stats.update_with_closed_position(-50.0);
        stats.update_with_closed_position(75.0);
        stats.update_with_closed_position(-25.0);

        // Total P&L: 100, Total trades: 4
        // Expectancy: 25
        assert_eq!(stats.expectancy(), 25.0);
    }
}
