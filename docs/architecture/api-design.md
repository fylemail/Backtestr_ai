# API Design

BackTestr_ai's API design focuses on Inter-Process Communication (IPC) between the Electron frontend and Rust backend, rather than traditional REST APIs. This approach provides optimal performance for a desktop application with real-time data requirements and complex computational workloads.

## IPC Protocol Design

### Protocol Overview & Specification

BackTestr_ai implements a custom high-performance IPC protocol designed specifically for real-time financial data communication between the Rust backend engine and Electron frontend. The protocol is built on MessagePack binary serialization for maximum efficiency and supports multiple communication patterns.

**Protocol Characteristics:**
- **Transport Layer**: Named Pipes (Windows-optimized)
- **Serialization**: MessagePack binary format with schema versioning
- **Message Patterns**: Request/Response, Pub/Sub, Streaming, Command/Event
- **Performance Target**: <100μs round-trip latency for critical messages
- **Reliability**: Automatic reconnection, message ordering, duplicate detection
- **Security**: Message authentication, replay protection, secure channels

**Communication Architecture:**
```
┌─────────────────┐    IPC Channel     ┌──────────────────┐
│   Electron UI   │◄─────────────────►│   Rust Engine    │
│   Process       │   Named Pipe      │   Process        │
│                 │   (Windows)       │                  │
│ ┌─────────────┐ │                   │ ┌──────────────┐ │
│ │IPC Manager  │ │                   │ │IPC Server    │ │
│ │- Reconnect  │ │                   │ │- Handler Pool│ │
│ │- Queue Mgmt │ │                   │ │- Broadcasting│ │
│ │- Health Mon │ │                   │ │- Auth        │ │
│ └─────────────┘ │                   │ └──────────────┘ │
└─────────────────┘                   └──────────────────┘
```

**Protocol Stack:**
1. **Application Layer**: Command/Response handling, event distribution
2. **Session Layer**: Connection management, authentication, heartbeat
3. **Transport Layer**: Named pipes with message framing
4. **Serialization Layer**: MessagePack with type safety and versioning

### 1. MessagePack Protocol Foundation

**Binary Serialization**: High-performance, compact binary format optimized for cross-language communication.

```rust
// Rust - Core message structure
use serde::{Deserialize, Serialize};
use rmp_serde as rmps;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IPCMessage {
    pub id: Uuid,
    pub timestamp: u64,
    pub message_type: MessageType,
    pub payload: MessagePayload,
    pub correlation_id: Option<Uuid>, // For request/response correlation
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageType {
    Request,
    Response,
    Event,
    Stream,
    Error,
    Heartbeat,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum MessagePayload {
    // Control Commands
    StartBacktest(BacktestConfig),
    StopBacktest { backtest_id: Uuid },
    PauseBacktest { backtest_id: Uuid },
    ResumeBacktest { backtest_id: Uuid },
    
    // Data Requests
    GetMarketData(MarketDataRequest),
    GetBacktestResults { backtest_id: Uuid },
    GetPerformanceMetrics { backtest_id: Uuid },
    GetAlgorithmStatus { algorithm_id: Uuid },
    
    // Real-time Streams
    PriceUpdate(PriceData),
    BacktestProgress(BacktestProgressData),
    OrderUpdate(OrderData),
    PositionUpdate(PositionData),
    
    // System Events
    SystemStatus(SystemStatusData),
    Error(ErrorData),
    
    // Responses
    Success { data: serde_json::Value },
    Failure { error: ErrorData },
}

// Serialization helpers
impl IPCMessage {
    pub fn serialize(&self) -> Result<Vec<u8>, rmps::encode::Error> {
        rmps::to_vec(self)
    }
    
    pub fn deserialize(data: &[u8]) -> Result<Self, rmps::decode::Error> {
        rmps::from_slice(data)
    }
    
    pub fn new_request(payload: MessagePayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            message_type: MessageType::Request,
            payload,
            correlation_id: None,
        }
    }
    
    pub fn new_response(request_id: Uuid, payload: MessagePayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            message_type: MessageType::Response,
            payload,
            correlation_id: Some(request_id),
        }
    }
}
```

```typescript
// TypeScript - Frontend message handling
interface IPCMessage {
  id: string;
  timestamp: number;
  messageType: MessageType;
  payload: MessagePayload;
  correlationId?: string;
}

enum MessageType {
  Request = 'Request',
  Response = 'Response',
  Event = 'Event',
  Stream = 'Stream',
  Error = 'Error',
  Heartbeat = 'Heartbeat',
}

type MessagePayload = 
  | { type: 'StartBacktest'; data: BacktestConfig }
  | { type: 'StopBacktest'; data: { backtestId: string } }
  | { type: 'GetMarketData'; data: MarketDataRequest }
  | { type: 'PriceUpdate'; data: PriceData }
  | { type: 'BacktestProgress'; data: BacktestProgressData }
  | { type: 'Success'; data: any }
  | { type: 'Failure'; data: ErrorData };

class IPCClient {
  private pendingRequests = new Map<string, {
    resolve: (value: any) => void;
    reject: (error: any) => void;
    timeout: NodeJS.Timeout;
  }>();
  
  constructor() {
    window.electronAPI.onMessage(this.handleMessage.bind(this));
  }
  
  async sendRequest<T>(payload: MessagePayload, timeoutMs = 10000): Promise<T> {
    const message: IPCMessage = {
      id: crypto.randomUUID(),
      timestamp: Date.now(),
      messageType: MessageType.Request,
      payload,
    };
    
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.pendingRequests.delete(message.id);
        reject(new Error(`Request timeout: ${message.id}`));
      }, timeoutMs);
      
      this.pendingRequests.set(message.id, { resolve, reject, timeout });
      window.electronAPI.sendMessage(message);
    });
  }
  
  private handleMessage(message: IPCMessage) {
    if (message.messageType === MessageType.Response && message.correlationId) {
      const pending = this.pendingRequests.get(message.correlationId);
      if (pending) {
        clearTimeout(pending.timeout);
        this.pendingRequests.delete(message.correlationId);
        
        if (message.payload.type === 'Success') {
          pending.resolve(message.payload.data);
        } else {
          pending.reject(new Error(message.payload.data.message));
        }
      }
    }
  }
}
```

### 2. Request/Response Patterns

**Async Request/Response**: Standardized patterns for command execution and data retrieval.

