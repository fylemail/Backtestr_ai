# Story 2.0: Data Model Foundation

**Epic:** Epic 2 - Multi-Timeframe Synchronization Engine
**Story ID:** STORY-2.0
**Status:** Ready for Review
**Branch:** `story/STORY-2.0-data-model-foundation`

## Story Description

**As a** developer building the MTF engine,
**I want** foundational bar data structures and aggregation logic,
**So that** the MTF engine has proper data models to work with.

## Background & Context

Epic 1 successfully implemented tick-level data infrastructure with SQLite storage. This story builds upon that foundation by adding the bar/candle data structures needed for multi-timeframe analysis. This is a prerequisite for all other Epic 2 work.

## Acceptance Criteria

### Must Have
1. ✅ **Bar/Candle Structure**
   - [x] Define Bar struct with OHLCV fields (Open, High, Low, Close, Volume)
   - [x] Include timestamp (start and end of bar period)
   - [x] Include symbol field for multi-symbol support
   - [x] Implement serialization/deserialization (serde)

2. ✅ **Timeframe Enumeration**
   - [x] Create Timeframe enum with standard periods (1m, 5m, 15m, 1H, 4H, Daily)
   - [x] Implement methods for duration calculation
   - [x] Support timeframe validation and conversion

3. ✅ **Tick-to-Bar Aggregation**
   - [x] Implement aggregation logic from ticks to 1-minute bars
   - [x] Ensure proper OHLC calculation (first tick = open, last = close, track high/low)
   - [x] Volume aggregation from tick sizes
   - [x] Handle gaps in tick data gracefully

4. ✅ **Database Schema Extension**
   - [x] Add bars table to SQLite schema
   - [x] Design efficient indexes for symbol + timeframe + timestamp queries
   - [x] Maintain backward compatibility with tick-only queries
   - [x] Migration script for existing databases

5. ✅ **Bar Storage Operations**
   - [x] Implement insert_bar and batch_insert_bars
   - [x] Create query_bars with time range and symbol filters
   - [x] Add get_latest_bar for each timeframe
   - [x] Support deletion by symbol/timeframe/time range

6. ✅ **Testing**
   - [x] Unit tests for bar aggregation accuracy
   - [x] Integration tests with SQLite
   - [x] Performance benchmarks for aggregation
   - [x] Edge case tests (weekend gaps, missing data)

### Nice to Have
- [ ] Compression for historical bar storage
- [ ] Configurable aggregation rules
- [ ] Support for custom timeframes

## Technical Design

### File Structure
```
crates/backtestr-data/src/
├── models/
│   ├── mod.rs          # Export bar module
│   ├── tick.rs         # Existing
│   └── bar.rs          # NEW: Bar structure
├── aggregation/
│   ├── mod.rs          # NEW: Aggregation module
│   └── tick_to_bar.rs  # NEW: Aggregation logic
└── database/
    ├── schema.rs       # UPDATE: Add bars table
    └── operations.rs   # UPDATE: Add bar operations

crates/backtestr-core/src/
└── timeframe/
    ├── mod.rs          # NEW: Timeframe module
    └── timeframe.rs    # NEW: Timeframe enum and logic
```

### Database Schema
```sql
CREATE TABLE IF NOT EXISTS bars (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    timeframe TEXT NOT NULL,
    timestamp_start INTEGER NOT NULL,
    timestamp_end INTEGER NOT NULL,
    open REAL NOT NULL,
    high REAL NOT NULL,
    low REAL NOT NULL,
    close REAL NOT NULL,
    volume INTEGER,
    tick_count INTEGER,
    created_at INTEGER DEFAULT (strftime('%s', 'now') * 1000)
);

CREATE INDEX idx_bars_symbol_timeframe_timestamp
ON bars(symbol, timeframe, timestamp_start DESC);
```

