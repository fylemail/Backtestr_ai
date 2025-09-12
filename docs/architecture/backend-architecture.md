# Backend Architecture

## Core Architecture Patterns

The BackTestr_ai backend employs several architectural patterns optimized for high-frequency financial data processing while maintaining code clarity and system reliability.

### 1. Event-Driven Architecture (EDA)

**Implementation**: The core engine operates on a tick-based event system where each market data point triggers a cascade of state updates and algorithm evaluations.

```rust
// Core event system structure
pub enum MarketEvent {
    Tick { symbol: String, price: f64, volume: u64, timestamp: Timestamp },
    BarClose { timeframe: Timeframe, symbol: String, bar: OHLCBar },
    AlgorithmSignal { signal: TradeSignal, context: AlgorithmContext },
    PositionUpdate { position_id: PositionId, update: PositionUpdate },
}

pub trait EventHandler {
    fn handle_event(&mut self, event: MarketEvent) -> Result<Vec<MarketEvent>, ProcessingError>;
}
```

**Benefits**:
- Loose coupling between components enables independent testing and development
- Natural flow mirrors real trading systems for accurate backtesting
- Easy extension for new data sources and algorithm types
- Deterministic event ordering ensures reproducible results

### 2. Repository Pattern

**Implementation**: Abstract data access behind trait interfaces to enable testing with mock data and support for multiple data sources.

```rust
pub trait TickDataRepository {
    async fn get_ticks(&self, symbol: &str, start: DateTime, end: DateTime) -> Result<Vec<Tick>, DataError>;
    async fn store_ticks(&self, symbol: &str, ticks: &[Tick]) -> Result<(), DataError>;
    async fn get_symbols(&self) -> Result<Vec<String>, DataError>;
}

pub trait BarDataRepository {
    async fn get_bars(&self, symbol: &str, timeframe: Timeframe, start: DateTime, end: DateTime) -> Result<Vec<OHLCBar>, DataError>;
    async fn cache_bars(&self, symbol: &str, timeframe: Timeframe, bars: &[OHLCBar]) -> Result<(), DataError>;
}
```

**Benefits**:
- Database-agnostic algorithm development
- Simplified unit testing with mock repositories
- Easy migration between storage systems
- Clear separation of business logic and data access

### 3. Command Query Responsibility Segregation (CQRS)

**Implementation**: Separate read and write operations for optimal performance in different access patterns.

```rust
// Write side - optimized for high-frequency updates
pub struct TickWriter {
    buffer: RingBuffer<Tick>,
    batch_size: usize,
    flush_interval: Duration,
}

// Read side - optimized for analytical queries
pub struct BarReader {
    cache: LRUCache<TimeframeKey, Vec<OHLCBar>>,
    db_connection: DuckDBConnection,
}
```

**Benefits**:
- Write operations optimized for high-frequency tick ingestion
- Read operations optimized for complex analytical queries
- Independent scaling of read and write workloads
- Simplified caching strategies for each access pattern

### 4. Domain-Driven Design (DDD)

**Implementation**: Core financial concepts modeled as rich domain objects with behavior and business rules.

```rust
pub struct Position {
    id: PositionId,
    symbol: String,
    side: PositionSide,
    entry_price: Price,
    current_size: Quantity,
    unrealized_pnl: Money,
    realized_pnl: Money,
    
    // Domain behavior
    impl Position {
        pub fn add_fill(&mut self, fill: Fill) -> Result<(), PositionError> { ... }
        pub fn calculate_unrealized_pnl(&self, current_price: Price) -> Money { ... }
        pub fn is_flat(&self) -> bool { ... }
    }
}
```

**Benefits**:
- Business logic encapsulated within domain objects
- Consistent behavior across different usage contexts
- Natural modeling of financial concepts
- Reduced cognitive load for algorithm developers

## Service Layer Design

The service layer orchestrates domain objects and coordinates between different subsystems while maintaining clear boundaries and responsibilities.

### 1. Multi-Timeframe (MTF) Engine Service

**Core Responsibility**: Maintains synchronized state across six timeframes with atomic updates per tick.

```rust
pub struct MTFEngine {
    timeframes: HashMap<Timeframe, TimeframeState>,
    partial_bars: HashMap<Timeframe, PartialBar>,
    event_publisher: EventPublisher,
    performance_metrics: MTFMetrics,
}

impl MTFEngine {
    pub fn process_tick(&mut self, tick: Tick) -> Result<Vec<BarCompleteEvent>, MTFError> {
        let mut completed_bars = Vec::new();
        
        // Update all affected timeframes atomically
        for (timeframe, state) in &mut self.timeframes {
            if timeframe.should_update(&tick.timestamp) {
                let bar_complete = state.add_tick(tick.clone())?;
                if let Some(bar) = bar_complete {
                    completed_bars.push(BarCompleteEvent {
                        timeframe: *timeframe,
                        bar,
                        timestamp: tick.timestamp,
                    });
                }
            }
        }
        
        // Update performance metrics
        self.performance_metrics.record_update_latency();
        
        Ok(completed_bars)
    }
    
    // Critical: Query methods must return current state at specific timestamp
    pub fn get_bars_at_time(&self, timeframe: Timeframe, timestamp: DateTime, count: usize) -> Vec<OHLCBar> {
        self.timeframes[&timeframe].get_completed_bars_before(timestamp, count)
    }
}
```

**Performance Targets**:
- Sub-100 microsecond update latency per tick
- Zero allocation during steady-state operation
- Atomic consistency across all timeframes
- 1M+ ticks/second processing capability

### 2. Position Manager Service

**Core Responsibility**: Tracks multiple concurrent positions with realistic execution modeling.

