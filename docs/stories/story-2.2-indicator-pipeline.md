# Story 2.2: Rust Indicator Pipeline

**Epic:** Epic 2 - Multi-Timeframe Synchronization Engine
**Story ID:** STORY-2.2
**Status:** Ready for Development
**Branch:** `story/STORY-2.2-indicator-pipeline`

## Story Description

**As a** developer,
**I want** high-performance indicators calculated in Rust,
**So that** common indicators run at maximum speed during backtesting.

## Background & Context

Technical indicators are essential for trading algorithms. By implementing them in Rust with incremental calculation, we achieve maximum performance. This story focuses on the 20 most commonly used indicators, with an architecture that allows easy extension.

## Acceptance Criteria

### Must Have
1. ✅ **Core Indicators Implementation (20 total)**
   - [ ] **Trend Indicators**
     - [ ] SMA (Simple Moving Average)
     - [ ] EMA (Exponential Moving Average)
     - [ ] WMA (Weighted Moving Average)
     - [ ] DEMA (Double Exponential Moving Average)
   - [ ] **Momentum Indicators**
     - [ ] RSI (Relative Strength Index)
     - [ ] MACD (Moving Average Convergence Divergence)
     - [ ] Stochastic Oscillator
     - [ ] CCI (Commodity Channel Index)
     - [ ] Williams %R
   - [ ] **Volatility Indicators**
     - [ ] Bollinger Bands
     - [ ] ATR (Average True Range)
     - [ ] Keltner Channels
     - [ ] Donchian Channels
   - [ ] **Volume Indicators**
     - [ ] OBV (On-Balance Volume)
     - [ ] Volume SMA
     - [ ] VWAP (Volume Weighted Average Price)
   - [ ] **Other Essential**
     - [ ] Pivot Points
     - [ ] Support/Resistance Levels
     - [ ] ADX (Average Directional Index)
     - [ ] Parabolic SAR

2. ✅ **Incremental Calculation**
   - [ ] No recalculation of entire history on new data
   - [ ] Maintain internal state for efficiency
   - [ ] Support warm-up periods for indicators
   - [ ] Handle insufficient data gracefully

3. ✅ **Per-Timeframe Caching**
   - [ ] Indicator values cached per timeframe
   - [ ] Automatic cache invalidation on new bars
   - [ ] Memory-efficient storage
   - [ ] Quick retrieval for queries

4. ✅ **Parallel Processing**
   - [ ] Independent indicators calculate in parallel using Rayon
   - [ ] Thread-safe indicator state management
   - [ ] Dependency graph for related indicators
   - [ ] Optimal CPU utilization

5. ✅ **Performance Requirements**
   - [ ] <50μs to update all indicators on new tick
   - [ ] <10μs to retrieve cached indicator value
   - [ ] Support 1000+ simultaneous indicators
   - [ ] Memory usage <500MB for all indicators on 1M bars

6. ✅ **Accuracy & Testing**
   - [ ] Unit tests validating against reference implementations
   - [ ] Test against TradingView/TA-Lib values
   - [ ] Edge case handling (divide by zero, NaN)
   - [ ] Precision within 0.0001% of reference

### Nice to Have
- [ ] Custom indicator framework
- [ ] Indicator combination/composite indicators
- [ ] GPU acceleration for suitable indicators

## Technical Design

### Architecture
```
┌──────────────────┐
│   Bar Updates    │
└────────┬─────────┘
         │
         ▼
┌──────────────────────────────┐
│   Indicator Pipeline Manager  │
└────────┬─────────────────────┘
         │
    ┌────┴────┐
    │ Parallel │
    └────┬────┘
         │
┌────────┼────────┬──────────┐
│        │        │          │
▼        ▼        ▼          ▼
SMA     EMA      RSI      MACD
│        │        │          │
└────────┴────────┴──────────┘
         │
         ▼
┌──────────────────┐
│  Cache Storage   │
└──────────────────┘
```

### File Structure
```
crates/backtestr-core/src/indicators/
├── mod.rs                  # Indicator module exports
├── indicator_trait.rs      # Common indicator interface
├── pipeline.rs            # Pipeline manager
├── cache.rs               # Caching system
├── trend/
│   ├── mod.rs
│   ├── sma.rs
│   ├── ema.rs
│   ├── wma.rs
│   └── dema.rs
├── momentum/
│   ├── mod.rs
│   ├── rsi.rs
│   ├── macd.rs
│   ├── stochastic.rs
│   ├── cci.rs
│   └── williams_r.rs
├── volatility/
│   ├── mod.rs
│   ├── bollinger.rs
│   ├── atr.rs
│   ├── keltner.rs
│   └── donchian.rs
├── volume/
│   ├── mod.rs
│   ├── obv.rs
│   ├── volume_sma.rs
│   └── vwap.rs
└── other/
    ├── mod.rs
    ├── pivot.rs
    ├── support_resistance.rs
    ├── adx.rs
    └── parabolic_sar.rs
```

