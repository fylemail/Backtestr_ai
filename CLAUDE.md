# CLAUDE.md - Agent Context for Progressive Development

## Current Development Focus

**Active Epic**: Epic 3 - Core Position Management & Execution (PLANNING)
**Current Story**: Ready to start Story 3.1 - Multi-Position Tracking
**Previous Epic**: Epic 2 - MTF Engine ✅ COMPLETE (PR #13 Ready)
**Development Philosophy**: Build ONLY what current epic needs
**Note**: SQLite chosen over DuckDB in Epic 1 - maintaining this decision

## Critical Boundaries - DO NOT BUILD YET

❌ **Python Integration** - Deferred to Epic 4
❌ **Frontend/Electron** - Deferred to Epic 5
❌ **Charting/Visualization** - Deferred to Epic 5
❌ **Walkback Replay** - Deferred to Epic 6
❌ **Statistical Analysis** - Deferred to Epic 7

## What We Built in Epic 1 (COMPLETE)

✅ **Rust Core Engine** - Basic tick processing
✅ **Simple SQLite** - Basic data storage
✅ **Tick Data Ingestion** - CSV/simple formats
✅ **Basic CLI** - Query interface

## What We Built in Epic 2 (COMPLETE)

✅ **Bar/Candle Data Model** - OHLCV structures (Story 2.0)
✅ **MTF Synchronization** - Multi-timeframe state engine (Story 2.1)
✅ **Indicator Pipeline** - 20 core indicators (Story 2.2)
✅ **Advanced Bar Formation** - Session mgmt, gap handling (Story 2.3)
✅ **State Persistence** - Checkpoint/recovery system (Story 2.4)

## What We're Building NOW (Epic 3)

🚧 **Multi-Position Tracking** - Unlimited concurrent positions (Story 3.1)
🚧 **Order Execution Engine** - Realistic execution with slippage (Story 3.2)
🚧 **Risk Management System** - Stops, margins, position sizing (Story 3.3)
🚧 **Trade Lifecycle Logging** - Comprehensive trade logging (Story 3.4)

## Current Working Commands

```bash
# These work NOW:
cargo build              # Build Rust workspace
cargo test              # Run tests
cargo clippy            # Lint code
cargo fmt               # Format code
cargo run --bin backtestr -- --help  # CLI help
cargo run --bin backtestr -- import <csv_file>  # Import CSV data
cargo run --bin backtestr -- query --symbol EURUSD  # Query tick data
cargo run --bin backtestr -- stats  # Show database statistics

# These DON'T work yet (don't try):
# npm/pnpm commands     # No frontend yet
# python/pytest         # No Python yet
# electron commands     # No UI yet
```

## Project Structure (Current State)

```
backtestr_ai/
├── crates/           # Rust workspace (ACTIVE)
│   ├── backtestr-core/  # Core engine
│   ├── backtestr-data/  # Data layer
│   └── backtestr-ipc/   # IPC (minimal)
├── src/              # Main application
├── data/             # Data storage
├── docs/             # Documentation
└── scripts/          # Build scripts

# NOT YET CREATED (don't add):
# ├── algorithms/     # Epic 4
# ├── electron/       # Epic 5
# └── python/         # Epic 4
```

## Development Guidelines

### When Adding Code

1. **Ask First**: "Is this needed for Epic 1?"
2. **If No**: Add feature flag or defer entirely
3. **If Yes**: Implement minimally, no over-engineering

### Feature Flags

```rust
// Use feature flags for future epic code
#[cfg(feature = "epic_2")]
pub mod advanced_features;

// Current epic code doesn't need flags
pub mod core_features;  // Always built
```

### CI/CD

- Single workflow: `.github/workflows/ci.yml`
- Tests ONLY Rust code
- No Python/Node.js checks until their epics

### Git Workflow

```bash
# Branch naming for Epic 2
git checkout -b story/STORY-2.0-data-model-foundation

# Don't create branches for future epics yet
# ❌ story/STORY-3.1-position-management
# ❌ story/STORY-4.1-python-bridge
```

## Common Mistakes to Avoid

1. **Building Python bridge** - Not until Epic 4
2. **Setting up Electron** - Not until Epic 5
3. **Complex CI workflows** - Keep it simple
4. **Unused dependencies** - Only add what's needed NOW
5. **Empty directories** - Don't create algorithms/, electron/ yet

## Performance Targets

### Epic 1 (Achieved)
- Tick ingestion: 10K ticks/second ✅
- Memory usage: < 500MB for 1M ticks ✅
- Query response: < 100ms for basic queries ✅

### Epic 2 (Current Targets)
- Tick processing: <100μs with all timeframes
- Indicator updates: <50μs for all 20 indicators
- State query: <10μs for snapshot
- Recovery time: <1 second

## Testing Requirements

### Epic 1 Tests
- Unit tests for data structures
- Integration tests for SQLite
- Basic performance benchmarks

### NOT Required Yet
- Python algorithm tests (Epic 4)
- UI component tests (Epic 5)
- Statistical validation (Epic 7)

## Epic 1 Completion Checklist

- [x] Story 1.1: Project setup ✅
- [x] Story 1.2: Basic SQLite integration ✅
- [x] Story 1.3: CSV tick data ingestion ✅
- [x] Story 1.4: Simple query interface ✅
- [x] Story 1.5: Basic performance validation ✅

**Epic 1 Status: COMPLETE** ✅

## Epic 2 Story Planning ✅ COMPLETE

### Story 2.0: Data Model Foundation ✅ COMPLETE
- [x] Bar/Candle data structures
- [x] Timeframe enumeration
- [x] Tick-to-bar aggregation
- [x] SQLite schema extension
**Status:** Merged to develop

### Story 2.1: MTF State Synchronization ✅ COMPLETE
- [x] In-memory MTF engine
- [x] Event-driven tick processing
- [x] Partial bar tracking
**Status:** Merged to develop

### Story 2.2: Indicator Pipeline ✅ COMPLETE
- [x] 20 core indicators
- [x] Incremental calculation
- [x] Per-timeframe caching
**Status:** Merged to develop

### Story 2.3: Advanced Bar Formation ✅ COMPLETE
- [x] Weekend gap handling
- [x] Session boundaries
- [x] Bar completion events
**Status:** Merged to master

### Story 2.4: State Persistence ✅ COMPLETE
- [x] State serialization
- [x] Recovery mechanisms
- [x] Hybrid storage strategy
**Status:** Merged to develop

## Epic 3 Story Planning (READY TO START)

### Story 3.1: Multi-Position Tracking System
- [ ] Position manager with unlimited positions
- [ ] Unique IDs and O(1) lookup
- [ ] Parent-child relationships
- [ ] Memory efficient for 100+ positions
**Status:** Ready to start

### Story 3.2: Order Execution Engine
- [ ] Three execution models (Perfect, Realistic, Worst-case)
- [ ] Bid-ask spread slippage
- [ ] Commission and swap calculations
**Status:** Planning

### Story 3.3: Risk Management System
- [ ] Stop loss and take profit orders
- [ ] Trailing stops
- [ ] Margin management
- [ ] Position sizing functions
**Status:** Planning

### Story 3.4: Trade Lifecycle Logging
- [ ] Detailed position logging
- [ ] Configurable log levels
- [ ] Performance metrics
**Status:** Planning

## Questions to Ask Yourself

Before writing ANY code:
1. Is this needed for Epic 3?
2. Will this work without Python/Frontend?
3. Can this be simpler?
4. Am I over-engineering?
5. Does this maintain zero look-ahead bias?

## Epic 2→3 Interface Contracts

### Defined Interfaces
Interface contracts have been established for Epic 3 integration:

1. **PositionEventHandler** - Receives events from MTF engine
   - `on_bar_complete()` - Called when bars complete
   - `on_tick_update()` - Real-time tick processing
   - `on_indicator_update()` - Indicator value changes

2. **ExecutionContext** - Provides market data for execution
   - `get_current_spread()` - Current bid/ask prices
   - `get_bar_context()` - Bar data for slippage calculation
   - `is_market_open()` - Session boundary checks

3. **RiskContext** - Provides data for risk calculations
   - `get_indicator()` - Access to calculated indicators
   - `get_volatility()` - ATR or similar metrics
   - `get_margin_requirement()` - Leverage settings

4. **PositionStateStore** - Extends persistence for positions
   - Compatible with Epic 2 checkpoint system
   - Integrated recovery mechanisms

### Performance Baselines (Epic 2 Complete)
- Tick processing: <100μs ✅
- Bar aggregation: <50μs ✅
- Indicator updates: <50μs ✅
- Memory (1M ticks): <500MB ✅
- State recovery: <1 second ✅

### Integration Points
- **EventDispatcher**: Primary communication channel
- **MTFStateManager**: Provides market context
- **IndicatorPipeline**: Feeds risk calculations
- **Interfaces defined in**: `crates/backtestr-core/src/interfaces/`
- **Mock implementations**: `crates/backtestr-core/src/mocks/`

## Getting Help

- Epic 2 Stories: `docs/stories/story-2.*.md`
- Epic 3 Planning: `docs/stories/epic-3/`
- Performance Baseline: `docs/development/epic-2-performance-baseline.md`
- Interface Contracts: `crates/backtestr-core/src/interfaces/epic3_contracts.rs`
- Git strategy: `docs/development/git-strategy.md`

---

**Remember**: We're building a foundation, not the entire building. Keep it simple, focused, and progressive.