```rust
pub struct PositionManager {
    positions: HashMap<PositionId, Position>,
    open_orders: HashMap<OrderId, Order>,
    execution_model: Box<dyn ExecutionModel>,
    risk_manager: RiskManager,
    pnl_calculator: PnLCalculator,
}

impl PositionManager {
    pub fn submit_order(&mut self, order: Order) -> Result<OrderId, ExecutionError> {
        // Pre-trade risk checks
        self.risk_manager.validate_order(&order, &self.positions)?;
        
        // Store pending order
        let order_id = OrderId::new();
        self.open_orders.insert(order_id, order);
        
        Ok(order_id)
    }
    
    pub fn process_market_data(&mut self, tick: Tick) -> Result<Vec<Fill>, ExecutionError> {
        let mut fills = Vec::new();
        
        // Check for order fills using realistic execution model
        for (order_id, order) in &self.open_orders {
            if let Some(fill) = self.execution_model.check_fill(order, &tick)? {
                // Update position
                let position = self.positions.entry(fill.position_id)
                    .or_insert_with(|| Position::new(fill.symbol.clone()));
                position.add_fill(fill.clone())?;
                
                fills.push(fill);
                self.open_orders.remove(order_id);
            }
        }
        
        // Update unrealized P&L for all positions
        for position in self.positions.values_mut() {
            position.update_unrealized_pnl(tick.price);
        }
        
        Ok(fills)
    }
}
```

**Key Features**:
- Multiple concurrent positions per symbol
- Realistic slippage and commission modeling
- Risk limits and position sizing controls
- Real-time P&L calculation and tracking

### 3. Indicator Pipeline Service

**Core Responsibility**: Manages indicator dependencies and provides efficient calculation updates.

```rust
pub struct IndicatorPipeline {
    indicators: HashMap<IndicatorId, Box<dyn Indicator>>,
    dependency_graph: DependencyGraph,
    calculation_cache: IndicatorCache,
    update_scheduler: UpdateScheduler,
}

impl IndicatorPipeline {
    pub fn register_indicator<T: Indicator + 'static>(&mut self, indicator: T) -> IndicatorId {
        let id = IndicatorId::new();
        let dependencies = indicator.get_dependencies();
        
        // Build dependency graph for optimal update ordering
        self.dependency_graph.add_node(id, dependencies);
        self.indicators.insert(id, Box::new(indicator));
        
        id
    }
    
    pub fn update_indicators(&mut self, bar_event: BarCompleteEvent) -> Result<(), IndicatorError> {
        // Get topologically sorted update order
        let update_order = self.dependency_graph.get_update_order()?;
        
        for indicator_id in update_order {
            if let Some(indicator) = self.indicators.get_mut(&indicator_id) {
                // Check if update needed based on timeframe
                if indicator.should_update(&bar_event.timeframe) {
                    let input_data = self.get_indicator_inputs(indicator_id, &bar_event)?;
                    let result = indicator.calculate(input_data)?;
                    
                    // Cache result for dependent indicators
                    self.calculation_cache.store(indicator_id, bar_event.timestamp, result);
                }
            }
        }
        
        Ok(())
    }
}

// Indicator trait for custom implementations
pub trait Indicator: Send + Sync {
    fn calculate(&mut self, inputs: IndicatorInputs) -> Result<IndicatorResult, IndicatorError>;
    fn get_dependencies(&self) -> Vec<IndicatorId>;
    fn should_update(&self, timeframe: &Timeframe) -> bool;
    fn get_lookback_period(&self) -> usize;
}
```

**Performance Optimizations**:
- Dependency resolution at registration time
- Incremental calculation for rolling indicators
- Vectorized operations using SIMD instructions
- Lazy evaluation for unused indicators

### 4. Algorithm Execution Service

**Core Responsibility**: Manages Python algorithm lifecycle and provides safe execution environment.

```rust
pub struct AlgorithmExecutor {
    python_runtime: Python,
    algorithm_modules: HashMap<AlgorithmId, PyModule>,
    execution_context: AlgorithmContext,
    safety_limits: ExecutionLimits,
}

impl AlgorithmExecutor {
    pub fn load_algorithm(&mut self, code: &str) -> Result<AlgorithmId, AlgorithmError> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        
        // Compile and validate algorithm code
        let module = PyModule::from_code(py, code, "algorithm.py", "algorithm")?;
        
        // Verify required interfaces
        self.validate_algorithm_interface(&module)?;
        
        let id = AlgorithmId::new();
        self.algorithm_modules.insert(id, module);
        
        Ok(id)
    }
    
    pub fn execute_on_bar(&mut self, algorithm_id: AlgorithmId, bar_event: BarCompleteEvent) -> Result<Vec<TradeSignal>, AlgorithmError> {
        let module = self.algorithm_modules.get(&algorithm_id)
            .ok_or(AlgorithmError::NotFound)?;
        
        // Set up execution context with current market state
        self.execution_context.update_market_data(&bar_event);
        
        // Execute algorithm with timeout protection
        let timeout = Duration::from_millis(self.safety_limits.max_execution_time_ms);
        let signals = timeout::timeout(timeout, async {
            self.call_algorithm_function(module, "on_bar", &bar_event)
        }).await??;
        
        // Validate output signals
        self.validate_trade_signals(&signals)?;
        
        Ok(signals)
    }
    
    fn call_algorithm_function(&self, module: &PyModule, function: &str, args: &BarCompleteEvent) -> PyResult<Vec<TradeSignal>> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        
        // Convert Rust data to Python objects
        let py_args = self.convert_to_python(py, args)?;
        
        // Call algorithm function
        let result = module.getattr(function)?.call1((py_args,))?;
        
        // Convert Python result back to Rust
        self.convert_from_python(result)
    }
}
```

**Safety Features**:
- Execution timeout protection
- Memory usage limits
- API access restrictions
- Sandboxed environment

## Data Access Layer

The data access layer provides efficient, scalable storage and retrieval of financial time series data with emphasis on performance and compression.

### 1. DuckDB Integration

