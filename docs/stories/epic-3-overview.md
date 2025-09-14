# Epic 3: Core Position Management & Execution - Overview

**Epic ID:** EPIC-3
**Status:** Planning
**Estimated Duration:** 4-6 weeks
**Dependencies:** Epic 2 (Multi-Timeframe Engine) ✅ COMPLETE

## Executive Summary

Epic 3 builds upon the MTF engine foundation to add comprehensive position management and realistic trade execution. This epic enables the platform to track multiple concurrent positions, simulate realistic order execution with slippage, and provide risk management capabilities.

## Key Deliverables

1. **Multi-Position Tracking System** - Unlimited concurrent positions with O(1) lookup
2. **Order Execution Engine** - Realistic execution with configurable slippage models
3. **Risk Management System** - Stop loss, take profit, trailing stops, margin management
4. **Trade Lifecycle Logging** - Comprehensive logging without visualization

## Stories Overview

### Story 3.1: Multi-Position Tracking System
- Position manager with unlimited positions per symbol
- Unique IDs and independent parameters per position
- Parent-child relationships for complex strategies
- Memory efficient even with 100+ positions

### Story 3.2: Order Execution Engine
- Three execution models: Perfect, Realistic, Worst-case
- Bid-ask spread based slippage
- Correct entry/exit prices (ask for buy, bid for sell)
- Commission and swap cost calculations

### Story 3.3: Risk Management System
- Stop loss and take profit orders
- Trailing stops with dynamic adjustment
- Margin call management
- Position sizing functions (fixed, percentage, Kelly)

### Story 3.4: Trade Lifecycle Logging
- Detailed position lifecycle logging
- Per-tick P&L updates
- Configurable log levels
- Parseable output for analysis

## Technical Architecture

### Core Components
```
crates/backtestr-core/
├── positions/
│   ├── position_manager.rs    # Multi-position tracking
│   ├── position.rs            # Position data structure
│   └── position_state.rs      # State management
├── execution/
│   ├── order_engine.rs        # Order execution logic
│   ├── slippage_models.rs     # Slippage calculations
│   └── fill_simulator.rs      # Fill simulation
├── risk/
│   ├── risk_manager.rs        # Risk management system
│   ├── stop_orders.rs         # Stop/TP/Trailing logic
│   └── margin_calculator.rs   # Margin calculations
└── logging/
    ├── trade_logger.rs        # Trade lifecycle logging
    └── position_reporter.rs   # Position reporting
```

### Data Flow
1. MTF Engine provides tick and bar data
2. Position Manager processes orders
3. Execution Engine simulates fills
4. Risk Manager enforces limits
5. Logger captures all events

## Performance Requirements

- Position operations: O(1) lookup, O(log n) insertion
- < 10μs per position update
- < 50μs for risk calculations
- < 5% overhead from logging
- Support 1000+ concurrent positions

## Success Criteria

1. ✅ All 4 stories completed with tests
2. ✅ Performance benchmarks met
3. ✅ Integration with MTF engine working
4. ✅ Comprehensive test coverage (>80%)
5. ✅ Documentation complete

## Risk Mitigation

1. **Complexity Risk**: Start with simple position tracking, add features incrementally
2. **Performance Risk**: Use efficient data structures (HashMap, BTreeMap)
3. **Integration Risk**: Design clean interfaces with MTF engine
4. **Testing Risk**: Implement comprehensive unit and integration tests

## Development Approach

### Phase 1: Foundation (Story 3.1)
- Basic position tracking
- Simple position lifecycle
- Core data structures

### Phase 2: Execution (Story 3.2)
- Order execution models
- Slippage implementation
- Cost calculations

### Phase 3: Risk (Story 3.3)
- Stop/TP orders
- Trailing stops
- Margin management

### Phase 4: Logging (Story 3.4)
- Trade logging system
- Performance optimization
- Integration testing

## NOT in Scope (Deferred)

- ❌ Visual position display (Epic 5)
- ❌ Python strategy integration (Epic 4)
- ❌ Statistical analysis (Epic 7)
- ❌ Replay/walkback (Epic 6)

## Getting Started

After Epic 2 completion:
1. Create branch: `epic/EPIC-3-position-management`
2. Start with Story 3.1: Multi-Position Tracking
3. Follow progressive development approach
4. Maintain zero look-ahead bias

## Notes

- Builds directly on Epic 2's MTF engine
- No external dependencies needed
- Focus on correctness over optimization initially
- Keep interfaces clean for future Python integration