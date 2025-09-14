# Story 2.1: MTF State Synchronization Engine

**Epic:** Epic 2 - Multi-Timeframe Synchronization Engine
**Story ID:** STORY-2.1
**Status:** Blocked by Story 2.0
**Branch:** `story/STORY-2.1-mtf-state-synchronization`

## Story Description

**As an** algorithmic trader,
**I want** perfect multi-timeframe state synchronization,
**So that** my algorithms can query accurate partial and completed bars across all timeframes at any moment.

## Background & Context

This is the core differentiator of BackTestr - maintaining perfect state consistency across all timeframes at every tick. Most backtesting platforms fail here, leading to look-ahead bias and incorrect results. Our MTF engine will process each tick and atomically update all affected timeframes.

## Acceptance Criteria

### Must Have
1. ✅ **MTF State Management**
   - [ ] Maintain synchronized state for 1m, 5m, 15m, 1H, 4H, Daily timeframes
   - [ ] In-memory state structure for all active timeframes
   - [ ] Support multiple symbols simultaneously
   - [ ] Thread-safe state access

2. ✅ **Atomic Tick Processing**
   - [ ] Each tick updates all affected timeframe states atomically
   - [ ] No partial updates - all or nothing
   - [ ] Maintain consistency during concurrent access
   - [ ] Event system for tick arrival

3. ✅ **Partial Bar Tracking**
   - [ ] Track progress within each timeframe (e.g., "32 seconds into 1m bar")
   - [ ] Current OHLC values available for in-progress bars
   - [ ] Percentage completion for each timeframe
   - [ ] Time until next bar completion

4. ✅ **State Query Interface**
   - [ ] Get complete MTF snapshot at any moment
   - [ ] Query specific timeframe state
   - [ ] Get all partial bars across timeframes
   - [ ] Historical bar access from memory cache

5. ✅ **Zero Look-Ahead Prevention**
   - [ ] Strict temporal ordering enforcement
   - [ ] No future data access possible
   - [ ] Comprehensive tests for look-ahead bias
   - [ ] Audit trail for state changes

6. ✅ **Performance Requirements**
   - [ ] <100μs per tick with all timeframes active
   - [ ] State query returns in <10μs
   - [ ] Support 100K+ ticks/second throughput
   - [ ] Memory usage <1GB for 1M ticks across all timeframes

### Nice to Have
- [ ] Configurable timeframe sets
- [ ] Custom timeframe definitions
- [ ] State visualization/debugging tools

## Technical Design

### Architecture
```
┌─────────────────┐
│   Tick Stream   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Event Dispatcher│
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────┐
│      MTF State Manager          │
│  ┌──────────┬──────────┬────┐  │
│  │ 1m State │ 5m State │... │  │
│  └──────────┴──────────┴────┘  │
└─────────────────────────────────┘
         │
         ▼
┌─────────────────┐
│  Query Interface │
└─────────────────┘
```

### File Structure
```
crates/backtestr-core/src/
├── mtf/
│   ├── mod.rs              # MTF module exports
│   ├── state_manager.rs    # Core MTF state manager
│   ├── timeframe_state.rs  # Per-timeframe state
│   ├── tick_processor.rs   # Tick processing logic
│   └── state_query.rs      # Query interface
├── events/
│   ├── mod.rs              # Event system
│   ├── tick_event.rs       # Tick event definition
│   └── bar_event.rs        # Bar completion events
```

### Core Data Structures
```rust
// Configuration constants
const DEFAULT_BAR_HISTORY: usize = 1000;  // per timeframe
const MAX_SYMBOLS: usize = 10;            // for initial implementation
const MAX_MEMORY_MB: usize = 1000;        // 1GB limit

pub struct MTFStateManager {
    states: HashMap<Symbol, SymbolMTFState>,
    event_dispatcher: EventDispatcher,
    config: MTFConfig,
}

pub struct MTFConfig {
    pub bar_history_limit: usize,  // Default: 1000
    pub max_symbols: usize,         // Default: 10
    pub max_memory_mb: usize,       // Default: 1000
}

pub struct SymbolMTFState {
    symbol: String,
    current_tick: Option<Tick>,
    timeframes: HashMap<Timeframe, TimeframeState>,
    last_update: i64,
}

pub struct TimeframeState {
    timeframe: Timeframe,
    current_bar: Option<PartialBar>,
    completed_bars: VecDeque<Bar>,  // Limited history
    bar_start_time: i64,
    bar_end_time: i64,
    tick_count: u32,
}

pub struct PartialBar {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: i64,
    tick_count: u32,
    completion_percentage: f32,
}
```

## Dependencies

- **Story 2.0:** Data Model Foundation (MUST be complete)
- **Blocks:** Stories 2.2, 2.3, 2.4

## Implementation Steps

1. **Phase 1: Core Structure**
   - Implement MTFStateManager
   - Create TimeframeState management
   - Basic tick processing

2. **Phase 2: Event System**
   - Tick event dispatcher
   - Bar completion events
   - State change notifications

3. **Phase 3: Query Interface**
   - Snapshot queries
   - Partial bar access
   - Performance optimization

4. **Phase 4: Testing & Validation**
   - Look-ahead bias tests
   - Performance benchmarks
   - Stress testing

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Zero look-ahead bias verified
- [ ] Performance targets achieved
- [ ] Unit tests >95% coverage
- [ ] Integration tests passing
- [ ] Deterministic behavior verified
- [ ] Code reviewed and approved
- [ ] Documentation complete
- [ ] CI/CD passing

## Performance Benchmarks

### Required Benchmarks
```rust
// Benchmark tick processing
// Target: <100μs per tick
bench_tick_processing_all_timeframes()

// Benchmark state query
// Target: <10μs
bench_state_query_snapshot()

// Benchmark throughput
// Target: >100K ticks/second
bench_mtf_throughput()

// Benchmark memory usage
// Target: <1GB for 1M ticks
bench_memory_usage()
```

## Risk Assessment

### Critical Risks
1. **Risk:** Race conditions in concurrent access
   - **Mitigation:** Use RwLock or lock-free structures
   - **Testing:** Stress test with concurrent access

2. **Risk:** Look-ahead bias in implementation
   - **Mitigation:** Strict temporal ordering, comprehensive tests
   - **Testing:** Automated bias detection tests

3. **Risk:** Memory growth unbounded
   - **Mitigation:** Configurable history limits, circular buffers
   - **Testing:** Long-running memory tests

## Testing Strategy

### Test Categories
1. **Correctness Tests**
   - Bar aggregation accuracy
   - State consistency verification
   - Look-ahead bias prevention

2. **Performance Tests**
   - Latency benchmarks
   - Throughput tests
   - Memory usage monitoring

3. **Stress Tests**
   - High-frequency tick streams
   - Multiple symbols
   - Long-running stability

## Notes

- This is the core innovation of BackTestr - get it right
- Consider future Epic 3 integration (position tracking)
- Maintain clean interfaces for Epic 4 Python integration
- Document the synchronization algorithm thoroughly