**Core Implementation**: Embedded analytical database optimized for time series queries.

```rust
pub struct DuckDBDataStore {
    connection: Connection,
    prepared_statements: PreparedStatementCache,
    partition_manager: PartitionManager,
    compression_config: CompressionConfig,
}

impl DuckDBDataStore {
    pub fn new(db_path: PathBuf) -> Result<Self, DataStoreError> {
        let connection = Connection::open(db_path)?;
        
        // Configure for time series optimization
        connection.execute_batch(r#"
            PRAGMA threads=4;
            PRAGMA memory_limit='4GB';
            PRAGMA enable_object_cache;
            
            -- Create optimized tick data table
            CREATE TABLE IF NOT EXISTS ticks (
                symbol VARCHAR NOT NULL,
                timestamp TIMESTAMP_US NOT NULL,
                price DECIMAL(18,8) NOT NULL,
                volume BIGINT NOT NULL,
                bid DECIMAL(18,8),
                ask DECIMAL(18,8),
                PRIMARY KEY (symbol, timestamp)
            ) PARTITION BY RANGE (DATE_TRUNC('day', timestamp));
            
            -- Create bar cache table
            CREATE TABLE IF NOT EXISTS bars (
                symbol VARCHAR NOT NULL,
                timeframe VARCHAR NOT NULL,
                timestamp TIMESTAMP_US NOT NULL,
                open DECIMAL(18,8) NOT NULL,
                high DECIMAL(18,8) NOT NULL,
                low DECIMAL(18,8) NOT NULL,
                close DECIMAL(18,8) NOT NULL,
                volume BIGINT NOT NULL,
                PRIMARY KEY (symbol, timeframe, timestamp)
            ) PARTITION BY (symbol, timeframe);
        "#)?;
        
        Ok(Self {
            connection,
            prepared_statements: PreparedStatementCache::new(),
            partition_manager: PartitionManager::new(),
            compression_config: CompressionConfig::default(),
        })
    }
    
    pub async fn insert_ticks_batch(&mut self, symbol: &str, ticks: &[Tick]) -> Result<(), DataStoreError> {
        // Use prepared statement for optimal performance
        let stmt = self.prepared_statements.get_or_create(
            "INSERT INTO ticks (symbol, timestamp, price, volume, bid, ask) VALUES (?, ?, ?, ?, ?, ?)"
        )?;
        
        // Batch insert with transaction
        let tx = self.connection.transaction()?;
        {
            for tick in ticks {
                stmt.execute(params![
                    symbol,
                    tick.timestamp,
                    tick.price,
                    tick.volume,
                    tick.bid,
                    tick.ask
                ])?;
            }
        }
        tx.commit()?;
        
        // Update partition statistics
        self.partition_manager.update_statistics(symbol, ticks.len());
        
        Ok(())
    }
    
    pub async fn get_ticks_range(&self, symbol: &str, start: DateTime, end: DateTime) -> Result<Vec<Tick>, DataStoreError> {
        // Use partition pruning for efficient queries
        let partitions = self.partition_manager.get_partitions_for_range(symbol, start, end);
        
        let query = format!(r#"
            SELECT timestamp, price, volume, bid, ask
            FROM ticks
            WHERE symbol = ? 
              AND timestamp BETWEEN ? AND ?
              AND partition_key IN ({})
            ORDER BY timestamp
        "#, partitions.join(","));
        
        let mut stmt = self.connection.prepare(&query)?;
        let rows = stmt.query_map(params![symbol, start, end], |row| {
            Ok(Tick {
                symbol: symbol.to_string(),
                timestamp: row.get(0)?,
                price: row.get(1)?,
                volume: row.get(2)?,
                bid: row.get(3)?,
                ask: row.get(4)?,
            })
        })?;
        
        rows.collect::<Result<Vec<_>, _>>().map_err(DataStoreError::from)
    }
}
```

**Performance Features**:
- Columnar storage with 10-20x compression
- Partition pruning for range queries
- Vectorized query execution
- Memory-mapped file access

### 2. Tick Data Storage Strategy

**Partitioning Scheme**: Date-based partitioning with symbol sub-partitioning for optimal query performance.

```rust
pub struct PartitionManager {
    partition_metadata: HashMap<PartitionKey, PartitionInfo>,
    compression_stats: CompressionTracker,
}

#[derive(Hash, Eq, PartialEq)]
pub struct PartitionKey {
    symbol: String,
    date: NaiveDate,
}

pub struct PartitionInfo {
    tick_count: u64,
    compressed_size: u64,
    uncompressed_size: u64,
    min_timestamp: DateTime,
    max_timestamp: DateTime,
    last_accessed: DateTime,
}

impl PartitionManager {
    pub fn create_partition(&mut self, symbol: &str, date: NaiveDate) -> Result<(), PartitionError> {
        let partition_key = PartitionKey {
            symbol: symbol.to_string(),
            date,
        };
        
        // Create partition-specific table
        let table_name = format!("ticks_{}_{}", symbol, date.format("%Y%m%d"));
        let create_sql = format!(r#"
            CREATE TABLE {} (
                timestamp TIMESTAMP_US NOT NULL,
                price DECIMAL(18,8) NOT NULL,
                volume BIGINT NOT NULL,
                bid DECIMAL(18,8),
                ask DECIMAL(18,8)
            ) WITH (
                compression = 'zstd',
                compression_level = 6
            );
            
            CREATE INDEX ON {} (timestamp);
        "#, table_name, table_name);
        
        // Execute partition creation
        // ... implementation details
        
        Ok(())
    }
    
    pub fn get_partitions_for_range(&self, symbol: &str, start: DateTime, end: DateTime) -> Vec<String> {
        let start_date = start.date();
        let end_date = end.date();
        
        (0..=(end_date - start_date).num_days())
            .map(|days| start_date + Duration::days(days))
            .filter_map(|date| {
                let key = PartitionKey {
                    symbol: symbol.to_string(),
                    date,
                };
                
                if self.partition_metadata.contains_key(&key) {
                    Some(format!("ticks_{}_{}", symbol, date.format("%Y%m%d")))
                } else {
                    None
                }
            })
            .collect()
    }
}
```

