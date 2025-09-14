//! Thread-safe caching layer for indicator values.
//!
//! This module provides a high-performance, lock-free cache for storing
//! and retrieving indicator values across multiple timeframes.

use super::indicator_trait::IndicatorValue;
use backtestr_data::Timeframe;
use dashmap::DashMap;
use std::collections::VecDeque;
use std::sync::Arc;

/// Thread-safe cache for storing indicator values with history.
///
/// Uses DashMap for lock-free concurrent access and VecDeque for
/// efficient history management.
///
/// # Examples
///
/// ```
/// use backtestr_core::indicators::{IndicatorCache, IndicatorValue};
/// use backtestr_core::Timeframe;
///
/// let cache = IndicatorCache::new(100);
/// cache.insert("RSI".to_string(), Timeframe::M1, IndicatorValue {
///     value: 65.5,
///     timestamp: 1234567890,
/// });
/// ```
#[derive(Debug, Clone)]
pub struct IndicatorCache {
    values: Arc<DashMap<(String, Timeframe), VecDeque<IndicatorValue>>>,
    max_history: usize,
}

impl IndicatorCache {
    /// Creates a new indicator cache with specified history limit.
    ///
    /// # Arguments
    ///
    /// * `max_history` - Maximum number of historical values to store per indicator/timeframe
    pub fn new(max_history: usize) -> Self {
        Self {
            values: Arc::new(DashMap::new()),
            max_history,
        }
    }

    /// Inserts a new indicator value into the cache.
    ///
    /// Automatically maintains history limit by removing oldest values.
    ///
    /// # Arguments
    ///
    /// * `indicator_name` - Name of the indicator
    /// * `timeframe` - Timeframe context
    /// * `value` - The indicator value to cache
    pub fn insert(&self, indicator_name: String, timeframe: Timeframe, value: IndicatorValue) {
        let mut entry = self.values.entry((indicator_name, timeframe)).or_default();

        entry.push_back(value);

        if entry.len() > self.max_history {
            entry.pop_front();
        }
    }

    /// Gets the most recent value for an indicator.
    ///
    /// # Arguments
    ///
    /// * `indicator_name` - Name of the indicator
    /// * `timeframe` - Timeframe to query
    ///
    /// # Returns
    ///
    /// The most recent indicator value, or `None` if not found.
    pub fn get(&self, indicator_name: &str, timeframe: Timeframe) -> Option<IndicatorValue> {
        self.values
            .get(&(indicator_name.to_string(), timeframe))
            .and_then(|values| values.back().copied())
    }

    /// Retrieves historical values for an indicator.
    ///
    /// # Arguments
    ///
    /// * `indicator_name` - Name of the indicator
    /// * `timeframe` - Timeframe to query
    /// * `count` - Maximum number of values to retrieve
    ///
    /// # Returns
    ///
    /// Vector of historical values in chronological order (oldest first).
    pub fn get_history(
        &self,
        indicator_name: &str,
        timeframe: Timeframe,
        count: usize,
    ) -> Vec<IndicatorValue> {
        self.values
            .get(&(indicator_name.to_string(), timeframe))
            .map(|values| {
                let len = values.len();
                let start = len.saturating_sub(count);
                values.range(start..).copied().collect()
            })
            .unwrap_or_default()
    }

    /// Clears all cached values.
    pub fn clear(&self) {
        self.values.clear();
    }

    /// Clears all cached values for a specific indicator.
    ///
    /// # Arguments
    ///
    /// * `indicator_name` - Name of the indicator to clear
    pub fn clear_indicator(&self, indicator_name: &str) {
        let keys_to_remove: Vec<_> = self
            .values
            .iter()
            .filter(|entry| entry.key().0 == indicator_name)
            .map(|entry| entry.key().clone())
            .collect();

        for key in keys_to_remove {
            self.values.remove(&key);
        }
    }

    pub fn clear_timeframe(&self, timeframe: Timeframe) {
        let keys_to_remove: Vec<_> = self
            .values
            .iter()
            .filter(|entry| entry.key().1 == timeframe)
            .map(|entry| entry.key().clone())
            .collect();

        for key in keys_to_remove {
            self.values.remove(&key);
        }
    }

    /// Gets cache statistics.
    ///
    /// # Returns
    ///
    /// Statistics about cache usage and performance.
    pub fn get_stats(&self) -> CacheStats {
        let total_entries = self.values.len();
        let total_values: usize = self.values.iter().map(|entry| entry.value().len()).sum();

        CacheStats {
            total_indicators: total_entries,
            total_values,
            max_history: self.max_history,
        }
    }
}

/// Statistics about cache usage and performance.
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_indicators: usize,
    pub total_values: usize,
    pub max_history: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_insert_and_get() {
        let cache = IndicatorCache::new(100);
        let value = IndicatorValue {
            value: 50.0,
            timestamp: 1000,
        };

        cache.insert("RSI".to_string(), Timeframe::M1, value);
        let retrieved = cache.get("RSI", Timeframe::M1);

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, 50.0);
    }

    #[test]
    fn test_cache_max_history() {
        let cache = IndicatorCache::new(3);

        for i in 0..5 {
            let value = IndicatorValue {
                value: i as f64,
                timestamp: i as i64,
            };
            cache.insert("SMA".to_string(), Timeframe::M5, value);
        }

        let history = cache.get_history("SMA", Timeframe::M5, 10);
        assert_eq!(history.len(), 3); // Should only have last 3 values
        assert_eq!(history[0].value, 2.0);
        assert_eq!(history[1].value, 3.0);
        assert_eq!(history[2].value, 4.0);
    }
}