```rust
// Rust - Request handler
use tokio::sync::{mpsc, oneshot};
use std::collections::HashMap;

pub struct IPCRequestHandler {
    pending_requests: Arc<Mutex<HashMap<Uuid, oneshot::Sender<MessagePayload>>>>,
    backtest_engine: Arc<BacktestEngine>,
    market_data_service: Arc<MarketDataService>,
}

impl IPCRequestHandler {
    pub async fn handle_request(&self, message: IPCMessage) -> Result<IPCMessage, IPCError> {
        let response_payload = match message.payload {
            MessagePayload::StartBacktest(config) => {
                match self.backtest_engine.start_backtest(config).await {
                    Ok(backtest_id) => MessagePayload::Success {
                        data: serde_json::json!({ "backtestId": backtest_id })
                    },
                    Err(e) => MessagePayload::Failure {
                        error: ErrorData {
                            code: "BACKTEST_START_FAILED".to_string(),
                            message: e.to_string(),
                            details: None,
                        }
                    }
                }
            },
            
            MessagePayload::GetMarketData(request) => {
                match self.market_data_service.get_data(request).await {
                    Ok(data) => MessagePayload::Success {
                        data: serde_json::to_value(data)?
                    },
                    Err(e) => MessagePayload::Failure {
                        error: ErrorData {
                            code: "MARKET_DATA_FAILED".to_string(),
                            message: e.to_string(),
                            details: Some(serde_json::json!({ "request": request })),
                        }
                    }
                }
            },
            
            _ => MessagePayload::Failure {
                error: ErrorData {
                    code: "UNSUPPORTED_REQUEST".to_string(),
                    message: "Request type not supported".to_string(),
                    details: None,
                }
            }
        };
        
        Ok(IPCMessage::new_response(message.id, response_payload))
    }
    
    pub async fn start_request_processor(&self, mut rx: mpsc::Receiver<IPCMessage>) {
        while let Some(message) = rx.recv().await {
            let handler = self.clone();
            tokio::spawn(async move {
                if let Ok(response) = handler.handle_request(message).await {
                    // Send response back through IPC channel
                    handler.send_response(response).await;
                }
            });
        }
    }
}
```

### 3. Connection Management

**Robust Connection Handling**: Automatic reconnection, heartbeat monitoring, and graceful degradation.

```rust
// Rust - Connection manager
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, WebSocketStream};

pub struct IPCConnectionManager {
    connections: Arc<Mutex<HashMap<Uuid, WebSocketStream<TcpStream>>>>,
    heartbeat_interval: Duration,
}

impl IPCConnectionManager {
    pub async fn start_server(&self, port: u16) -> Result<(), IPCError> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        
        while let Ok((stream, _)) = listener.accept().await {
            let ws_stream = accept_async(stream).await?;
            let connection_id = Uuid::new_v4();
            
            self.connections.lock().await.insert(connection_id, ws_stream);
            
            // Start heartbeat for this connection
            self.start_heartbeat(connection_id).await;
        }
        
        Ok(())
    }
    
    async fn start_heartbeat(&self, connection_id: Uuid) {
        let connections = self.connections.clone();
        let interval = self.heartbeat_interval;
        
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            
            loop {
                ticker.tick().await;
                
                let heartbeat = IPCMessage {
                    id: Uuid::new_v4(),
                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                    message_type: MessageType::Heartbeat,
                    payload: MessagePayload::SystemStatus(SystemStatusData {
                        status: "healthy".to_string(),
                        uptime: get_uptime(),
                        memory_usage: get_memory_usage(),
                    }),
                    correlation_id: None,
                };
                
                if let Err(_) = self.send_to_connection(connection_id, heartbeat).await {
                    // Connection lost, clean up
                    connections.lock().await.remove(&connection_id);
                    break;
                }
            }
        });
    }
}
```

## Message Types & Contracts

### 1. Data Streaming Messages

**Real-time Market Data**: Optimized for high-frequency price updates and chart rendering.

```rust
// Rust - Market data streaming
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceData {
    pub symbol: String,
    pub timestamp: u64,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: u64,
    pub timeframe: TimeFrame,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BacktestProgressData {
    pub backtest_id: Uuid,
    pub current_time: u64,
    pub progress_percent: f32,
    pub orders_executed: u32,
    pub positions_opened: u32,
    pub positions_closed: u32,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub equity_curve_point: EquityCurvePoint,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EquityCurvePoint {
    pub timestamp: u64,
    pub equity: f64,
    pub drawdown: f64,
    pub returns: f64,
}

// Streaming service
pub struct StreamingService {
    subscribers: Arc<Mutex<HashMap<String, Vec<mpsc::Sender<IPCMessage>>>>>,
}

impl StreamingService {
    pub async fn subscribe(&self, topic: &str, sender: mpsc::Sender<IPCMessage>) {
        self.subscribers
            .lock()
            .await
            .entry(topic.to_string())
            .or_insert_with(Vec::new)
            .push(sender);
    }
    
    pub async fn publish(&self, topic: &str, data: MessagePayload) {
        let message = IPCMessage {
            id: Uuid::new_v4(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            message_type: MessageType::Stream,
            payload: data,
            correlation_id: None,
        };
        
        if let Some(subscribers) = self.subscribers.lock().await.get(topic) {
            let mut to_remove = Vec::new();
            
            for (i, sender) in subscribers.iter().enumerate() {
                if sender.send(message.clone()).await.is_err() {
                    to_remove.push(i);
                }
            }
            
            // Clean up disconnected subscribers
            for &index in to_remove.iter().rev() {
                self.subscribers.lock().await.get_mut(topic).unwrap().remove(index);
            }
        }
    }
}
```

### 2. Control Commands

**Algorithm Execution Control**: Commands for managing backtest lifecycle and algorithm execution.

```rust
// Rust - Control command contracts
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BacktestConfig {
    pub algorithm_id: Uuid,
    pub symbol: String,
    pub start_date: String,
    pub end_date: String,
    pub initial_capital: f64,
    pub timeframe: TimeFrame,
    pub commission: f64,
    pub slippage: f64,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketDataRequest {
    pub symbol: String,
    pub timeframe: TimeFrame,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub limit: Option<u32>,
    pub indicators: Vec<IndicatorConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndicatorConfig {
    pub name: String,
    pub parameters: HashMap<String, f64>,
    pub output_names: Vec<String>,
}

// Command execution
pub struct CommandProcessor {
    backtest_engine: Arc<BacktestEngine>,
    algorithm_manager: Arc<AlgorithmManager>,
}

impl CommandProcessor {
    pub async fn execute_command(&self, command: MessagePayload) -> Result<MessagePayload, IPCError> {
        match command {
            MessagePayload::StartBacktest(config) => {
                // Validate configuration
                self.validate_backtest_config(&config)?;
                
                // Start backtest with streaming updates
                let backtest_id = self.backtest_engine.start_backtest(config).await?;
                
                Ok(MessagePayload::Success {
                    data: serde_json::json!({
                        "backtestId": backtest_id,
                        "status": "started"
                    })
                })
            },
            
            MessagePayload::StopBacktest { backtest_id } => {
                self.backtest_engine.stop_backtest(backtest_id).await?;
                
                Ok(MessagePayload::Success {
                    data: serde_json::json!({
                        "backtestId": backtest_id,
                        "status": "stopped"
                    })
                })
            },
            
            _ => Err(IPCError::UnsupportedCommand),
        }
    }
    
    fn validate_backtest_config(&self, config: &BacktestConfig) -> Result<(), IPCError> {
        if config.initial_capital <= 0.0 {
            return Err(IPCError::InvalidConfiguration("Initial capital must be positive".to_string()));
        }
        
        if config.start_date >= config.end_date {
            return Err(IPCError::InvalidConfiguration("Start date must be before end date".to_string()));
        }
        
        Ok(())
    }
}
```

### 3. Status Updates

**System Health Monitoring**: Comprehensive status reporting for system components.

