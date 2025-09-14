//! State serialization for MTF engine components

use crate::mtf::{MTFStateManager, PartialBar, SymbolMTFState};
use backtestr_data::{Bar, Tick, Timeframe};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const CHECKPOINT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointData {
    pub version: u32,
    pub timestamp: i64,
    pub tick_count: u64,
    pub mtf_state: MTFStateSnapshot,
    pub indicator_states: HashMap<String, IndicatorSnapshot>,
    pub metadata: CheckpointMetadata,
    #[serde(skip)]
    pub checksum: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MTFStateSnapshot {
    pub current_tick: Option<Tick>,
    pub symbol_states: HashMap<String, SymbolMTFStateSnapshot>,
    pub partial_bars: HashMap<(String, Timeframe), PartialBarSnapshot>,
    pub completed_bar_ids: HashMap<(String, Timeframe), Vec<i64>>,
    pub last_processed_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolMTFStateSnapshot {
    pub symbol: String,
    pub timeframe_bars: HashMap<Timeframe, Option<Bar>>,
    pub bar_counts: HashMap<Timeframe, usize>,
    pub last_tick_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialBarSnapshot {
    pub symbol: String,
    pub timeframe: Timeframe,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    pub tick_count: u32,
    pub start_time: i64,
    pub last_update: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorSnapshot {
    pub name: String,
    pub timeframe: Timeframe,
    pub values: Vec<f64>,
    pub parameters: HashMap<String, f64>,
    pub last_update: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    pub created_at: i64,
    pub backtest_id: String,
    pub symbol_count: usize,
    pub total_bars: usize,
    pub engine_version: String,
}

impl MTFStateManager {
    pub fn create_snapshot(&self) -> Result<MTFStateSnapshot, anyhow::Error> {
        // Use the existing public method get_all_symbols
        let symbols = self.get_all_symbols();

        let mut symbol_states = HashMap::new();
        for symbol in symbols {
            if let Some(state) = self.get_symbol_state(&symbol) {
                symbol_states.insert(symbol, state.to_snapshot());
            }
        }

        let partial_bars = self
            .get_internal_partial_bars()
            .into_iter()
            .map(|((symbol, tf), bar)| {
                let snapshot = PartialBarSnapshot {
                    symbol: symbol.clone(),
                    timeframe: tf,
                    open: bar.open,
                    high: bar.high,
                    low: bar.low,
                    close: bar.close,
                    volume: bar.volume as u64,
                    tick_count: bar.tick_count,
                    start_time: chrono::Utc::now().timestamp_millis() - bar.milliseconds_elapsed,
                    last_update: chrono::Utc::now().timestamp_millis(),
                };
                ((symbol, tf), snapshot)
            })
            .collect();

        Ok(MTFStateSnapshot {
            current_tick: self.get_internal_current_tick(),
            symbol_states,
            partial_bars,
            completed_bar_ids: self.get_internal_completed_bar_ids(),
            last_processed_timestamp: self.get_internal_last_timestamp(),
        })
    }

    pub fn restore_from_snapshot(
        &mut self,
        snapshot: MTFStateSnapshot,
    ) -> Result<(), anyhow::Error> {
        for (symbol, state_snapshot) in snapshot.symbol_states {
            self.restore_symbol_state(symbol, state_snapshot)?;
        }

        for ((symbol, timeframe), partial) in snapshot.partial_bars {
            self.restore_partial_bar(symbol, timeframe, partial)?;
        }

        self.set_completed_bar_ids(snapshot.completed_bar_ids);
        self.set_last_timestamp(snapshot.last_processed_timestamp);

        if let Some(tick) = snapshot.current_tick {
            self.set_current_tick(tick);
        }

        Ok(())
    }

    fn get_internal_partial_bars(&self) -> HashMap<(String, Timeframe), PartialBar> {
        // Use public API to get partial bars
        let mut partial_bars = HashMap::new();

        for symbol in self.get_all_symbols() {
            if let Some(state) = self.get_symbol_state(&symbol) {
                for (timeframe, partial_opt) in state.get_all_partial_bars() {
                    if let Some(partial) = partial_opt {
                        partial_bars.insert((symbol.clone(), timeframe), partial);
                    }
                }
            }
        }

        partial_bars
    }

    fn get_internal_completed_bar_ids(&self) -> HashMap<(String, Timeframe), Vec<i64>> {
        HashMap::new() // TODO: Implement
    }

    fn get_internal_last_timestamp(&self) -> i64 {
        0 // TODO: Implement
    }

    fn get_internal_current_tick(&self) -> Option<Tick> {
        None // TODO: Implement
    }

    fn restore_symbol_state(
        &mut self,
        _symbol: String,
        _snapshot: SymbolMTFStateSnapshot,
    ) -> Result<(), anyhow::Error> {
        Ok(()) // TODO: Implement
    }

    fn restore_partial_bar(
        &mut self,
        _symbol: String,
        _timeframe: Timeframe,
        _snapshot: PartialBarSnapshot,
    ) -> Result<(), anyhow::Error> {
        Ok(()) // TODO: Implement
    }

    fn set_completed_bar_ids(&mut self, _ids: HashMap<(String, Timeframe), Vec<i64>>) {
        // TODO: Implement
    }

    fn set_last_timestamp(&mut self, _timestamp: i64) {
        // TODO: Implement
    }

    fn set_current_tick(&mut self, _tick: Tick) {
        // TODO: Implement
    }

    pub fn restore_indicators(
        &mut self,
        _indicators: HashMap<String, IndicatorSnapshot>,
    ) -> Result<(), anyhow::Error> {
        Ok(()) // TODO: Implement
    }
}

impl SymbolMTFState {
    fn to_snapshot(&self) -> SymbolMTFStateSnapshot {
        SymbolMTFStateSnapshot {
            symbol: String::new(), // TODO: Get from actual state
            timeframe_bars: HashMap::new(),
            bar_counts: HashMap::new(),
            last_tick_timestamp: 0,
        }
    }
}
