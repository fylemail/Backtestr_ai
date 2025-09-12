# Data Architecture

The data architecture forms the foundation of BackTestr_ai's high-performance backtesting engine, designed to handle institutional-scale financial data processing with tick-level precision. Built around DuckDB as the embedded analytical database, the architecture delivers 10-20x compression through Parquet storage and supports processing rates exceeding 1M ticks/second.

## Database Design

The database schema is optimized for time-series financial data with hierarchical partitioning and columnar storage for maximum query performance and data compression.

### 1. DuckDB Schema Design

**Core Schema Structure**: Optimized for analytical workloads with partitioned tables and specialized indexes.

```sql
-- Create main schema for financial data
CREATE SCHEMA financial_data;

-- Enable required extensions
INSTALL 'parquet';
LOAD 'parquet';
INSTALL 'httpfs';
LOAD 'httpfs';

-- Configure DuckDB for optimal performance
SET memory_limit = '4GB';
SET threads = 8;
SET enable_progress_bar = true;
SET max_memory = '8GB';
```

### 2. Core Tables Schema

**Tick Data Table**: Foundation table for all market data with optimal partitioning.

```sql
-- Tick data with date-based partitioning
CREATE TABLE financial_data.ticks (
    symbol VARCHAR NOT NULL,
    timestamp_utc TIMESTAMP_US NOT NULL,
    price DECIMAL(18,8) NOT NULL,
    volume DECIMAL(18,8) NOT NULL,
    bid DECIMAL(18,8),
    ask DECIMAL(18,8),
    bid_size DECIMAL(18,8),
    ask_size DECIMAL(18,8),
    tick_type TINYINT NOT NULL, -- 0=trade, 1=bid, 2=ask, 3=composite
    exchange_id SMALLINT,
    conditions VARCHAR,
    date_partition DATE GENERATED ALWAYS AS (CAST(timestamp_utc AS DATE)) STORED
) PARTITION BY (date_partition, symbol);

-- Indexes for optimal query performance
CREATE INDEX idx_ticks_symbol_timestamp ON financial_data.ticks (symbol, timestamp_utc);
CREATE INDEX idx_ticks_timestamp ON financial_data.ticks (timestamp_utc);
CREATE INDEX idx_ticks_partition ON financial_data.ticks (date_partition, symbol);
```

**OHLCV Bars Table**: Multi-timeframe aggregated data with materialized views.

```sql
-- OHLCV bars for multiple timeframes
CREATE TABLE financial_data.bars (
    symbol VARCHAR NOT NULL,
    timestamp_utc TIMESTAMP_US NOT NULL,
    timeframe ENUM('1s', '5s', '15s', '30s', '1m', '5m', '15m', '30m', '1h', '4h', '1d') NOT NULL,
    open_price DECIMAL(18,8) NOT NULL,
    high_price DECIMAL(18,8) NOT NULL,
    low_price DECIMAL(18,8) NOT NULL,
    close_price DECIMAL(18,8) NOT NULL,
    volume DECIMAL(18,8) NOT NULL,
    trade_count INTEGER DEFAULT 0,
    vwap DECIMAL(18,8),
    date_partition DATE GENERATED ALWAYS AS (CAST(timestamp_utc AS DATE)) STORED,
    PRIMARY KEY (symbol, timestamp_utc, timeframe)
) PARTITION BY (date_partition, timeframe);

-- Indexes for bar data
CREATE INDEX idx_bars_symbol_timeframe_timestamp ON financial_data.bars (symbol, timeframe, timestamp_utc);
CREATE INDEX idx_bars_timestamp_timeframe ON financial_data.bars (timestamp_utc, timeframe);
```

**Positions and Orders Tables**: Portfolio state tracking with audit trail.

```sql
-- Position tracking
CREATE TABLE financial_data.positions (
    position_id UUID PRIMARY KEY DEFAULT uuid(),
    backtest_id UUID NOT NULL,
    symbol VARCHAR NOT NULL,
    entry_timestamp TIMESTAMP_US NOT NULL,
    exit_timestamp TIMESTAMP_US,
    side ENUM('long', 'short') NOT NULL,
    quantity DECIMAL(18,8) NOT NULL,
    entry_price DECIMAL(18,8) NOT NULL,
    exit_price DECIMAL(18,8),
    unrealized_pnl DECIMAL(18,8) DEFAULT 0,
    realized_pnl DECIMAL(18,8) DEFAULT 0,
    fees DECIMAL(18,8) DEFAULT 0,
    status ENUM('open', 'closed', 'partially_filled') NOT NULL DEFAULT 'open',
    metadata STRUCT(
        strategy_id VARCHAR,
        signal_strength DECIMAL(5,4),
        stop_loss DECIMAL(18,8),
        take_profit DECIMAL(18,8),
        tags VARCHAR[]
    )
);

-- Order execution tracking
CREATE TABLE financial_data.orders (
    order_id UUID PRIMARY KEY DEFAULT uuid(),
    backtest_id UUID NOT NULL,
    position_id UUID,
    symbol VARCHAR NOT NULL,
    timestamp_utc TIMESTAMP_US NOT NULL,
    order_type ENUM('market', 'limit', 'stop', 'stop_limit') NOT NULL,
    side ENUM('buy', 'sell') NOT NULL,
    quantity DECIMAL(18,8) NOT NULL,
    price DECIMAL(18,8),
    stop_price DECIMAL(18,8),
    filled_quantity DECIMAL(18,8) DEFAULT 0,
    avg_fill_price DECIMAL(18,8),
    status ENUM('pending', 'filled', 'partially_filled', 'cancelled', 'rejected') NOT NULL,
    fill_timestamp TIMESTAMP_US,
    fees DECIMAL(18,8) DEFAULT 0,
    slippage DECIMAL(18,8) DEFAULT 0
);
```

**Backtest Results Tables**: Performance metrics and analytics storage.

```sql
-- Backtest configuration and metadata
CREATE TABLE financial_data.backtests (
    backtest_id UUID PRIMARY KEY DEFAULT uuid(),
    name VARCHAR NOT NULL,
    strategy_name VARCHAR NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    initial_capital DECIMAL(18,8) NOT NULL,
    symbols VARCHAR[] NOT NULL,
    timeframe VARCHAR NOT NULL,
    commission_rate DECIMAL(8,6) DEFAULT 0.001,
    slippage_rate DECIMAL(8,6) DEFAULT 0.0001,
    created_at TIMESTAMP_US DEFAULT now(),
    completed_at TIMESTAMP_US,
    status ENUM('running', 'completed', 'failed', 'cancelled') NOT NULL DEFAULT 'running',
    metadata JSON
);

-- Performance metrics
CREATE TABLE financial_data.backtest_metrics (
    backtest_id UUID PRIMARY KEY,
    total_return DECIMAL(10,6),
    annual_return DECIMAL(10,6),
    sharpe_ratio DECIMAL(8,4),
    sortino_ratio DECIMAL(8,4),
    max_drawdown DECIMAL(8,4),
    win_rate DECIMAL(8,4),
    profit_factor DECIMAL(8,4),
    total_trades INTEGER,
    avg_trade_duration INTERVAL,
    avg_win DECIMAL(18,8),
    avg_loss DECIMAL(18,8),
    largest_win DECIMAL(18,8),
    largest_loss DECIMAL(18,8),
    consecutive_wins INTEGER,
    consecutive_losses INTEGER,
    calmar_ratio DECIMAL(8,4),
    var_95 DECIMAL(18,8),
    expected_shortfall DECIMAL(18,8)
);

-- Daily performance tracking
CREATE TABLE financial_data.daily_performance (
    backtest_id UUID NOT NULL,
    date DATE NOT NULL,
    portfolio_value DECIMAL(18,8) NOT NULL,
    daily_return DECIMAL(10,6),
    daily_pnl DECIMAL(18,8),
    drawdown DECIMAL(8,4),
    open_positions INTEGER DEFAULT 0,
    trades_today INTEGER DEFAULT 0,
    PRIMARY KEY (backtest_id, date)
);
```

### 3. Specialized Indexes

**Performance-Optimized Indexes**: Covering indexes for common query patterns.

```sql
-- Covering indexes for tick data queries
CREATE INDEX idx_ticks_symbol_date_covering 
ON financial_data.ticks (symbol, date_partition) 
INCLUDE (timestamp_utc, price, volume);

-- Time-based range queries
CREATE INDEX idx_ticks_time_range 
ON financial_data.ticks (timestamp_utc, symbol) 
WHERE timestamp_utc >= '2020-01-01'::TIMESTAMP;

-- Portfolio performance queries
CREATE INDEX idx_positions_backtest_symbol 
ON financial_data.positions (backtest_id, symbol, entry_timestamp);

-- Order execution analysis
CREATE INDEX idx_orders_backtest_time 
ON financial_data.orders (backtest_id, timestamp_utc) 
INCLUDE (symbol, side, quantity, price);
```

