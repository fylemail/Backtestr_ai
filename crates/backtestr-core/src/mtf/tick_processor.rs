use backtestr_data::{Bar, Tick};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct TickProcessor {
    ticks_processed: u64,
    bars_completed: u64,
    last_processing_time_us: u64,
}

impl TickProcessor {
    pub fn new() -> Self {
        Self {
            ticks_processed: 0,
            bars_completed: 0,
            last_processing_time_us: 0,
        }
    }

    pub fn process(
        &mut self,
        tick: &Tick,
        process_fn: impl FnOnce(&Tick) -> Result<Vec<Bar>, String>,
    ) -> Result<ProcessingResult, String> {
        let start = Instant::now();

        // Execute the processing function
        let completed_bars = process_fn(tick)?;

        let elapsed_us = start.elapsed().as_micros() as u64;

        self.ticks_processed += 1;
        self.bars_completed += completed_bars.len() as u64;
        self.last_processing_time_us = elapsed_us;

        Ok(ProcessingResult {
            tick_timestamp: tick.timestamp,
            completed_bars,
            processing_time_us: elapsed_us,
            total_ticks_processed: self.ticks_processed,
            total_bars_completed: self.bars_completed,
        })
    }

    pub fn get_stats(&self) -> ProcessorStats {
        ProcessorStats {
            ticks_processed: self.ticks_processed,
            bars_completed: self.bars_completed,
            last_processing_time_us: self.last_processing_time_us,
            average_processing_time_us: if self.ticks_processed > 0 {
                self.last_processing_time_us // Would need rolling average in production
            } else {
                0
            },
        }
    }

    pub fn reset_stats(&mut self) {
        self.ticks_processed = 0;
        self.bars_completed = 0;
        self.last_processing_time_us = 0;
    }
}

impl Default for TickProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub tick_timestamp: i64,
    pub completed_bars: Vec<Bar>,
    pub processing_time_us: u64,
    pub total_ticks_processed: u64,
    pub total_bars_completed: u64,
}

#[derive(Debug, Clone)]
pub struct ProcessorStats {
    pub ticks_processed: u64,
    pub bars_completed: u64,
    pub last_processing_time_us: u64,
    pub average_processing_time_us: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use backtestr_data::Timeframe;

    #[test]
    fn test_tick_processor_creation() {
        let processor = TickProcessor::new();
        let stats = processor.get_stats();
        assert_eq!(stats.ticks_processed, 0);
        assert_eq!(stats.bars_completed, 0);
    }

    #[test]
    fn test_tick_processing() {
        let mut processor = TickProcessor::new();
        let tick = Tick::new_with_millis("EURUSD".to_string(), 1704067230000, 1.0920, 1.0922);

        let result = processor.process(&tick, |_t| Ok(vec![]));
        assert!(result.is_ok());

        let res = result.unwrap();
        assert_eq!(res.tick_timestamp, 1704067230000);
        assert_eq!(res.completed_bars.len(), 0);
        assert_eq!(res.total_ticks_processed, 1);

        let stats = processor.get_stats();
        assert_eq!(stats.ticks_processed, 1);
        assert_eq!(stats.bars_completed, 0);
    }

    #[test]
    fn test_tick_processing_with_bars() {
        let mut processor = TickProcessor::new();
        let tick = Tick::new_with_millis("EURUSD".to_string(), 1704067230000, 1.0920, 1.0922);

        // Simulate processing that completes a bar
        let bar = Bar::new(
            "EURUSD".to_string(),
            Timeframe::M1,
            1704067200000,
            1704067260000,
            1.0920,
            1.0925,
            1.0918,
            1.0923,
        );

        let result = processor.process(&tick, |_t| Ok(vec![bar.clone()]));
        assert!(result.is_ok());

        let res = result.unwrap();
        assert_eq!(res.completed_bars.len(), 1);
        assert_eq!(res.total_bars_completed, 1);

        let stats = processor.get_stats();
        assert_eq!(stats.bars_completed, 1);
    }

    #[test]
    fn test_processing_error_handling() {
        let mut processor = TickProcessor::new();
        let tick = Tick::new_with_millis("EURUSD".to_string(), 1704067230000, 1.0920, 1.0922);

        let result = processor.process(&tick, |_t| Err("Test error".to_string()));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Test error");
    }

    #[test]
    fn test_reset_stats() {
        let mut processor = TickProcessor::new();
        let tick = Tick::new_with_millis("EURUSD".to_string(), 1704067230000, 1.0920, 1.0922);

        processor.process(&tick, |_t| Ok(vec![])).unwrap();
        assert_eq!(processor.get_stats().ticks_processed, 1);

        processor.reset_stats();
        assert_eq!(processor.get_stats().ticks_processed, 0);
        assert_eq!(processor.get_stats().bars_completed, 0);
    }
}
