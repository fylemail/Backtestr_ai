# CLAUDE.md - Agent Context for Progressive Development

## Current Development Focus

**Active Epic**: Epic 2 - Multi-Timeframe Synchronization Engine
**Current Story**: 2.0 - Data Model Foundation (Starting)
**Development Philosophy**: Build ONLY what current epic needs
**Note**: SQLite chosen over DuckDB in Epic 1 - maintaining this decision

## Critical Boundaries - DO NOT BUILD YET

❌ **Python Integration** - Deferred to Epic 4
❌ **Frontend/Electron** - Deferred to Epic 5
❌ **Position Management** - Deferred to Epic 3
❌ **Charting/Visualization** - Deferred to Epic 5
❌ **Walkback Replay** - Deferred to Epic 6
❌ **Statistical Analysis** - Deferred to Epic 7

## What We Built in Epic 1 (COMPLETE)

✅ **Rust Core Engine** - Basic tick processing
✅ **Simple SQLite** - Basic data storage
✅ **Tick Data Ingestion** - CSV/simple formats
✅ **Basic CLI** - Query interface

## What We're Building NOW (Epic 2)

🚧 **Bar/Candle Data Model** - OHLCV structures
🚧 **MTF Synchronization** - Multi-timeframe state engine
🚧 **Indicator Pipeline** - 20 core indicators
🚧 **State Persistence** - Checkpoint/recovery system  

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

## Epic 2 Story Planning (NEW)

### Story 2.0: Data Model Foundation (NEW - Critical)
- [ ] Bar/Candle data structures
- [ ] Timeframe enumeration
- [ ] Tick-to-bar aggregation
- [ ] SQLite schema extension

### Story 2.1: MTF State Synchronization
- [ ] In-memory MTF engine
- [ ] Event-driven tick processing
- [ ] Partial bar tracking

### Story 2.2: Indicator Pipeline
- [ ] 20 core indicators
- [ ] Incremental calculation
- [ ] Per-timeframe caching

### Story 2.3: Advanced Bar Formation
- [ ] Weekend gap handling
- [ ] Session boundaries
- [ ] Bar completion events

### Story 2.4: State Persistence
- [ ] State serialization
- [ ] Recovery mechanisms
- [ ] Hybrid storage strategy

## Questions to Ask Yourself

Before writing ANY code:
1. Is this needed for Epic 2?
2. Will this work without Python/Frontend?
3. Can this be simpler?
4. Am I over-engineering?
5. Does this maintain zero look-ahead bias?

## Getting Help

- Epic 2 Stories: `docs/stories/story-2.*.md`
- Configuration: `docs/stories/epic-2-configuration-schema.md`
- Review Analysis: `docs/stories/epic-2-story-review.md`
- Git strategy: `docs/development/git-strategy.md`

---

**Remember**: We're building a foundation, not the entire building. Keep it simple, focused, and progressive.