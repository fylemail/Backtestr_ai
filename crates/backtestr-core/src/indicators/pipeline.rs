//! Indicator pipeline for managing and executing multiple indicators efficiently.
//!
//! This module provides a high-performance pipeline that can process multiple
//! indicators in parallel when beneficial, with automatic caching of results.

use anyhow::Result;
use dashmap::DashMap;
use rayon::prelude::*;
use std::sync::Arc;
use std::time::Instant;
use tracing::debug;

use backtestr_data::Timeframe;

use super::cache::IndicatorCache;
use super::indicator_trait::{BarData, Indicator, IndicatorDefaults, IndicatorValue};

/// High-performance pipeline for managing multiple technical indicators.
///
/// The pipeline automatically chooses between sequential and parallel processing
/// based on the number of indicators, caches results per timeframe, and provides
/// thread-safe access to all indicators.
///
/// # Performance Characteristics
///
/// - Sequential processing for <5 indicators (lower overhead)
/// - Parallel processing via Rayon for 5+ indicators
/// - Thread-safe indicator storage using DashMap
/// - Per-timeframe caching for efficient retrieval
///
/// # Examples
///
/// ```
/// use backtestr_core::indicators::{IndicatorPipeline, SMA, BarData};
/// use backtestr_core::Timeframe;
///
/// let mut pipeline = IndicatorPipeline::new(1000);
/// pipeline.register_indicator("SMA_20".to_string(), Box::new(SMA::new(20)));
///
/// let bar = BarData {
///     open: 100.0,
///     high: 102.0,
///     low: 99.0,
///     close: 101.0,
///     volume: 10000.0,
///     timestamp: 1234567890,
/// };
/// let result = pipeline.update_all(&bar, Timeframe::M1).unwrap();
/// ```
pub struct IndicatorPipeline {
    indicators: Arc<DashMap<String, Box<dyn Indicator<Input = BarData, Output = f64>>>>,
    cache: IndicatorCache,
    defaults: IndicatorDefaults,
    parallel_threshold: usize,
}

impl IndicatorPipeline {
    pub fn new(cache_size: usize) -> Self {
        Self {
            indicators: Arc::new(DashMap::new()),
            cache: IndicatorCache::new(cache_size),
            defaults: IndicatorDefaults::default(),
            parallel_threshold: 5, // Use parallel processing if more than 5 indicators
        }
    }

    pub fn with_defaults(cache_size: usize, defaults: IndicatorDefaults) -> Self {
        Self {
            indicators: Arc::new(DashMap::new()),
            cache: IndicatorCache::new(cache_size),
            defaults,
            parallel_threshold: 5,
        }
    }

    pub fn register_indicator(&self, name: String, indicator: Box<dyn Indicator<Input = BarData, Output = f64>>) {
        debug!("Registering indicator: {}", name);
        self.indicators.insert(name, indicator);
    }

    pub fn update_all(&self, bar: &BarData, timeframe: Timeframe) -> Result<UpdateResult> {
        let start = Instant::now();
        let indicator_count = self.indicators.len();

        if indicator_count == 0 {
            return Ok(UpdateResult {
                updated_count: 0,
                failed_count: 0,
                duration_micros: start.elapsed().as_micros() as u64,
            });
        }

        let results = if indicator_count > self.parallel_threshold {
            self.update_parallel(bar, timeframe)
        } else {
            self.update_sequential(bar, timeframe)
        };

        let (updated_count, failed_count) = results;

        Ok(UpdateResult {
            updated_count,
            failed_count,
            duration_micros: start.elapsed().as_micros() as u64,
        })
    }

    fn update_sequential(&self, bar: &BarData, timeframe: Timeframe) -> (usize, usize) {
        let mut updated = 0;
        let mut failed = 0;

        for mut entry in self.indicators.iter_mut() {
            let (name, indicator) = entry.pair_mut();

            if let Some(value) = indicator.update(bar.clone()) {
                let indicator_value = IndicatorValue {
                    value,
                    timestamp: bar.timestamp,
                };
                self.cache.insert(name.clone(), timeframe, indicator_value);
                updated += 1;
            } else {
                failed += 1;
            }
        }

        (updated, failed)
    }