```typescript
// TypeScript - Status update types
interface SystemStatusData {
  status: 'healthy' | 'warning' | 'error';
  uptime: number;
  memoryUsage: MemoryUsage;
  cpuUsage: number;
  activeBacktests: number;
  queuedRequests: number;
  lastHeartbeat: number;
}

interface MemoryUsage {
  used: number;
  total: number;
  percentage: number;
}

interface OrderData {
  orderId: string;
  backtestId: string;
  symbol: string;
  side: 'buy' | 'sell';
  quantity: number;
  price: number;
  orderType: 'market' | 'limit' | 'stop';
  status: 'pending' | 'filled' | 'cancelled' | 'rejected';
  timestamp: number;
  fillPrice?: number;
  fillQuantity?: number;
}

interface PositionData {
  positionId: string;
  backtestId: string;
  symbol: string;
  quantity: number;
  averagePrice: number;
  currentPrice: number;
  unrealizedPnL: number;
  realizedPnL: number;
  timestamp: number;
}

// Status monitoring service
class StatusMonitor {
  private lastHeartbeat = 0;
  private systemStatus: SystemStatusData | null = null;
  private statusCallbacks: Array<(status: SystemStatusData) => void> = [];
  
  constructor(private ipcClient: IPCClient) {
    this.setupStatusMonitoring();
  }
  
  private setupStatusMonitoring() {
    window.electronAPI.onMessage((message: IPCMessage) => {
      if (message.messageType === MessageType.Heartbeat) {
        this.handleHeartbeat(message);
      } else if (message.payload.type === 'SystemStatus') {
        this.handleStatusUpdate(message.payload.data);
      }
    });
    
    // Check for missed heartbeats
    setInterval(() => {
      const now = Date.now();
      if (now - this.lastHeartbeat > 30000) { // 30 second timeout
        this.handleConnectionLoss();
      }
    }, 5000);
  }
  
  private handleHeartbeat(message: IPCMessage) {
    this.lastHeartbeat = message.timestamp;
    if (message.payload.type === 'SystemStatus') {
      this.systemStatus = message.payload.data as SystemStatusData;
      this.notifyStatusCallbacks();
    }
  }
  
  public onStatusChange(callback: (status: SystemStatusData) => void): () => void {
    this.statusCallbacks.push(callback);
    return () => {
      const index = this.statusCallbacks.indexOf(callback);
      if (index > -1) {
        this.statusCallbacks.splice(index, 1);
      }
    };
  }
}
```

## Error Handling

### 1. Error Codes & Classification

**Structured Error System**: Comprehensive error categorization with recovery guidance.

```rust
// Rust - Error handling system
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize, Clone)]
pub enum IPCError {
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Algorithm error: {0}")]
    Algorithm(String),
    
    #[error("Data error: {0}")]
    Data(String),
    
    #[error("System error: {0}")]
    System(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Configuration error: {0}")]
    InvalidConfiguration(String),
    
    #[error("Unsupported command")]
    UnsupportedCommand,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorData {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub recovery_actions: Vec<RecoveryAction>,
    pub severity: ErrorSeverity,
    pub component: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecoveryAction {
    pub action: String,
    pub description: String,
    pub automatic: bool,
    pub user_required: bool,
}

impl From<IPCError> for ErrorData {
    fn from(error: IPCError) -> Self {
        let (code, severity, recovery_actions) = match &error {
            IPCError::Serialization(_) => (
                "IPC_SERIALIZATION_ERROR",
                ErrorSeverity::Error,
                vec![RecoveryAction {
                    action: "retry_request".to_string(),
                    description: "Retry the request with valid data format".to_string(),
                    automatic: false,
                    user_required: true,
                }]
            ),
            
            IPCError::Network(_) => (
                "IPC_NETWORK_ERROR",
                ErrorSeverity::Warning,
                vec![
                    RecoveryAction {
                        action: "reconnect".to_string(),
                        description: "Automatically reconnect to backend".to_string(),
                        automatic: true,
                        user_required: false,
                    },
                    RecoveryAction {
                        action: "restart_application".to_string(),
                        description: "Restart the application if connection cannot be restored".to_string(),
                        automatic: false,
                        user_required: true,
                    }
                ]
            ),
            
            IPCError::Algorithm(_) => (
                "ALGORITHM_ERROR",
                ErrorSeverity::Error,
                vec![RecoveryAction {
                    action: "check_algorithm_code".to_string(),
                    description: "Review and fix algorithm implementation".to_string(),
                    automatic: false,
                    user_required: true,
                }]
            ),
            
            _ => ("UNKNOWN_ERROR", ErrorSeverity::Error, vec![]),
        };
        
        Self {
            code: code.to_string(),
            message: error.to_string(),
            details: None,
            recovery_actions,
            severity,
            component: "IPC".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}
```

### 2. Recovery Strategies

**Automatic Recovery**: Self-healing mechanisms for common failure scenarios.

```rust
// Rust - Recovery strategies
pub struct RecoveryManager {
    max_retries: u32,
    backoff_strategy: BackoffStrategy,
    circuit_breaker: CircuitBreaker,
}

#[derive(Clone)]
pub enum BackoffStrategy {
    Linear { interval: Duration },
    Exponential { base: Duration, max: Duration },
    Fixed { interval: Duration },
}

impl RecoveryManager {
    pub async fn execute_with_recovery<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>> + Clone,
        E: std::fmt::Debug + Clone,
    {
        let mut attempts = 0;
        let mut delay = Duration::from_millis(100);
        
        loop {
            match self.circuit_breaker.can_execute() {
                false => return Err(/* Circuit breaker open error */),
                true => {}
            }
            
            match operation().await {
                Ok(result) => {
                    self.circuit_breaker.record_success();
                    return Ok(result);
                },
                Err(e) => {
                    attempts += 1;
                    self.circuit_breaker.record_failure();
                    
                    if attempts >= self.max_retries {
                        return Err(e);
                    }
                    
                    // Apply backoff strategy
                    match &self.backoff_strategy {
                        BackoffStrategy::Linear { interval } => {
                            delay = *interval * attempts;
                        },
                        BackoffStrategy::Exponential { base, max } => {
                            delay = std::cmp::min(*base * 2_u32.pow(attempts - 1), *max);
                        },
                        BackoffStrategy::Fixed { interval } => {
                            delay = *interval;
                        },
                    }
                    
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
}

pub struct CircuitBreaker {
    failure_threshold: u32,
    recovery_timeout: Duration,
    failure_count: Arc<AtomicU32>,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    state: Arc<Mutex<CircuitBreakerState>>,
}

#[derive(Debug, Clone)]
enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}
```

```typescript
// TypeScript - Frontend error handling
class ErrorHandler {
  private errorQueue: ErrorData[] = [];
  private maxQueueSize = 100;
  private errorCallbacks: Array<(error: ErrorData) => void> = [];
  
  constructor(private ipcClient: IPCClient) {
    this.setupErrorHandling();
  }
  
  private setupErrorHandling() {
    window.electronAPI.onMessage((message: IPCMessage) => {
      if (message.messageType === MessageType.Error) {
        this.handleError(message.payload.data as ErrorData);
      }
    });
  }
  
  public handleError(error: ErrorData) {
    // Add to error queue
    this.errorQueue.push(error);
    if (this.errorQueue.length > this.maxQueueSize) {
      this.errorQueue.shift();
    }
    
    // Execute automatic recovery actions
    this.executeAutomaticRecovery(error);
    
    // Notify error callbacks
    this.errorCallbacks.forEach(callback => callback(error));
    
    // Log error for debugging
    console.error(`[${error.component}] ${error.code}: ${error.message}`, error.details);
  }
  
  private async executeAutomaticRecovery(error: ErrorData) {
    for (const action of error.recovery_actions) {
      if (action.automatic) {
        try {
          await this.executeRecoveryAction(action);
        } catch (recoveryError) {
          console.error('Recovery action failed:', recoveryError);
        }
      }
    }
  }
  
  private async executeRecoveryAction(action: RecoveryAction): Promise<void> {
    switch (action.action) {
      case 'reconnect':
        await this.ipcClient.reconnect();
        break;
        
      case 'retry_request':
        // Retry logic would be handled by the specific component
        break;
        
      case 'clear_cache':
        localStorage.clear();
        break;
        
      default:
        console.warn(`Unknown recovery action: ${action.action}`);
    }
  }
  
  public getRecentErrors(severity?: ErrorSeverity): ErrorData[] {
    if (severity) {
      return this.errorQueue.filter(error => error.severity === severity);
    }
    return [...this.errorQueue];
  }
  
  public onError(callback: (error: ErrorData) => void): () => void {
    this.errorCallbacks.push(callback);
    return () => {
      const index = this.errorCallbacks.indexOf(callback);
      if (index > -1) {
        this.errorCallbacks.splice(index, 1);
      }
    };
  }
}
```