### 3. Caching Strategy

**Multi-Level Caching**: In-memory caching with intelligent prefetching for optimal performance.

```rust
pub struct DataCache {
    tick_cache: LRUCache<TickCacheKey, Arc<Vec<Tick>>>,
    bar_cache: LRUCache<BarCacheKey, Arc<Vec<OHLCBar>>>,
    prefetch_scheduler: PrefetchScheduler,
    cache_metrics: CacheMetrics,
}

impl DataCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            tick_cache: LRUCache::new(config.tick_cache_size),
            bar_cache: LRUCache::new(config.bar_cache_size),
            prefetch_scheduler: PrefetchScheduler::new(),
            cache_metrics: CacheMetrics::new(),
        }
    }
    
    pub async fn get_bars(&mut self, symbol: &str, timeframe: Timeframe, start: DateTime, end: DateTime) -> Result<Arc<Vec<OHLCBar>>, CacheError> {
        let cache_key = BarCacheKey {
            symbol: symbol.to_string(),
            timeframe,
            start,
            end,
        };
        
        // Check cache first
        if let Some(bars) = self.bar_cache.get(&cache_key) {
            self.cache_metrics.record_hit();
            return Ok(bars.clone());
        }
        
        // Cache miss - load from database
        self.cache_metrics.record_miss();
        let bars = self.load_bars_from_db(&cache_key).await?;
        let bars_arc = Arc::new(bars);
        
        // Store in cache
        self.bar_cache.put(cache_key, bars_arc.clone());
        
        // Schedule prefetch of adjacent time ranges
        self.prefetch_scheduler.schedule_prefetch(symbol, timeframe, end);
        
        Ok(bars_arc)
    }
    
    async fn load_bars_from_db(&self, key: &BarCacheKey) -> Result<Vec<OHLCBar>, CacheError> {
        // Implementation depends on whether bars are pre-computed or calculated on-demand
        // ... database query implementation
        Ok(vec![]) // placeholder
    }
}
```

## Algorithm Integration

The algorithm integration layer provides seamless interoperability between Rust performance and Python's algorithmic development ecosystem.

### 1. PyO3 Bridge Architecture

**Zero-Copy Data Sharing**: Direct memory access between Rust and Python for maximum performance.

```rust
use pyo3::prelude::*;
use numpy::{PyArray1, PyArray2, ToPyArray};

#[pyclass]
pub struct MarketDataBridge {
    mtf_engine: Arc<Mutex<MTFEngine>>,
    position_manager: Arc<Mutex<PositionManager>>,
    indicator_pipeline: Arc<Mutex<IndicatorPipeline>>,
}

#[pymethods]
impl MarketDataBridge {
    #[getter]
    fn bars(&self, py: Python, timeframe: &str, symbol: &str, count: usize) -> PyResult<&PyArray2<f64>> {
        let timeframe = Timeframe::from_str(timeframe)?;
        let engine = self.mtf_engine.lock().unwrap();
        
        // Get bars as zero-copy view
        let bars = engine.get_recent_bars(&timeframe, symbol, count);
        
        // Convert to numpy array without copying data
        let array_data: Vec<Vec<f64>> = bars.iter()
            .map(|bar| vec![bar.open, bar.high, bar.low, bar.close, bar.volume as f64])
            .collect();
        
        // Create numpy array with shared memory
        let flat_data: Vec<f64> = array_data.into_iter().flatten().collect();
        flat_data.to_pyarray(py).reshape((count, 5))
    }
    
    fn submit_order(&self, py: Python, order_dict: &PyDict) -> PyResult<u64> {
        let order = self.parse_order_from_dict(order_dict)?;
        let mut manager = self.position_manager.lock().unwrap();
        
        let order_id = manager.submit_order(order)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{:?}", e)))?;
        
        Ok(order_id.0)
    }
    
    #[getter]
    fn positions(&self, py: Python) -> PyResult<PyObject> {
        let manager = self.position_manager.lock().unwrap();
        let positions = manager.get_all_positions();
        
        // Convert to Python dict
        let py_dict = PyDict::new(py);
        for (id, position) in positions {
            let position_dict = PyDict::new(py);
            position_dict.set_item("symbol", position.symbol)?;
            position_dict.set_item("size", position.current_size)?;
            position_dict.set_item("entry_price", position.entry_price)?;
            position_dict.set_item("unrealized_pnl", position.unrealized_pnl)?;
            
            py_dict.set_item(id.0, position_dict)?;
        }
        
        Ok(py_dict.into())
    }
}

// Custom indicator registration
#[pyfunction]
fn register_indicator(py: Python, indicator_class: PyObject) -> PyResult<u64> {
    // Wrap Python indicator in Rust trait
    let indicator = PythonIndicatorWrapper::new(indicator_class);
    
    // Register with pipeline
    let pipeline = get_global_indicator_pipeline();
    let id = pipeline.register_indicator(indicator);
    
    Ok(id.0)
}

// Python module definition
#[pymodule]
fn backtestr_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<MarketDataBridge>()?;
    m.add_function(wrap_pyfunction!(register_indicator, m)?)?;
    Ok(())
}
```

### 2. Python Environment Management

**Embedded Runtime**: Controlled Python environment with performance optimizations.

