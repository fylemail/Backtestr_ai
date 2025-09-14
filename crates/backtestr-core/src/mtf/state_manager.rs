use crate::mtf::{TickProcessor, TimeframeState};
use backtestr_data::{Bar, Tick, Timeframe};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

const DEFAULT_BAR_HISTORY: usize = 1000;
const MAX_SYMBOLS: usize = 10;
const MAX_MEMORY_MB: usize = 1000;

#[derive(Debug, Clone)]
pub struct MTFConfig {
    pub bar_history_limit: usize,
    pub max_symbols: usize,
    pub max_memory_mb: usize,
    pub enabled_timeframes: Vec<Timeframe>,
}

impl Default for MTFConfig {
    fn default() -> Self {
        Self {
            bar_history_limit: DEFAULT_BAR_HISTORY,
            max_symbols: MAX_SYMBOLS,
            max_memory_mb: MAX_MEMORY_MB,
            enabled_timeframes: Timeframe::all(),
        }
    }
}

#[derive(Clone)]
pub struct MTFStateManager {
    states: Arc<RwLock<HashMap<String, SymbolMTFState>>>,
    config: MTFConfig,
    #[allow(dead_code)]
    tick_processor: TickProcessor,
}

impl MTFStateManager {
    pub fn new(config: MTFConfig) -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
            config,
            tick_processor: TickProcessor::new(),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(MTFConfig::default())
    }

    pub fn process_tick(&self, tick: &Tick) -> Result<Vec<Bar>, String> {
        // Validate symbol count
        {
            let states = self
                .states
                .read()
                .map_err(|e| format!("Lock error: {}", e))?;
            if !states.contains_key(&tick.symbol) && states.len() >= self.config.max_symbols {
                return Err(format!(
                    "Maximum symbols ({}) reached. Cannot add {}",
                    self.config.max_symbols, tick.symbol
                ));
            }
        }

        // Process tick atomically
        let mut states = self
            .states
            .write()
            .map_err(|e| format!("Lock error: {}", e))?;

        let symbol_state = states.entry(tick.symbol.clone()).or_insert_with(|| {
            SymbolMTFState::new(
                tick.symbol.clone(),
                &self.config.enabled_timeframes,
                self.config.bar_history_limit,
            )
        });

        // Use mid-price for bar aggregation
        let price = (tick.bid + tick.ask) / 2.0;
        let volume = tick.bid_size.unwrap_or(0) + tick.ask_size.unwrap_or(0);

        symbol_state.process_tick(tick.timestamp, price, volume)
    }

    pub fn get_symbol_state(&self, symbol: &str) -> Option<SymbolMTFState> {
        self.states
            .read()
            .ok()
            .and_then(|states| states.get(symbol).cloned())
    }

    pub fn get_all_symbols(&self) -> Vec<String> {
        self.states
            .read()
            .ok()
            .map(|states| states.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn clear_symbol(&self, symbol: &str) -> Result<(), String> {
        let mut states = self
            .states
            .write()
            .map_err(|e| format!("Lock error: {}", e))?;
        states.remove(symbol);
        Ok(())
    }

    pub fn clear_all(&self) -> Result<(), String> {
        let mut states = self
            .states
            .write()
            .map_err(|e| format!("Lock error: {}", e))?;
        states.clear();
        Ok(())
    }

    pub fn get_memory_usage_estimate(&self) -> usize {
        let states = match self.states.read() {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let mut total_bytes = 0;

        for (symbol, state) in states.iter() {
            // Estimate symbol string memory
            total_bytes += symbol.len() + 24; // String overhead

            // Estimate per-timeframe memory
            for tf_state in state.timeframes.values() {
                // Each bar is approximately 100 bytes
                total_bytes += tf_state.completed_bars.len() * 100;
                // Partial bar
                total_bytes += 100;
                // Overhead
                total_bytes += 64;
            }

            // State overhead
            total_bytes += 128;
        }

        total_bytes
    }
}

#[derive(Debug, Clone)]
pub struct SymbolMTFState {
    pub symbol: String,
    pub current_tick: Option<Tick>,
    pub timeframes: HashMap<Timeframe, TimeframeState>,
    pub last_update: i64,
}

impl SymbolMTFState {
    pub fn new(symbol: String, timeframes: &[Timeframe], history_limit: usize) -> Self {
        let mut tf_states = HashMap::new();
        for &tf in timeframes {
            tf_states.insert(tf, TimeframeState::with_history_limit(tf, history_limit));
        }

        Self {
            symbol,
            current_tick: None,
            timeframes: tf_states,
            last_update: 0,
        }
    }

    pub fn process_tick(
        &mut self,
        timestamp: i64,
        price: f64,
        volume: i64,
    ) -> Result<Vec<Bar>, String> {
        // Update last tick
        self.current_tick = Some(Tick::new_with_millis(
            self.symbol.clone(),
            timestamp,
            price,
            price,
        ));
        self.last_update = timestamp;

        // Process tick for all timeframes atomically
        let mut completed_bars = Vec::new();

        for tf_state in self.timeframes.values_mut() {
            if let Some(bar) = tf_state.process_tick(&self.symbol, timestamp, price, volume) {
                completed_bars.push(bar);
            }
        }

        Ok(completed_bars)
    }

    pub fn get_timeframe_state(&self, timeframe: Timeframe) -> Option<&TimeframeState> {
        self.timeframes.get(&timeframe)
    }

    pub fn get_all_partial_bars(&self) -> HashMap<Timeframe, Option<crate::mtf::PartialBar>> {
        self.timeframes
            .iter()
            .map(|(&tf, state)| (tf, state.current_bar.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mtf_manager_creation() {
        let manager = MTFStateManager::with_default_config();
        assert_eq!(manager.get_all_symbols().len(), 0);
    }

    #[test]
    fn test_process_tick_creates_state() {
        let manager = MTFStateManager::with_default_config();
        let tick = Tick::new_with_millis("EURUSD".to_string(), 1704067230000, 1.0920, 1.0922);

        let result = manager.process_tick(&tick);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0); // No bars completed yet

        assert_eq!(manager.get_all_symbols().len(), 1);
        assert!(manager.get_symbol_state("EURUSD").is_some());
    }

    #[test]
    fn test_process_tick_completes_bars() {
        let manager = MTFStateManager::with_default_config();

        // First tick
        let tick1 = Tick::new_with_millis("EURUSD".to_string(), 1704067230000, 1.0920, 1.0922);
        manager.process_tick(&tick1).unwrap();

        // Tick in next minute - should complete M1 bar
        let tick2 = Tick::new_with_millis("EURUSD".to_string(), 1704067290000, 1.0925, 1.0927);
        let completed = manager.process_tick(&tick2).unwrap();

        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].timeframe, Timeframe::M1);
    }

    #[test]
    fn test_symbol_limit() {
        let config = MTFConfig {
            max_symbols: 2,
            ..Default::default()
        };
        let manager = MTFStateManager::new(config);

        // Add first symbol
        let tick1 = Tick::new_with_millis("EURUSD".to_string(), 1704067230000, 1.0920, 1.0922);
        assert!(manager.process_tick(&tick1).is_ok());

        // Add second symbol
        let tick2 = Tick::new_with_millis("GBPUSD".to_string(), 1704067230000, 1.3920, 1.3922);
        assert!(manager.process_tick(&tick2).is_ok());

        // Try to add third symbol - should fail
        let tick3 = Tick::new_with_millis("USDJPY".to_string(), 1704067230000, 110.20, 110.22);
        let result = manager.process_tick(&tick3);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Maximum symbols"));
    }

    #[test]
    fn test_clear_symbol() {
        let manager = MTFStateManager::with_default_config();
        let tick = Tick::new_with_millis("EURUSD".to_string(), 1704067230000, 1.0920, 1.0922);

        manager.process_tick(&tick).unwrap();
        assert_eq!(manager.get_all_symbols().len(), 1);

        manager.clear_symbol("EURUSD").unwrap();
        assert_eq!(manager.get_all_symbols().len(), 0);
    }

    #[test]
    fn test_memory_estimate() {
        let manager = MTFStateManager::with_default_config();
        let tick = Tick::new_with_millis("EURUSD".to_string(), 1704067230000, 1.0920, 1.0922);

        manager.process_tick(&tick).unwrap();
        let memory = manager.get_memory_usage_estimate();
        assert!(memory > 0);
        assert!(memory < 10000); // Should be relatively small for one tick
    }
}