## Real-time Communication

### 1. Streaming Architecture

**WebSocket-like Streaming**: Bidirectional real-time data flow over IPC channels.

```rust
// Rust - Real-time streaming service
use tokio::sync::broadcast;
use futures_util::{SinkExt, StreamExt};

pub struct RealTimeStreamer {
    price_broadcast: broadcast::Sender<PriceData>,
    backtest_broadcast: broadcast::Sender<BacktestProgressData>,
    order_broadcast: broadcast::Sender<OrderData>,
    position_broadcast: broadcast::Sender<PositionData>,
    subscribers: Arc<Mutex<HashMap<Uuid, StreamSubscription>>>,
}

#[derive(Debug, Clone)]
pub struct StreamSubscription {
    pub connection_id: Uuid,
    pub topics: HashSet<String>,
    pub filters: HashMap<String, serde_json::Value>,
    pub last_activity: Instant,
}

impl RealTimeStreamer {
    pub fn new() -> Self {
        let (price_tx, _) = broadcast::channel(10000);
        let (backtest_tx, _) = broadcast::channel(1000);
        let (order_tx, _) = broadcast::channel(5000);
        let (position_tx, _) = broadcast::channel(1000);
        
        Self {
            price_broadcast: price_tx,
            backtest_broadcast: backtest_tx,
            order_broadcast: order_tx,
            position_broadcast: position_tx,
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub async fn subscribe(&self, connection_id: Uuid, topics: Vec<String>) -> Result<(), IPCError> {
        let mut subscribers = self.subscribers.lock().await;
        
        let subscription = StreamSubscription {
            connection_id,
            topics: topics.into_iter().collect(),
            filters: HashMap::new(),
            last_activity: Instant::now(),
        };
        
        subscribers.insert(connection_id, subscription);
        
        // Start streaming to this subscriber
        self.start_streaming_to_subscriber(connection_id).await;
        
        Ok(())
    }
    
    async fn start_streaming_to_subscriber(&self, connection_id: Uuid) {
        let subscribers = self.subscribers.clone();
        let price_rx = self.price_broadcast.subscribe();
        let backtest_rx = self.backtest_broadcast.subscribe();
        let order_rx = self.order_broadcast.subscribe();
        let position_rx = self.position_broadcast.subscribe();
        
        tokio::spawn(async move {
            let mut price_stream = BroadcastStream::new(price_rx);
            let mut backtest_stream = BroadcastStream::new(backtest_rx);
            let mut order_stream = BroadcastStream::new(order_rx);
            let mut position_stream = BroadcastStream::new(position_rx);
            
            loop {
                tokio::select! {
                    price_data = price_stream.next() => {
                        if let Some(Ok(data)) = price_data {
                            if Self::should_send_to_subscriber(&subscribers, connection_id, "price_updates").await {
                                Self::send_stream_message(connection_id, MessagePayload::PriceUpdate(data)).await;
                            }
                        }
                    },
                    
                    backtest_data = backtest_stream.next() => {
                        if let Some(Ok(data)) = backtest_data {
                            if Self::should_send_to_subscriber(&subscribers, connection_id, "backtest_progress").await {
                                Self::send_stream_message(connection_id, MessagePayload::BacktestProgress(data)).await;
                            }
                        }
                    },
                    
                    order_data = order_stream.next() => {
                        if let Some(Ok(data)) = order_data {
                            if Self::should_send_to_subscriber(&subscribers, connection_id, "order_updates").await {
                                Self::send_stream_message(connection_id, MessagePayload::OrderUpdate(data)).await;
                            }
                        }
                    },
                    
                    position_data = position_stream.next() => {
                        if let Some(Ok(data)) = position_data {
                            if Self::should_send_to_subscriber(&subscribers, connection_id, "position_updates").await {
                                Self::send_stream_message(connection_id, MessagePayload::PositionUpdate(data)).await;
                            }
                        }
                    },
                    
                    // Clean up disconnected subscribers
                    _ = tokio::time::sleep(Duration::from_secs(60)) => {
                        let mut subs = subscribers.lock().await;
                        if let Some(sub) = subs.get(&connection_id) {
                            if sub.last_activity.elapsed() > Duration::from_secs(300) {
                                subs.remove(&connection_id);
                                break;
                            }
                        }
                    }
                }
            }
        });
    }
    
    pub async fn publish_price_update(&self, data: PriceData) {
        let _ = self.price_broadcast.send(data);
    }
    
    pub async fn publish_backtest_progress(&self, data: BacktestProgressData) {
        let _ = self.backtest_broadcast.send(data);
    }
    
    async fn should_send_to_subscriber(
        subscribers: &Arc<Mutex<HashMap<Uuid, StreamSubscription>>>,
        connection_id: Uuid,
        topic: &str,
    ) -> bool {
        if let Some(subscription) = subscribers.lock().await.get(&connection_id) {
            return subscription.topics.contains(topic);
        }
        false
    }
}
```

### 2. Data Buffering & Batching

**Efficient Data Transmission**: Smart batching and compression for high-frequency updates.

