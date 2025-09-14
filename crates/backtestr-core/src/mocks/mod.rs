//! Mock implementations for testing Epic 3 integration
//!
//! This module provides mock implementations of Epic 3 interfaces
//! for testing integration with the Epic 2 MTF engine.

pub mod mock_position_manager;

pub use mock_position_manager::{
    MockPositionManager, MockPositionStateStore, PerformanceMetrics,
};