# Story 2.0: Data Model Foundation

**Epic:** Epic 2 - Multi-Timeframe Synchronization Engine
**Story ID:** STORY-2.0
**Status:** Ready for Development
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
   - [ ] Define Bar struct with OHLCV fields (Open, High, Low, Close, Volume)
   - [ ] Include timestamp (start and end of bar period)
   - [ ] Include symbol field for multi-symbol support
   - [ ] Implement serialization/deserialization (serde)

2. ✅ **Timeframe Enumeration**
   - [ ] Create Timeframe enum with standard periods (1m, 5m, 15m, 1H, 4H, Daily)
   - [ ] Implement methods for duration calculation
   - [ ] Support timeframe validation and conversion

3. ✅ **Tick-to-Bar Aggregation**
   - [ ] Implement aggregation logic from ticks to 1-minute bars
   - [ ] Ensure proper OHLC calculation (first tick = open, last = close, track high/low)
   - [ ] Volume aggregation from tick sizes
   - [ ] Handle gaps in tick data gracefully

4. ✅ **Database Schema Extension**
   - [ ] Add bars table to SQLite schema
   - [ ] Design efficient indexes for symbol + timeframe + timestamp queries
   - [ ] Maintain backward compatibility with tick-only queries
   - [ ] Migration script for existing databases

5. ✅ **Bar Storage Operations**
   - [ ] Implement insert_bar and batch_insert_bars
   - [ ] Create query_bars with time range and symbol filters
   - [ ] Add get_latest_bar for each timeframe
   - [ ] Support deletion by symbol/timeframe/time range

6. ✅ **Testing**
   - [ ] Unit tests for bar aggregation accuracy
   - [ ] Integration tests with SQLite
   - [ ] Performance benchmarks for aggregation
   - [ ] Edge case tests (weekend gaps, missing data)

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

- [ ] All acceptance criteria met
- [ ] Unit tests passing with >90% coverage
- [ ] Integration tests passing
- [ ] Performance benchmarks meet targets (>10K ticks/second aggregation)
- [ ] Code reviewed and approved
- [ ] Documentation updated
- [ ] CI/CD pipeline passing
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