```rust
// Rust - Data batching service
pub struct DataBatcher {
    pending_batches: Arc<Mutex<HashMap<String, Vec<MessagePayload>>>>,
    batch_size: usize,
    batch_timeout: Duration,
    compression_threshold: usize,
}

impl DataBatcher {
    pub fn new() -> Self {
        Self {
            pending_batches: Arc::new(Mutex::new(HashMap::new())),
            batch_size: 100,
            batch_timeout: Duration::from_millis(50),
            compression_threshold: 1024,
        }
    }
    
    pub async fn add_to_batch(&self, topic: String, payload: MessagePayload) {
        let mut batches = self.pending_batches.lock().await;
        let batch = batches.entry(topic.clone()).or_insert_with(Vec::new);
        
        batch.push(payload);
        
        // Send batch if it reaches the size limit
        if batch.len() >= self.batch_size {
            let batch_data = batch.drain(..).collect();
            drop(batches);
            self.send_batch(topic, batch_data).await;
        }
    }
    
    pub async fn start_batch_timer(&self) {
        let batches = self.pending_batches.clone();
        let timeout = self.batch_timeout;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(timeout);
            
            loop {
                interval.tick().await;
                
                let mut to_send = Vec::new();
                {
                    let mut batch_map = batches.lock().await;
                    for (topic, batch) in batch_map.iter_mut() {
                        if !batch.is_empty() {
                            to_send.push((topic.clone(), batch.drain(..).collect()));
                        }
                    }
                }
                
                for (topic, batch_data) in to_send {
                    Self::send_batch_static(topic, batch_data).await;
                }
            }
        });
    }
    
    async fn send_batch(&self, topic: String, batch: Vec<MessagePayload>) {
        let batch_message = BatchMessage {
            topic,
            messages: batch,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            compressed: false,
        };
        
        let serialized = match rmp_serde::to_vec(&batch_message) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to serialize batch: {}", e);
                return;
            }
        };
        
        // Compress if data is large
        let final_data = if serialized.len() > self.compression_threshold {
            match self.compress_data(&serialized) {
                Ok(compressed) => {
                    let mut compressed_batch = batch_message;
                    compressed_batch.compressed = true;
                    compressed
                },
                Err(_) => serialized,
            }
        } else {
            serialized
        };
        
        // Send through IPC channel
        self.send_to_frontend(final_data).await;
    }
    
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::Write;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        encoder.finish()
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct BatchMessage {
    topic: String,
    messages: Vec<MessagePayload>,
    timestamp: u64,
    compressed: bool,
}
```

```typescript
// TypeScript - Frontend batch processing
class DataBatchProcessor {
  private decompressionWorker: Worker;
  private processingQueue: BatchMessage[] = [];
  
  constructor() {
    this.decompressionWorker = new Worker('/workers/decompression-worker.js');
    this.setupBatchProcessing();
  }
  
  private setupBatchProcessing() {
    window.electronAPI.onMessage((message: IPCMessage) => {
      if (message.payload.type === 'Batch') {
        this.processBatch(message.payload.data as BatchMessage);
      }
    });
  }
  
  private async processBatch(batch: BatchMessage) {
    let messages = batch.messages;
    
    // Decompress if needed
    if (batch.compressed) {
      messages = await this.decompressBatch(batch);
    }
    
    // Process messages based on topic
    switch (batch.topic) {
      case 'price_updates':
        this.processPriceUpdates(messages);
        break;
        
      case 'backtest_progress':
        this.processBacktestProgress(messages);
        break;
        
      case 'order_updates':
        this.processOrderUpdates(messages);
        break;
        
      default:
        console.warn(`Unknown batch topic: ${batch.topic}`);
    }
  }
  
  private async decompressBatch(batch: BatchMessage): Promise<MessagePayload[]> {
    return new Promise((resolve, reject) => {
      this.decompressionWorker.postMessage({
        type: 'decompress',
        data: batch.messages,
      });
      
      this.decompressionWorker.onmessage = (event) => {
        if (event.data.success) {
          resolve(event.data.messages);
        } else {
          reject(new Error(event.data.error));
        }
      };
    });
  }
  
  private processPriceUpdates(messages: MessagePayload[]) {
    // Batch update chart data
    const priceUpdates = messages
      .filter(msg => msg.type === 'PriceUpdate')
      .map(msg => msg.data as PriceData);
    
    if (priceUpdates.length > 0) {
      // Use requestAnimationFrame for smooth chart updates
      requestAnimationFrame(() => {
        this.updateCharts(priceUpdates);
      });
    }
  }
  
  private updateCharts(priceUpdates: PriceData[]) {
    // Efficient chart updates using canvas or WebGL
    const chartService = ChartService.getInstance();
    chartService.batchUpdatePrices(priceUpdates);
  }
}
```

## State Synchronization

### 1. State Management Architecture

**Consistent State Across Processes**: Ensuring frontend and backend maintain synchronized state.

```rust
// Rust - State synchronization manager
use dashmap::DashMap;
use serde_json::Value;

pub struct StateSyncManager {
    state_store: Arc<DashMap<String, StateEntry>>,
    subscribers: Arc<DashMap<String, Vec<StateSubscriber>>>,
    sync_interval: Duration,
}

#[derive(Debug, Clone)]
struct StateEntry {
    value: Value,
    version: u64,
    last_modified: Instant,
    checksum: u64,
}

#[derive(Debug, Clone)]
struct StateSubscriber {
    connection_id: Uuid,
    last_sync_version: u64,
}

impl StateSyncManager {
    pub fn new() -> Self {
        Self {
            state_store: Arc::new(DashMap::new()),
            subscribers: Arc::new(DashMap::new()),
            sync_interval: Duration::from_millis(100),
        }
    }
    
    pub async fn update_state(&self, key: &str, value: Value) -> Result<(), StateError> {
        let checksum = self.calculate_checksum(&value);
        let now = Instant::now();
        
        let entry = StateEntry {
            value: value.clone(),
            version: self.get_next_version(key),
            last_modified: now,
            checksum,
        };
        
        self.state_store.insert(key.to_string(), entry.clone());
        
        // Notify subscribers of state change
        self.notify_state_change(key, &entry).await;
        
        Ok(())
    }
    
    pub fn get_state(&self, key: &str) -> Option<(Value, u64)> {
        self.state_store.get(key).map(|entry| {
            (entry.value.clone(), entry.version)
        })
    }
    
    pub async fn subscribe_to_state(&self, connection_id: Uuid, key: &str) {
        let subscriber = StateSubscriber {
            connection_id,
            last_sync_version: 0,
        };
        
        self.subscribers
            .entry(key.to_string())
            .or_insert_with(Vec::new)
            .push(subscriber);
        
        // Send current state to new subscriber
        if let Some(entry) = self.state_store.get(key) {
            self.send_state_update(connection_id, key, &entry).await;
        }
    }
    
    pub async fn start_sync_service(&self) {
        let state_store = self.state_store.clone();
        let subscribers = self.subscribers.clone();
        let interval = self.sync_interval;
        
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            
            loop {
                ticker.tick().await;
                
                // Check for state conflicts and resolve them
                for mut entry in state_store.iter_mut() {
                    let key = entry.key();
                    let state_entry = entry.value_mut();
                    
                    // Validate state integrity
                    let current_checksum = Self::calculate_checksum_static(&state_entry.value);
                    if current_checksum != state_entry.checksum {
                        // State corruption detected, trigger recovery
                        eprintln!("State corruption detected for key: {}", key);
                        // Implement recovery logic here
                    }
                }
                
                // Send incremental updates to subscribers
                Self::send_incremental_updates(&state_store, &subscribers).await;
            }
        });
    }
    
    async fn notify_state_change(&self, key: &str, entry: &StateEntry) {
        if let Some(subs) = self.subscribers.get(key) {
            for subscriber in subs.iter() {
                if subscriber.last_sync_version < entry.version {
                    self.send_state_update(subscriber.connection_id, key, entry).await;
                }
            }
        }
    }
    
    async fn send_state_update(&self, connection_id: Uuid, key: &str, entry: &StateEntry) {
        let update_message = StateUpdateMessage {
            key: key.to_string(),
            value: entry.value.clone(),
            version: entry.version,
            timestamp: entry.last_modified.elapsed().as_millis() as u64,
        };
        
        let ipc_message = IPCMessage {
            id: Uuid::new_v4(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            message_type: MessageType::Event,
            payload: MessagePayload::StateUpdate(update_message),
            correlation_id: None,
        };
        
        // Send through IPC channel
        if let Err(e) = self.send_to_connection(connection_id, ipc_message).await {
            eprintln!("Failed to send state update: {}", e);
        }
    }
    
    fn calculate_checksum(&self, value: &Value) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        format!("{:?}", value).hash(&mut hasher);
        hasher.finish()
    }
    
    fn get_next_version(&self, key: &str) -> u64 {
        self.state_store
            .get(key)
            .map(|entry| entry.version + 1)
            .unwrap_or(1)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct StateUpdateMessage {
    key: String,
    value: Value,
    version: u64,
    timestamp: u64,
}
```

