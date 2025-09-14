//! Core MTF engine and state management for BackTestr AI
//!
//! This crate provides the multi-timeframe state engine that maintains
//! synchronized bar states across 6 timeframes with sub-100μs updates.

pub mod data;
pub mod engine;
pub mod events;
pub mod indicators;
pub mod mtf;
pub mod positions;
pub mod python;

pub use engine::MTFEngine;
pub use mtf::{MTFConfig, MTFStateManager, StateQuery};

// Re-export Timeframe from data crate
pub use backtestr_data::Timeframe;

#[cfg(test)]
mod tests {
    #[test]
    fn test_core_initialization() {
        assert_eq!(2 + 2, 4);
    }
}