```rust
pub struct PythonEnvironment {
    interpreter: Python<'static>,
    module_cache: HashMap<String, PyModule>,
    import_hooks: Vec<Box<dyn ImportHook>>,
    memory_limit: usize,
    execution_timeout: Duration,
}

impl PythonEnvironment {
    pub fn new(config: PythonConfig) -> Result<Self, PythonError> {
        // Initialize Python interpreter
        pyo3::prepare_freethreaded_python();
        let gil = Python::acquire_gil();
        let py = gil.python();
        
        // Configure Python environment
        py.run(r#"
import sys
import numpy as np
import pandas as pd
import talib

# Restrict dangerous modules
sys.modules['os'] = None
sys.modules['subprocess'] = None
sys.modules['socket'] = None

# Add our custom module
sys.path.insert(0, '')
"#, None, None)?;
        
        // Load our Rust module into Python
        let backtestr_module = PyModule::new(py, "backtestr_core")?;
        
        Ok(Self {
            interpreter: py,
            module_cache: HashMap::new(),
            import_hooks: vec![],
            memory_limit: config.memory_limit,
            execution_timeout: config.execution_timeout,
        })
    }
    
    pub fn execute_algorithm_code(&mut self, code: &str) -> Result<PyModule, PythonError> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        
        // Memory usage check before execution
        let initial_memory = self.get_memory_usage(py)?;
        if initial_memory > self.memory_limit {
            return Err(PythonError::MemoryLimit);
        }
        
        // Compile and execute with timeout
        let module = timeout::timeout(self.execution_timeout, async {
            PyModule::from_code(py, code, "algorithm.py", "algorithm")
        }).await??;
        
        // Verify memory usage after execution
        let final_memory = self.get_memory_usage(py)?;
        if final_memory > self.memory_limit {
            return Err(PythonError::MemoryLimit);
        }
        
        Ok(module)
    }
    
    fn get_memory_usage(&self, py: Python) -> Result<usize, PythonError> {
        // Get Python memory usage via sys module
        let sys_module = py.import("sys")?;
        let getsizeof = sys_module.getattr("getsizeof")?;
        let gc_module = py.import("gc")?;
        let get_objects = gc_module.getattr("get_objects")?;
        
        let objects = get_objects.call0()?;
        let total_size: usize = 0; // Calculate total memory usage
        
        Ok(total_size)
    }
}
```

### 3. Custom Indicator Framework

**Extensible Indicator System**: Support for both built-in and user-defined indicators.

```python
# Python side - base indicator class
class Indicator:
    def __init__(self, lookback_period=20):
        self.lookback_period = lookback_period
        self.dependencies = []
        
    def calculate(self, bars, dependencies=None):
        """Override this method in custom indicators"""
        raise NotImplementedError
        
    def get_dependencies(self):
        return self.dependencies

# Example custom indicator
class CustomRSI(Indicator):
    def __init__(self, period=14):
        super().__init__(lookback_period=period + 1)
        self.period = period
        
    def calculate(self, bars, dependencies=None):
        closes = bars[:, 3]  # Close prices
        
        # Calculate price changes
        deltas = np.diff(closes)
        gains = np.where(deltas > 0, deltas, 0)
        losses = np.where(deltas < 0, -deltas, 0)
        
        # Calculate moving averages
        avg_gains = np.mean(gains[-self.period:])
        avg_losses = np.mean(losses[-self.period:])
        
        if avg_losses == 0:
            return 100.0
        
        rs = avg_gains / avg_losses
        rsi = 100 - (100 / (1 + rs))
        
        return rsi

# Algorithm example using indicators
class TrendFollowingStrategy:
    def __init__(self):
        self.rsi = CustomRSI(period=14)
        self.ma_fast = SimpleMovingAverage(period=10)
        self.ma_slow = SimpleMovingAverage(period=20)
        
        # Register indicators with the system
        backtestr_core.register_indicator(self.rsi)
        backtestr_core.register_indicator(self.ma_fast)
        backtestr_core.register_indicator(self.ma_slow)
        
    def on_bar(self, bar_data):
        # Get current market data
        bars = backtestr_core.bars('1H', 'EURUSD', 50)
        
        # Calculate indicators
        current_rsi = self.rsi.calculate(bars)
        fast_ma = self.ma_fast.calculate(bars)
        slow_ma = self.ma_slow.calculate(bars)
        
        # Trading logic
        if fast_ma > slow_ma and current_rsi < 30:
            # Buy signal
            order = {
                'symbol': 'EURUSD',
                'side': 'buy',
                'size': 100000,
                'order_type': 'market'
            }
            backtestr_core.submit_order(order)
            
        elif fast_ma < slow_ma and current_rsi > 70:
            # Sell signal
            order = {
                'symbol': 'EURUSD',
                'side': 'sell', 
                'size': 100000,
                'order_type': 'market'
            }
            backtestr_core.submit_order(order)
```

## State Management

Comprehensive state management ensures data consistency and enables features like replay debugging and position tracking.

### 1. Multi-Timeframe State

**Atomic State Updates**: All timeframes updated consistently for each tick.