### 2. Conflict Resolution

**Optimistic Concurrency Control**: Handling state conflicts with automatic resolution strategies.

```typescript
// TypeScript - Frontend state synchronization
interface StateManager {
  localState: Map<string, StateEntry>;
  pendingUpdates: Map<string, PendingUpdate>;
  conflictResolver: ConflictResolver;
}

interface StateEntry {
  value: any;
  version: number;
  lastModified: number;
  checksum: string;
}

interface PendingUpdate {
  value: any;
  localVersion: number;
  timestamp: number;
  retryCount: number;
}

class FrontendStateSyncManager {
  private localState = new Map<string, StateEntry>();
  private pendingUpdates = new Map<string, PendingUpdate>();
  private conflictResolver = new ConflictResolver();
  private syncCallbacks = new Map<string, Array<(value: any) => void>>();
  
  constructor(private ipcClient: IPCClient) {
    this.setupStateSync();
  }
  
  private setupStateSync() {
    window.electronAPI.onMessage((message: IPCMessage) => {
      if (message.payload.type === 'StateUpdate') {
        this.handleStateUpdate(message.payload.data as StateUpdateMessage);
      }
    });
  }
  
  async updateState(key: string, value: any): Promise<void> {
    const currentEntry = this.localState.get(key);
    const newVersion = currentEntry ? currentEntry.version + 1 : 1;
    
    // Store as pending update
    this.pendingUpdates.set(key, {
      value,
      localVersion: newVersion,
      timestamp: Date.now(),
      retryCount: 0,
    });
    
    try {
      // Send update to backend
      await this.ipcClient.sendRequest({
        type: 'UpdateState',
        data: {
          key,
          value,
          expectedVersion: currentEntry?.version || 0,
        },
      });
      
      // Update local state optimistically
      this.setLocalState(key, {
        value,
        version: newVersion,
        lastModified: Date.now(),
        checksum: this.calculateChecksum(value),
      });
      
      // Remove from pending
      this.pendingUpdates.delete(key);
      
    } catch (error) {
      // Handle update conflict
      await this.handleUpdateConflict(key, error);
    }
  }
  
  private async handleStateUpdate(update: StateUpdateMessage) {
    const currentEntry = this.localState.get(update.key);
    const pendingUpdate = this.pendingUpdates.get(update.key);
    
    // Check for conflicts
    if (pendingUpdate && currentEntry) {
      if (update.version <= currentEntry.version) {
        // Ignore older updates
        return;
      }
      
      // Conflict detected - resolve it
      const resolvedValue = await this.conflictResolver.resolve(
        update.key,
        currentEntry.value,
        update.value,
        pendingUpdate.value
      );
      
      if (resolvedValue !== null) {
        this.setLocalState(update.key, {
          value: resolvedValue,
          version: update.version,
          lastModified: update.timestamp,
          checksum: this.calculateChecksum(resolvedValue),
        });
      }
    } else {
      // No conflict, apply update
      this.setLocalState(update.key, {
        value: update.value,
        version: update.version,
        lastModified: update.timestamp,
        checksum: this.calculateChecksum(update.value),
      });
    }
  }
  
  private async handleUpdateConflict(key: string, error: any) {
    const pendingUpdate = this.pendingUpdates.get(key);
    if (!pendingUpdate) return;
    
    pendingUpdate.retryCount++;
    
    if (pendingUpdate.retryCount < 3) {
      // Retry with exponential backoff
      const delay = Math.pow(2, pendingUpdate.retryCount) * 1000;
      setTimeout(() => {
        this.retryUpdate(key);
      }, delay);
    } else {
      // Give up and notify user
      console.error(`Failed to update state for key ${key} after 3 retries`);
      this.pendingUpdates.delete(key);
    }
  }
  
  private async retryUpdate(key: string) {
    const pendingUpdate = this.pendingUpdates.get(key);
    if (!pendingUpdate) return;
    
    try {
      // Get latest state from backend first
      const latestState = await this.ipcClient.sendRequest({
        type: 'GetState',
        data: { key },
      });
      
      // Resolve conflict with latest state
      const resolvedValue = await this.conflictResolver.resolve(
        key,
        latestState.value,
        pendingUpdate.value,
        pendingUpdate.value
      );
      
      if (resolvedValue !== null) {
        await this.updateState(key, resolvedValue);
      }
    } catch (error) {
      await this.handleUpdateConflict(key, error);
    }
  }
  
  public subscribeToState(key: string, callback: (value: any) => void): () => void {
    if (!this.syncCallbacks.has(key)) {
      this.syncCallbacks.set(key, []);
    }
    
    this.syncCallbacks.get(key)!.push(callback);
    
    // Send current value immediately
    const currentEntry = this.localState.get(key);
    if (currentEntry) {
      callback(currentEntry.value);
    }
    
    return () => {
      const callbacks = this.syncCallbacks.get(key);
      if (callbacks) {
        const index = callbacks.indexOf(callback);
        if (index > -1) {
          callbacks.splice(index, 1);
        }
      }
    };
  }
  
  private setLocalState(key: string, entry: StateEntry) {
    this.localState.set(key, entry);
    
    // Notify subscribers
    const callbacks = this.syncCallbacks.get(key);
    if (callbacks) {
      callbacks.forEach(callback => callback(entry.value));
    }
  }
  
  private calculateChecksum(value: any): string {
    return btoa(JSON.stringify(value)).slice(0, 16);
  }
}

class ConflictResolver {
  async resolve(key: string, baseValue: any, remoteValue: any, localValue: any): Promise<any> {
    // Implement conflict resolution strategies based on data type and key
    
    if (key.startsWith('backtest.')) {
      return this.resolveBacktestConflict(baseValue, remoteValue, localValue);
    }
    
    if (key.startsWith('algorithm.')) {
      return this.resolveAlgorithmConflict(baseValue, remoteValue, localValue);
    }
    
    // Default: last-writer-wins
    return remoteValue;
  }
  
  private resolveBacktestConflict(base: any, remote: any, local: any): any {
    // For backtest state, prefer the more recent state
    if (remote.lastUpdate > local.lastUpdate) {
      return remote;
    }
    return local;
  }
  
  private resolveAlgorithmConflict(base: any, remote: any, local: any): any {
    // For algorithm state, merge non-conflicting changes
    return {
      ...remote,
      ...local,
      lastModified: Math.max(remote.lastModified || 0, local.lastModified || 0),
    };
  }
}
```

## Performance Considerations

### 1. Batching Strategies

**Intelligent Data Aggregation**: Optimizing throughput while maintaining responsiveness.

