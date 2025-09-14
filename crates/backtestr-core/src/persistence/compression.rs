//! Compression utilities for checkpoint data using ZSTD

use anyhow::{Context, Result};

pub fn compress_data(data: &[u8], level: i32) -> Result<Vec<u8>> {
    // ZSTD compression levels: 1-22 (default is 3)
    let compression_level = match level {
        0 => 1,      // Minimum compression
        1..=9 => level, // Map 1-9 directly
        10..=22 => level, // ZSTD supports higher levels
        _ => 3,      // Default ZSTD level
    };

    zstd::encode_all(data, compression_level)
        .context("Failed to compress data with ZSTD")
}

pub fn decompress_data(compressed: &[u8]) -> Result<Vec<u8>> {
    zstd::decode_all(compressed)
        .context("Failed to decompress data with ZSTD")
}

pub fn estimate_compression_ratio(original_size: usize, compressed_size: usize) -> f64 {
    if original_size == 0 {
        return 0.0;
    }
    1.0 - (compressed_size as f64 / original_size as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress_roundtrip() {
        let original = b"Hello, this is test data for compression!".repeat(100);

        let compressed = compress_data(&original, 6).unwrap();
        assert!(compressed.len() < original.len());

        let decompressed = decompress_data(&compressed).unwrap();
        assert_eq!(original, decompressed.as_slice());
    }

    #[test]
    fn test_compression_levels() {
        let data = b"Test data".repeat(1000);

        let fast = compress_data(&data, 1).unwrap();
        let best = compress_data(&data, 9).unwrap();

        // Higher compression should produce smaller output
        assert!(best.len() <= fast.len());
    }

    #[test]
    fn test_compression_ratio() {
        let ratio = estimate_compression_ratio(1000, 250);
        assert_eq!(ratio, 0.75);

        let ratio_zero = estimate_compression_ratio(0, 100);
        assert_eq!(ratio_zero, 0.0);
    }
}