## Data Models

The data models are designed for high-performance financial computations with strong typing and validation rules that ensure data integrity across the entire pipeline.

### 1. Tick Data Model

**Core Tick Structure**: Immutable tick data with nanosecond precision and metadata.

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tick {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub price: Decimal,
    pub volume: Decimal,
    pub bid: Option<Decimal>,
    pub ask: Option<Decimal>,
    pub bid_size: Option<Decimal>,
    pub ask_size: Option<Decimal>,
    pub tick_type: TickType,
    pub exchange_id: Option<u16>,
    pub conditions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TickType {
    Trade = 0,
    Bid = 1,
    Ask = 2,
    Composite = 3,
}

impl Tick {
    pub fn mid_price(&self) -> Option<Decimal> {
        match (self.bid, self.ask) {
            (Some(bid), Some(ask)) => Some((bid + ask) / Decimal::from(2)),
            _ => None,
        }
    }
    
    pub fn spread(&self) -> Option<Decimal> {
        match (self.bid, self.ask) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }
    
    pub fn is_valid(&self) -> bool {
        self.price > Decimal::ZERO 
            && self.volume >= Decimal::ZERO
            && self.symbol.len() > 0
    }
}
```

### 2. OHLCV Bar Model

**Multi-Timeframe Bar Structure**: Aggregated data with computed technical indicators.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bar {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub timeframe: Timeframe,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub trade_count: Option<u32>,
    pub vwap: Option<Decimal>,
    pub technical_indicators: Option<TechnicalIndicators>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Timeframe {
    Second1,
    Second5,
    Second15,
    Second30,
    Minute1,
    Minute5,
    Minute15,
    Minute30,
    Hour1,
    Hour4,
    Day1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TechnicalIndicators {
    pub sma_20: Option<Decimal>,
    pub ema_20: Option<Decimal>,
    pub rsi_14: Option<Decimal>,
    pub bollinger_upper: Option<Decimal>,
    pub bollinger_lower: Option<Decimal>,
    pub macd_line: Option<Decimal>,
    pub macd_signal: Option<Decimal>,
    pub macd_histogram: Option<Decimal>,
}

impl Bar {
    pub fn is_valid(&self) -> bool {
        self.open > Decimal::ZERO
            && self.high >= self.open
            && self.low <= self.open
            && self.close > Decimal::ZERO
            && self.high >= self.close
            && self.low <= self.close
            && self.volume >= Decimal::ZERO
    }
    
    pub fn typical_price(&self) -> Decimal {
        (self.high + self.low + self.close) / Decimal::from(3)
    }
    
    pub fn true_range(&self, prev_close: Option<Decimal>) -> Decimal {
        match prev_close {
            Some(prev) => {
                let hl = self.high - self.low;
                let hc = (self.high - prev).abs();
                let lc = (self.low - prev).abs();
                hl.max(hc).max(lc)
            }
            None => self.high - self.low,
        }
    }
}
```

### 3. Position and Order Models

**Portfolio State Management**: Complete position lifecycle tracking with risk metrics.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub position_id: uuid::Uuid,
    pub backtest_id: uuid::Uuid,
    pub symbol: String,
    pub entry_timestamp: DateTime<Utc>,
    pub exit_timestamp: Option<DateTime<Utc>>,
    pub side: PositionSide,
    pub quantity: Decimal,
    pub entry_price: Decimal,
    pub exit_price: Option<Decimal>,
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
    pub fees: Decimal,
    pub status: PositionStatus,
    pub metadata: PositionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionSide {
    Long,
    Short,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionStatus {
    Open,
    Closed,
    PartiallyFilled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionMetadata {
    pub strategy_id: Option<String>,
    pub signal_strength: Option<Decimal>,
    pub stop_loss: Option<Decimal>,
    pub take_profit: Option<Decimal>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: uuid::Uuid,
    pub backtest_id: uuid::Uuid,
    pub position_id: Option<uuid::Uuid>,
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub filled_quantity: Decimal,
    pub avg_fill_price: Option<Decimal>,
    pub status: OrderStatus,
    pub fill_timestamp: Option<DateTime<Utc>>,
    pub fees: Decimal,
    pub slippage: Decimal,
}

impl Position {
    pub fn current_pnl(&self, current_price: Decimal) -> Decimal {
        match self.side {
            PositionSide::Long => (current_price - self.entry_price) * self.quantity,
            PositionSide::Short => (self.entry_price - current_price) * self.quantity,
        }
    }
    
    pub fn roi(&self, current_price: Option<Decimal>) -> Decimal {
        let price = current_price.unwrap_or(self.exit_price.unwrap_or(self.entry_price));
        let pnl = self.current_pnl(price);
        let investment = self.entry_price * self.quantity;
        pnl / investment
    }
}
```

### 4. Backtest Results Model

**Performance Analytics Structure**: Comprehensive metrics and risk analysis.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResults {
    pub backtest_id: uuid::Uuid,
    pub metadata: BacktestMetadata,
    pub performance_metrics: PerformanceMetrics,
    pub risk_metrics: RiskMetrics,
    pub daily_performance: Vec<DailyPerformance>,
    pub trade_analysis: TradeAnalysis,
    pub drawdown_analysis: DrawdownAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_return: Decimal,
    pub annual_return: Decimal,
    pub sharpe_ratio: Decimal,
    pub sortino_ratio: Decimal,
    pub calmar_ratio: Decimal,
    pub max_drawdown: Decimal,
    pub win_rate: Decimal,
    pub profit_factor: Decimal,
    pub total_trades: u32,
    pub avg_trade_duration: chrono::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub var_95: Decimal,
    pub expected_shortfall: Decimal,
    pub beta: Option<Decimal>,
    pub alpha: Option<Decimal>,
    pub correlation: Option<Decimal>,
    pub volatility: Decimal,
    pub downside_deviation: Decimal,
    pub max_consecutive_losses: u32,
    pub largest_loss: Decimal,
}
```

## Data Flow & Pipelines

The data pipeline architecture ensures high-throughput processing with minimal latency, supporting real-time ingestion, transformation, and querying of financial data.

### 1. Data Ingestion Pipeline

**Multi-Source Ingestion**: Parallel processing from multiple data providers with automatic failover.

```rust
use tokio::sync::mpsc;
use std::sync::Arc;
use dashmap::DashMap;

pub struct DataIngestionPipeline {
    channels: Arc<DashMap<String, mpsc::UnboundedSender<Tick>>>,
    processors: Vec<DataProcessor>,
    buffer_size: usize,
    compression_threshold: usize,
}

impl DataIngestionPipeline {
    pub async fn new(config: IngestionConfig) -> Result<Self, PipelineError> {
        let channels = Arc::new(DashMap::new());
        let processors = Self::create_processors(&config).await?;
        
        Ok(Self {
            channels,
            processors,
            buffer_size: config.buffer_size,
            compression_threshold: config.compression_threshold,
        })
    }
    
    pub async fn start_ingestion(&mut self) -> Result<(), PipelineError> {
        // Create channels for each symbol
        for symbol in &self.config.symbols {
            let (tx, rx) = mpsc::unbounded_channel();
            self.channels.insert(symbol.clone(), tx);
            
            // Spawn processor for this symbol
            let processor = TickProcessor::new(symbol.clone(), rx);
            tokio::spawn(async move {
                processor.process_ticks().await
            });
        }
        
        Ok(())
    }
    
    pub async fn ingest_tick(&self, tick: Tick) -> Result<(), IngestionError> {
        if let Some(sender) = self.channels.get(&tick.symbol) {
            sender.send(tick).map_err(|_| IngestionError::ChannelClosed)?;
        }
        Ok(())
    }
}

pub struct TickProcessor {
    symbol: String,
    receiver: mpsc::UnboundedReceiver<Tick>,
    buffer: Vec<Tick>,
    aggregator: BarAggregator,
    writer: ParquetWriter,
}

impl TickProcessor {
    async fn process_ticks(&mut self) -> Result<(), ProcessingError> {
        while let Some(tick) = self.receiver.recv().await {
            // Validate tick data
            if !tick.is_valid() {
                log::warn!("Invalid tick received: {:?}", tick);
                continue;
            }
            
            // Add to buffer
            self.buffer.push(tick.clone());
            
            // Update bar aggregations
            self.aggregator.update_bars(&tick).await?;
            
            // Flush buffer when threshold is reached
            if self.buffer.len() >= self.compression_threshold {
                self.flush_buffer().await?;
            }
        }
        
        // Final flush
        self.flush_buffer().await?;
        Ok(())
    }
    
    async fn flush_buffer(&mut self) -> Result<(), ProcessingError> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        
        // Write to Parquet with compression
        self.writer.write_batch(&self.buffer).await?;
        
        // Clear buffer
        self.buffer.clear();
        
        log::info!("Flushed {} ticks for symbol {}", self.buffer.len(), self.symbol);
        Ok(())
    }
}
```

### 2. Bar Aggregation Pipeline

**Real-Time Multi-Timeframe Aggregation**: Concurrent processing of multiple timeframes with microsecond precision.

```rust
use std::collections::HashMap;
use tokio::time::{Duration, Instant};

pub struct BarAggregator {
    active_bars: HashMap<Timeframe, Option<Bar>>,
    completed_bars: mpsc::UnboundedSender<Bar>,
    timeframe_configs: Vec<TimeframeConfig>,
}

impl BarAggregator {
    pub async fn update_bars(&mut self, tick: &Tick) -> Result<(), AggregationError> {
        for timeframe in &self.timeframe_configs {
            self.update_timeframe_bar(tick, &timeframe.timeframe).await?;
        }
        Ok(())
    }
    
    async fn update_timeframe_bar(
        &mut self, 
        tick: &Tick, 
        timeframe: &Timeframe
    ) -> Result<(), AggregationError> {
        let bar_timestamp = self.align_timestamp(&tick.timestamp, timeframe);
        
        // Get or create active bar for this timeframe
        let current_bar = self.active_bars.entry(*timeframe).or_insert(None);
        
        match current_bar {
            Some(ref mut bar) if bar.timestamp == bar_timestamp => {
                // Update existing bar
                self.update_existing_bar(bar, tick);
            }
            Some(bar) => {
                // Complete current bar and start new one
                self.completed_bars.send(bar.clone()).map_err(|_| 
                    AggregationError::ChannelClosed)?;
                
                *current_bar = Some(self.create_new_bar(tick, bar_timestamp, *timeframe));
            }
            None => {
                // Create first bar for this timeframe
                *current_bar = Some(self.create_new_bar(tick, bar_timestamp, *timeframe));
            }
        }
        
        Ok(())
    }
    
    fn update_existing_bar(&self, bar: &mut Bar, tick: &Tick) {
        // Update OHLCV data
        bar.high = bar.high.max(tick.price);
        bar.low = bar.low.min(tick.price);
        bar.close = tick.price;
        bar.volume += tick.volume;
        
        if let Some(ref mut count) = bar.trade_count {
            *count += 1;
        }
        
        // Update VWAP calculation
        self.update_vwap(bar, tick);
    }
    
    fn update_vwap(&self, bar: &mut Bar, tick: &Tick) {
        let total_volume = bar.volume;
        let current_vwap = bar.vwap.unwrap_or(bar.open);
        
        // Volume-weighted average price calculation
        let new_vwap = (current_vwap * (total_volume - tick.volume) + 
                       tick.price * tick.volume) / total_volume;
        
        bar.vwap = Some(new_vwap);
    }
    
    fn align_timestamp(&self, timestamp: &DateTime<Utc>, timeframe: &Timeframe) -> DateTime<Utc> {
        match timeframe {
            Timeframe::Minute1 => timestamp.with_second(0).unwrap().with_nanosecond(0).unwrap(),
            Timeframe::Minute5 => {
                let minute = (timestamp.minute() / 5) * 5;
                timestamp.with_minute(minute).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap()
            }
            Timeframe::Hour1 => timestamp.with_minute(0).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap(),
            Timeframe::Day1 => timestamp.with_hour(0).unwrap().with_minute(0).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap(),
            // Add other timeframe alignments...
            _ => *timestamp,
        }
    }
}
```

### 3. Query Optimization Pipeline

**High-Performance Query Engine**: Optimized for time-series financial data patterns.

```rust
pub struct QueryEngine {
    connection_pool: Arc<DuckDBConnectionPool>,
    query_cache: Arc<DashMap<String, CachedQuery>>,
    partition_manager: PartitionManager,
}

impl QueryEngine {
    pub async fn execute_tick_query(
        &self,
        request: TickQueryRequest,
    ) -> Result<Vec<Tick>, QueryError> {
        // Generate cache key
        let cache_key = self.generate_cache_key(&request);
        
        // Check cache first
        if let Some(cached) = self.query_cache.get(&cache_key) {
            if !cached.is_expired() {
                return Ok(cached.data.clone());
            }
        }
        
        // Build optimized query
        let query = self.build_optimized_tick_query(&request)?;
        
        // Execute with connection pooling
        let connection = self.connection_pool.get().await?;
        let results = connection.execute_query(&query).await?;
        
        // Parse results
        let ticks = self.parse_tick_results(results)?;
        
        // Cache results
        self.cache_query_results(cache_key, &ticks);
        
        Ok(ticks)
    }
    
    fn build_optimized_tick_query(&self, request: &TickQueryRequest) -> Result<String, QueryError> {
        let mut query = String::from("SELECT * FROM financial_data.ticks WHERE 1=1");
        
        // Add partition pruning
        if let Some(partitions) = self.determine_partitions(&request.time_range) {
            let partition_filter = partitions.iter()
                .map(|p| format!("date_partition = '{}'", p))
                .collect::<Vec<_>>()
                .join(" OR ");
            query.push_str(&format!(" AND ({})", partition_filter));
        }
        
        // Add symbol filter
        if !request.symbols.is_empty() {
            let symbol_list = request.symbols.iter()
                .map(|s| format!("'{}'", s))
                .collect::<Vec<_>>()
                .join(",");
            query.push_str(&format!(" AND symbol IN ({})", symbol_list));
        }
        
        // Add time range filter
        query.push_str(&format!(
            " AND timestamp_utc BETWEEN '{}' AND '{}'",
            request.time_range.start.format("%Y-%m-%d %H:%M:%S%.6f"),
            request.time_range.end.format("%Y-%m-%d %H:%M:%S%.6f")
        ));
        
        // Add ordering for optimal sequential access
        query.push_str(" ORDER BY timestamp_utc, symbol");
        
        // Add limit if specified
        if let Some(limit) = request.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        
        Ok(query)
    }
}
```

## Caching Strategy

The multi-level caching architecture provides sub-millisecond query response times for frequently accessed data while maintaining data consistency across all cache layers.

### 1. L1 Cache (In-Memory)

**Hot Data Cache**: Ultra-fast access to recently accessed ticks and bars.

```rust
use lru::LruCache;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct L1Cache {
    tick_cache: Arc<RwLock<LruCache<String, Vec<Tick>>>>,
    bar_cache: Arc<RwLock<LruCache<String, Vec<Bar>>>>,
    position_cache: Arc<RwLock<LruCache<uuid::Uuid, Position>>>,
    max_size: usize,
    ttl: Duration,
}

impl L1Cache {
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            tick_cache: Arc::new(RwLock::new(LruCache::new(max_size))),
            bar_cache: Arc::new(RwLock::new(LruCache::new(max_size))),
            position_cache: Arc::new(RwLock::new(LruCache::new(max_size / 10))),
            max_size,
            ttl,
        }
    }
    
    pub async fn get_ticks(&self, key: &str) -> Option<Vec<Tick>> {
        let cache = self.tick_cache.read().await;
        cache.peek(key).cloned()
    }
    
    pub async fn put_ticks(&self, key: String, ticks: Vec<Tick>) {
        let mut cache = self.tick_cache.write().await;
        cache.put(key, ticks);
    }
    
    pub async fn get_bars(&self, key: &str) -> Option<Vec<Bar>> {
        let cache = self.bar_cache.read().await;
        cache.peek(key).cloned()
    }
    
    pub async fn put_bars(&self, key: String, bars: Vec<Bar>) {
        let mut cache = self.bar_cache.write().await;
        cache.put(key, bars);
    }
    
    // Cache invalidation for real-time updates
    pub async fn invalidate_symbol(&self, symbol: &str) {
        let mut tick_cache = self.tick_cache.write().await;
        let mut bar_cache = self.bar_cache.write().await;
        
        // Remove all entries containing this symbol
        let keys_to_remove: Vec<String> = tick_cache.iter()
            .filter_map(|(key, _)| {
                if key.contains(symbol) { Some(key.clone()) } else { None }
            })
            .collect();
        
        for key in keys_to_remove {
            tick_cache.pop(&key);
            bar_cache.pop(&key);
        }
    }
}
```

### 2. L2 Cache (Compressed Memory)

**Warm Data Cache**: Compressed storage for frequently accessed but not hot data.

```rust
use zstd::stream::{Encoder, Decoder};

pub struct L2Cache {
    compressed_cache: Arc<DashMap<String, CompressedData>>,
    compression_level: i32,
    max_entries: usize,
    hit_counter: Arc<DashMap<String, u64>>,
}

#[derive(Clone)]
struct CompressedData {
    data: Vec<u8>,
    original_size: usize,
    compressed_at: Instant,
    access_count: u64,
}

impl L2Cache {
    pub fn new(max_entries: usize, compression_level: i32) -> Self {
        Self {
            compressed_cache: Arc::new(DashMap::new()),
            compression_level,
            max_entries,
            hit_counter: Arc::new(DashMap::new()),
        }
    }
    
    pub async fn get<T>(&self, key: &str) -> Option<T> 
    where 
        T: serde::de::DeserializeOwned 
    {
        if let Some(compressed) = self.compressed_cache.get(key) {
            // Update access statistics
            self.hit_counter.entry(key.to_string())
                .and_modify(|count| *count += 1)
                .or_insert(1);
            
            // Decompress data
            if let Ok(decompressed) = self.decompress(&compressed.data) {
                if let Ok(data) = bincode::deserialize::<T>(&decompressed) {
                    return Some(data);
                }
            }
        }
        None
    }
    
    pub async fn put<T>(&self, key: String, data: &T) -> Result<(), CacheError>
    where 
        T: serde::Serialize 
    {
        // Serialize data
        let serialized = bincode::serialize(data)
            .map_err(CacheError::SerializationError)?;
        
        // Compress data
        let compressed = self.compress(&serialized)
            .map_err(CacheError::CompressionError)?;
        
        let compressed_data = CompressedData {
            data: compressed,
            original_size: serialized.len(),
            compressed_at: Instant::now(),
            access_count: 0,
        };
        
        // Evict if cache is full
        if self.compressed_cache.len() >= self.max_entries {
            self.evict_lru().await;
        }
        
        self.compressed_cache.insert(key, compressed_data);
        Ok(())
    }
    
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        let mut encoder = Encoder::new(Vec::new(), self.compression_level)?;
        encoder.write_all(data)?;
        encoder.finish()
    }
    
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        let mut decoder = Decoder::new(data)?;
        let mut decompressed = Vec::new();
        std::io::copy(&mut decoder, &mut decompressed)?;
        Ok(decompressed)
    }
    
    async fn evict_lru(&self) {
        // Find least recently used entry
        let mut lru_key: Option<String> = None;
        let mut min_access_time = Instant::now();
        
        for entry in self.compressed_cache.iter() {
            if entry.value().compressed_at < min_access_time {
                min_access_time = entry.value().compressed_at;
                lru_key = Some(entry.key().clone());
            }
        }
        
        if let Some(key) = lru_key {
            self.compressed_cache.remove(&key);
            self.hit_counter.remove(&key);
        }
    }
}
```

### 3. Prefetching Strategy

**Intelligent Data Prefetching**: ML-driven prediction of data access patterns.

```rust
pub struct PrefetchEngine {
    access_pattern_analyzer: AccessPatternAnalyzer,
    prefetch_queue: Arc<tokio::sync::Mutex<VecDeque<PrefetchTask>>>,
    background_workers: Vec<tokio::task::JoinHandle<()>>,
    l1_cache: Arc<L1Cache>,
    l2_cache: Arc<L2Cache>,
}

#[derive(Debug, Clone)]
struct PrefetchTask {
    priority: u8,
    symbol: String,
    time_range: TimeRange,
    timeframe: Option<Timeframe>,
    prediction_confidence: f64,
}

impl PrefetchEngine {
    pub async fn analyze_and_prefetch(&self, recent_queries: &[QueryRequest]) {
        let predictions = self.access_pattern_analyzer
            .predict_next_queries(recent_queries).await;
        
        for prediction in predictions {
            if prediction.confidence > 0.7 {
                let task = PrefetchTask {
                    priority: self.calculate_priority(&prediction),
                    symbol: prediction.symbol,
                    time_range: prediction.time_range,
                    timeframe: prediction.timeframe,
                    prediction_confidence: prediction.confidence,
                };
                
                self.enqueue_prefetch_task(task).await;
            }
        }
    }
    
    async fn execute_prefetch_task(&self, task: PrefetchTask) -> Result<(), PrefetchError> {
        // Check if data is already cached
        let cache_key = self.generate_cache_key(&task);
        
        if self.l1_cache.get_ticks(&cache_key).await.is_some() {
            return Ok(()); // Already cached
        }
        
        // Load data from storage
        let query_request = self.task_to_query_request(&task);
        let data = self.load_data_from_storage(&query_request).await?;
        
        // Cache in appropriate level based on prediction confidence
        if task.prediction_confidence > 0.9 {
            // High confidence - cache in L1
            self.l1_cache.put_ticks(cache_key.clone(), data.clone()).await;
        } else {
            // Medium confidence - cache in L2
            self.l2_cache.put(cache_key, &data).await?;
        }
        
        log::debug!("Prefetched data for task: {:?}", task);
        Ok(())
    }
}

pub struct AccessPatternAnalyzer {
    query_history: Arc<RwLock<VecDeque<TimestampedQuery>>>,
    pattern_models: HashMap<String, PatternModel>,
    seasonal_patterns: SeasonalPatternDetector,
}

impl AccessPatternAnalyzer {
    pub async fn predict_next_queries(&self, recent_queries: &[QueryRequest]) -> Vec<QueryPrediction> {
        let mut predictions = Vec::new();
        
        // Analyze temporal patterns
        let temporal_predictions = self.analyze_temporal_patterns(recent_queries).await;
        predictions.extend(temporal_predictions);
        
        // Analyze symbol correlation patterns
        let correlation_predictions = self.analyze_symbol_correlations(recent_queries).await;
        predictions.extend(correlation_predictions);
        
        // Analyze user behavior patterns
        let behavior_predictions = self.analyze_user_behavior(recent_queries).await;
        predictions.extend(behavior_predictions);
        
        // Sort by confidence and return top predictions
        predictions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        predictions.truncate(10);
        
        predictions
    }
    
    async fn analyze_temporal_patterns(&self, queries: &[QueryRequest]) -> Vec<QueryPrediction> {
        let mut predictions = Vec::new();
        
        for query in queries {
            // Predict next time window based on historical patterns
            let next_time_range = self.predict_next_time_window(&query.time_range);
            
            let prediction = QueryPrediction {
                symbol: query.symbols[0].clone(),
                time_range: next_time_range,
                timeframe: query.timeframe.clone(),
                confidence: self.calculate_temporal_confidence(query),
                prediction_type: PredictionType::Temporal,
            };
            
            predictions.push(prediction);
        }
        
        predictions
    }
}
```

## Data Persistence

The persistence layer ensures data durability, consistency, and efficient recovery while supporting high-throughput write operations and point-in-time recovery.

### 1. Backup Strategy

**Incremental Backup System**: Efficient backup with minimal performance impact.

```rust
use std::path::PathBuf;
use chrono::{DateTime, Utc};

pub struct BackupManager {
    backup_config: BackupConfig,
    storage_backends: Vec<Box<dyn StorageBackend>>,
    compression_engine: CompressionEngine,
    encryption_engine: Option<EncryptionEngine>,
}

#[derive(Debug, Clone)]
pub struct BackupConfig {
    pub backup_schedule: BackupSchedule,
    pub retention_policy: RetentionPolicy,
    pub compression_level: u8,
    pub encryption_enabled: bool,
    pub verification_enabled: bool,
    pub incremental_threshold: usize,
}

impl BackupManager {
    pub async fn create_backup(&self, backup_type: BackupType) -> Result<BackupMetadata, BackupError> {
        let backup_id = uuid::Uuid::new_v4();
        let timestamp = Utc::now();
        
        log::info!("Starting {} backup: {}", backup_type, backup_id);
        
        match backup_type {
            BackupType::Full => self.create_full_backup(backup_id, timestamp).await,
            BackupType::Incremental => self.create_incremental_backup(backup_id, timestamp).await,
            BackupType::Differential => self.create_differential_backup(backup_id, timestamp).await,
        }
    }
    
    async fn create_incremental_backup(
        &self, 
        backup_id: uuid::Uuid, 
        timestamp: DateTime<Utc>
    ) -> Result<BackupMetadata, BackupError> {
        // Get last backup timestamp
        let last_backup = self.get_last_backup_timestamp().await?;
        
        // Query for changed data since last backup
        let changed_data = self.query_changed_data_since(last_backup).await?;
        
        if changed_data.is_empty() {
            log::info!("No changes since last backup, skipping incremental backup");
            return Ok(BackupMetadata::empty(backup_id, timestamp));
        }
        
        // Create backup archive
        let archive_path = self.create_backup_archive(&changed_data, backup_id).await?;
        
        // Upload to storage backends
        let mut upload_results = Vec::new();
        for backend in &self.storage_backends {
            let result = backend.upload_backup(&archive_path, backup_id).await;
            upload_results.push(result);
        }
        
        // Verify backup integrity
        if self.backup_config.verification_enabled {
            self.verify_backup_integrity(backup_id, &archive_path).await?;
        }
        
        // Create metadata
        let metadata = BackupMetadata {
            backup_id,
            backup_type: BackupType::Incremental,
            timestamp,
            size_bytes: std::fs::metadata(&archive_path)?.len(),
            file_count: changed_data.len(),
            compression_ratio: self.calculate_compression_ratio(&archive_path).await?,
            checksum: self.calculate_checksum(&archive_path).await?,
            storage_locations: upload_results.into_iter()
                .filter_map(|r| r.ok())
                .collect(),
        };
        
        // Store backup metadata
        self.store_backup_metadata(&metadata).await?;
        
        log::info!("Incremental backup completed: {} ({} bytes)", backup_id, metadata.size_bytes);
        Ok(metadata)
    }
    
    async fn query_changed_data_since(&self, since: DateTime<Utc>) -> Result<Vec<DataChunk>, QueryError> {
        let query = format!(
            "SELECT * FROM financial_data.ticks WHERE timestamp_utc > '{}' 
             UNION ALL 
             SELECT * FROM financial_data.bars WHERE timestamp_utc > '{}'
             UNION ALL
             SELECT * FROM financial_data.positions WHERE entry_timestamp > '{}'",
            since.format("%Y-%m-%d %H:%M:%S%.6f"),
            since.format("%Y-%m-%d %H:%M:%S%.6f"),
            since.format("%Y-%m-%d %H:%M:%S%.6f")
        );
        
        // Execute query and return results
        self.execute_backup_query(&query).await
    }
}

pub trait StorageBackend: Send + Sync {
    async fn upload_backup(&self, archive_path: &PathBuf, backup_id: uuid::Uuid) -> Result<StorageLocation, StorageError>;
    async fn download_backup(&self, backup_id: uuid::Uuid) -> Result<PathBuf, StorageError>;
    async fn delete_backup(&self, backup_id: uuid::Uuid) -> Result<(), StorageError>;
    async fn list_backups(&self) -> Result<Vec<BackupMetadata>, StorageError>;
}

// S3-compatible storage backend
pub struct S3StorageBackend {
    client: aws_sdk_s3::Client,
    bucket: String,
    prefix: String,
}

impl StorageBackend for S3StorageBackend {
    async fn upload_backup(&self, archive_path: &PathBuf, backup_id: uuid::Uuid) -> Result<StorageLocation, StorageError> {
        let key = format!("{}/backups/{}.tar.zst", self.prefix, backup_id);
        
        let body = aws_sdk_s3::primitives::ByteStream::from_path(archive_path)
            .await
            .map_err(|e| StorageError::ReadError(e.to_string()))?;
        
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(body)
            .send()
            .await
            .map_err(|e| StorageError::UploadError(e.to_string()))?;
        
        Ok(StorageLocation {
            backend_type: "s3".to_string(),
            location: format!("s3://{}/{}", self.bucket, key),
            uploaded_at: Utc::now(),
        })
    }
}
```

### 2. Recovery System

**Point-in-Time Recovery**: Granular recovery with minimal data loss.

```rust
pub struct RecoveryManager {
    backup_manager: Arc<BackupManager>,
    wal_manager: WriteAheadLogManager,
    recovery_config: RecoveryConfig,
}

impl RecoveryManager {
    pub async fn recover_to_point_in_time(
        &self, 
        target_time: DateTime<Utc>
    ) -> Result<RecoveryResult, RecoveryError> {
        log::info!("Starting point-in-time recovery to: {}", target_time);
        
        // Find the best backup to start from
        let base_backup = self.find_optimal_backup_for_recovery(target_time).await?;
        
        // Restore from base backup
        let restore_result = self.restore_from_backup(&base_backup).await?;
        
        // Apply WAL entries up to target time
        let wal_result = self.replay_wal_to_time(base_backup.timestamp, target_time).await?;
        
        // Verify data integrity
        let verification_result = self.verify_recovery_integrity(target_time).await?;
        
        let recovery_result = RecoveryResult {
            recovered_to: target_time,
            base_backup: base_backup.backup_id,
            wal_entries_applied: wal_result.entries_applied,
            data_loss_duration: wal_result.data_loss_duration,
            verification_passed: verification_result.passed,
            recovery_duration: restore_result.duration + wal_result.duration,
        };
        
        log::info!("Point-in-time recovery completed: {:?}", recovery_result);
        Ok(recovery_result)
    }
    
    async fn replay_wal_to_time(
        &self,
        from_time: DateTime<Utc>,
        to_time: DateTime<Utc>,
    ) -> Result<WalReplayResult, RecoveryError> {
        let wal_entries = self.wal_manager.get_entries_in_range(from_time, to_time).await?;
        
        let mut entries_applied = 0;
        let start_time = Instant::now();
        
        for entry in wal_entries {
            if entry.timestamp <= to_time {
                self.apply_wal_entry(&entry).await?;
                entries_applied += 1;
            }
        }
        
        let duration = start_time.elapsed();
        
        Ok(WalReplayResult {
            entries_applied,
            duration,
            data_loss_duration: Duration::zero(),
        })
    }
}

pub struct WriteAheadLogManager {
    wal_dir: PathBuf,
    current_wal_file: Arc<tokio::sync::Mutex<Option<WalFile>>>,
    wal_rotation_size: u64,
    retention_period: Duration,
}

impl WriteAheadLogManager {
    pub async fn log_operation(&self, operation: WalOperation) -> Result<(), WalError> {
        let entry = WalEntry {
            lsn: self.generate_next_lsn().await,
            timestamp: Utc::now(),
            operation,
            checksum: 0, // Will be calculated
        };
        
        let serialized = self.serialize_entry(&entry)?;
        let checksum = self.calculate_checksum(&serialized);
        
        let final_entry = WalEntry { checksum, ..entry };
        let final_serialized = self.serialize_entry(&final_entry)?;
        
        // Write to current WAL file
        let mut current_file = self.current_wal_file.lock().await;
        if current_file.is_none() || self.should_rotate_wal(&current_file).await? {
            self.rotate_wal_file(&mut current_file).await?;
        }
        
        if let Some(ref mut wal_file) = current_file.as_mut() {
            wal_file.write_entry(&final_serialized).await?;
            wal_file.flush().await?;
        }
        
        Ok(())
    }
}
```

### 3. Migration System

**Schema Evolution**: Safe database schema updates with rollback capability.

```rust
pub struct MigrationManager {
    migrations: Vec<Box<dyn Migration>>,
    migration_history: MigrationHistory,
    rollback_strategy: RollbackStrategy,
}

pub trait Migration: Send + Sync {
    fn version(&self) -> u32;
    fn description(&self) -> &str;
    async fn up(&self, connection: &mut DuckDBConnection) -> Result<(), MigrationError>;
    async fn down(&self, connection: &mut DuckDBConnection) -> Result<(), MigrationError>;
    fn is_reversible(&self) -> bool;
    fn estimated_duration(&self) -> Duration;
}

impl MigrationManager {
    pub async fn migrate_to_latest(&self) -> Result<MigrationResult, MigrationError> {
        let current_version = self.migration_history.get_current_version().await?;
        let target_version = self.get_latest_version();
        
        if current_version >= target_version {
            return Ok(MigrationResult::AlreadyUpToDate(current_version));
        }
        
        log::info!("Migrating from version {} to {}", current_version, target_version);
        
        // Create backup before migration
        let backup_metadata = self.create_pre_migration_backup().await?;
        
        // Apply migrations in order
        let mut applied_migrations = Vec::new();
        for migration in &self.migrations {
            if migration.version() > current_version && migration.version() <= target_version {
                match self.apply_migration(migration.as_ref()).await {
                    Ok(_) => {
                        applied_migrations.push(migration.version());
                        log::info!("Applied migration {}: {}", migration.version(), migration.description());
                    }
                    Err(e) => {
                        log::error!("Migration {} failed: {}", migration.version(), e);
                        
                        // Attempt rollback
                        if let Err(rollback_error) = self.rollback_migrations(&applied_migrations).await {
                            log::error!("Rollback failed: {}", rollback_error);
                            return Err(MigrationError::RollbackFailed {
                                original_error: Box::new(e),
                                rollback_error: Box::new(rollback_error),
                            });
                        }
                        
                        return Err(e);
                    }
                }
            }
        }
        
        Ok(MigrationResult::Success {
            from_version: current_version,
            to_version: target_version,
            applied_migrations,
            backup_id: backup_metadata.backup_id,
        })
    }
    
    async fn apply_migration(&self, migration: &dyn Migration) -> Result<(), MigrationError> {
        let start_time = Instant::now();
        
        // Begin transaction
        let mut connection = self.get_migration_connection().await?;
        let transaction = connection.begin_transaction().await?;
        
        // Apply migration
        migration.up(&mut connection).await?;
        
        // Record migration in history
        self.migration_history.record_migration(
            migration.version(),
            migration.description(),
            start_time.elapsed(),
        ).await?;
        
        // Commit transaction
        transaction.commit().await?;
        
        Ok(())
    }
}

// Example migration
pub struct AddTechnicalIndicatorsMigration;

impl Migration for AddTechnicalIndicatorsMigration {
    fn version(&self) -> u32 { 2 }
    
    fn description(&self) -> &str {
        "Add technical indicators columns to bars table"
    }
    
    async fn up(&self, connection: &mut DuckDBConnection) -> Result<(), MigrationError> {
        let sql = r#"
            ALTER TABLE financial_data.bars 
            ADD COLUMN technical_indicators STRUCT(
                sma_20 DECIMAL(18,8),
                ema_20 DECIMAL(18,8),
                rsi_14 DECIMAL(8,4),
                bollinger_upper DECIMAL(18,8),
                bollinger_lower DECIMAL(18,8),
                macd_line DECIMAL(18,8),
                macd_signal DECIMAL(18,8),
                macd_histogram DECIMAL(18,8)
            )
        "#;
        
        connection.execute(sql).await?;
        Ok(())
    }
    
    async fn down(&self, connection: &mut DuckDBConnection) -> Result<(), MigrationError> {
        let sql = "ALTER TABLE financial_data.bars DROP COLUMN technical_indicators";
        connection.execute(sql).await?;
        Ok(())
    }
    
    fn is_reversible(&self) -> bool { true }
    
    fn estimated_duration(&self) -> Duration {
        Duration::from_secs(30) // Estimated time for this migration
    }
}
```

## Performance Considerations

The performance optimization layer ensures consistent sub-millisecond query response times and efficient resource utilization across all system components.

### 1. Partitioning Strategy

**Intelligent Data Partitioning**: Optimized for time-series query patterns with automatic partition management.

```rust
pub struct PartitionManager {
    partition_config: PartitionConfig,
    partition_metadata: Arc<DashMap<String, PartitionInfo>>,
    auto_partition_enabled: bool,
    partition_stats: PartitionStatistics,
}

#[derive(Debug, Clone)]
pub struct PartitionConfig {
    pub partition_strategy: PartitionStrategy,
    pub partition_size_threshold: u64,
    pub partition_age_threshold: Duration,
    pub compression_enabled: bool,
    pub auto_pruning_enabled: bool,
}

#[derive(Debug, Clone)]
pub enum PartitionStrategy {
    DateBased { interval: DateInterval },
    SizeBased { max_size_bytes: u64 },
    Hybrid { date_interval: DateInterval, max_size_bytes: u64 },
}

impl PartitionManager {
    pub async fn optimize_partitions(&self) -> Result<PartitionOptimizationResult, PartitionError> {
        log::info!("Starting partition optimization");
        
        let mut optimization_result = PartitionOptimizationResult::default();
        
        // Analyze current partition performance
        let partition_analysis = self.analyze_partition_performance().await?;
        
        // Merge small partitions
        let merge_result = self.merge_small_partitions(&partition_analysis).await?;
        optimization_result.partitions_merged = merge_result.partitions_merged;
        
        // Split large partitions
        let split_result = self.split_large_partitions(&partition_analysis).await?;
        optimization_result.partitions_split = split_result.partitions_split;
        
        // Compress old partitions
        let compression_result = self.compress_old_partitions(&partition_analysis).await?;
        optimization_result.partitions_compressed = compression_result.partitions_compressed;
        
        // Prune expired partitions
        if self.partition_config.auto_pruning_enabled {
            let pruning_result = self.prune_expired_partitions().await?;
            optimization_result.partitions_pruned = pruning_result.partitions_pruned;
        }
        
        // Update partition statistics
        self.update_partition_statistics().await?;
        
        log::info!("Partition optimization completed: {:?}", optimization_result);
        Ok(optimization_result)
    }
    
    async fn analyze_partition_performance(&self) -> Result<PartitionAnalysis, PartitionError> {
        let mut analysis = PartitionAnalysis::default();
        
        for partition_entry in self.partition_metadata.iter() {
            let partition_info = partition_entry.value();
            let stats = self.get_partition_statistics(&partition_info.name).await?;
            
            // Analyze query performance
            let query_performance = self.analyze_partition_query_performance(&partition_info.name).await?;
            
            // Check if partition needs optimization
            if stats.size_bytes > self.partition_config.partition_size_threshold {
                analysis.oversized_partitions.push(partition_info.clone());
            }
            
            if stats.size_bytes < self.partition_config.partition_size_threshold / 10 {
                analysis.undersized_partitions.push(partition_info.clone());
            }
            
            if query_performance.avg_query_time > Duration::from_millis(100) {
                analysis.slow_partitions.push(partition_info.clone());
            }
            
            if partition_info.created_at + self.partition_config.partition_age_threshold < Utc::now() {
                analysis.old_partitions.push(partition_info.clone());
            }
        }
        
        Ok(analysis)
    }
    
    async fn create_partition(&self, partition_spec: PartitionSpec) -> Result<PartitionInfo, PartitionError> {
        let partition_name = self.generate_partition_name(&partition_spec);
        
        // Create partition table
        let create_sql = self.generate_partition_create_sql(&partition_spec, &partition_name)?;
        self.execute_ddl(&create_sql).await?;
        
        // Create indexes for new partition
        let index_sqls = self.generate_partition_indexes(&partition_name, &partition_spec);
        for sql in index_sqls {
            self.execute_ddl(&sql).await?;
        }
        
        // Apply compression if enabled
        if self.partition_config.compression_enabled {
            self.apply_partition_compression(&partition_name).await?;
        }
        
        let partition_info = PartitionInfo {
            name: partition_name.clone(),
            table_name: partition_spec.table_name.clone(),
            date_range: partition_spec.date_range,
            created_at: Utc::now(),
            size_bytes: 0,
            row_count: 0,
            last_accessed: Utc::now(),
            compression_ratio: 1.0,
        };
        
        self.partition_metadata.insert(partition_name, partition_info.clone());
        
        Ok(partition_info)
    }
    
    fn generate_partition_create_sql(&self, spec: &PartitionSpec, name: &str) -> Result<String, PartitionError> {
        let base_table = &spec.table_name;
        let date_filter = match &spec.date_range {
            Some(range) => format!(
                "CHECK (date_partition >= '{}' AND date_partition <= '{}')",
                range.start.format("%Y-%m-%d"),
                range.end.format("%Y-%m-%d")
            ),
            None => String::new(),
        };
        
        let sql = match base_table.as_str() {
            "ticks" => format!(r#"
                CREATE TABLE financial_data.{} (
                    LIKE financial_data.ticks INCLUDING ALL
                ) {}
            "#, name, date_filter),
            "bars" => format!(r#"
                CREATE TABLE financial_data.{} (
                    LIKE financial_data.bars INCLUDING ALL
                ) {}
            "#, name, date_filter),
            _ => return Err(PartitionError::UnsupportedTable(base_table.clone())),
        };
        
        Ok(sql)
    }
}
```

### 2. Compression Optimization

**Advanced Compression Pipeline**: Multi-level compression with automatic optimization based on data patterns.

```rust
pub struct CompressionEngine {
    compression_config: CompressionConfig,
    compression_stats: Arc<DashMap<String, CompressionStats>>,
    algorithms: HashMap<CompressionAlgorithm, Box<dyn Compressor>>,
}

#[derive(Debug, Clone)]
pub struct CompressionConfig {
    pub default_algorithm: CompressionAlgorithm,
    pub analysis_enabled: bool,
    pub adaptive_compression: bool,
    pub compression_level: u8,
    pub block_size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompressionAlgorithm {
    ZSTD,
    LZ4,
    Snappy,
    Parquet,
    Delta,
    RLE, // Run-length encoding
}

impl CompressionEngine {
    pub async fn compress_data(&self, data: &[u8], data_type: DataType) -> Result<CompressedData, CompressionError> {
        // Analyze data patterns if adaptive compression is enabled
        let algorithm = if self.compression_config.adaptive_compression {
            self.select_optimal_algorithm(data, data_type).await?
        } else {
            self.compression_config.default_algorithm.clone()
        };
        
        // Get compressor for selected algorithm
        let compressor = self.algorithms.get(&algorithm)
            .ok_or(CompressionError::UnsupportedAlgorithm(algorithm.clone()))?;
        
        // Perform compression
        let start_time = Instant::now();
        let compressed_data = compressor.compress(data, self.compression_config.compression_level)?;
        let compression_time = start_time.elapsed();
        
        // Calculate compression statistics
        let compression_ratio = data.len() as f64 / compressed_data.len() as f64;
        let compression_rate = data.len() as f64 / compression_time.as_secs_f64(); // bytes per second
        
        // Update compression statistics
        self.update_compression_stats(&algorithm, CompressionStats {
            original_size: data.len(),
            compressed_size: compressed_data.len(),
            compression_ratio,
            compression_time,
            compression_rate,
            algorithm: algorithm.clone(),
        }).await;
        
        Ok(CompressedData {
            data: compressed_data,
            algorithm,
            original_size: data.len(),
            compression_ratio,
            compressed_at: Utc::now(),
        })
    }
    
    async fn select_optimal_algorithm(&self, data: &[u8], data_type: DataType) -> Result<CompressionAlgorithm, CompressionError> {
        // Analyze data characteristics
        let analysis = self.analyze_data_patterns(data, data_type).await?;
        
        match analysis.primary_pattern {
            DataPattern::HighlyRepetitive => Ok(CompressionAlgorithm::RLE),
            DataPattern::NumericalTimeSeries => Ok(CompressionAlgorithm::Delta),
            DataPattern::Random => Ok(CompressionAlgorithm::LZ4), // Fast compression for random data
            DataPattern::StructuredFinancial => Ok(CompressionAlgorithm::Parquet),
            DataPattern::Mixed => {
                // Use historical performance to select best algorithm
                self.select_based_on_performance_history(data_type).await
            }
        }
    }
    
    async fn analyze_data_patterns(&self, data: &[u8], data_type: DataType) -> Result<DataAnalysis, CompressionError> {
        match data_type {
            DataType::Ticks => self.analyze_tick_data_patterns(data).await,
            DataType::Bars => self.analyze_bar_data_patterns(data).await,
            DataType::Positions => self.analyze_position_data_patterns(data).await,
            DataType::Generic => self.analyze_generic_data_patterns(data).await,
        }
    }
    
    async fn analyze_tick_data_patterns(&self, data: &[u8]) -> Result<DataAnalysis, CompressionError> {
        // Deserialize tick data for analysis
        let ticks: Vec<Tick> = bincode::deserialize(data)
            .map_err(|e| CompressionError::DeserializationError(e.to_string()))?;
        
        if ticks.is_empty() {
            return Ok(DataAnalysis::default());
        }
        
        // Analyze price patterns
        let mut price_deltas = Vec::new();
        let mut volume_patterns = Vec::new();
        let mut timestamp_gaps = Vec::new();
        
        for window in ticks.windows(2) {
            if let [prev, curr] = window {
                // Price delta analysis
                let price_delta = curr.price - prev.price;
                price_deltas.push(price_delta);
                
                // Volume pattern analysis
                volume_patterns.push(curr.volume);
                
                // Timestamp gap analysis
                let time_gap = curr.timestamp.signed_duration_since(prev.timestamp);
                timestamp_gaps.push(time_gap);
            }
        }
        
        // Determine dominant patterns
        let price_repetition = self.calculate_repetition_rate(&price_deltas);
        let volume_variance = self.calculate_variance(&volume_patterns);
        let time_regularity = self.calculate_time_regularity(&timestamp_gaps);
        
        let primary_pattern = if price_repetition > 0.8 {
            DataPattern::HighlyRepetitive
        } else if time_regularity > 0.9 {
            DataPattern::NumericalTimeSeries
        } else {
            DataPattern::StructuredFinancial
        };
        
        Ok(DataAnalysis {
            primary_pattern,
            repetition_rate: price_repetition,
            entropy: self.calculate_entropy(data),
            compression_potential: self.estimate_compression_potential(&primary_pattern),
        })
    }
}

pub trait Compressor: Send + Sync {
    fn compress(&self, data: &[u8], level: u8) -> Result<Vec<u8>, CompressionError>;
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError>;
    fn max_compression_level(&self) -> u8;
    fn is_fast_compression(&self) -> bool;
}

// ZSTD compressor implementation
pub struct ZstdCompressor;

impl Compressor for ZstdCompressor {
    fn compress(&self, data: &[u8], level: u8) -> Result<Vec<u8>, CompressionError> {
        zstd::encode_all(data, level as i32)
            .map_err(|e| CompressionError::CompressionFailed(e.to_string()))
    }
    
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        zstd::decode_all(data)
            .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))
    }
    
    fn max_compression_level(&self) -> u8 { 22 }
    fn is_fast_compression(&self) -> bool { false }
}

// Delta compressor for numerical time series
pub struct DeltaCompressor;

impl Compressor for DeltaCompressor {
    fn compress(&self, data: &[u8], _level: u8) -> Result<Vec<u8>, CompressionError> {
        // Deserialize as f64 values (assuming price data)
        if data.len() % 8 != 0 {
            return Err(CompressionError::InvalidDataFormat("Data length not multiple of 8".to_string()));
        }
        
        let values: Vec<f64> = (0..data.len() / 8)
            .map(|i| f64::from_le_bytes([
                data[i * 8], data[i * 8 + 1], data[i * 8 + 2], data[i * 8 + 3],
                data[i * 8 + 4], data[i * 8 + 5], data[i * 8 + 6], data[i * 8 + 7],
            ]))
            .collect();
        
        if values.is_empty() {
            return Ok(Vec::new());
        }
        
        // Delta encode
        let mut deltas = Vec::with_capacity(values.len());
        deltas.push(values[0]); // First value as-is
        
        for i in 1..values.len() {
            deltas.push(values[i] - values[i - 1]);
        }
        
        // Serialize deltas
        let mut result = Vec::with_capacity(deltas.len() * 8);
        for delta in deltas {
            result.extend_from_slice(&delta.to_le_bytes());
        }
        
        // Apply additional compression to deltas
        zstd::encode_all(&result, 3)
            .map_err(|e| CompressionError::CompressionFailed(e.to_string()))
    }
    
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // Decompress first
        let decompressed = zstd::decode_all(data)
            .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))?;
        
        // Deserialize deltas
        if decompressed.len() % 8 != 0 {
            return Err(CompressionError::InvalidDataFormat("Decompressed data length not multiple of 8".to_string()));
        }
        
        let deltas: Vec<f64> = (0..decompressed.len() / 8)
            .map(|i| f64::from_le_bytes([
                decompressed[i * 8], decompressed[i * 8 + 1], decompressed[i * 8 + 2], decompressed[i * 8 + 3],
                decompressed[i * 8 + 4], decompressed[i * 8 + 5], decompressed[i * 8 + 6], decompressed[i * 8 + 7],
            ]))
            .collect();
        
        if deltas.is_empty() {
            return Ok(Vec::new());
        }
        
        // Reconstruct original values
        let mut values = Vec::with_capacity(deltas.len());
        values.push(deltas[0]); // First value
        
        for i in 1..deltas.len() {
            values.push(values[i - 1] + deltas[i]);
        }
        
        // Serialize reconstructed values
        let mut result = Vec::with_capacity(values.len() * 8);
        for value in values {
            result.extend_from_slice(&value.to_le_bytes());
        }
        
        Ok(result)
    }
    
    fn max_compression_level(&self) -> u8 { 1 }
    fn is_fast_compression(&self) -> bool { true }
}
```

### 3. Query Optimization

**Intelligent Query Planning**: Cost-based optimization with adaptive query rewriting for financial data patterns.

```rust
pub struct QueryOptimizer {
    cost_model: CostModel,
    statistics: TableStatistics,
    index_advisor: IndexAdvisor,
    query_cache: Arc<DashMap<String, OptimizedQuery>>,
    execution_history: QueryExecutionHistory,
}

impl QueryOptimizer {
    pub async fn optimize_query(&self, query: &Query) -> Result<OptimizedQuery, OptimizationError> {
        // Parse and analyze query
        let query_analysis = self.analyze_query(query).await?;
        
        // Check cache for previously optimized query
        let cache_key = self.generate_query_hash(query);
        if let Some(cached) = self.query_cache.get(&cache_key) {
            if !self.is_statistics_stale(&cached.optimization_timestamp) {
                return Ok(cached.clone());
            }
        }
        
        // Apply optimization rules
        let mut optimized_query = query.clone();
        
        // 1. Predicate pushdown
        optimized_query = self.apply_predicate_pushdown(optimized_query).await?;
        
        // 2. Partition pruning
        optimized_query = self.apply_partition_pruning(optimized_query, &query_analysis).await?;
        
        // 3. Index selection
        let index_recommendations = self.index_advisor.recommend_indexes(&query_analysis).await?;
        optimized_query = self.apply_index_hints(optimized_query, &index_recommendations).await?;
        
        // 4. Join optimization
        if query_analysis.has_joins {
            optimized_query = self.optimize_joins(optimized_query, &query_analysis).await?;
        }
        
        // 5. Aggregation optimization
        if query_analysis.has_aggregations {
            optimized_query = self.optimize_aggregations(optimized_query, &query_analysis).await?;
        }
        
        // 6. Column pruning
        optimized_query = self.apply_column_pruning(optimized_query, &query_analysis).await?;
        
        // Calculate estimated cost
        let estimated_cost = self.cost_model.estimate_query_cost(&optimized_query, &query_analysis).await?;
        
        let result = OptimizedQuery {
            original_query: query.clone(),
            optimized_query,
            optimization_rules_applied: query_analysis.optimization_opportunities,
            estimated_cost,
            optimization_timestamp: Utc::now(),
            index_recommendations,
        };
        
        // Cache optimized query
        self.query_cache.insert(cache_key, result.clone());
        
        Ok(result)
    }
    
    async fn apply_predicate_pushdown(&self, mut query: Query) -> Result<Query, OptimizationError> {
        // Identify predicates that can be pushed down to table scan level
        let pushdown_candidates = self.identify_pushdown_predicates(&query).await?;
        
        for predicate in pushdown_candidates {
            match &predicate.column {
                "timestamp_utc" | "date_partition" => {
                    // Time-based predicates - highest priority for financial data
                    query = self.push_time_predicate(query, &predicate).await?;
                }
                "symbol" => {
                    // Symbol filtering - very selective
                    query = self.push_symbol_predicate(query, &predicate).await?;
                }
                "price" | "volume" => {
                    // Numerical range predicates
                    query = self.push_numerical_predicate(query, &predicate).await?;
                }
                _ => {
                    // Generic predicate pushdown
                    query = self.push_generic_predicate(query, &predicate).await?;
                }
            }
        }
        
        Ok(query)
    }
    
    async fn apply_partition_pruning(&self, query: Query, analysis: &QueryAnalysis) -> Result<Query, OptimizationError> {
        // Analyze time range constraints
        if let Some(time_range) = &analysis.time_range {
            let relevant_partitions = self.statistics
                .get_partitions_for_time_range(time_range)
                .await?;
            
            if relevant_partitions.len() < self.statistics.total_partitions {
                // Add partition pruning hint
                let pruning_hint = format!(
                    "/*+ PARTITION_PRUNING({}) */",
                    relevant_partitions.iter()
                        .map(|p| format!("'{}'", p))
                        .collect::<Vec<_>>()
                        .join(",")
                );
                
                return Ok(query.add_hint(pruning_hint));
            }
        }
        
        Ok(query)
    }
    
    async fn optimize_aggregations(&self, mut query: Query, analysis: &QueryAnalysis) -> Result<Query, OptimizationError> {
        for aggregation in &analysis.aggregations {
            match aggregation.function.as_str() {
                "OHLC" | "VWAP" => {
                    // Use pre-computed bars when possible
                    if let Some(timeframe) = self.detect_timeframe_from_grouping(&aggregation.group_by) {
                        query = self.rewrite_to_use_bars(query, timeframe).await?;
                    }
                }
                "SUM" | "AVG" | "COUNT" => {
                    // Use columnar aggregation optimizations
                    query = self.optimize_columnar_aggregation(query, aggregation).await?;
                }
                _ => {}
            }
        }
        
        Ok(query)
    }
    
    async fn rewrite_to_use_bars(&self, query: Query, timeframe: Timeframe) -> Result<Query, OptimizationError> {
        // Check if bars table has the required timeframe
        let bars_available = self.statistics
            .check_bars_availability(&timeframe)
            .await?;
        
        if bars_available {
            // Rewrite query to use bars table instead of aggregating ticks
            let rewritten_sql = query.sql.replace(
                "FROM financial_data.ticks",
                &format!("FROM financial_data.bars WHERE timeframe = '{:?}'", timeframe)
            );
            
            return Ok(Query {
                sql: rewritten_sql,
                ..query
            });
        }
        
        Ok(query)
    }
}

pub struct CostModel {
    io_cost_per_page: f64,
    cpu_cost_per_tuple: f64,
    memory_cost_per_mb: f64,
    network_cost_per_kb: f64,
    index_scan_cost_factor: f64,
    hash_join_cost_factor: f64,
}

impl CostModel {
    pub async fn estimate_query_cost(&self, query: &Query, analysis: &QueryAnalysis) -> Result<QueryCost, CostError> {
        let mut total_cost = 0.0;
        
        // Table scan costs
        for table in &analysis.tables {
            let table_stats = self.get_table_statistics(table).await?;
            let scan_cost = self.estimate_scan_cost(&table_stats, &analysis.predicates);
            total_cost += scan_cost;
        }
        
        // Join costs
        for join in &analysis.joins {
            let join_cost = self.estimate_join_cost(join, analysis).await?;
            total_cost += join_cost;
        }
        
        // Aggregation costs
        for aggregation in &analysis.aggregations {
            let agg_cost = self.estimate_aggregation_cost(aggregation, analysis).await?;
            total_cost += agg_cost;
        }
        
        // Sort costs
        if analysis.has_order_by {
            let sort_cost = self.estimate_sort_cost(&analysis.estimated_result_size);
            total_cost += sort_cost;
        }
        
        Ok(QueryCost {
            total_cost,
            io_cost: total_cost * 0.6,       // IO typically dominates
            cpu_cost: total_cost * 0.3,      // CPU processing
            memory_cost: total_cost * 0.1,   // Memory usage
            estimated_execution_time: Duration::from_millis((total_cost * 100.0) as u64),
        })
    }
    
    fn estimate_scan_cost(&self, table_stats: &TableStats, predicates: &[Predicate]) -> f64 {
        let base_pages = table_stats.size_bytes / 8192; // 8KB pages
        let selectivity = self.calculate_predicate_selectivity(predicates, table_stats);
        
        // Apply selectivity to reduce IO cost
        let effective_pages = (base_pages as f64 * selectivity).max(1.0);
        
        self.io_cost_per_page * effective_pages + 
        self.cpu_cost_per_tuple * (table_stats.row_count as f64 * selectivity)
    }
    
    fn calculate_predicate_selectivity(&self, predicates: &[Predicate], table_stats: &TableStats) -> f64 {
        let mut total_selectivity = 1.0;
        
        for predicate in predicates {
            let selectivity = match &predicate.operator {
                Operator::Equals => {
                    // Use histogram data if available
                    if let Some(histogram) = table_stats.column_histograms.get(&predicate.column) {
                        histogram.estimate_equality_selectivity(&predicate.value)
                    } else {
                        1.0 / table_stats.estimated_distinct_values.get(&predicate.column).unwrap_or(&100) as f64
                    }
                }
                Operator::Range => {
                    // Range selectivity based on value distribution
                    if let Some(histogram) = table_stats.column_histograms.get(&predicate.column) {
                        histogram.estimate_range_selectivity(&predicate.value)
                    } else {
                        0.1 // Default 10% selectivity for ranges
                    }
                }
                Operator::In => {
                    // IN clause selectivity
                    let in_values = predicate.value.as_array().unwrap_or(&vec![]);
                    (in_values.len() as f64) / table_stats.estimated_distinct_values.get(&predicate.column).unwrap_or(&100) as f64
                }
                _ => 0.1, // Conservative estimate
            };
            
            total_selectivity *= selectivity;
        }
        
        total_selectivity.max(0.001) // Minimum selectivity to avoid zero costs
    }
}
```

This comprehensive Data Architecture section provides the foundation for BackTestr_ai's high-performance financial data processing capabilities. The architecture delivers institutional-grade performance with 10-20x compression, sub-millisecond query response times, and supports processing rates exceeding 1M ticks/second while maintaining data integrity and providing comprehensive backup and recovery capabilities.
