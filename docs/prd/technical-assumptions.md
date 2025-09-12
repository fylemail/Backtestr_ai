# Technical Assumptions

## Repository Structure: Monorepo

Single repository containing all packages (Rust engine, Python analysis, React UI, Node.js orchestration) with shared tooling and coordinated versioning. Enables atomic commits across components and simplified CI/CD. Using npm/yarn workspaces for JavaScript packages and Cargo workspace for Rust components.

## Service Architecture

**Hybrid single-process with UI separation** for optimal performance:
- **Main Process (Rust + Embedded Python)** - Tick processing, MTF state management, algorithm execution, and statistics calculation in shared memory
- **UI Process (Electron)** - Separate process for UI responsiveness, receives batched updates via IPC
- **PyO3 Integration** - Python embedded directly in Rust process, eliminating IPC overhead for indicators
- **DuckDB Embedded** - In-process database for zero-copy data access

Communication between main process and UI via MessagePack over IPC, batched to minimize overhead. UI updates throttled to 60 FPS regardless of tick rate.

## Database Schema Design

```sql
-- DuckDB Schema (Columnar storage optimized for time-series)

-- Tick data table (partitioned by date)
CREATE TABLE ticks (
    symbol VARCHAR(10),
    timestamp BIGINT,  -- Microseconds since epoch
    bid DECIMAL(10,5),
    ask DECIMAL(10,5),
    bid_volume INTEGER,
    ask_volume INTEGER,
    PRIMARY KEY (symbol, timestamp)
) PARTITION BY (DATE(timestamp));

-- Compressed bar cache for common timeframes
CREATE TABLE bars_cache (
    symbol VARCHAR(10),
    timeframe VARCHAR(5),
    timestamp BIGINT,
    open DECIMAL(10,5),
    high DECIMAL(10,5),
    low DECIMAL(10,5),
    close DECIMAL(10,5),
    volume BIGINT,
    tick_count INTEGER,
    PRIMARY KEY (symbol, timeframe, timestamp)
);

-- Backtest results storage
CREATE TABLE backtests (
    id UUID PRIMARY KEY,
    created_at TIMESTAMP,
    config JSON,
    summary_stats JSON,
    execution_time_ms INTEGER
);

-- Trade history
CREATE TABLE trades (
    backtest_id UUID,
    trade_id VARCHAR(50),
    symbol VARCHAR(10),
    entry_time BIGINT,
    exit_time BIGINT,
    entry_price DECIMAL(10,5),
    exit_price DECIMAL(10,5),
    position_size DECIMAL(10,2),
    pnl DECIMAL(10,2),
    pnl_pips DECIMAL(10,1),
    mae_pips DECIMAL(10,1),
    mfe_pips DECIMAL(10,1),
    entry_reason TEXT,
    exit_reason TEXT,
    FOREIGN KEY (backtest_id) REFERENCES backtests(id)
);

-- Position tracking (for multi-position support)
CREATE TABLE positions (
    backtest_id UUID,
    position_id VARCHAR(50),
    parent_trade_id VARCHAR(50),
    symbol VARCHAR(10),
    open_time BIGINT,
    close_time BIGINT,
    size DECIMAL(10,2),
    entry_price DECIMAL(10,5),
    exit_price DECIMAL(10,5),
    stop_loss DECIMAL(10,5),
    take_profit DECIMAL(10,5),
    commission DECIMAL(10,2),
    swap DECIMAL(10,2),
    pnl DECIMAL(10,2),
    FOREIGN KEY (backtest_id) REFERENCES backtests(id)
);
```

## Testing Requirements

**Comprehensive testing pyramid** essential for financial accuracy:
- **Unit tests** - Every calculation, especially MTF state synchronization and position tracking
- **Integration tests** - IPC communication, data flow between processes, and state consistency
- **Performance tests** - Tick processing speed, memory usage, and chart rendering benchmarks
- **Accuracy tests** - Compare backtest results with known live trading outcomes
- **Regression tests** - Ensure deterministic results across code changes
- **Manual testing conveniences** - Debug mode with state inspection, test data generators, and replay tools

## Additional Technical Assumptions and Requests

- **Rust 1.75+** with Tokio for async operations and PyO3 for Python binding
- **Python 3.11+** with NumPy, Pandas, and TA-Lib for technical analysis
- **Node.js 20 LTS** with Electron 28+ for desktop application framework
- **React 18** with Lightweight Charts for visualization and Zustand for state management
- **DuckDB 0.9+** embedded database for tick data storage with zero-copy Arrow integration
- **MessagePack** for IPC serialization between main process and UI
- **GitHub Actions** for CI/CD with Windows-specific builders and deployment pipeline
- **Code signing certificate** required for Windows distribution and Microsoft Store compatibility
- **Deterministic floating point** operations using fixed-point arithmetic for financial calculations
- **Memory-mapped files** for efficient tick data access without loading entire datasets
- **SIMD optimizations** in Rust for parallel tick processing where available
- **Windows-specific optimizations** - Native Windows APIs for file I/O, memory management, and high-resolution timers
- **Error budgeting** - Maximum 1 microsecond cumulative rounding error per million calculations
- **Windows integration** - File associations, system tray, notifications, and Windows-native context menus

## Deployment and Distribution Strategy

**Windows-Only MVP Benefits:**
- Single installer targeting Windows 10/11 (64-bit)
- Windows-native packaging with MSI and Microsoft Store compatibility
- Simplified testing matrix (1 platform vs 3)
- Focused customer support for Windows-specific issues
- Potential Microsoft Store distribution for broader reach
- Windows-optimized performance without cross-platform overhead
- Native integration with Windows security and update mechanisms
