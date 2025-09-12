# Epic 2: Multi-Timeframe Synchronization Engine

**Goal:** Build the revolutionary MTF state synchronization system that maintains perfect state consistency across all timeframes at every tick - the core differentiator that no competitor offers correctly.

## Story 2.1: Complete MTF State Synchronization

**As a** algorithmic trader,  
**I want** perfect multi-timeframe state synchronization,  
**so that** my algorithms can query accurate partial and completed bars across all timeframes at any moment.

### Acceptance Criteria
1: MTF engine maintains synchronized state for 1m, 5m, 15m, 1H, 4H, Daily timeframes
2: Each tick updates all affected timeframe states atomically
3: Partial bar progress tracked (e.g., "32 seconds into 1m bar")
4: Current OHLC values available for in-progress bars
5: Zero look-ahead bias verified through comprehensive tests
6: Performance maintained at <100μs per tick with all timeframes active
7: State query returns complete snapshot in <10μs
8: Deterministic results across multiple identical runs

## Story 2.2: Rust Indicator Pipeline

**As a** developer,  
**I want** high-performance indicators calculated in Rust,  
**so that** common indicators run at maximum speed.

### Acceptance Criteria
1: 20 core indicators implemented (SMA, EMA, RSI, MACD, Bollinger, ATR, etc.)
2: Incremental calculation for efficiency (no recalculation of entire history)
3: Indicator values cached per timeframe
4: Parallel calculation for independent indicators using Rayon
5: <50μs to update all indicators on new tick
6: Unit tests validating accuracy against reference implementations
7: Support for both partial and completed bar calculations
8: Memory efficient with configurable history limits

## Story 2.3: Bar Formation and Aggregation

**As a** system,  
**I want** accurate bar formation from tick data,  
**so that** higher timeframes correctly aggregate lower timeframe data.

### Acceptance Criteria
1: Tick data correctly forms 1-minute bars with proper OHLC values
2: Higher timeframes aggregate from lower timeframes accurately
3: Weekend gaps handled correctly without phantom bars
4: Session boundaries respected (daily bars close at NY close)
5: Volume aggregation sums correctly across timeframes
6: Partial bar updates trigger appropriate recalculations
7: Historical bar storage limited to configured limits (e.g., 1000 bars)
8: Bar completion events fired for algorithm notifications

## Story 2.4: MTF State Persistence and Recovery

**As a** user,  
**I want** MTF state to persist and recover,  
**so that** interrupted backtests can resume without reprocessing.

### Acceptance Criteria
1: MTF state serializable to disk every 60 seconds
2: State recovery restores exact tick-level position
3: Indicator values restored without recalculation
4: Partial bars restored with correct progress
5: Recovery time <1 second for typical state
6: Corruption detection with fallback to last valid state
7: Option to disable persistence for maximum performance
8: State files compressed to minimize disk usage