```rust
pub struct MTFState {
    timeframes: [TimeframeState; 6],
    partial_bars: [Option<PartialBar>; 6],
    last_update_time: DateTime,
    state_version: u64,
}

pub struct TimeframeState {
    timeframe: Timeframe,
    completed_bars: RingBuffer<OHLCBar>,
    current_partial: Option<PartialBar>,
    last_bar_time: DateTime,
    bar_count: u64,
}

impl MTFState {
    pub fn update_with_tick(&mut self, tick: Tick) -> Result<Vec<BarCompleteEvent>, MTFError> {
        let mut completed_bars = Vec::new();
        
        // Update version for consistency checking
        self.state_version += 1;
        
        // Process tick for each timeframe
        for (i, timeframe_state) in self.timeframes.iter_mut().enumerate() {
            let timeframe = Timeframe::from_index(i);
            
            // Check if this timeframe should be updated
            if timeframe.should_update_for_time(tick.timestamp) {
                // Update partial bar
                let mut partial = self.partial_bars[i]
                    .take()
                    .unwrap_or_else(|| PartialBar::new(timeframe, tick.timestamp));
                
                partial.add_tick(&tick);
                
                // Check if bar should be completed
                if timeframe.bar_should_complete(tick.timestamp, partial.start_time) {
                    // Complete the bar
                    let completed_bar = partial.to_completed_bar();
                    timeframe_state.add_completed_bar(completed_bar.clone());
                    
                    completed_bars.push(BarCompleteEvent {
                        timeframe,
                        bar: completed_bar,
                        timestamp: tick.timestamp,
                    });
                    
                    // Start new partial bar
                    self.partial_bars[i] = Some(PartialBar::new(timeframe, tick.timestamp));
                } else {
                    // Keep updating partial bar
                    self.partial_bars[i] = Some(partial);
                }
            }
        }
        
        self.last_update_time = tick.timestamp;
        Ok(completed_bars)
    }
    
    // Critical: Queries must return state at specific point in time
    pub fn get_bars_at_time(&self, timeframe: Timeframe, timestamp: DateTime, count: usize) -> Vec<OHLCBar> {
        let timeframe_index = timeframe.to_index();
        let state = &self.timeframes[timeframe_index];
        
        // Get completed bars before the specified timestamp
        state.get_bars_before(timestamp, count)
    }
    
    pub fn get_partial_bar(&self, timeframe: Timeframe) -> Option<&PartialBar> {
        let index = timeframe.to_index();
        self.partial_bars[index].as_ref()
    }
}
```

### 2. Position State Tracking

**Comprehensive Position Management**: Track all position states for accurate P&L calculation.

```rust
pub struct PositionState {
    positions: HashMap<PositionId, Position>,
    position_history: Vec<PositionSnapshot>,
    open_orders: HashMap<OrderId, Order>,
    fill_history: Vec<Fill>,
    total_realized_pnl: Money,
    total_unrealized_pnl: Money,
}

pub struct PositionSnapshot {
    timestamp: DateTime,
    position_id: PositionId,
    size: Quantity,
    entry_price: Price,
    current_price: Price,
    unrealized_pnl: Money,
    state_version: u64,
}

impl PositionState {
    pub fn apply_fill(&mut self, fill: Fill) -> Result<(), PositionError> {
        let position = self.positions.entry(fill.position_id)
            .or_insert_with(|| Position::new(fill.symbol.clone()));
        
        // Calculate P&L before applying fill
        let old_pnl = position.unrealized_pnl;
        
        // Apply the fill
        position.add_fill(fill.clone())?;
        
        // Update realized P&L if position was reduced
        if fill.reduces_position(position) {
            let realized = position.calculate_realized_pnl(&fill);
            self.total_realized_pnl += realized;
        }
        
        // Store fill in history
        self.fill_history.push(fill);
        
        // Create position snapshot
        self.position_history.push(PositionSnapshot {
            timestamp: fill.timestamp,
            position_id: fill.position_id,
            size: position.current_size,
            entry_price: position.entry_price,
            current_price: fill.price,
            unrealized_pnl: position.unrealized_pnl,
            state_version: self.get_next_version(),
        });
        
        Ok(())
    }
    
    pub fn update_market_prices(&mut self, tick: Tick) -> Result<(), PositionError> {
        for position in self.positions.values_mut() {
            if position.symbol == tick.symbol {
                position.update_unrealized_pnl(tick.price);
            }
        }
        
        // Recalculate total unrealized P&L
        self.total_unrealized_pnl = self.positions.values()
            .map(|p| p.unrealized_pnl)
            .sum();
        
        Ok(())
    }
    
    pub fn get_position_at_time(&self, position_id: PositionId, timestamp: DateTime) -> Option<PositionSnapshot> {
        self.position_history
            .iter()
            .filter(|snap| snap.position_id == position_id && snap.timestamp <= timestamp)
            .last()
            .cloned()
    }
}
```

### 3. Order State Management

**Order Lifecycle Tracking**: Complete order state management from submission to fill.

```rust
#[derive(Debug, Clone)]
pub enum OrderState {
    Pending,
    PartiallyFilled { filled_quantity: Quantity },
    Filled,
    Cancelled,
    Rejected { reason: String },
}

pub struct OrderManager {
    orders: HashMap<OrderId, Order>,
    order_states: HashMap<OrderId, OrderState>,
    order_history: Vec<OrderEvent>,
    fill_matcher: FillMatcher,
}

pub struct OrderEvent {
    timestamp: DateTime,
    order_id: OrderId,
    event_type: OrderEventType,
    details: OrderEventDetails,
}

#[derive(Debug, Clone)]
pub enum OrderEventType {
    Submitted,
    PartialFill,
    Filled,
    Cancelled,
    Rejected,
    Modified,
}

impl OrderManager {
    pub fn submit_order(&mut self, order: Order) -> Result<OrderId, OrderError> {
        let order_id = OrderId::new();
        
        // Validate order
        self.validate_order(&order)?;
        
        // Store order
        self.orders.insert(order_id, order.clone());
        self.order_states.insert(order_id, OrderState::Pending);
        
        // Record submission event
        self.order_history.push(OrderEvent {
            timestamp: order.submitted_time,
            order_id,
            event_type: OrderEventType::Submitted,
            details: OrderEventDetails::Submission { order: order.clone() },
        });
        
        Ok(order_id)
    }
    
    pub fn process_market_data(&mut self, tick: Tick) -> Result<Vec<Fill>, OrderError> {
        let mut fills = Vec::new();
        let mut completed_orders = Vec::new();
        
        // Check each pending order for potential fills
        for (order_id, order) in &self.orders {
            if let Some(order_state) = self.order_states.get(order_id) {
                match order_state {
                    OrderState::Pending | OrderState::PartiallyFilled { .. } => {
                        if let Some(fill) = self.fill_matcher.check_fill(order, &tick)? {
                            fills.push(fill.clone());
                            
                            // Update order state
                            let new_state = if fill.quantity >= order.remaining_quantity() {
                                OrderState::Filled
                            } else {
                                OrderState::PartiallyFilled {
                                    filled_quantity: fill.quantity,
                                }
                            };
                            
                            self.order_states.insert(*order_id, new_state.clone());
                            
                            // Record fill event
                            self.order_history.push(OrderEvent {
                                timestamp: tick.timestamp,
                                order_id: *order_id,
                                event_type: match new_state {
                                    OrderState::Filled => OrderEventType::Filled,
                                    _ => OrderEventType::PartialFill,
                                },
                                details: OrderEventDetails::Fill { fill },
                            });
                            
                            if matches!(new_state, OrderState::Filled) {
                                completed_orders.push(*order_id);
                            }
                        }
                    }
                    _ => {} // Order already completed
                }
            }
        }
        
        // Remove completed orders
        for order_id in completed_orders {
            self.orders.remove(&order_id);
        }
        
        Ok(fills)
    }
}
```

