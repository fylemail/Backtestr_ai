# Epic 2 Story Review & Gap Analysis

## Review Summary

After reviewing all 5 Epic 2 stories, I've identified several areas that need clarification to prevent developer assumptions and ensure smooth implementation.

## Story 2.0: Data Model Foundation

### ✅ Strengths
- Clear data structures defined
- Database schema provided
- File locations specified
- Performance targets clear

### 🔴 Gaps & Ambiguities

1. **Timeframe Implementation Details**
   - **Issue:** How to calculate duration for each timeframe?
   - **Needed:** Explicit duration mapping
   - **Clarification:**
   ```rust
   impl Timeframe {
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
   }
   ```

2. **Aggregation Algorithm**
   - **Issue:** Exact tick-to-bar logic not specified
   - **Needed:** How to handle first tick, gaps, volume
   - **Clarification:**
   ```rust
   // When does a bar start?
   // - First tick after bar period begins
   // - OR at period boundary even without ticks?

   // Answer: Bars start with first tick, no empty bars
   ```

3. **Migration Strategy**
   - **Issue:** How to handle existing SQLite databases?
   - **Needed:** Migration script or version check
   - **Clarification:** Add database version table

## Story 2.1: MTF State Synchronization

### ✅ Strengths
- Architecture diagram clear
- Performance requirements specific
- Zero look-ahead bias emphasized

### 🔴 Gaps & Ambiguities

1. **Tick Event Definition**
   - **Issue:** What triggers a tick event?
   - **Needed:** Clear event source
   - **Clarification:**
   ```rust
   // Tick events come from:
   // 1. Real-time feed (future)
   // 2. Historical replay (current focus)
   // 3. CSV import processing
   ```

2. **Concurrent Symbol Handling**
   - **Issue:** How many symbols simultaneously?
   - **Needed:** Limits and resource management
   - **Clarification:** Start with single symbol, add multi-symbol in phase 2

3. **Memory Limits**
   - **Issue:** "Configurable history limits" but not specified
   - **Needed:** Default values
   - **Clarification:**
   ```rust
   const DEFAULT_BAR_HISTORY: usize = 1000; // per timeframe
   const MAX_SYMBOLS: usize = 10;
   ```

## Story 2.2: Indicator Pipeline

### ✅ Strengths
- All 20 indicators listed explicitly
- Example RSI implementation provided
- Parallel processing mentioned

### 🔴 Gaps & Ambiguities

1. **Indicator Parameters**
   - **Issue:** Default periods not specified
   - **Needed:** Standard parameters for each indicator
   - **Clarification:**
   ```rust
   // Standard periods:
   SMA: 20
   EMA: 20
   RSI: 14
   MACD: (12, 26, 9)
   Bollinger: (20, 2.0)
   ATR: 14
   // ... etc
   ```

2. **Warm-up Data Handling**
   - **Issue:** What to return during warm-up?
   - **Needed:** Explicit behavior
   - **Clarification:** Return `None` until sufficient data

3. **Dependency Resolution**
   - **Issue:** MACD depends on EMAs - how to handle?
   - **Needed:** Execution order
   - **Clarification:** Use dependency graph with topological sort

## Story 2.3: Advanced Bar Formation

### ✅ Strengths
- Session handling detailed
- Gap detection explained
- Event system outlined

### 🔴 Gaps & Ambiguities

1. **Market Hours Configuration**
   - **Issue:** Market hours format not specified
   - **Needed:** Configuration structure
   - **Clarification:**
   ```rust
   pub struct MarketHours {
       symbol: String,
       timezone: Tz,
       open_time: NaiveTime,  // e.g., 09:30
       close_time: NaiveTime, // e.g., 16:00
       trading_days: Vec<Weekday>,
   }
   ```

2. **Weekend Gap Definition**
   - **Issue:** What constitutes a gap?
   - **Needed:** Time threshold
   - **Clarification:** Gap = no ticks for >2x normal bar period

3. **Session Close Times**
   - **Issue:** Daily close time varies by market
   - **Needed:** Per-symbol configuration
   - **Clarification:** Default to NY 5pm, configurable per symbol

## Story 2.4: State Persistence

### ✅ Strengths
- File format specified
- Compression mentioned
- Recovery process detailed

### 🔴 Gaps & Ambiguities

1. **Checkpoint Trigger**
   - **Issue:** Timer-based only or tick-count based too?
   - **Needed:** Clear trigger conditions
   - **Clarification:**
   ```rust
   // Checkpoint when:
   // 1. 60 seconds elapsed (configurable)
   // 2. OR 1M ticks processed
   // 3. OR manual checkpoint requested
   ```

2. **State Version Compatibility**
   - **Issue:** How to handle structure changes?
   - **Needed:** Versioning strategy
   - **Clarification:** Semantic versioning, minor versions compatible

3. **Recovery Point Selection**
   - **Issue:** Which checkpoint to use?
   - **Needed:** Selection algorithm
   - **Clarification:** Latest valid checkpoint, with fallback chain

## Cross-Story Concerns

### 1. Error Handling Strategy
- **Issue:** Not consistently defined across stories
- **Recommendation:** Use `anyhow::Result` for all public APIs
- **Internal errors:** Custom error types per module

### 2. Logging Strategy
- **Issue:** No logging mentioned
- **Recommendation:** Use `tracing` crate
- **Levels:** ERROR for failures, WARN for retries, INFO for state changes, DEBUG for details

### 3. Configuration Management
- **Issue:** Many "configurable" items without config structure
- **Recommendation:** Central configuration file
```toml
[mtf]
max_symbols = 10
bar_history_limit = 1000

[indicators]
parallel_threshold = 5

[persistence]
checkpoint_interval_secs = 60
max_checkpoints = 5
compression_level = 6
```

### 4. Testing Data
- **Issue:** Test data sources not specified
- **Recommendation:**
  - Use Epic 1 test CSV files
  - Generate synthetic tick data for edge cases
  - Create reference indicator values

## Recommended Story Updates

### Priority 1 (Blocking Issues)
1. Add Timeframe duration calculation to Story 2.0
2. Clarify tick-to-bar aggregation rules in Story 2.0
3. Define indicator default parameters in Story 2.2
4. Specify market hours configuration in Story 2.3

### Priority 2 (Important Clarifications)
1. Add memory limit defaults to Story 2.1
2. Define gap detection thresholds in Story 2.3
3. Clarify checkpoint triggers in Story 2.4
4. Add error handling strategy to all stories

### Priority 3 (Nice to Have)
1. Add example configuration file
2. Include logging examples
3. Provide test data specifications
4. Add performance profiling approach

## Developer Assumptions to Prevent

1. **Don't assume** bar periods align to wall clock (they don't)
2. **Don't assume** all markets trade 24/7 (forex does, stocks don't)
3. **Don't assume** indicators can look ahead (strict temporal ordering)
4. **Don't assume** unlimited memory (enforce limits)
5. **Don't assume** single-threaded access (use proper synchronization)

## Next Steps

1. Update each story with Priority 1 clarifications
2. Add configuration schema to Story 2.0
3. Create test data generation script
4. Document error handling strategy
5. Add examples for edge cases

## Sign-off Checklist

Before development begins, ensure:
- [ ] All Priority 1 issues addressed
- [ ] Configuration schema defined
- [ ] Test data approach documented
- [ ] Error handling strategy clear
- [ ] Memory limits specified
- [ ] Performance benchmarks have test data
- [ ] Integration points with Epic 1 verified