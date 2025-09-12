# Goals and Background Context

## Goals

- Deliver a desktop backtesting platform that achieves 95%+ correlation between backtest and live trading results
- Solve the multi-timeframe synchronization problem that causes 70-80% of algorithmic strategies to fail in live markets
- Enable algorithmic traders to validate complex forex strategies in hours instead of months of demo trading
- Process 10 years of tick-level data with sub-millisecond latency for real-time walkback visualization
- Provide institutional-grade statistical analysis and performance heatmaps for comprehensive strategy validation
- Eliminate the $15,000+ annual cost barrier of institutional platforms while exceeding their accuracy

## Background Context

BackTestr_ai addresses the critical validation gap in algorithmic trading where existing platforms fail catastrophically at multi-timeframe strategy testing. Our research confirmed that no current platform - including QuantConnect, MetaTrader, and TradingView - correctly synchronizes multiple timeframe states at the moment of algorithmic decision-making. This fundamental flaw causes strategies that appear profitable in backtests to lose money in live markets, costing traders an average of $50,000 annually in failed deployments.

The platform's revolutionary MTF (Multi-Timeframe) State Engine maintains synchronized partial and completed bar states across all timeframes at every tick, enabling algorithms to query current market structure exactly as they would in live trading. Built as a Windows desktop application using Rust for performance-critical tick processing and free charting libraries, BackTestr_ai delivers institutional-level accuracy without the associated costs or complexity.

**MVP Strategy:** The strategic decision to focus exclusively on Windows for the MVP reduces development complexity by approximately 70%. This enables the team to concentrate on perfecting the core MTF synchronization engine and user experience rather than managing cross-platform compatibility issues. The Windows-first approach allows for native integration with Windows-specific performance optimizations, file system conventions, and user interface patterns that professional traders expect.

## Change Log

| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2024-01-11 | 1.0 | Initial PRD creation from Project Brief | John (PM) |
| 2025-09-11 | 1.1 | Strategic pivot to Windows-only MVP - 70% complexity reduction | BMad (PM) |
