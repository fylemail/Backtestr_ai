# Developer Task: Epic 2→3 Interface Contracts & Integration Testing

## Context
**Project**: BackTestr AI - Rust-based backtesting engine
**Current State**: Epic 2 (MTF Engine) complete, PR #13 ready to merge
**Next Phase**: Epic 3 (Position Management & Execution)
**Git Branch Strategy**: Following docs/development/git-strategy.md
**Your Model**: claude-opus-4.1-20250805

## Objective
Define clean interface contracts between Epic 2 (MTF Engine) and Epic 3 (Position Management) components, implement integration verification tests, and document performance baselines before starting Epic 3 implementation.

## Task Requirements

### 1. Create Transition Branch
```bash
# Create from develop (NOT from story branches)
git checkout develop
git pull origin develop
git checkout -b story/transition-epic-2-to-3-interfaces

# Verify Epic 2 is fully integrated
git log --oneline -10  # Should show Story 2.4 merged
```

### 2. Define Interface Contracts

Create file: `crates/backtestr-core/src/interfaces/epic3_contracts.rs`

Required interfaces to define:

#### A. Position Event Handler Interface
```rust
// Extension of existing EventHandler trait for position management
pub trait PositionEventHandler: EventHandler {
    // Called when bar completes (from MTF engine)
    fn on_bar_complete(&mut self, bar: &Bar, timeframe: Timeframe, symbol: &str);

    // Called on each tick for position P&L updates
    fn on_tick_update(&mut self, tick: &Tick, symbol: &str);

    // Called when indicator updates (from indicator pipeline)
    fn on_indicator_update(&mut self, indicator: &IndicatorValue, timeframe: Timeframe, symbol: &str);
}
```

#### B. Execution Context Interface
```rust
// Provides market context for order execution
pub trait ExecutionContext {
    // Get current bid/ask from tick data
    fn get_current_spread(&self, symbol: &str) -> Option<(f64, f64)>;

    // Get bar data for slippage calculation
    fn get_bar_context(&self, symbol: &str, timeframe: Timeframe) -> Option<&Bar>;

    // Check if market is open (session boundaries)
    fn is_market_open(&self, symbol: &str, timestamp: i64) -> bool;
}
```

#### C. Risk Context Interface
```rust
// Provides data for risk calculations
pub trait RiskContext {
    // Get indicator values for risk decisions
    fn get_indicator(&self, symbol: &str, timeframe: Timeframe, name: &str) -> Option<f64>;

    // Get volatility for position sizing
    fn get_volatility(&self, symbol: &str, timeframe: Timeframe) -> Option<f64>;

    // Get margin requirements
    fn get_margin_requirement(&self, symbol: &str) -> f64;
}
```

#### D. State Persistence Extension
```rust
// Extension for position state persistence
pub trait PositionStateStore {
    // Save position snapshot
    fn save_positions(&self, positions: &PositionSnapshot) -> Result<()>;

    // Restore positions on recovery
    fn restore_positions(&self) -> Result<PositionSnapshot>;

    // Get checkpoint compatibility
    fn is_compatible_with_mtf(&self, mtf_version: &str) -> bool;
}
```

### 3. Create Integration Tests

Create file: `crates/backtestr-core/tests/epic2_to_epic3_integration.rs`

Required test scenarios:

#### A. Event Flow Verification
```rust
#[test]
fn test_mtf_to_position_event_flow() {
    // Setup MTF engine with test data
    // Create mock position handler
    // Verify events flow correctly:
    // - Tick → MTF → Position Handler
    // - Bar completion → Position Handler
    // - Indicator update → Position Handler
}
```

#### B. Performance Baseline Tests
```rust
#[test]
fn benchmark_event_throughput() {
    // Measure current performance WITHOUT position management:
    // - Tick processing: Target <100μs
    // - Bar aggregation: Target <50μs
    // - Indicator calculation: Target <50μs
    // Document results in epic-2-performance-baseline.md
}
```

#### C. State Recovery Integration
```rust
#[test]
fn test_mtf_state_recovery_compatibility() {
    // Create MTF state with checkpoints
    // Simulate crash
    // Recover state
    // Verify ready for position management attachment
}
```

#### D. Memory Usage Baseline
```rust
#[test]
fn benchmark_memory_usage() {
    // Load 1M ticks
    // Process through all timeframes
    // Measure memory usage
    // Document baseline (target <500MB)
}
```

### 4. Create Performance Baseline Documentation

Create file: `docs/development/epic-2-performance-baseline.md`

Document:
- Current tick processing latency
- Bar aggregation performance
- Indicator pipeline throughput
- Memory usage with various data sizes
- State persistence/recovery times
- Event dispatcher overhead

