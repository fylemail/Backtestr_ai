# Epic 2 Pre-Start Alignment Analysis

## Executive Summary

This document analyzes the alignment between Epic 1's actual implementation and the requirements for Epic 2 and subsequent epics. Key findings and adjustments needed before starting Epic 2 development.

## Epic 1 Implementation Review

### What We Built
1. **SQLite Database** (not DuckDB as originally planned)
   - Simple tick storage with basic schema
   - File-based and in-memory support
   - Batch insert optimization (1000 rows/batch)

2. **Data Model**
   - Simple Tick structure (symbol, timestamp, bid, ask, bid_size, ask_size)
   - Timestamp stored as milliseconds since Unix epoch (i64)
   - No bar/candle structures yet

3. **CLI Interface**
   - Import CSV files
   - Query ticks by symbol and time range
   - Statistics and deletion commands
   - JSON and table output formats

4. **Performance Baseline**
   - Tick insertion: >10K/second ✓
   - Memory usage: <500MB for 1M ticks ✓
   - Query response: <100ms ✓

### What We Didn't Build
- Bar/Candle data structures
- Any timeframe aggregation
- Real-time tick processing
- Event systems
- Complex data models

## Epic 2 Requirements Analysis

### Story 2.1: MTF State Synchronization
**Current Gaps:**
- No bar/candle data structures exist
- No timeframe concept in current model
- No event system for tick updates
- SQLite may not be optimal for real-time MTF updates

**Required Additions:**
- Bar data model (OHLCV)
- Timeframe enumeration (1m, 5m, 15m, 1H, 4H, Daily)
- Event-driven architecture for tick processing
- In-memory MTF state manager

### Story 2.2: Rust Indicator Pipeline
**Current Foundation:**
- Empty `backtestr-core/src/indicators/mod.rs` exists
- No indicator infrastructure

**Required Additions:**
- Indicator trait system
- Incremental calculation framework
- Per-timeframe caching system

### Story 2.3: Bar Formation
**Critical Gap:** No bar formation logic exists

**Required Additions:**
- Bar aggregation from ticks
- Time-based bar boundaries
- Volume aggregation

### Story 2.4: State Persistence
**Current State:**
- SQLite for tick persistence only
- No state serialization

**Required Additions:**
- State snapshot system
- Binary serialization (bincode/serde)
- Recovery mechanisms

## Epic 3-7 Dependencies

### Epic 3: Position Management
- **Dependency on Epic 2:** Needs bar data for entry/exit
- **Alignment:** Can work with Epic 2's bar structures

### Epic 4: Python Bridge
- **Current State:** Empty `backtestr-core/src/python/mod.rs`
- **Alignment:** Should be deferred until core Rust engine complete

### Epic 5: Chart Visualization
- **Dependency:** Needs bar data from Epic 2
- **Note:** Original plan mentions Lightweight Charts (JavaScript)
- **Consideration:** May need IPC earlier than planned

### Epic 6: Walkback Replay
- **Dependency:** Needs complete MTF engine from Epic 2
- **Alignment:** Correctly sequenced after Epic 2

### Epic 7: Statistical Analysis
- **Dependency:** Needs position data from Epic 3
- **Alignment:** Correctly sequenced

## Key Architectural Decisions Needed

### 1. Database Strategy
**Issue:** SQLite chosen over DuckDB in Epic 1
**Impact on Epic 2:**
- SQLite may struggle with real-time MTF updates
- Consider hybrid approach: SQLite for historical, in-memory for live processing

### 2. Bar Storage
**Options:**
1. Store bars in SQLite alongside ticks
2. Generate bars on-demand from ticks
3. Hybrid: Cache recent bars, generate historical on-demand

**Recommendation:** Option 3 for best performance/storage balance

### 3. Event Architecture
**Need:** Epic 2 requires event-driven tick processing
**Options:**
1. Build custom event system
2. Use tokio channels
3. Simple callback system

**Recommendation:** Start with simple callbacks, evolve as needed

### 4. Data Model Evolution
**Current:** Simple tick model
**Epic 2 Needs:**
- Bar/Candle structure
- Timeframe enumeration
- MTF state containers

## Recommended Epic 2 Story Adjustments

### Story 2.0: Data Model Foundation (NEW)
**Priority:** Do First
**Scope:**
- Define Bar/Candle structures
- Create Timeframe enum
- Design MTF state containers
- Implement tick-to-bar aggregation logic

### Story 2.1: MTF Engine Core (Modified)
**Dependencies:** Story 2.0
**Focus:** Pure in-memory MTF engine first
**Defer:** Persistence until Story 2.4

### Story 2.2: Indicator Pipeline (As-is)
**No changes needed**

### Story 2.3: Bar Formation (Merge with 2.0)
**Recommendation:** Combine with Story 2.0 as foundational

### Story 2.4: Persistence (Modified)
**Add:** Hybrid storage strategy implementation

## Migration Path from Epic 1

### Phase 1: Data Model Extension
1. Add bar structures to `backtestr-data`
2. Extend database schema for bars
3. Keep tick infrastructure intact

### Phase 2: MTF Engine in `backtestr-core`
1. Build MTF state manager
2. Implement tick processor
3. Create bar aggregation

### Phase 3: Integration
1. Connect CLI to MTF engine
2. Add bar-based queries
3. Performance validation

## Risks and Mitigations

### Risk 1: SQLite Performance
**Mitigation:** Design for pluggable storage backend

### Risk 2: Memory Usage with MTF
**Mitigation:** Configurable history limits per timeframe

### Risk 3: Complexity Creep
**Mitigation:** Strict story boundaries, no Epic 3+ features

## Recommendations

1. **Create Story 2.0** for data model foundation
2. **Keep SQLite** but design for future storage flexibility
3. **Build MTF engine** as pure in-memory first
4. **Defer Python/IPC** until Epic 4/5 as planned
5. **Document architecture decisions** as we go

## Next Steps

1. Update Epic 2 documentation with Story 2.0
2. Create detailed technical design for MTF engine
3. Define bar storage strategy
4. Create Epic 2 story branches
5. Begin Story 2.0 implementation