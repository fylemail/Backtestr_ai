# Story 2.3: Advanced Bar Formation & Aggregation

**Epic:** Epic 2 - Multi-Timeframe Synchronization Engine
**Story ID:** STORY-2.3
**Status:** Blocked by Story 2.0
**Branch:** `story/STORY-2.3-bar-formation-advanced`

## Story Description

**As a** system,
**I want** accurate bar formation with advanced features,
**So that** higher timeframes correctly aggregate lower timeframe data with proper handling of gaps and sessions.

## Background & Context

While Story 2.0 provides basic tick-to-bar aggregation, this story adds the advanced features needed for production trading: weekend gap handling, session boundaries, volume aggregation, and bar completion events. These features are critical for accurate backtesting of forex and futures markets.

## Acceptance Criteria

### Must Have
1. ✅ **Multi-Timeframe Aggregation**
   - [ ] 1-minute bars aggregate correctly from ticks
   - [ ] Higher timeframes aggregate from lower timeframes
   - [ ] 5m bars from 1m bars (5 bars → 1)
   - [ ] 15m bars from 5m bars (3 bars → 1)
   - [ ] 1H bars from 15m bars (4 bars → 1)
   - [ ] 4H bars from 1H bars (4 bars → 1)
   - [ ] Daily bars from 4H bars (6 bars → 1)

2. ✅ **Weekend Gap Handling**
   - [ ] No phantom bars during market closure
   - [ ] Friday close to Sunday open handled correctly
   - [ ] Holiday gaps identified and marked
   - [ ] Configurable market hours per symbol

3. ✅ **Session Boundaries**
   - [ ] Daily bars close at configurable time (e.g., NY 5pm)
   - [ ] Weekly bars close on Friday
   - [ ] Monthly bars close on last trading day
   - [ ] Session overlap handling for 24-hour markets

4. ✅ **Volume Aggregation**
   - [ ] Tick volume sums correctly across timeframes
   - [ ] Real volume (if available) aggregates properly
   - [ ] Volume-weighted calculations maintained
   - [ ] Zero-volume bars handled appropriately

5. ✅ **Bar Completion Events**
   - [ ] Events fired when bars complete
   - [ ] Subscribers notified per timeframe
   - [ ] Event includes completed bar data
   - [ ] Guaranteed delivery order (1m before 5m, etc.)

6. ✅ **Historical Bar Management**
   - [ ] Configurable history limits per timeframe
   - [ ] Automatic cleanup of old bars
   - [ ] Memory-efficient storage (circular buffers)
   - [ ] Quick access to recent bars

### Nice to Have
- [ ] Custom session definitions
- [ ] Renko/Range bars
- [ ] Volume-based bars
- [ ] Market profile generation

## Technical Design

### Bar Aggregation Flow
```
Ticks → 1m bars → 5m bars → 15m bars → 1H bars → 4H bars → Daily bars
         ↓         ↓          ↓          ↓         ↓          ↓
      Events    Events     Events     Events    Events     Events
```

### File Structure
```
crates/backtestr-core/src/
├── aggregation/
│   ├── mod.rs
│   ├── bar_aggregator.rs      # Core aggregation logic
│   ├── session_manager.rs     # Session boundary handling
│   ├── gap_detector.rs        # Gap detection logic
│   └── volume_aggregator.rs   # Volume calculations
├── events/
│   ├── bar_completion.rs      # Bar completion events
│   └── event_bus.rs          # Event distribution
```

### Core Components

```rust
pub struct BarAggregator {
    aggregation_rules: HashMap<Timeframe, AggregationRule>,
    session_manager: SessionManager,
    gap_detector: GapDetector,
    event_bus: EventBus,
}

pub struct AggregationRule {
    source_timeframe: Timeframe,
    target_timeframe: Timeframe,
    bars_per_aggregation: usize,
    aggregation_method: AggregationMethod,
}

pub struct SessionManager {
    market_hours: HashMap<String, MarketHours>,
    holidays: HashSet<NaiveDate>,
    session_close_times: HashMap<Timeframe, NaiveTime>,
}

pub struct MarketHours {
    pub symbol: String,
    pub timezone: chrono_tz::Tz,
    pub open_time: NaiveTime,     // e.g., 09:30
    pub close_time: NaiveTime,    // e.g., 16:00
    pub trading_days: Vec<chrono::Weekday>,
    pub session_break: Option<(NaiveTime, NaiveTime)>, // For markets with breaks
}

// Default market hours for common markets
impl Default for MarketHours {
    fn default() -> Self {
        // Forex default (24/5)
        MarketHours {
            symbol: "DEFAULT".to_string(),
            timezone: chrono_tz::US::Eastern,
            open_time: NaiveTime::from_hms_opt(17, 0, 0).unwrap(), // Sunday 5pm ET
            close_time: NaiveTime::from_hms_opt(17, 0, 0).unwrap(), // Friday 5pm ET
            trading_days: vec![
                Weekday::Mon, Weekday::Tue, Weekday::Wed,
                Weekday::Thu, Weekday::Fri
            ],
            session_break: None,
        }
    }
}

pub struct GapDetector {
    max_gap_duration: Duration,
    market_schedule: MarketSchedule,
}

pub enum BarCompletionEvent {
    MinuteBar(Bar),
    FiveMinuteBar(Bar),
    FifteenMinuteBar(Bar),
    HourBar(Bar),
    FourHourBar(Bar),
    DailyBar(Bar),
}
```