```rust
// Rust - Advanced batching strategies
pub struct AdaptiveBatcher {
    batch_configs: HashMap<String, BatchConfig>,
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
    adaptive_tuning: AdaptiveTuning,
}

#[derive(Debug, Clone)]
struct BatchConfig {
    min_batch_size: usize,
    max_batch_size: usize,
    target_latency: Duration,
    max_wait_time: Duration,
    compression_threshold: usize,
    priority: BatchPriority,
}

#[derive(Debug, Clone)]
enum BatchPriority {
    RealTime,    // Sub-millisecond latency
    Interactive, // < 16ms for 60fps
    Background,  // < 100ms
    Bulk,        // Best effort
}

#[derive(Debug)]
struct PerformanceMetrics {
    average_latency: Duration,
    throughput_per_second: f64,
    cpu_usage: f32,
    memory_pressure: f32,
    network_congestion: f32,
}

impl AdaptiveBatcher {
    pub fn new() -> Self {
        let mut batch_configs = HashMap::new();
        
        // Configure different batching strategies for different data types
        batch_configs.insert("price_updates".to_string(), BatchConfig {
            min_batch_size: 1,
            max_batch_size: 1000,
            target_latency: Duration::from_millis(1),
            max_wait_time: Duration::from_millis(5),
            compression_threshold: 5000,
            priority: BatchPriority::RealTime,
        });
        
        batch_configs.insert("backtest_progress".to_string(), BatchConfig {
            min_batch_size: 5,
            max_batch_size: 100,
            target_latency: Duration::from_millis(16),
            max_wait_time: Duration::from_millis(50),
            compression_threshold: 2000,
            priority: BatchPriority::Interactive,
        });
        
        batch_configs.insert("order_updates".to_string(), BatchConfig {
            min_batch_size: 1,
            max_batch_size: 500,
            target_latency: Duration::from_millis(5),
            max_wait_time: Duration::from_millis(10),
            compression_threshold: 3000,
            priority: BatchPriority::RealTime,
        });
        
        Self {
            batch_configs,
            performance_metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
            adaptive_tuning: AdaptiveTuning::new(),
        }
    }
    
    pub async fn optimize_batch_config(&self, topic: &str) {
        let metrics = self.performance_metrics.lock().await;
        let current_config = self.batch_configs.get(topic).unwrap();
        
        let optimized_config = self.adaptive_tuning.optimize_config(current_config, &metrics);
        
        // Update configuration if significant improvement is expected
        if self.should_update_config(current_config, &optimized_config, &metrics) {
            self.batch_configs.insert(topic.to_string(), optimized_config);
        }
    }
    
    fn should_update_config(
        current: &BatchConfig,
        proposed: &BatchConfig,
        metrics: &PerformanceMetrics,
    ) -> bool {
        // Only update if we expect significant improvement
        let latency_improvement = current.target_latency.as_nanos() as f64 / 
                                 proposed.target_latency.as_nanos() as f64;
        
        let throughput_factor = if metrics.cpu_usage < 0.7 { 1.2 } else { 0.8 };
        
        latency_improvement > 1.1 || throughput_factor > 1.1
    }
}

struct AdaptiveTuning {
    learning_rate: f64,
    exploration_factor: f64,
}

impl AdaptiveTuning {
    fn new() -> Self {
        Self {
            learning_rate: 0.1,
            exploration_factor: 0.05,
        }
    }
    
    fn optimize_config(&self, current: &BatchConfig, metrics: &PerformanceMetrics) -> BatchConfig {
        let mut optimized = current.clone();
        
        // Adjust batch size based on CPU usage and latency
        if metrics.cpu_usage > 0.8 {
            // High CPU usage - increase batch size to reduce overhead
            optimized.max_batch_size = std::cmp::min(
                optimized.max_batch_size * 2,
                10000
            );
            optimized.max_wait_time = Duration::from_millis(
                optimized.max_wait_time.as_millis() as u64 * 2
            );
        } else if metrics.average_latency > current.target_latency * 2 {
            // High latency - reduce batch size
            optimized.max_batch_size = std::cmp::max(
                optimized.max_batch_size / 2,
                1
            );
            optimized.max_wait_time = Duration::from_millis(
                optimized.max_wait_time.as_millis() as u64 / 2
            );
        }
        
        // Adjust compression threshold based on network conditions
        if metrics.network_congestion > 0.7 {
            optimized.compression_threshold = std::cmp::min(
                optimized.compression_threshold / 2,
                500
            );
        }
        
        optimized
    }
}
```

### 2. Compression & Serialization

**Data Optimization**: Advanced compression and serialization strategies.

```rust
// Rust - Compression and serialization optimization
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use zstd;

pub struct DataOptimizer {
    compression_cache: LruCache<u64, Vec<u8>>,
    serialization_pool: ObjectPool<Vec<u8>>,
    compression_stats: CompressionStats,
}

#[derive(Debug, Default)]
struct CompressionStats {
    total_bytes_in: u64,
    total_bytes_out: u64,
    compression_time: Duration,
    decompression_time: Duration,
    cache_hits: u64,
    cache_misses: u64,
}

impl DataOptimizer {
    pub fn new() -> Self {
        Self {
            compression_cache: LruCache::new(NonZeroUsize::new(1000).unwrap()),
            serialization_pool: ObjectPool::new(|| Vec::with_capacity(8192), Vec::clear),
            compression_stats: CompressionStats::default(),
        }
    }
    
    pub fn serialize_and_compress<T: Serialize>(&mut self, data: &T, compression_level: CompressionLevel) -> Result<Vec<u8>, SerializationError> {
        let start_time = Instant::now();
        
        // Get buffer from pool
        let mut buffer = self.serialization_pool.get();
        
        // Serialize using MessagePack
        rmp_serde::encode::write(&mut *buffer, data)?;
        
        let serialized_size = buffer.len();
        self.compression_stats.total_bytes_in += serialized_size as u64;
        
        // Calculate hash for caching
        let hash = self.calculate_hash(&buffer);
        
        // Check cache first
        if let Some(cached) = self.compression_cache.get(&hash) {
            self.compression_stats.cache_hits += 1;
            return Ok(cached.clone());
        }
        
        self.compression_stats.cache_misses += 1;
        
        // Choose compression algorithm based on data characteristics and level
        let compressed = match compression_level {
            CompressionLevel::None => buffer.clone(),
            CompressionLevel::Fast => {
                compress_prepend_size(&buffer)
            },
            CompressionLevel::Balanced => {
                zstd::bulk::compress(&buffer, 3)?
            },
            CompressionLevel::Maximum => {
                zstd::bulk::compress(&buffer, 19)?
            },
        };
        
        let compressed_size = compressed.len();
        self.compression_stats.total_bytes_out += compressed_size as u64;
        self.compression_stats.compression_time += start_time.elapsed();
        
        // Cache the result if compression was effective
        let compression_ratio = compressed_size as f64 / serialized_size as f64;
        if compression_ratio < 0.8 {
            self.compression_cache.put(hash, compressed.clone());
        }
        
        // Return buffer to pool
        buffer.clear();
        self.serialization_pool.put(buffer);
        
        Ok(compressed)
    }
    
    pub fn decompress_and_deserialize<T: DeserializeOwned>(&mut self, data: &[u8], compression_level: CompressionLevel) -> Result<T, SerializationError> {
        let start_time = Instant::now();
        
        // Decompress based on compression level
        let decompressed = match compression_level {
            CompressionLevel::None => data.to_vec(),
            CompressionLevel::Fast => {
                decompress_size_prepended(data)?
            },
            CompressionLevel::Balanced | CompressionLevel::Maximum => {
                zstd::bulk::decompress(data, 1024 * 1024)? // 1MB max
            },
        };
        
        self.compression_stats.decompression_time += start_time.elapsed();
        
        // Deserialize
        let result: T = rmp_serde::from_slice(&decompressed)?;
        
        Ok(result)
    }
    
    pub fn get_optimal_compression_level(&self, data_size: usize, priority: BatchPriority) -> CompressionLevel {
        match priority {
            BatchPriority::RealTime => {
                if data_size > 10240 { CompressionLevel::Fast } else { CompressionLevel::None }
            },
            BatchPriority::Interactive => {
                if data_size > 5120 { CompressionLevel::Balanced } else { CompressionLevel::Fast }
            },
            BatchPriority::Background => CompressionLevel::Balanced,
            BatchPriority::Bulk => CompressionLevel::Maximum,
        }
    }
    
    fn calculate_hash(&self, data: &[u8]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }
    
    pub fn get_compression_stats(&self) -> &CompressionStats {
        &self.compression_stats
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CompressionLevel {
    None,
    Fast,
    Balanced,
    Maximum,
}
```