Use this format:
```markdown
# Epic 2 Performance Baseline

## Test Environment
- OS: Windows [version]
- CPU: [model]
- RAM: [amount]
- Rust: 1.75.0

## Baseline Metrics

### Tick Processing
- Single tick: Xμs (median), Yμs (p99)
- With all timeframes: Xμs (median), Yμs (p99)
- With indicators: Xμs (median), Yμs (p99)

### Memory Usage
- 100K ticks: X MB
- 1M ticks: Y MB
- 10M ticks: Z MB

[etc...]
```

### 5. Create Mock Position Manager

Create file: `crates/backtestr-core/src/mocks/mock_position_manager.rs`

Implement a basic mock that:
- Implements the PositionEventHandler trait
- Logs all received events
- Measures event processing time
- Can simulate various load scenarios

### 6. Update CLAUDE.md Context

Add to CLAUDE.md:
```markdown
## Epic 2→3 Interface Contracts

### Defined Interfaces
- PositionEventHandler: Receives events from MTF engine
- ExecutionContext: Provides market data for execution
- RiskContext: Provides data for risk calculations
- PositionStateStore: Extends persistence for positions

### Performance Baselines (Epic 2 Complete)
- Tick processing: Xμs
- Bar aggregation: Yμs
- Memory (1M ticks): Z MB
- [Add actual measured values]

### Integration Points
- EventDispatcher: Primary communication channel
- MTFStateManager: Provides market context
- IndicatorPipeline: Feeds risk calculations
```

### 7. Verify No Breaking Changes

Run comprehensive test suite:
```bash
# Run all tests
cargo test --all-features --workspace

# Run benchmarks
cargo bench --bench mtf_performance

# Check for any warnings
cargo clippy -- -D warnings

# Verify formatting
cargo fmt --check
```

### 8. Create Pull Request

Create PR with title:
```
[Transition] Epic 2→3 Interface Contracts and Integration Tests
```

PR Description must include:
- [ ] All interface contracts defined
- [ ] Integration tests passing
- [ ] Performance baselines documented
- [ ] No breaking changes to Epic 2
- [ ] CLAUDE.md updated with interfaces
- [ ] Ready for Epic 3 Story 3.1

## Important Constraints

1. **DO NOT** modify existing Epic 2 code (except adding `pub` where needed)
2. **DO NOT** implement actual position management logic (just interfaces)
3. **DO NOT** add external dependencies
4. **DO NOT** create visualization or Python integration
5. **MAINTAIN** zero look-ahead bias principle
6. **ENSURE** all existing tests still pass

## Success Criteria

- [ ] Clean separation of concerns between Epic 2 and Epic 3
- [ ] All interfaces compile but don't require implementation
- [ ] Performance baseline documented with actual measurements
- [ ] Integration test framework ready for Epic 3
- [ ] No regression in Epic 2 functionality
- [ ] PR can be merged to develop without conflicts

## Files to Create/Modify

New files:
- `crates/backtestr-core/src/interfaces/epic3_contracts.rs`
- `crates/backtestr-core/src/interfaces/mod.rs` (if not exists)
- `crates/backtestr-core/tests/epic2_to_epic3_integration.rs`
- `crates/backtestr-core/src/mocks/mock_position_manager.rs`
- `crates/backtestr-core/src/mocks/mod.rs` (if not exists)
- `docs/development/epic-2-performance-baseline.md`

Modified files:
- `crates/backtestr-core/src/lib.rs` (export interfaces module)
- `CLAUDE.md` (add interface documentation)
- `Cargo.toml` (ensure test dependencies are present)

## Estimated Time
2-3 hours for complete implementation

## Questions to Validate Before Starting

1. Are all Epic 2 tests currently passing?
2. Is PR #13 merged or should we wait?
3. Are there any specific performance concerns to investigate?
4. Should we create the epic/EPIC-3-position-management branch now or after this task?

## Command Sequence

```bash
# 1. Setup
git checkout develop
git pull origin develop
git checkout -b story/transition-epic-2-to-3-interfaces

# 2. Create interfaces
# [implement interfaces]

# 3. Test
cargo test --all-features
cargo bench

# 4. Document
# [create performance baseline doc]

# 5. Commit
git add .
git commit -m "feat(transition): define Epic 2→3 interface contracts

- Created position management interfaces
- Added integration test framework
- Documented performance baselines
- Prepared for Epic 3 implementation

No breaking changes to Epic 2 functionality"

# 6. Push and create PR
git push -u origin story/transition-epic-2-to-3-interfaces
```

---

**Note**: This transition work ensures clean boundaries between epics and prevents future refactoring. It's essential for maintaining the progressive development approach defined in CLAUDE.md.