# Epic 5: Chart Visualization System

**Goal:** Create professional multi-timeframe visualization using Lightweight Charts, enabling traders to see their algorithm's decisions across all timeframes simultaneously.

## Story 5.1: Electron Application Shell

**As a** user,  
**I want** a desktop application,  
**so that** I can run backtests locally with full performance.

### Acceptance Criteria
1: Electron app boots with proper window management
2: Menu bar with File, Edit, View, Tools, Help menus
3: Window remembers size and position between sessions
4: Native file dialogs for data import/export
5: Keyboard shortcuts for common actions
6: Auto-updater configured for releases
7: Crash reporting for stability monitoring
8: Splash screen while loading resources

## Story 5.2: Six-Panel Chart Layout

**As a** trader,  
**I want** six synchronized timeframe charts,  
**so that** I can see market structure across all timeframes.

### Acceptance Criteria
1: 6-panel grid layout (2x3) fills available screen space
2: Each panel shows different timeframe (1m, 5m, 15m, 1H, 4H, Daily)
3: Panels resize proportionally with window
4: Timeframe labels clearly visible on each panel
5: Dark theme optimized for extended viewing
6: Panel borders subtle but clear
7: Minimum panel size enforced for readability
8: Layout persists between sessions

## Story 5.3: Lightweight Charts Integration

**As a** trader,  
**I want** professional financial charts,  
**so that** I can analyze price action effectively.

### Acceptance Criteria
1: Lightweight Charts renders candlesticks for all timeframes
2: Zoom and pan synchronized across all panels
3: Crosshair synchronized showing same time on all timeframes
4: Volume bars displayed below price
5: Price scale shows 5 decimal places for forex
6: Time scale shows appropriate labels per timeframe
7: Chart updates smooth at 60 FPS
8: Memory usage stable during long sessions

## Story 5.4: Trade Visualization

**As a** trader,  
**I want** to see all trades on charts,  
**so that** I can understand entry and exit decisions.

### Acceptance Criteria
1: Entry markers (green arrows) at exact entry points
2: Exit markers (red arrows) at exact exit points
3: Position lines connect entry to exit
4: Multiple positions shown with unique colors
5: Hover shows position details (ID, size, P&L)
6: Stop loss and take profit lines visible
7: Running P&L displayed for open positions
8: Trade markers visible across all relevant timeframes