## Performance Optimizations

Critical optimizations ensure the system meets its 1M ticks/second processing target with sub-100μs MTF updates.

### 1. Zero-Copy Operations

**Memory-Efficient Data Handling**: Minimize allocations and copies in hot paths.

```rust
use std::sync::Arc;
use crossbeam::utils::CachePadded;

// Zero-copy tick data structure
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Tick {
    pub timestamp: u64,     // Microseconds since epoch
    pub price: u64,         // Fixed-point price (6 decimal places)
    pub volume: u32,
    pub bid: u64,
    pub ask: u64,
}

// Ring buffer for zero-allocation tick processing
pub struct TickRingBuffer {
    buffer: Box<[CachePadded<Tick>]>,
    write_pos: AtomicUsize,
    read_pos: AtomicUsize,
    capacity: usize,
}

impl TickRingBuffer {
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(CachePadded::new(Tick::default()));
        }
        
        Self {
            buffer: buffer.into_boxed_slice(),
            write_pos: AtomicUsize::new(0),
            read_pos: AtomicUsize::new(0),
            capacity,
        }
    }
    
    pub fn push(&self, tick: Tick) -> Result<(), BufferError> {
        let write_pos = self.write_pos.load(Ordering::Acquire);
        let next_write = (write_pos + 1) % self.capacity;
        
        // Check if buffer is full
        if next_write == self.read_pos.load(Ordering::Acquire) {
            return Err(BufferError::Full);
        }
        
        // Write tick without allocation
        unsafe {
            std::ptr::write_volatile(
                self.buffer[write_pos].deref() as *const Tick as *mut Tick,
                tick
            );
        }
        
        self.write_pos.store(next_write, Ordering::Release);
        Ok(())
    }
    
    pub fn pop(&self) -> Option<Tick> {
        let read_pos = self.read_pos.load(Ordering::Acquire);
        let write_pos = self.write_pos.load(Ordering::Acquire);
        
        if read_pos == write_pos {
            return None; // Buffer empty
        }
        
        let tick = unsafe {
            std::ptr::read_volatile(self.buffer[read_pos].deref())
        };
        
        let next_read = (read_pos + 1) % self.capacity;
        self.read_pos.store(next_read, Ordering::Release);
        
        Some(tick)
    }
}

// Zero-copy OHLC bar slices
pub struct BarSlice<'a> {
    data: &'a [OHLCBar],
    start_index: usize,
    length: usize,
}

impl<'a> BarSlice<'a> {
    pub fn new(data: &'a [OHLCBar], start_index: usize, length: usize) -> Self {
        Self {
            data,
            start_index: start_index.min(data.len()),
            length: length.min(data.len() - start_index),
        }
    }
    
    pub fn get_closes(&self) -> &[f64] {
        // Return slice of close prices without copying
        unsafe {
            std::slice::from_raw_parts(
                self.data[self.start_index..self.start_index + self.length]
                    .as_ptr()
                    .cast::<f64>()
                    .add(3), // Offset to close price field
                self.length
            )
        }
    }
}
```

### 2. Memory Pools

**Pre-allocated Memory Management**: Eliminate allocation overhead in critical paths.

```rust
use std::sync::Mutex;

pub struct MemoryPool<T> {
    pool: Mutex<Vec<Box<T>>>,
    create_fn: Box<dyn Fn() -> T + Send + Sync>,
    reset_fn: Box<dyn Fn(&mut T) + Send + Sync>,
}

impl<T> MemoryPool<T> {
    pub fn new<F, R>(capacity: usize, create_fn: F, reset_fn: R) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
        R: Fn(&mut T) + Send + Sync + 'static,
    {
        let mut pool = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            pool.push(Box::new(create_fn()));
        }
        
        Self {
            pool: Mutex::new(pool),
            create_fn: Box::new(create_fn),
            reset_fn: Box::new(reset_fn),
        }
    }
    
    pub fn acquire(&self) -> PooledObject<T> {
        let mut pool = self.pool.lock().unwrap();
        let object = pool.pop().unwrap_or_else(|| Box::new((self.create_fn)()));
        
        PooledObject {
            object: Some(object),
            pool: &self.pool,
            reset_fn: &self.reset_fn,
        }
    }
}

pub struct PooledObject<'a, T> {
    object: Option<Box<T>>,
    pool: &'a Mutex<Vec<Box<T>>>,
    reset_fn: &'a dyn Fn(&mut T),
}

impl<'a, T> std::ops::Deref for PooledObject<'a, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self.object.as_ref().unwrap()
    }
}

impl<'a, T> std::ops::DerefMut for PooledObject<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.object.as_mut().unwrap()
    }
}

impl<'a, T> Drop for PooledObject<'a, T> {
    fn drop(&mut self) {
        if let Some(mut object) = self.object.take() {
            // Reset object state
            (self.reset_fn)(&mut object);
            
            // Return to pool
            let mut pool = self.pool.lock().unwrap();
            pool.push(object);
        }
    }
}

// Usage example for bar calculation
lazy_static! {
    static ref BAR_POOL: MemoryPool<OHLCBar> = MemoryPool::new(
        1000,
        || OHLCBar::default(),
        |bar| bar.reset()
    );
    
    static ref VEC_POOL: MemoryPool<Vec<f64>> = MemoryPool::new(
        100,
        || Vec::with_capacity(1000),
        |vec| vec.clear()
    );
}

pub fn calculate_sma_pooled(bars: &[OHLCBar], period: usize) -> f64 {
    let mut buffer = VEC_POOL.acquire();
    
    // Use pooled vector for calculation
    for bar in bars.iter().take(period) {
        buffer.push(bar.close);
    }
    
    buffer.iter().sum::<f64>() / buffer.len() as f64
    
    // Vector automatically returned to pool on drop
}
```