### 3. Connection Pooling & Throttling

**Resource Management**: Efficient connection management and rate limiting.

```rust
// Rust - Connection pooling and throttling
use governor::{Quota, RateLimiter, state::{InMemoryState, NotKeyed}};
use std::num::NonZeroU32;

pub struct ConnectionPool {
    connections: Arc<Mutex<VecDeque<PooledConnection>>>,
    rate_limiter: RateLimiter<NotKeyed, InMemoryState, governor::clock::DefaultClock>,
    connection_config: ConnectionConfig,
    active_connections: Arc<AtomicUsize>,
    throttling_config: ThrottlingConfig,
}

#[derive(Debug, Clone)]
struct ConnectionConfig {
    max_connections: usize,
    connection_timeout: Duration,
    idle_timeout: Duration,
    keep_alive_interval: Duration,
}

#[derive(Debug, Clone)]
struct ThrottlingConfig {
    max_requests_per_second: NonZeroU32,
    burst_size: NonZeroU32,
    priority_thresholds: HashMap<BatchPriority, NonZeroU32>,
}

struct PooledConnection {
    id: Uuid,
    stream: WebSocketStream<TcpStream>,
    last_used: Instant,
    request_count: u64,
    error_count: u32,
}

impl ConnectionPool {
    pub fn new(config: ConnectionConfig, throttling_config: ThrottlingConfig) -> Self {
        let quota = Quota::per_second(throttling_config.max_requests_per_second)
            .allow_burst(throttling_config.burst_size);
        
        Self {
            connections: Arc::new(Mutex::new(VecDeque::new())),
            rate_limiter: RateLimiter::direct(quota),
            connection_config: config,
            active_connections: Arc::new(AtomicUsize::new(0)),
            throttling_config,
        }
    }
    
    pub async fn get_connection(&self, priority: BatchPriority) -> Result<PooledConnection, ConnectionError> {
        // Apply rate limiting based on priority
        let rate_limit = self.throttling_config.priority_thresholds
            .get(&priority)
            .unwrap_or(&self.throttling_config.max_requests_per_second);
        
        if !self.check_rate_limit(priority, *rate_limit).await {
            return Err(ConnectionError::RateLimited);
        }
        
        // Try to get existing connection
        let mut connections = self.connections.lock().await;
        
        // Clean up stale connections
        self.cleanup_stale_connections(&mut connections).await;
        
        if let Some(mut conn) = connections.pop_front() {
            conn.last_used = Instant::now();
            return Ok(conn);
        }
        
        drop(connections);
        
        // Create new connection if under limit
        let active_count = self.active_connections.load(Ordering::SeqCst);
        if active_count < self.connection_config.max_connections {
            self.create_new_connection().await
        } else {
            Err(ConnectionError::PoolExhausted)
        }
    }
    
    pub async fn return_connection(&self, mut connection: PooledConnection) {
        connection.last_used = Instant::now();
        
        if connection.error_count < 5 { // Threshold for connection health
            let mut connections = self.connections.lock().await;
            connections.push_back(connection);
        } else {
            // Connection has too many errors, discard it
            self.active_connections.fetch_sub(1, Ordering::SeqCst);
        }
    }
    
    async fn create_new_connection(&self) -> Result<PooledConnection, ConnectionError> {
        let stream = TcpStream::connect("127.0.0.1:8080")
            .timeout(self.connection_config.connection_timeout)
            .await??;
        
        let ws_stream = tokio_tungstenite::client_async("ws://127.0.0.1:8080", stream)
            .timeout(self.connection_config.connection_timeout)
            .await??
            .0;
        
        self.active_connections.fetch_add(1, Ordering::SeqCst);
        
        Ok(PooledConnection {
            id: Uuid::new_v4(),
            stream: ws_stream,
            last_used: Instant::now(),
            request_count: 0,
            error_count: 0,
        })
    }
    
    async fn cleanup_stale_connections(&self, connections: &mut VecDeque<PooledConnection>) {
        let idle_threshold = Instant::now() - self.connection_config.idle_timeout;
        
        while let Some(conn) = connections.front() {
            if conn.last_used < idle_threshold {
                connections.pop_front();
                self.active_connections.fetch_sub(1, Ordering::SeqCst);
            } else {
                break;
            }
        }
    }
    
    async fn check_rate_limit(&self, priority: BatchPriority, limit: NonZeroU32) -> bool {
        match priority {
            BatchPriority::RealTime => {
                // Real-time requests get priority - check with reduced weight
                self.rate_limiter.check_n(NonZeroU32::new(1).unwrap()).is_ok()
            },
            BatchPriority::Interactive => {
                // Interactive requests use normal rate limiting
                self.rate_limiter.check().is_ok()
            },
            BatchPriority::Background | BatchPriority::Bulk => {
                // Background requests are heavily rate limited
                self.rate_limiter.check_n(NonZeroU32::new(2).unwrap()).is_ok()
            },
        }
    }
    
    pub async fn start_keep_alive_service(&self) {
        let connections = self.connections.clone();
        let interval = self.connection_config.keep_alive_interval;
        
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            
            loop {
                ticker.tick().await;
                
                let mut conns = connections.lock().await;
                for conn in conns.iter_mut() {
                    // Send ping frame to keep connection alive
                    if let Err(_) = conn.stream.send(Message::Ping(vec![])).await {
                        conn.error_count += 1;
                    }
                }
            }
        });
    }
    
    pub fn get_pool_stats(&self) -> PoolStats {
        PoolStats {
            active_connections: self.active_connections.load(Ordering::SeqCst),
            max_connections: self.connection_config.max_connections,
            rate_limit_hits: self.rate_limiter.get_state().len(),
        }
    }
}

#[derive(Debug)]
pub struct PoolStats {
    pub active_connections: usize,
    pub max_connections: usize,
    pub rate_limit_hits: usize,
}

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("Connection pool exhausted")]
    PoolExhausted,
    
    #[error("Rate limited")]
    RateLimited,
    
    #[error("Connection timeout")]
    Timeout,
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
}
```

This comprehensive API Design section provides a robust foundation for BackTestr_ai's IPC communication system, focusing on performance, reliability, and real-time capabilities essential for a high-performance desktop trading application. The design emphasizes efficient data transmission, automatic error recovery, and intelligent resource management to ensure smooth operation under varying load conditions.
