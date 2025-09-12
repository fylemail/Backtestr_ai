# Requirements

## Functional

- **FR1:** The platform must maintain synchronized state across 6 timeframes (1m, 5m, 15m, 1H, 4H, Daily) with current bar progress and completed bar history accessible at any tick timestamp
- **FR2:** Process tick-level forex data for EUR/USD, GBP/USD, USD/JPY, and AUD/USD with bid/ask spread preservation and volume information
- **FR3:** Execute algorithms with complex multi-condition logic including nested if/then statements, multiple indicator calculations, dynamic position sizing, and support for multiple concurrent positions per currency pair with unique position IDs
- **FR4:** Provide three execution modeling profiles - Perfect (zero slippage), Realistic (typical 0.1-0.3 pip slippage), and Worst-case (0.5-1.0 pip slippage with widened spreads)
- **FR5:** Filter out configurable time periods including weekends, major holidays (Christmas, New Year, etc.), and between-session gaps (NY close to Asian open)
- **FR6:** Enable tick-by-tick walkback replay with variable speed control (1x, 10x, 50x, max speed) and frame-by-frame stepping
- **FR7:** Display 6 synchronized chart panels with crosshair linking, trade markers showing all open/closed positions, and current bar highlighting across all timeframes
- **FR8:** Calculate comprehensive statistics including Sharpe ratio, profit factor, win rate, maximum drawdown, MAE/MFE, trade duration analysis, and aggregate metrics for overlapping positions
- **FR9:** Generate performance heatmaps showing P&L breakdown by hour of day, day of week, volatility quintiles, and trading sessions
- **FR10:** Import tick data from CSV, FIX protocol, and binary formats with automatic format detection and validation
- **FR11:** Save and load backtest configurations including date ranges, execution settings, and filter parameters as reusable test scenarios
- **FR12:** Export results as CSV (trade list), JSON (full metrics), and PDF (professional reports with charts)
- **FR13:** Support algorithm code in Python format with access to 50+ built-in technical indicators
- **FR14:** Compress and store tick data using DuckDB with 10-20x compression ratios maintaining query performance
- **FR15:** Track unlimited concurrent positions per currency pair with independent stop loss, take profit, position size, and entry/exit timestamps
- **FR16:** Calculate position correlation, aggregate exposure, margin usage, and combined P&L when multiple positions are open simultaneously
- **FR17:** Visualize multiple positions on charts with distinct colors/markers, showing entry/exit points and position IDs for each trade
- **FR18:** Support position management operations including partial closes, position scaling (pyramiding), and hedging strategies

## Non Functional

- **NFR1:** Process 1 million ticks per second on modern hardware (Intel i7/AMD Ryzen 7 or better)
- **NFR2:** Maintain 60 FPS smooth rendering during walkback replay across all 6 chart panels
- **NFR3:** Use less than 8GB RAM for 1 year of tick data during active backtesting with up to 100 concurrent positions
- **NFR4:** Achieve 100% deterministic results - identical outputs for identical inputs across multiple runs
- **NFR5:** Complete statistical analysis generation within 5 seconds after backtest completion
- **NFR6:** Support offline operation for backtesting once data is downloaded (internet only required for data updates and initial setup)
- **NFR7:** Run on Windows 10/11 (64-bit) with full native Windows integration and performance optimization
- **NFR8:** Respond to UI interactions within 100ms for all user actions
- **NFR9:** Handle 10+ years of historical tick data (approximately 15-20GB compressed per currency pair)
- **NFR10:** Maintain tick processing latency under 100 microseconds for MTF state updates
- **NFR11:** Support monitors from 1920x1080 to 4K resolution with responsive scaling
- **NFR12:** Auto-save backtest state every 60 seconds to prevent data loss
- **NFR13:** Handle position arrays efficiently with O(1) position lookup by ID and O(n) iteration for aggregate calculations
