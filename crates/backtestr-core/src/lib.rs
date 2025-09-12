#![doc = include_str!("../../../README.md")]

pub mod data;
pub mod engine;
pub mod indicators;
pub mod positions;
pub mod python;

pub use engine::MTFEngine;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_initialization() {
        assert_eq!(2 + 2, 4);
    }
}