### Aggregation Algorithm
```rust
impl BarAggregator {
    pub fn aggregate_bars(
        &mut self,
        source_bars: &[Bar],
        target_timeframe: Timeframe
    ) -> Option<Bar> {
        let rule = self.aggregation_rules.get(&target_timeframe)?;

        if source_bars.len() < rule.bars_per_aggregation {
            return None;
        }

        // Check session boundary
        if self.session_manager.is_session_boundary(
            target_timeframe,
            source_bars.last()?.timestamp_end
        ) {
            // Force bar completion even if not enough source bars
            return Some(self.create_session_bar(source_bars));
        }

        // Check for gaps
        if self.gap_detector.has_gap(source_bars) {
            // Handle gap appropriately
            return self.handle_gap_aggregation(source_bars);
        }

        // Normal aggregation
        Some(Bar {
            symbol: source_bars[0].symbol.clone(),
            timeframe: target_timeframe,
            timestamp_start: source_bars[0].timestamp_start,
            timestamp_end: source_bars.last()?.timestamp_end,
            open: source_bars[0].open,
            high: source_bars.iter().map(|b| b.high).max_by(f64::total_cmp)?,
            low: source_bars.iter().map(|b| b.low).min_by(f64::total_cmp)?,
            close: source_bars.last()?.close,
            volume: source_bars.iter().map(|b| b.volume.unwrap_or(0)).sum(),
            tick_count: source_bars.iter().map(|b| b.tick_count.unwrap_or(0)).sum(),
        })
    }
}
```

## Dependencies

- **Story 2.0:** Data Model Foundation (bar structures)
- **Story 2.1:** MTF State Synchronization (receives events from)
- **Blocks:** Story 2.4 (persistence needs completed bars)

## Implementation Steps

1. **Phase 1: Basic Aggregation**
   - Implement timeframe aggregation rules
   - Test aggregation accuracy
   - Add volume aggregation

2. **Phase 2: Session Management**
   - Implement market hours configuration
   - Add session boundary detection
   - Handle daily/weekly/monthly closes

3. **Phase 3: Gap Handling**
   - Detect market gaps
   - Implement gap handling logic
   - Test weekend/holiday scenarios

4. **Phase 4: Event System**
   - Implement bar completion events
   - Add event subscribers
   - Test event ordering

## Definition of Done

- [ ] All aggregation rules implemented
- [ ] Weekend gaps handled correctly
- [ ] Session boundaries respected
- [ ] Volume aggregation accurate
- [ ] Event system functional
- [ ] Unit tests >90% coverage
- [ ] Integration tests passing
- [ ] Performance benchmarks met
- [ ] Code reviewed
- [ ] Documentation complete

## Performance Targets

- Bar aggregation: <10μs per aggregation
- Event dispatch: <5μs per event
- Memory usage: <100MB for 1M bars across all timeframes
- Gap detection: <1μs per check

## Risk Assessment

1. **Risk:** Incorrect aggregation causing data corruption
   - **Mitigation:** Extensive testing against known good data
   - **Validation:** Compare with TradingView/MT5

2. **Risk:** Session boundary edge cases
   - **Mitigation:** Comprehensive timezone testing
   - **Testing:** Test all market transitions

3. **Risk:** Event ordering issues
   - **Mitigation:** Strict ordering enforcement
   - **Testing:** Concurrent event testing

## Testing Strategy

### Aggregation Tests
- Test all timeframe combinations
- Verify OHLCV calculations
- Test partial aggregations

### Gap Tests
- Weekend gap scenarios
- Holiday gaps
- Intraday gaps

### Session Tests
- Daily close at NY 5pm
- Weekly close on Friday
- Monthly close accuracy

### Event Tests
- Event delivery order
- Event data accuracy
- Subscriber notifications

## Notes

- Core aggregation logic from Story 2.0 is reused here
- Focus on the advanced features that make this production-ready
- Consider futures markets with different session times
- Document any market-specific handling