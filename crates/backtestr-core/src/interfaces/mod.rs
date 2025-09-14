//! Interface contracts for clean separation between epics
//!
//! This module contains interface definitions that enable
//! different epics to communicate without tight coupling.

pub mod epic3_contracts;

pub use epic3_contracts::{
    ExecutionContext, ExecutionModel, OrderSide, PositionEventHandler, PositionSnapshot,
    PositionStateStore, RiskContext, TradeEvent, TradeLogger,
};
