# Epic 6: Walkback Replay Engine

**Goal:** Build the tick-by-tick replay system that enables visual debugging of algorithm decisions, showing exactly why trades were taken at each moment.

## Story 6.1: Replay Infrastructure

**As a** developer,  
**I want** a replay engine architecture,  
**so that** tick data can be replayed at variable speeds.

### Acceptance Criteria
1: Replay engine separate from live execution engine
2: Tick queue feeds data at controlled rate
3: State snapshot captured at each tick
4: Replay position tracked with tick precision
5: Memory buffering for smooth playback
6: Jump to any timestamp instantly
7: Replay state independent of backtest results
8: Performance maintains 60 FPS during replay

## Story 6.2: Playback Controls

**As a** trader,  
**I want** intuitive playback controls,  
**so that** I can analyze trades at my preferred speed.

### Acceptance Criteria
1: Play/pause button with spacebar shortcut
2: Speed selector (1x, 10x, 50x, max)
3: Frame step forward/backward with arrow keys
4: Progress bar showing position in backtest
5: Jump to next/previous trade buttons
6: Current timestamp displayed prominently
7: Ticks per second counter
8: Estimated time remaining at current speed

## Story 6.3: State Inspector

**As a** trader,  
**I want** to inspect algorithm state during replay,  
**so that** I can understand decision making.

### Acceptance Criteria
1: Side panel shows current algorithm variables
2: Indicator values displayed for all timeframes
3: Position list with current P&L
4: Recent log messages from algorithm
5: MTF state visualization showing partial bars
6: Tick data (bid, ask, spread) prominently shown
7: Decision points highlighted when trades triggered
8: State exportable for debugging

## Story 6.4: Replay Synchronization

**As a** trader,  
**I want** charts synchronized with replay,  
**so that** I see exactly what the algorithm saw.

### Acceptance Criteria
1: All charts update simultaneously per tick
2: Current bar highlighting shows active bars
3: Indicator overlays update with replay
4: Trade markers appear at execution moment
5: Partial bar progress visible
6: No look-ahead - future data hidden
7: Smooth animation between ticks
8: Pause freezes all chart updates
