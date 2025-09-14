use crate::mtf::{MTFStateManager, PartialBar};
use backtestr_data::{Bar, Tick, Timeframe};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MTFSnapshot {
    pub symbol: String,
    pub timestamp: i64,
    pub current_tick: Option<Tick>,
    pub partial_bars: HashMap<Timeframe, Option<PartialBar>>,
    pub completed_bars: HashMap<Timeframe, Vec<Bar>>,
    pub query_time_us: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeframeSnapshot {
    pub timeframe: Timeframe,
    pub partial_bar: Option<PartialBar>,
    pub latest_bars: Vec<Bar>,
    pub completion_percentage: f32,
    pub time_remaining_ms: i64,
}

pub struct StateQuery<'a> {
    manager: &'a MTFStateManager,
}

impl<'a> StateQuery<'a> {
    pub fn new(manager: &'a MTFStateManager) -> Self {
        Self { manager }
    }

    pub fn get_snapshot(&self, symbol: &str) -> Option<MTFSnapshot> {
        let start = Instant::now();

        let state = self.manager.get_symbol_state(symbol)?;

        let mut partial_bars = HashMap::new();
        let mut completed_bars = HashMap::new();

        for (&timeframe, tf_state) in &state.timeframes {
            partial_bars.insert(timeframe, tf_state.current_bar.clone());
            completed_bars.insert(timeframe, tf_state.get_latest_bars(10));
        }

        let query_time_us = start.elapsed().as_micros() as u64;

        Some(MTFSnapshot {
            symbol: symbol.to_string(),
            timestamp: state.last_update,
            current_tick: state.current_tick,
            partial_bars,
            completed_bars,
            query_time_us,
        })
    }

    pub fn get_timeframe_snapshot(
        &self,
        symbol: &str,
        timeframe: Timeframe,
    ) -> Option<TimeframeSnapshot> {
        let state = self.manager.get_symbol_state(symbol)?;
        let tf_state = state.get_timeframe_state(timeframe)?;

        Some(TimeframeSnapshot {
            timeframe,
            partial_bar: tf_state.current_bar.clone(),
            latest_bars: tf_state.get_latest_bars(10),
            completion_percentage: tf_state.get_completion_percentage(),
            time_remaining_ms: tf_state.get_time_remaining_ms(),
        })
    }

    pub fn get_all_partial_bars(
        &self,
        symbol: &str,
    ) -> Option<HashMap<Timeframe, Option<PartialBar>>> {
        let state = self.manager.get_symbol_state(symbol)?;
        Some(state.get_all_partial_bars())
    }

    pub fn get_latest_completed_bars(
        &self,
        symbol: &str,
        timeframe: Timeframe,
        count: usize,
    ) -> Option<Vec<Bar>> {
        let state = self.manager.get_symbol_state(symbol)?;
        let tf_state = state.get_timeframe_state(timeframe)?;
        Some(tf_state.get_latest_bars(count))
    }

    pub fn get_all_symbols(&self) -> Vec<String> {
        self.manager.get_all_symbols()
    }

    pub fn has_symbol(&self, symbol: &str) -> bool {
        self.manager.get_symbol_state(symbol).is_some()
    }

    pub fn get_memory_usage(&self) -> usize {
        self.manager.get_memory_usage_estimate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mtf::MTFConfig;

    #[test]
    fn test_state_query_creation() {
        let manager = MTFStateManager::with_default_config();
        let query = StateQuery::new(&manager);
        assert_eq!(query.get_all_symbols().len(), 0);
    }

    #[test]
    fn test_get_snapshot_empty() {
        let manager = MTFStateManager::with_default_config();
        let query = StateQuery::new(&manager);

        let snapshot = query.get_snapshot("EURUSD");
        assert!(snapshot.is_none());
    }

    #[test]
    fn test_get_snapshot_with_data() {
        let manager = MTFStateManager::with_default_config();
        let tick = Tick::new_with_millis("EURUSD".to_string(), 1704067230000, 1.0920, 1.0922);

        manager.process_tick(&tick).unwrap();

        let query = StateQuery::new(&manager);
        let snapshot = query.get_snapshot("EURUSD");

        assert!(snapshot.is_some());
        let snap = snapshot.unwrap();
        assert_eq!(snap.symbol, "EURUSD");
        assert_eq!(snap.timestamp, 1704067230000);
        assert!(snap.current_tick.is_some());
        assert!(!snap.partial_bars.is_empty());
    }

    #[test]
    fn test_get_timeframe_snapshot() {
        let manager = MTFStateManager::with_default_config();
        let tick = Tick::new_with_millis("EURUSD".to_string(), 1704067230000, 1.0920, 1.0922);

        manager.process_tick(&tick).unwrap();

        let query = StateQuery::new(&manager);
        let snapshot = query.get_timeframe_snapshot("EURUSD", Timeframe::M1);

        assert!(snapshot.is_some());
        let snap = snapshot.unwrap();
        assert_eq!(snap.timeframe, Timeframe::M1);
        assert!(snap.partial_bar.is_some());
        assert_eq!(snap.completion_percentage, 50.0);
        assert_eq!(snap.time_remaining_ms, 30000);
    }

    #[test]
    fn test_get_all_partial_bars() {
        let config = MTFConfig {
            enabled_timeframes: vec![Timeframe::M1, Timeframe::M5],
            ..Default::default()
        };
        let manager = MTFStateManager::new(config);
        let tick = Tick::new_with_millis("EURUSD".to_string(), 1704067230000, 1.0920, 1.0922);

        manager.process_tick(&tick).unwrap();

        let query = StateQuery::new(&manager);
        let partial_bars = query.get_all_partial_bars("EURUSD");

        assert!(partial_bars.is_some());
        let bars = partial_bars.unwrap();
        assert_eq!(bars.len(), 2);
        assert!(bars.contains_key(&Timeframe::M1));
        assert!(bars.contains_key(&Timeframe::M5));
    }

    #[test]
    fn test_has_symbol() {
        let manager = MTFStateManager::with_default_config();
        let tick = Tick::new_with_millis("EURUSD".to_string(), 1704067230000, 1.0920, 1.0922);

        manager.process_tick(&tick).unwrap();

        let query = StateQuery::new(&manager);
        assert!(query.has_symbol("EURUSD"));
        assert!(!query.has_symbol("GBPUSD"));
    }

    #[test]
    fn test_get_memory_usage() {
        let manager = MTFStateManager::with_default_config();
        let tick = Tick::new_with_millis("EURUSD".to_string(), 1704067230000, 1.0920, 1.0922);

        manager.process_tick(&tick).unwrap();

        let query = StateQuery::new(&manager);
        let memory = query.get_memory_usage();
        assert!(memory > 0);
    }
}