    fn update_parallel(&self, bar: &BarData, timeframe: Timeframe) -> (usize, usize) {
        let results: Vec<(String, Option<f64>)> = self
            .indicators
            .iter_mut()
            .par_bridge()
            .map(|mut entry| {
                let (name, indicator) = entry.pair_mut();
                let result = indicator.update(bar.clone());
                (name.clone(), result)
            })
            .collect();

        let mut updated = 0;
        let mut failed = 0;

        for (name, result) in results {
            if let Some(value) = result {
                let indicator_value = IndicatorValue {
                    value,
                    timestamp: bar.timestamp,
                };
                self.cache.insert(name, timeframe, indicator_value);
                updated += 1;
            } else {
                failed += 1;
            }
        }

        (updated, failed)
    }

    pub fn get_value(&self, indicator_name: &str, timeframe: Timeframe) -> Option<f64> {
        self.cache.get(indicator_name, timeframe).map(|v| v.value)
    }

    pub fn get_indicator_value(&self, indicator_name: &str, timeframe: Timeframe) -> Option<IndicatorValue> {
        self.cache.get(indicator_name, timeframe)
    }

    pub fn get_history(&self, indicator_name: &str, timeframe: Timeframe, count: usize) -> Vec<IndicatorValue> {
        self.cache.get_history(indicator_name, timeframe, count)
    }

    pub fn reset_indicator(&self, indicator_name: &str) {
        if let Some(mut indicator) = self.indicators.get_mut(indicator_name) {
            indicator.reset();
            self.cache.clear_indicator(indicator_name);
        }
    }

    pub fn reset_all(&self) {
        for mut entry in self.indicators.iter_mut() {
            entry.value_mut().reset();
        }
        self.cache.clear();
    }

    pub fn remove_indicator(&self, indicator_name: &str) -> bool {
        self.cache.clear_indicator(indicator_name);
        self.indicators.remove(indicator_name).is_some()
    }

    pub fn get_indicator_names(&self) -> Vec<String> {
        self.indicators.iter().map(|entry| entry.key().clone()).collect()
    }

    pub fn get_stats(&self) -> PipelineStats {
        PipelineStats {
            total_indicators: self.indicators.len(),
            cache_stats: self.cache.get_stats(),
        }
    }

    pub fn set_parallel_threshold(&mut self, threshold: usize) {
        self.parallel_threshold = threshold;
    }
}

#[derive(Debug, Clone)]
pub struct UpdateResult {
    pub updated_count: usize,
    pub failed_count: usize,
    pub duration_micros: u64,
}

#[derive(Debug, Clone)]
pub struct PipelineStats {
    pub total_indicators: usize,
    pub cache_stats: super::cache::CacheStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MockIndicator {
        name: String,
        value: f64,
    }

    impl Indicator for MockIndicator {
        type Input = BarData;
        type Output = f64;

        fn name(&self) -> &str {
            &self.name
        }

        fn warm_up_period(&self) -> usize {
            1
        }

        fn update(&mut self, _input: BarData) -> Option<f64> {
            self.value += 1.0;
            Some(self.value)
        }

        fn current(&self) -> Option<f64> {
            Some(self.value)
        }

        fn reset(&mut self) {
            self.value = 0.0;
        }
    }

    #[test]
    fn test_pipeline_update() {
        let pipeline = IndicatorPipeline::new(100);
        let mock_indicator = Box::new(MockIndicator {
            name: "TEST".to_string(),
            value: 0.0,
        });

        pipeline.register_indicator("TEST".to_string(), mock_indicator);

        let bar = BarData {
            open: 100.0,
            high: 101.0,
            low: 99.0,
            close: 100.5,
            volume: 1000.0,
            timestamp: 1000,
        };

        let result = pipeline.update_all(&bar, Timeframe::M1).unwrap();
        assert_eq!(result.updated_count, 1);
        assert_eq!(result.failed_count, 0);

        let value = pipeline.get_value("TEST", Timeframe::M1);
        assert_eq!(value, Some(1.0));
    }
}