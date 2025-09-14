# Epic 2 Configuration Schema

This document defines all configurable parameters for Epic 2 to prevent hardcoded values and ensure flexibility.

## Configuration File: `backtestr.toml`

```toml
# BackTestr Configuration
# Epic 2: Multi-Timeframe Synchronization Engine

[database]
# SQLite configuration
path = "./data/backtestr.db"
connection_pool_size = 5
busy_timeout_ms = 5000

[mtf]
# Multi-Timeframe Engine Configuration
max_symbols = 10                    # Maximum concurrent symbols
bar_history_limit = 1000            # Bars to keep per timeframe
max_memory_mb = 1000                # Memory limit (1GB)
tick_processing_threads = 4          # Parallel processing threads

[mtf.timeframes]
# Enabled timeframes (comment out to disable)
enabled = ["M1", "M5", "M15", "H1", "H4", "D1"]

[aggregation]
# Bar Aggregation Settings
use_bid_ask_midpoint = true         # Use midpoint for OHLC
create_empty_bars = false            # Don't create bars without ticks
volume_aggregation = "sum"          # sum | average | last

[indicators]
# Indicator Pipeline Configuration
parallel_threshold = 5               # Min indicators for parallel processing
cache_size_per_indicator = 1000     # Values to cache

# Default indicator parameters
[indicators.defaults]
sma_period = 20
ema_period = 20
rsi_period = 14
macd_fast = 12
macd_slow = 26
macd_signal = 9
bollinger_period = 20
bollinger_std_dev = 2.0
atr_period = 14
stochastic_k_period = 14
stochastic_d_period = 3
cci_period = 20
williams_r_period = 14
adx_period = 14
vwap_period = 0                      # 0 = session-based
pivot_type = "standard"              # standard | fibonacci | woodie

[sessions]
# Market Session Configuration
default_timezone = "US/Eastern"
daily_close_time = "17:00"           # 5pm ET
weekly_close_day = "Friday"

# Market-specific sessions
[[sessions.markets]]
symbol_pattern = "EUR*"              # Forex pairs
timezone = "US/Eastern"
open_time = "17:00"                  # Sunday 5pm
close_time = "17:00"                 # Friday 5pm
trading_days = ["Mon", "Tue", "Wed", "Thu", "Fri"]

[[sessions.markets]]
symbol_pattern = "ES*"               # E-mini S&P
timezone = "US/Central"
open_time = "17:00"                  # Sunday 5pm CT
close_time = "16:00"                 # Friday 4pm CT
trading_days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri"]
session_break = ["16:15", "16:30"]   # Daily maintenance

[gaps]
# Gap Detection Configuration
max_gap_multiplier = 2.0             # Gap if > 2x normal bar period
weekend_gap_handling = "ignore"      # ignore | mark | fill
holiday_calendar = "US"              # US | UK | EU | NONE

[persistence]
# State Persistence Configuration
enabled = true
checkpoint_interval_secs = 60        # Checkpoint every minute
checkpoint_on_tick_count = 1000000   # Also checkpoint every 1M ticks
max_checkpoints = 5                  # Keep last 5 checkpoints
checkpoint_path = "./data/checkpoints"
compression_enabled = true
compression_level = 6                # 1-9, higher = better compression

[persistence.recovery]
auto_recover = true                  # Auto-recover on startup
fallback_checkpoints = 3             # Try last 3 checkpoints
validate_checksums = true
version_check = "compatible"         # strict | compatible | none

[logging]
# Logging Configuration
level = "INFO"                       # ERROR | WARN | INFO | DEBUG | TRACE
file = "./logs/backtestr.log"
max_file_size_mb = 100
max_files = 10
console_output = true

[logging.modules]
# Per-module log levels
"backtestr_core::mtf" = "DEBUG"
"backtestr_core::indicators" = "INFO"
"backtestr_data::aggregation" = "INFO"

[performance]
# Performance Tuning
tick_buffer_size = 10000             # Ticks to buffer before processing
batch_insert_size = 1000             # Database batch size
cache_line_size = 64                 # CPU cache optimization

[testing]
# Test Configuration
test_data_path = "./test-data"
reference_data_path = "./test-data/reference"
benchmark_iterations = 100
benchmark_warmup = 10

# Error Handling Strategy
[errors]
strategy = "log_and_continue"        # fail_fast | log_and_continue | retry
max_retries = 3
retry_delay_ms = 100
```

## Environment Variables

Environment variables override config file settings:

```bash
# Database
BACKTESTR_DB_PATH=./data/backtestr.db

# Memory limits
BACKTESTR_MAX_MEMORY_MB=2000

# Logging
BACKTESTR_LOG_LEVEL=DEBUG

# Persistence
BACKTESTR_CHECKPOINT_INTERVAL=30
BACKTESTR_CHECKPOINT_PATH=./checkpoints

# Testing
BACKTESTR_TEST_MODE=true
```

## Configuration Loading Priority

1. Default values (hardcoded)
2. Configuration file (`backtestr.toml`)
3. Environment variables
4. Command-line arguments

## Usage in Code

```rust
use serde::Deserialize;
use config::{Config, ConfigError, File, Environment};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: DatabaseConfig,
    pub mtf: MTFConfig,
    pub indicators: IndicatorConfig,
    pub sessions: SessionConfig,
    pub persistence: PersistenceConfig,
    pub logging: LoggingConfig,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::builder()
            // Start with defaults
            .set_default("mtf.max_symbols", 10)?
            .set_default("mtf.bar_history_limit", 1000)?

            // Add config file
            .add_source(File::with_name("backtestr").required(false))

            // Add environment variables
            .add_source(Environment::with_prefix("BACKTESTR"))

            .build()?;

        s.try_deserialize()
    }
}
```

## Validation Rules

1. **Memory Limits**: `max_memory_mb` must be >= 100
2. **History Limits**: `bar_history_limit` must be >= 100
3. **Checkpoint Interval**: Must be >= 10 seconds
4. **Compression Level**: Must be 1-9
5. **Batch Sizes**: Must be > 0 and <= 10000

## Per-Story Configuration

### Story 2.0 (Data Model)
- `aggregation.*` - Bar formation rules
- `database.*` - SQLite configuration

### Story 2.1 (MTF Engine)
- `mtf.*` - Core MTF settings
- `mtf.timeframes` - Active timeframes

### Story 2.2 (Indicators)
- `indicators.*` - Pipeline configuration
- `indicators.defaults` - Default parameters

### Story 2.3 (Advanced Bar Formation)
- `sessions.*` - Market hours
- `gaps.*` - Gap detection

### Story 2.4 (Persistence)
- `persistence.*` - Checkpoint settings
- `persistence.recovery` - Recovery options

## Notes

- Configuration is loaded once at startup
- Changes require restart (no hot-reload in Epic 2)
- Invalid configuration prevents startup
- Configuration validation happens before any processing
- All paths are relative to working directory unless absolute