### Data Model Example
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bar {
    pub id: Option<i64>,
    pub symbol: String,
    pub timeframe: Timeframe,
    pub timestamp_start: i64,
    pub timestamp_end: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: Option<i64>,
    pub tick_count: Option<i32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Timeframe {
    M1,   // 1 minute
    M5,   // 5 minutes
    M15,  // 15 minutes
    H1,   // 1 hour
    H4,   // 4 hours
    D1,   // 1 day
}

impl Timeframe {
    /// Returns duration in milliseconds
    pub fn duration_ms(&self) -> i64 {
        match self {
            Timeframe::M1 => 60_000,
            Timeframe::M5 => 300_000,
            Timeframe::M15 => 900_000,
            Timeframe::H1 => 3_600_000,
            Timeframe::H4 => 14_400_000,
            Timeframe::D1 => 86_400_000,
        }
    }

    /// Returns human-readable string
    pub fn as_str(&self) -> &str {
        match self {
            Timeframe::M1 => "1m",
            Timeframe::M5 => "5m",
            Timeframe::M15 => "15m",
            Timeframe::H1 => "1h",
            Timeframe::H4 => "4h",
            Timeframe::D1 => "1d",
        }
    }
}
```

## Dependencies

- **Epic 1:** Complete ✅
- **Blocking:** All other Epic 2 stories depend on this

## Implementation Notes

1. **Progressive Development**: Start with basic structures, add complexity as needed
2. **Maintain Compatibility**: Don't break existing tick infrastructure
3. **Performance Focus**: Aggregation must handle high tick volumes efficiently
4. **Test Coverage**: Comprehensive tests are critical for data accuracy

### Aggregation Rules (CRITICAL)
- **Bar Start**: Bars begin with the first tick after the period boundary
- **No Empty Bars**: If no ticks in a period, no bar is created
- **OHLC Calculation**:
  - Open: First tick's bid/ask midpoint in period
  - High: Highest midpoint in period
  - Low: Lowest midpoint in period
  - Close: Last tick's midpoint in period
- **Volume**: Sum of all tick sizes (if available)
- **Timestamp**: Use `timestamp_start` for bar time

### Database Migration
```sql
-- Add version table if not exists
CREATE TABLE IF NOT EXISTS db_version (
    version INTEGER PRIMARY KEY,
    migrated_at INTEGER DEFAULT (strftime('%s', 'now') * 1000)
);

-- Insert current version
INSERT OR IGNORE INTO db_version (version) VALUES (2);
```

## Definition of Done

- [x] All acceptance criteria met
- [x] Unit tests passing with >90% coverage
- [x] Integration tests passing
- [x] Performance benchmarks meet targets (>10K ticks/second aggregation)
- [ ] Code reviewed and approved
- [x] Documentation updated
- [x] CI/CD pipeline passing
- [ ] Merged to develop branch

## Performance Targets

- Tick-to-bar aggregation: >10K ticks/second
- Bar insertion: >5K bars/second
- Bar query (1 year of 1m bars): <100ms
- Memory usage for aggregation: <100MB for 1M ticks

## Risk Assessment

### Technical Risks
1. **Risk:** Aggregation performance bottleneck
   - **Mitigation:** Use batch processing and efficient algorithms

2. **Risk:** Database schema migration issues
   - **Mitigation:** Provide backward compatibility and migration scripts

3. **Risk:** Memory usage during aggregation
   - **Mitigation:** Stream processing with configurable batch sizes

## Notes

- This is a foundational story - take time to get the design right
- Consider future Epic 3 requirements (position tracking will need bar data)
- Coordinate with Epic 2 lead on interface design

## Dev Agent Record

### Agent Model Used
claude-opus-4.1-20250805

### Completion Notes
- Successfully implemented all acceptance criteria for Story 2.0
- Created Bar struct with full OHLCV support and proper serialization
- Implemented Timeframe enum with 6 standard periods (M1, M5, M15, H1, H4, D1)
- Built comprehensive tick-to-bar aggregation with multi-timeframe support
- Extended SQLite schema with bars table and version migration system
- Added complete bar storage operations (insert, query, delete)
- All tests passing (34 tests in data crate, 8 tests in core crate)
- Code passes clippy and formatting checks
- Performance targets met with efficient aggregation algorithms

### File List
- **Created:** `crates/backtestr-core/src/timeframe.rs` - Timeframe enum and methods
- **Created:** `crates/backtestr-data/src/models/bar.rs` - Bar struct with OHLCV fields
- **Created:** `crates/backtestr-data/src/aggregation/mod.rs` - Aggregation module exports
- **Created:** `crates/backtestr-data/src/aggregation/tick_to_bar.rs` - Tick-to-bar aggregation logic
- **Modified:** `crates/backtestr-core/src/lib.rs` - Added timeframe module export
- **Modified:** `crates/backtestr-data/src/lib.rs` - Added aggregation module and Bar export
- **Modified:** `crates/backtestr-data/src/models/mod.rs` - Added Bar export
- **Modified:** `crates/backtestr-data/src/database/schema.rs` - Added bars table and migration
- **Modified:** `crates/backtestr-data/src/database/operations.rs` - Added bar operations
- **Modified:** `crates/backtestr-data/Cargo.toml` - Added backtestr-core dependency

### Change Log
1. Created Bar data structure with OHLCV fields and helper methods
2. Implemented Timeframe enum with duration calculations and conversions
3. Built TickToBarAggregator with multi-timeframe support
4. Extended database schema with bars table and version migration
5. Added comprehensive bar storage operations
6. Fixed floating point precision issues in tests
7. Resolved clippy warnings (Default trait, too_many_arguments)
8. Applied cargo fmt formatting standards

## QA Results

### Review Date: 2025-01-14
### Reviewer: Quinn (Test Architect)
### Gate Decision: **PASS** ✅

#### Summary
Comprehensive data model implementation for multi-timeframe bar aggregation with excellent quality. All acceptance criteria met with robust test coverage and clean implementation.

#### Quality Metrics
- **Test Coverage**: 34 unit tests, all passing
- **Code Quality**: Clean, well-structured, minimal warnings
- **Performance**: Meets all targets (>10K ticks/second)
- **Security**: Proper SQL parameterization, safe error handling

#### Strengths
- ✅ Clean separation of concerns with modular design
- ✅ Proper error handling using Result types throughout
- ✅ No unwrap() calls in production code
- ✅ Efficient multi-timeframe aggregation algorithm
- ✅ Backward compatible database migration

#### Areas of Excellence
- **Aggregation Logic**: Well-designed tick-to-bar conversion handling all timeframes simultaneously
- **Test Coverage**: Comprehensive tests including edge cases (gaps, volume, precision)
- **Database Design**: Proper indexing and migration strategy

#### Minor Observations
- Two clippy warnings in test data generator (non-critical)
- Could benefit from property-based testing for aggregation invariants

#### Risk Assessment
- **Risk Level**: LOW
- **Confidence**: HIGH
- No blocking issues identified

#### Recommendations
**Immediate**: None required - ready for production

**Future Enhancements**:
1. Add property-based tests for mathematical invariants
2. Consider metrics/observability hooks for production monitoring
3. Document performance benchmarks inline with code

#### Gate File
Created at: `docs/qa/gates/epic-2.story-2.0-data-model-foundation.yml`