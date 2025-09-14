use backtestr_data::models::Bar;

pub struct VolumeAggregator {
    volume_weighted: bool,
}

impl Default for VolumeAggregator {
    fn default() -> Self {
        Self {
            volume_weighted: false,
        }
    }
}

impl VolumeAggregator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_volume_weighting(mut self, enabled: bool) -> Self {
        self.volume_weighted = enabled;
        self
    }

    pub fn aggregate_volume(&self, bars: &[Bar]) -> Option<i64> {
        if bars.is_empty() {
            return None;
        }

        let total_volume: i64 = bars
            .iter()
            .filter_map(|b| b.volume)
            .sum();

        if total_volume > 0 {
            Some(total_volume)
        } else {
            None
        }
    }

    pub fn aggregate_tick_count(&self, bars: &[Bar]) -> Option<i32> {
        if bars.is_empty() {
            return None;
        }

        let total_ticks: i32 = bars
            .iter()
            .filter_map(|b| b.tick_count)
            .sum();

        if total_ticks > 0 {
            Some(total_ticks)
        } else {
            None
        }
    }

    pub fn calculate_vwap(&self, bars: &[Bar]) -> Option<f64> {
        if bars.is_empty() {
            return None;
        }

        let mut total_volume = 0i64;
        let mut volume_weighted_sum = 0.0;

        for bar in bars {
            if let Some(volume) = bar.volume {
                let typical_price = (bar.high + bar.low + bar.close) / 3.0;
                volume_weighted_sum += typical_price * volume as f64;
                total_volume += volume;
            }
        }

        if total_volume > 0 {
            Some(volume_weighted_sum / total_volume as f64)
        } else {
            None
        }
    }

    pub fn calculate_volume_weighted_price(&self, bars: &[Bar], price_type: PriceType) -> Option<f64> {
        if bars.is_empty() || !self.volume_weighted {
            return None;
        }

        let mut total_volume = 0i64;
        let mut weighted_sum = 0.0;

        for bar in bars {
            if let Some(volume) = bar.volume {
                let price = match price_type {
                    PriceType::Open => bar.open,
                    PriceType::High => bar.high,
                    PriceType::Low => bar.low,
                    PriceType::Close => bar.close,
                    PriceType::Typical => (bar.high + bar.low + bar.close) / 3.0,
                    PriceType::Weighted => (bar.high + bar.low + bar.close * 2.0) / 4.0,
                    PriceType::Median => (bar.high + bar.low) / 2.0,
                };

                weighted_sum += price * volume as f64;
                total_volume += volume;
            }
        }

        if total_volume > 0 {
            Some(weighted_sum / total_volume as f64)
        } else {
            None
        }
    }

    pub fn calculate_average_bar_volume(&self, bars: &[Bar]) -> Option<f64> {
        if bars.is_empty() {
            return None;
        }

        let volumes: Vec<i64> = bars
            .iter()
            .filter_map(|b| b.volume)
            .collect();

        if volumes.is_empty() {
            return None;
        }

        let sum: i64 = volumes.iter().sum();
        Some(sum as f64 / volumes.len() as f64)
    }

    pub fn calculate_volume_profile(&self, bars: &[Bar], num_levels: usize) -> VolumeProfile {
        let mut profile = VolumeProfile::new(num_levels);

        if bars.is_empty() {
            return profile;
        }

        // Find price range
        let mut min_price = f64::MAX;
        let mut max_price = f64::MIN;

        for bar in bars {
            min_price = min_price.min(bar.low);
            max_price = max_price.max(bar.high);
        }

        if min_price >= max_price {
            return profile;
        }

        profile.min_price = min_price;
        profile.max_price = max_price;

        let price_step = (max_price - min_price) / num_levels as f64;

        // Initialize levels
        for i in 0..num_levels {
            let level_price = min_price + (i as f64 + 0.5) * price_step;
            profile.levels.push(VolumeLevel {
                price: level_price,
                volume: 0,
                tick_count: 0,
            });
        }

        // Distribute volume across levels
        for bar in bars {
            let volume = bar.volume.unwrap_or(0);
            let ticks = bar.tick_count.unwrap_or(0);

            // Find which levels this bar touches
            let low_level = ((bar.low - min_price) / price_step).floor() as usize;
            let high_level = ((bar.high - min_price) / price_step).ceil() as usize;

            let num_touched_levels = (high_level - low_level + 1).max(1);
            let volume_per_level = volume / num_touched_levels as i64;
            let ticks_per_level = ticks / num_touched_levels as i32;

            for level_idx in low_level..=high_level.min(num_levels - 1) {
                if level_idx < profile.levels.len() {
                    profile.levels[level_idx].volume += volume_per_level;
                    profile.levels[level_idx].tick_count += ticks_per_level;
                }
            }
        }

        // Find POC (Point of Control)
        let mut max_volume = 0i64;
        let mut poc_price = 0.0;

        for level in &profile.levels {
            if level.volume > max_volume {
                max_volume = level.volume;
                poc_price = level.price;
            }
        }

        profile.poc = poc_price;
        profile
    }

    pub fn is_high_volume_bar(&self, bar: &Bar, average_volume: f64) -> bool {
        if let Some(volume) = bar.volume {
            volume as f64 > average_volume * 1.5
        } else {
            false
        }
    }

    pub fn is_low_volume_bar(&self, bar: &Bar, average_volume: f64) -> bool {
        if let Some(volume) = bar.volume {
            (volume as f64) < average_volume * 0.5
        } else {
            true
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PriceType {
    Open,
    High,
    Low,
    Close,
    Typical,
    Weighted,
    Median,
}

#[derive(Debug, Clone)]
pub struct VolumeLevel {
    pub price: f64,
    pub volume: i64,
    pub tick_count: i32,
}

#[derive(Debug, Clone)]
pub struct VolumeProfile {
    pub levels: Vec<VolumeLevel>,
    pub poc: f64, // Point of Control (price with highest volume)
    pub min_price: f64,
    pub max_price: f64,
}

impl VolumeProfile {
    pub fn new(num_levels: usize) -> Self {
        Self {
            levels: Vec::with_capacity(num_levels),
            poc: 0.0,
            min_price: 0.0,
            max_price: 0.0,
        }
    }

    pub fn get_value_area(&self, percentage: f64) -> (f64, f64) {
        if self.levels.is_empty() {
            return (0.0, 0.0);
        }

        let total_volume: i64 = self.levels.iter().map(|l| l.volume).sum();
        let target_volume = (total_volume as f64 * percentage / 100.0) as i64;

        // Sort levels by volume
        let mut sorted_levels = self.levels.clone();
        sorted_levels.sort_by(|a, b| b.volume.cmp(&a.volume));

        let mut accumulated_volume = 0i64;
        let mut value_area_levels = Vec::new();

        for level in sorted_levels {
            accumulated_volume += level.volume;
            value_area_levels.push(level.price);

            if accumulated_volume >= target_volume {
                break;
            }
        }

        if value_area_levels.is_empty() {
            return (0.0, 0.0);
        }

        let min_va = value_area_levels.iter().copied().fold(f64::INFINITY, f64::min);
        let max_va = value_area_levels.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        (min_va, max_va)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use backtestr_data::timeframe::Timeframe;

    fn create_test_bar_with_volume(symbol: &str, volume: i64, tick_count: i32) -> Bar {
        Bar::new(
            symbol.to_string(),
            Timeframe::M1,
            1704067200000,
            1704067260000,
            1.0920,
            1.0925,
            1.0915,
            1.0922,
        )
        .with_volume(volume)
        .with_tick_count(tick_count)
    }

    #[test]
    fn test_aggregate_volume() {
        let aggregator = VolumeAggregator::new();

        let bars = vec![
            create_test_bar_with_volume("EURUSD", 1000, 50),
            create_test_bar_with_volume("EURUSD", 1500, 75),
            create_test_bar_with_volume("EURUSD", 2000, 100),
        ];

        assert_eq!(aggregator.aggregate_volume(&bars), Some(4500));
        assert_eq!(aggregator.aggregate_tick_count(&bars), Some(225));
    }

    #[test]
    fn test_vwap_calculation() {
        let aggregator = VolumeAggregator::new();

        let mut bars = Vec::new();
        bars.push(
            Bar::new(
                "EURUSD".to_string(),
                Timeframe::M1,
                1704067200000,
                1704067260000,
                1.0920,
                1.0930,
                1.0910,
                1.0925,
            )
            .with_volume(1000),
        );
        bars.push(
            Bar::new(
                "EURUSD".to_string(),
                Timeframe::M1,
                1704067260000,
                1704067320000,
                1.0925,
                1.0935,
                1.0920,
                1.0930,
            )
            .with_volume(1500),
        );

        let vwap = aggregator.calculate_vwap(&bars);
        assert!(vwap.is_some());

        // Verify VWAP is within expected range
        let vwap_value = vwap.unwrap();
        assert!(vwap_value > 1.0920 && vwap_value < 1.0935);
    }

    #[test]
    fn test_average_volume() {
        let aggregator = VolumeAggregator::new();

        let bars = vec![
            create_test_bar_with_volume("EURUSD", 1000, 50),
            create_test_bar_with_volume("EURUSD", 2000, 100),
            create_test_bar_with_volume("EURUSD", 3000, 150),
        ];

        let avg = aggregator.calculate_average_bar_volume(&bars);
        assert_eq!(avg, Some(2000.0));
    }

    #[test]
    fn test_volume_classification() {
        let aggregator = VolumeAggregator::new();
        let average_volume = 1000.0;

        let high_volume_bar = create_test_bar_with_volume("EURUSD", 2000, 100);
        let low_volume_bar = create_test_bar_with_volume("EURUSD", 400, 20);
        let normal_volume_bar = create_test_bar_with_volume("EURUSD", 1000, 50);

        assert!(aggregator.is_high_volume_bar(&high_volume_bar, average_volume));
        assert!(aggregator.is_low_volume_bar(&low_volume_bar, average_volume));
        assert!(!aggregator.is_high_volume_bar(&normal_volume_bar, average_volume));
        assert!(!aggregator.is_low_volume_bar(&normal_volume_bar, average_volume));
    }

    #[test]
    fn test_volume_profile() {
        let aggregator = VolumeAggregator::new();

        let bars = vec![
            Bar::new(
                "EURUSD".to_string(),
                Timeframe::M1,
                1704067200000,
                1704067260000,
                1.0920,
                1.0925,
                1.0915,
                1.0922,
            )
            .with_volume(1000),
            Bar::new(
                "EURUSD".to_string(),
                Timeframe::M1,
                1704067260000,
                1704067320000,
                1.0922,
                1.0930,
                1.0920,
                1.0928,
            )
            .with_volume(2000),
            Bar::new(
                "EURUSD".to_string(),
                Timeframe::M1,
                1704067320000,
                1704067380000,
                1.0928,
                1.0935,
                1.0925,
                1.0932,
            )
            .with_volume(1500),
        ];

        let profile = aggregator.calculate_volume_profile(&bars, 10);

        assert_eq!(profile.levels.len(), 10);
        assert!(profile.poc > 0.0);
        assert_eq!(profile.min_price, 1.0915);
        assert_eq!(profile.max_price, 1.0935);

        // Test value area calculation
        let (va_low, va_high) = profile.get_value_area(70.0);
        assert!(va_low < va_high);
        assert!(va_low >= profile.min_price);
        assert!(va_high <= profile.max_price);
    }
}