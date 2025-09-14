//! Validation and checksum utilities for checkpoint integrity

use std::hash::Hasher;
use twox_hash::XxHash64;

pub fn calculate_checksum(data: &[u8]) -> u64 {
    let mut hasher = XxHash64::with_seed(0);
    hasher.write(data);
    hasher.finish()
}

pub fn validate_checksum(data: &[u8], expected: u64) -> bool {
    calculate_checksum(data) == expected
}

pub struct ChecksumValidator {
    hasher: XxHash64,
}

impl ChecksumValidator {
    pub fn new() -> Self {
        Self {
            hasher: XxHash64::with_seed(0),
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.hasher.write(data);
    }

    pub fn finalize(self) -> u64 {
        self.hasher.finish()
    }

    pub fn verify(data: &[u8], expected: u64) -> bool {
        validate_checksum(data, expected)
    }
}

impl Default for ChecksumValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_calculation() {
        let data = b"test data for checksum";
        let checksum1 = calculate_checksum(data);
        let checksum2 = calculate_checksum(data);

        assert_eq!(checksum1, checksum2);
    }

    #[test]
    fn test_checksum_validation() {
        let data = b"test data";
        let checksum = calculate_checksum(data);

        assert!(validate_checksum(data, checksum));
        assert!(!validate_checksum(data, checksum + 1));
    }

    #[test]
    fn test_checksum_validator() {
        let data1 = b"part1";
        let data2 = b"part2";

        let mut validator = ChecksumValidator::new();
        validator.update(data1);
        validator.update(data2);
        let checksum = validator.finalize();

        let combined = [data1.as_ref(), data2.as_ref()].concat();
        let expected = calculate_checksum(&combined);

        assert_eq!(checksum, expected);
    }

    #[test]
    fn test_different_data_different_checksums() {
        let data1 = b"data1";
        let data2 = b"data2";

        let checksum1 = calculate_checksum(data1);
        let checksum2 = calculate_checksum(data2);

        assert_ne!(checksum1, checksum2);
    }
}