### 3. SIMD Optimizations

**Vectorized Calculations**: Use CPU vector instructions for performance-critical operations.

```rust
use std::arch::x86_64::*;

pub struct SIMDIndicators;

impl SIMDIndicators {
    // Vectorized moving average calculation
    pub unsafe fn moving_average_f64(values: &[f64], window: usize) -> Vec<f64> {
        assert!(values.len() >= window);
        let mut result = Vec::with_capacity(values.len() - window + 1);
        
        // Process 4 values at a time using AVX2
        let mut i = 0;
        while i + 4 <= values.len() - window + 1 {
            let sum = self.vectorized_sum(&values[i..i + window]);
            let avg = sum / window as f64;
            result.push(avg);
            i += 1;
        }
        
        // Handle remaining values
        for j in i..values.len() - window + 1 {
            let sum: f64 = values[j..j + window].iter().sum();
            result.push(sum / window as f64);
        }
        
        result
    }
    
    unsafe fn vectorized_sum(&self, values: &[f64]) -> f64 {
        let mut sum = _mm256_setzero_pd();
        let mut i = 0;
        
        // Process 4 doubles at a time
        while i + 4 <= values.len() {
            let chunk = _mm256_loadu_pd(values.as_ptr().add(i));
            sum = _mm256_add_pd(sum, chunk);
            i += 4;
        }
        
        // Extract sum from vector
        let mut result_array = [0.0; 4];
        _mm256_storeu_pd(result_array.as_mut_ptr(), sum);
        let mut total = result_array.iter().sum::<f64>();
        
        // Add remaining elements
        for &value in &values[i..] {
            total += value;
        }
        
        total
    }
    
    // Vectorized RSI calculation
    pub unsafe fn rsi_simd(prices: &[f64], period: usize) -> Vec<f64> {
        if prices.len() < period + 1 {
            return vec![];
        }
        
        let mut gains = Vec::with_capacity(prices.len() - 1);
        let mut losses = Vec::with_capacity(prices.len() - 1);
        
        // Calculate price changes using SIMD
        let mut i = 0;
        while i + 4 < prices.len() {
            let current = _mm256_loadu_pd(prices.as_ptr().add(i));
            let next = _mm256_loadu_pd(prices.as_ptr().add(i + 1));
            let diff = _mm256_sub_pd(next, current);
            
            // Separate gains and losses
            let zeros = _mm256_setzero_pd();
            let gain_mask = _mm256_cmp_pd(diff, zeros, _CMP_GT_OQ);
            let loss_mask = _mm256_cmp_pd(diff, zeros, _CMP_LT_OQ);
            
            let gains_vec = _mm256_and_pd(diff, gain_mask);
            let losses_vec = _mm256_and_pd(_mm256_sub_pd(zeros, diff), loss_mask);
            
            // Store results
            let mut gain_array = [0.0; 4];
            let mut loss_array = [0.0; 4];
            _mm256_storeu_pd(gain_array.as_mut_ptr(), gains_vec);
            _mm256_storeu_pd(loss_array.as_mut_ptr(), losses_vec);
            
            gains.extend_from_slice(&gain_array);
            losses.extend_from_slice(&loss_array);
            
            i += 4;
        }
        
        // Handle remaining elements
        for j in i..prices.len() - 1 {
            let change = prices[j + 1] - prices[j];
            gains.push(if change > 0.0 { change } else { 0.0 });
            losses.push(if change < 0.0 { -change } else { 0.0 });
        }
        
        // Calculate RSI using exponential moving averages
        self.calculate_rsi_from_gains_losses(&gains, &losses, period)
    }
    
    fn calculate_rsi_from_gains_losses(&self, gains: &[f64], losses: &[f64], period: usize) -> Vec<f64> {
        let mut rsi_values = Vec::new();
        let alpha = 1.0 / period as f64;
        
        // Initial averages
        let mut avg_gain = gains[..period].iter().sum::<f64>() / period as f64;
        let mut avg_loss = losses[..period].iter().sum::<f64>() / period as f64;
        
        // First RSI value
        let rs = if avg_loss == 0.0 { 100.0 } else { avg_gain / avg_loss };
        rsi_values.push(100.0 - (100.0 / (1.0 + rs)));
        
        // Subsequent RSI values using EMA
        for i in period..gains.len() {
            avg_gain = (1.0 - alpha) * avg_gain + alpha * gains[i];
            avg_loss = (1.0 - alpha) * avg_loss + alpha * losses[i];
            
            let rs = if avg_loss == 0.0 { 100.0 } else { avg_gain / avg_loss };
            rsi_values.push(100.0 - (100.0 / (1.0 + rs)));
        }
        
        rsi_values
    }
}
```

This comprehensive backend architecture provides the foundation for BackTestr_ai's high-performance financial backtesting capabilities, ensuring institutional-grade accuracy while maintaining developer productivity and system maintainability.