### Core Interfaces
```rust
pub trait Indicator: Send + Sync {
    type Input;
    type Output;

    fn name(&self) -> &str;
    fn warm_up_period(&self) -> usize;
    fn update(&mut self, input: Self::Input) -> Option<Self::Output>;
    fn current(&self) -> Option<Self::Output>;
    fn reset(&mut self);
}

// Standard indicator parameters (defaults)
pub struct IndicatorDefaults {
    pub sma_period: usize,           // 20
    pub ema_period: usize,           // 20
    pub rsi_period: usize,           // 14
    pub macd_fast: usize,            // 12
    pub macd_slow: usize,            // 26
    pub macd_signal: usize,          // 9
    pub bollinger_period: usize,     // 20
    pub bollinger_std_dev: f64,      // 2.0
    pub atr_period: usize,           // 14
    pub stochastic_k_period: usize,  // 14
    pub stochastic_d_period: usize,  // 3
    pub cci_period: usize,           // 20
    pub williams_r_period: usize,    // 14
    pub adx_period: usize,           // 14
}

pub struct IndicatorPipeline {
    indicators: HashMap<String, Box<dyn Indicator>>,
    cache: IndicatorCache,
    dependency_graph: DependencyGraph,
}

pub struct IndicatorCache {
    values: HashMap<(String, Timeframe), VecDeque<IndicatorValue>>,
    max_history: usize,
}
```

### Example Implementation (RSI)
```rust
pub struct RSI {
    period: usize,
    gains: VecDeque<f64>,
    losses: VecDeque<f64>,
    avg_gain: Option<f64>,
    avg_loss: Option<f64>,
    previous_close: Option<f64>,
}

impl Indicator for RSI {
    type Input = f64;  // Close price
    type Output = f64; // RSI value

    fn update(&mut self, close: f64) -> Option<f64> {
        if let Some(prev) = self.previous_close {
            let change = close - prev;
            let gain = change.max(0.0);
            let loss = (-change).max(0.0);

            // Update rolling windows
            self.gains.push_back(gain);
            self.losses.push_back(loss);

            if self.gains.len() > self.period {
                self.gains.pop_front();
                self.losses.pop_front();
            }

            // Calculate RSI
            if self.gains.len() == self.period {
                // Incremental calculation
                let avg_gain = self.gains.iter().sum::<f64>() / self.period as f64;
                let avg_loss = self.losses.iter().sum::<f64>() / self.period as f64;

                if avg_loss == 0.0 {
                    return Some(100.0);
                }

                let rs = avg_gain / avg_loss;
                let rsi = 100.0 - (100.0 / (1.0 + rs));

                self.avg_gain = Some(avg_gain);
                self.avg_loss = Some(avg_loss);
                self.previous_close = Some(close);

                return Some(rsi);
            }
        }

        self.previous_close = Some(close);
        None
    }
}
```

## Dependencies

- **Story 2.1:** MTF State Synchronization (for bar updates)
- **Blocks:** Story 2.4 (persistence needs indicators)

## Implementation Steps

1. **Phase 1: Framework**
   - Indicator trait definition
   - Pipeline manager
   - Cache system

2. **Phase 2: Core Indicators**
   - Implement trend indicators
   - Implement momentum indicators
   - Test against references

3. **Phase 3: Advanced Indicators**
   - Implement volatility indicators
   - Implement volume indicators
   - Other essential indicators

4. **Phase 4: Optimization**
   - Parallel processing with Rayon
   - Performance tuning
   - Memory optimization

## Definition of Done

- [ ] All 20 indicators implemented
- [ ] Incremental calculation verified
- [ ] Parallel processing working
- [ ] Performance targets met
- [ ] Accuracy validated against references
- [ ] Unit tests >95% coverage
- [ ] Integration tests passing
- [ ] Benchmarks documented
- [ ] Code reviewed
- [ ] CI/CD passing

## Performance Benchmarks

```rust
// Update all indicators benchmark
// Target: <50μs for 20 indicators
bench_update_all_indicators()

// Cache retrieval benchmark
// Target: <10μs
bench_cache_retrieval()

// Parallel vs sequential comparison
bench_parallel_performance()

// Memory usage benchmark
// Target: <500MB for 1M bars
bench_memory_usage()
```

## Risk Assessment

1. **Risk:** Accuracy deviation from references
   - **Mitigation:** Extensive testing against TA-Lib/TradingView
   - **Validation:** Automated comparison tests

2. **Risk:** Performance degradation with many indicators
   - **Mitigation:** Parallel processing, efficient algorithms
   - **Testing:** Load testing with 1000+ indicators

3. **Risk:** Memory growth with long histories
   - **Mitigation:** Configurable history limits
   - **Testing:** Long-running memory tests

## Testing Strategy

### Accuracy Tests
- Compare against TA-Lib for all indicators
- Test edge cases (startup, insufficient data)
- Validate warm-up period behavior

### Performance Tests
- Benchmark individual indicators
- Test pipeline with all indicators
- Measure parallel speedup

### Integration Tests
- Test with MTF engine
- Verify cache invalidation
- Test indicator dependencies

## Notes

- Focus on correctness first, then optimize
- Document any deviations from standard calculations
- Consider future Python integration (Epic 4)
- Keep interfaces clean for extensibility