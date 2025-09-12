# Frontend Architecture

The frontend architecture leverages modern web technologies within an Electron desktop environment to deliver a professional trading interface with 60 FPS performance and institutional-grade user experience. The design emphasizes real-time data visualization, responsive controls, and seamless integration with the high-performance Rust backend.

## Component Architecture

The frontend follows a hierarchical component structure optimized for financial applications, with clear separation between layout, data presentation, and user interaction concerns.

### 1. Layout System

**Main Application Layout**: Professional trading interface with docked panels and flexible workspace management.

```tsx
// Main application shell with docked panel system
export const AppShell: React.FC = () => {
  const { theme } = useTheme();
  const { layout, updateLayout } = useLayoutStore();
  
  return (
    <div className={`app-shell ${theme}`}>
      <TitleBar />
      <div className="main-content">
        <Sidebar />
        <WorkspaceContainer>
          <ChartGrid />
          <BottomPanel />
        </WorkspaceContainer>
      </div>
      <StatusBar />
    </div>
  );
};

// Flexible workspace with drag-and-drop panel management
export const WorkspaceContainer: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { panels, updatePanelLayout } = usePanelStore();
  
  return (
    <DockLayout
      config={panels}
      onLayoutChange={updatePanelLayout}
      className="workspace-container"
    >
      {children}
    </DockLayout>
  );
};

// Title bar with window controls and menu
export const TitleBar: React.FC = () => {
  const { isMaximized, minimize, maximize, close } = useWindowControls();
  
  return (
    <div className="title-bar" data-tauri-drag-region>
      <div className="title-content">
        <AppLogo />
        <span className="app-title">BackTestr AI</span>
      </div>
      <div className="window-controls">
        <button onClick={minimize} className="control-button minimize">
          <MinimizeIcon />
        </button>
        <button onClick={maximize} className="control-button maximize">
          {isMaximized ? <RestoreIcon /> : <MaximizeIcon />}
        </button>
        <button onClick={close} className="control-button close">
          <CloseIcon />
        </button>
      </div>
    </div>
  );
};
```

**Responsive Grid System**: Adaptive layout for different screen sizes and panel configurations.

```tsx
// Chart grid with synchronized timeframe panels
export const ChartGrid: React.FC = () => {
  const { chartConfig, updateChartConfig } = useChartStore();
  const { mtfData } = useMarketDataStore();
  
  const timeframes = ['1m', '5m', '15m', '1H', '4H', 'Daily'] as const;
  
  return (
    <div className="chart-grid">
      {timeframes.map((timeframe) => (
        <ChartPanel
          key={timeframe}
          timeframe={timeframe}
          data={mtfData[timeframe]}
          config={chartConfig[timeframe]}
          onConfigChange={(config) => updateChartConfig(timeframe, config)}
          className={`chart-panel-${timeframe.toLowerCase()}`}
        />
      ))}
    </div>
  );
};

// Individual chart panel with toolbar and controls
export const ChartPanel: React.FC<ChartPanelProps> = ({
  timeframe,
  data,
  config,
  onConfigChange,
  className
}) => {
  const chartRef = useRef<IChartApi>(null);
  const { isLoading } = useChartData(timeframe);
  
  useChartSync(chartRef, timeframe); // Synchronize with other panels
  
  return (
    <div className={`chart-panel ${className}`}>
      <ChartToolbar
        timeframe={timeframe}
        config={config}
        onConfigChange={onConfigChange}
      />
      <div className="chart-container">
        {isLoading ? (
          <LoadingSpinner />
        ) : (
          <LightweightChart
            ref={chartRef}
            data={data}
            config={config}
            onCrosshairMove={handleCrosshairSync}
          />
        )}
      </div>
      <ChartStatusBar timeframe={timeframe} />
    </div>
  );
};
```

### 2. Chart Components

**Lightweight Charts Integration**: Professional charting with real-time updates and synchronization.

```tsx
import { createChart, IChartApi, ISeriesApi, LineData, CandlestickData } from 'lightweight-charts';

// Main chart component with performance optimizations
export const LightweightChart = React.forwardRef<IChartApi, LightweightChartProps>(
  ({ data, config, onCrosshairMove }, ref) => {
    const containerRef = useRef<HTMLDivElement>(null);
    const chartRef = useRef<IChartApi | null>(null);
    const seriesRef = useRef<ISeriesApi<'Candlestick'> | null>(null);
    
    // Initialize chart with optimized settings
    useEffect(() => {
      if (!containerRef.current) return;
      
      const chart = createChart(containerRef.current, {
        width: containerRef.current.clientWidth,
        height: containerRef.current.clientHeight,
        layout: {
          background: { color: '#1a1a1a' },
          textColor: '#d1d5db',
        },
        grid: {
          vertLines: { color: '#374151' },
          horzLines: { color: '#374151' },
        },
        crosshair: {
          mode: 0, // Normal crosshair
          vertLine: {
            labelBackgroundColor: '#3b82f6',
          },
          horzLine: {
            labelBackgroundColor: '#3b82f6',
          },
        },
        timeScale: {
          timeVisible: true,
          secondsVisible: false,
          borderColor: '#4b5563',
        },
        rightPriceScale: {
          borderColor: '#4b5563',
          scaleMargins: {
            top: 0.1,
            bottom: 0.1,
          },
        },
      });
      
      // Create candlestick series
      const candlestickSeries = chart.addCandlestickSeries({
        upColor: '#22c55e',
        downColor: '#ef4444',
        borderUpColor: '#22c55e',
        borderDownColor: '#ef4444',
        wickUpColor: '#22c55e',
        wickDownColor: '#ef4444',
      });
      
      chartRef.current = chart;
      seriesRef.current = candlestickSeries;
      
      // Expose chart API to parent
      if (ref) {
        if (typeof ref === 'function') {
          ref(chart);
        } else {
          ref.current = chart;
        }
      }
      
      // Set up crosshair synchronization
      chart.subscribeCrosshairMove(onCrosshairMove);
      
      return () => {
        chart.remove();
      };
    }, []);
    
    // Update data with batching for performance
    useEffect(() => {
      if (seriesRef.current && data) {
        // Batch updates to prevent excessive redraws
        requestAnimationFrame(() => {
          seriesRef.current!.setData(data);
        });
      }
    }, [data]);
    
    // Handle resize
    useEffect(() => {
      const handleResize = () => {
        if (chartRef.current && containerRef.current) {
          chartRef.current.applyOptions({
            width: containerRef.current.clientWidth,
            height: containerRef.current.clientHeight,
          });
        }
      };
      
      window.addEventListener('resize', handleResize);
      return () => window.removeEventListener('resize', handleResize);
    }, []);
    
    return <div ref={containerRef} className="chart-container" />;
  }
);

// Chart synchronization hook for crosshair and zoom coordination
export const useChartSync = (chartRef: React.RefObject<IChartApi>, timeframe: string) => {
  const { syncState, updateSyncState } = useChartSyncStore();
  
  useEffect(() => {
    const chart = chartRef.current;
    if (!chart) return;
    
    // Subscribe to time scale changes
    const timeScale = chart.timeScale();
    const handleVisibleTimeRangeChange = () => {
      const visibleRange = timeScale.getVisibleRange();
      if (visibleRange) {
        updateSyncState({
          timeframe,
          visibleRange,
          timestamp: Date.now(),
        });
      }
    };
    
    timeScale.subscribeVisibleTimeRangeChange(handleVisibleTimeRangeChange);
    
    return () => {
      timeScale.unsubscribeVisibleTimeRangeChange(handleVisibleTimeRangeChange);
    };
  }, [chartRef, timeframe, updateSyncState]);
  
  // Apply sync state from other charts
  useEffect(() => {
    const chart = chartRef.current;
    if (!chart || syncState.timeframe === timeframe) return;
    
    const timeScale = chart.timeScale();
    if (syncState.visibleRange) {
      timeScale.setVisibleRange(syncState.visibleRange);
    }
  }, [syncState, chartRef, timeframe]);
};
```

**Indicator Overlays**: Dynamic indicator visualization with efficient rendering.

```tsx
// Indicator overlay system for technical analysis
export const IndicatorOverlay: React.FC<IndicatorOverlayProps> = ({
  chart,
  indicators,
  timeframe
}) => {
  const seriesRefs = useRef<Map<string, ISeriesApi<any>>>(new Map());
  
  useEffect(() => {
    if (!chart) return;
    
    // Add indicator series to chart
    indicators.forEach((indicator) => {
      if (!seriesRefs.current.has(indicator.id)) {
        let series: ISeriesApi<any>;
        
        switch (indicator.type) {
          case 'line':
            series = chart.addLineSeries({
              color: indicator.color,
              lineWidth: indicator.lineWidth || 2,
              priceLineVisible: false,
              lastValueVisible: false,
            });
            break;
          case 'histogram':
            series = chart.addHistogramSeries({
              color: indicator.color,
              priceFormat: {
                type: 'volume',
              },
              priceScaleId: 'volume',
            });
            break;
          default:
            return;
        }
        
        seriesRefs.current.set(indicator.id, series);
      }
      
      // Update indicator data
      const series = seriesRefs.current.get(indicator.id);
      if (series && indicator.data) {
        series.setData(indicator.data);
      }
    });
    
    // Cleanup removed indicators
    seriesRefs.current.forEach((series, id) => {
      if (!indicators.find(ind => ind.id === id)) {
        chart.removeSeries(series);
        seriesRefs.current.delete(id);
      }
    });
  }, [chart, indicators]);
  
  return null; // This component doesn't render anything directly
};

// Indicator configuration panel
export const IndicatorConfig: React.FC<IndicatorConfigProps> = ({
  timeframe,
  availableIndicators,
  onAddIndicator,
  onRemoveIndicator
}) => {
  const [selectedIndicator, setSelectedIndicator] = useState<string>('');
  const [indicatorParams, setIndicatorParams] = useState<Record<string, any>>({});
  
  const handleAddIndicator = () => {
    if (!selectedIndicator) return;
    
    const indicatorDef = availableIndicators.find(ind => ind.name === selectedIndicator);
    if (!indicatorDef) return;
    
    onAddIndicator({
      id: `${selectedIndicator}_${Date.now()}`,
      name: selectedIndicator,
      type: indicatorDef.type,
      params: indicatorParams,
      color: generateIndicatorColor(),
      timeframe,
    });
    
    setSelectedIndicator('');
    setIndicatorParams({});
  };
  
  return (
    <div className="indicator-config">
      <div className="add-indicator">
        <select
          value={selectedIndicator}
          onChange={(e) => setSelectedIndicator(e.target.value)}
          className="indicator-select"
        >
          <option value="">Select Indicator</option>
          {availableIndicators.map((indicator) => (
            <option key={indicator.name} value={indicator.name}>
              {indicator.displayName}
            </option>
          ))}
        </select>
        
        {selectedIndicator && (
          <IndicatorParamEditor
            indicatorName={selectedIndicator}
            params={indicatorParams}
            onParamsChange={setIndicatorParams}
          />
        )}
        
        <button onClick={handleAddIndicator} className="add-button">
          Add Indicator
        </button>
      </div>
    </div>
  );
};
```

### 3. Control Interfaces

**Algorithm Controls**: User interface for algorithm configuration and execution control.

```tsx
// Main algorithm control panel
export const AlgorithmControls: React.FC = () => {
  const { 
    algorithm, 
    isRunning, 
    runStatus,
    loadAlgorithm,
    startBacktest,
    stopBacktest,
    pauseBacktest
  } = useAlgorithmStore();
  
  const { backtestConfig, updateConfig } = useBacktestConfigStore();
  
  return (
    <div className="algorithm-controls">
      <div className="control-header">
        <h3>Algorithm Controls</h3>
        <RunStatusIndicator status={runStatus} />
      </div>
      
      <div className="control-sections">
        <AlgorithmSelector
          currentAlgorithm={algorithm}
          onAlgorithmLoad={loadAlgorithm}
        />
        
        <BacktestConfig
          config={backtestConfig}
          onConfigChange={updateConfig}
        />
        
        <ExecutionControls
          isRunning={isRunning}
          onStart={startBacktest}
          onStop={stopBacktest}
          onPause={pauseBacktest}
        />
      </div>
    </div>
  );
};

// Algorithm selection and loading interface
export const AlgorithmSelector: React.FC<AlgorithmSelectorProps> = ({
  currentAlgorithm,
  onAlgorithmLoad
}) => {
  const [algorithmCode, setAlgorithmCode] = useState('');
  const [isEditing, setIsEditing] = useState(false);
  
  const handleLoadFromFile = async () => {
    try {
      const file = await window.electronAPI.openFile({
        filters: [{ name: 'Python Files', extensions: ['py'] }]
      });
      
      if (file) {
        const content = await window.electronAPI.readFile(file.path);
        setAlgorithmCode(content);
        onAlgorithmLoad(content);
      }
    } catch (error) {
      console.error('Failed to load algorithm file:', error);
    }
  };
  
  const handleSaveAlgorithm = async () => {
    try {
      await window.electronAPI.saveFile({
        content: algorithmCode,
        defaultPath: 'algorithm.py',
        filters: [{ name: 'Python Files', extensions: ['py'] }]
      });
    } catch (error) {
      console.error('Failed to save algorithm:', error);
    }
  };
  
  return (
    <div className="algorithm-selector">
      <div className="selector-header">
        <span>Algorithm</span>
        <div className="action-buttons">
          <button onClick={handleLoadFromFile} className="load-button">
            <FileOpenIcon />
            Load
          </button>
          <button onClick={handleSaveAlgorithm} className="save-button">
            <SaveIcon />
            Save
          </button>
          <button 
            onClick={() => setIsEditing(!isEditing)} 
            className="edit-button"
          >
            <EditIcon />
            Edit
          </button>
        </div>
      </div>
      
      {isEditing ? (
        <CodeEditor
          value={algorithmCode}
          onChange={setAlgorithmCode}
          language="python"
          onSave={() => onAlgorithmLoad(algorithmCode)}
        />
      ) : (
        <div className="algorithm-display">
          {currentAlgorithm ? (
            <AlgorithmSummary algorithm={currentAlgorithm} />
          ) : (
            <div className="no-algorithm">No algorithm loaded</div>
          )}
        </div>
      )}
    </div>
  );
};

// Backtest configuration interface
export const BacktestConfig: React.FC<BacktestConfigProps> = ({
  config,
  onConfigChange
}) => {
  const updateField = useCallback((field: keyof BacktestConfig, value: any) => {
    onConfigChange({
      ...config,
      [field]: value
    });
  }, [config, onConfigChange]);
  
  return (
    <div className="backtest-config">
      <h4>Backtest Settings</h4>
      
      <div className="config-grid">
        <div className="config-field">
          <label>Symbol</label>
          <input
            type="text"
            value={config.symbol}
            onChange={(e) => updateField('symbol', e.target.value)}
            placeholder="EURUSD"
          />
        </div>
        
        <div className="config-field">
          <label>Start Date</label>
          <input
            type="datetime-local"
            value={config.startDate}
            onChange={(e) => updateField('startDate', e.target.value)}
          />
        </div>
        
        <div className="config-field">
          <label>End Date</label>
          <input
            type="datetime-local"
            value={config.endDate}
            onChange={(e) => updateField('endDate', e.target.value)}
          />
        </div>
        
        <div className="config-field">
          <label>Initial Balance</label>
          <CurrencyInput
            value={config.initialBalance}
            onChange={(value) => updateField('initialBalance', value)}
          />
        </div>
        
        <div className="config-field">
          <label>Commission</label>
          <input
            type="number"
            step="0.01"
            value={config.commission}
            onChange={(e) => updateField('commission', parseFloat(e.target.value))}
          />
        </div>
        
        <div className="config-field">
          <label>Slippage (pips)</label>
          <input
            type="number"
            step="0.1"
            value={config.slippage}
            onChange={(e) => updateField('slippage', parseFloat(e.target.value))}
          />
        </div>
      </div>
    </div>
  );
};

// Execution control buttons with status feedback
export const ExecutionControls: React.FC<ExecutionControlsProps> = ({
  isRunning,
  onStart,
  onStop,
  onPause
}) => {
  const [speed, setSpeed] = useState(1);
  const { progress } = useBacktestProgress();
  
  return (
    <div className="execution-controls">
      <div className="primary-controls">
        <button
          onClick={onStart}
          disabled={isRunning}
          className="start-button primary"
        >
          <PlayIcon />
          Start
        </button>
        
        <button
          onClick={onPause}
          disabled={!isRunning}
          className="pause-button secondary"
        >
          <PauseIcon />
          Pause
        </button>
        
        <button
          onClick={onStop}
          disabled={!isRunning}
          className="stop-button danger"
        >
          <StopIcon />
          Stop
        </button>
      </div>
      
      <div className="speed-control">
        <label>Speed</label>
        <input
          type="range"
          min="0.1"
          max="10"
          step="0.1"
          value={speed}
          onChange={(e) => setSpeed(parseFloat(e.target.value))}
          className="speed-slider"
        />
        <span className="speed-value">{speed}x</span>
      </div>
      
      {isRunning && (
        <div className="progress-indicator">
          <div className="progress-bar">
            <div 
              className="progress-fill"
              style={{ width: `${progress.percentage}%` }}
            />
          </div>
          <div className="progress-text">
            {progress.current} / {progress.total} ({progress.percentage.toFixed(1)}%)
          </div>
        </div>
      )}
    </div>
  );
};
```

## State Management

The frontend uses Zustand for lightweight, type-safe state management with clear separation between UI state, server state, and real-time data streams.

### 1. Zustand Stores

**Application State Architecture**: Modular stores with specific responsibilities.

```typescript
// Main application state store
interface AppStore {
  // Theme and UI preferences
  theme: 'dark' | 'light';
  sidebarOpen: boolean;
  activePanels: string[];
  
  // Window state
  isMaximized: boolean;
  windowSize: { width: number; height: number };
  
  // Actions
  setTheme: (theme: 'dark' | 'light') => void;
  toggleSidebar: () => void;
  updateWindowSize: (size: { width: number; height: number }) => void;
}

export const useAppStore = create<AppStore>((set, get) => ({
  theme: 'dark',
  sidebarOpen: true,
  activePanels: ['charts', 'analysis', 'logs'],
  isMaximized: false,
  windowSize: { width: 1200, height: 800 },
  
  setTheme: (theme) => set({ theme }),
  toggleSidebar: () => set({ sidebarOpen: !get().sidebarOpen }),
  updateWindowSize: (windowSize) => set({ windowSize }),
}));

// Chart state management
interface ChartStore {
  chartConfigs: Record<string, ChartConfig>;
  syncSettings: {
    crosshair: boolean;
    zoom: boolean;
    timeRange: boolean;
  };
  overlays: Record<string, IndicatorConfig[]>;
  
  updateChartConfig: (timeframe: string, config: Partial<ChartConfig>) => void;
  addIndicator: (timeframe: string, indicator: IndicatorConfig) => void;
  removeIndicator: (timeframe: string, indicatorId: string) => void;
  setSyncSettings: (settings: Partial<ChartStore['syncSettings']>) => void;
}

export const useChartStore = create<ChartStore>((set, get) => ({
  chartConfigs: {
    '1m': { autoScale: true, showVolume: true, candleStyle: 'candles' },
    '5m': { autoScale: true, showVolume: true, candleStyle: 'candles' },
    '15m': { autoScale: true, showVolume: true, candleStyle: 'candles' },
    '1H': { autoScale: true, showVolume: true, candleStyle: 'candles' },
    '4H': { autoScale: true, showVolume: true, candleStyle: 'candles' },
    'Daily': { autoScale: true, showVolume: true, candleStyle: 'candles' },
  },
  syncSettings: {
    crosshair: true,
    zoom: true,
    timeRange: true,
  },
  overlays: {},
  
  updateChartConfig: (timeframe, config) => set((state) => ({
    chartConfigs: {
      ...state.chartConfigs,
      [timeframe]: { ...state.chartConfigs[timeframe], ...config }
    }
  })),
  
  addIndicator: (timeframe, indicator) => set((state) => ({
    overlays: {
      ...state.overlays,
      [timeframe]: [...(state.overlays[timeframe] || []), indicator]
    }
  })),
  
  removeIndicator: (timeframe, indicatorId) => set((state) => ({
    overlays: {
      ...state.overlays,
      [timeframe]: state.overlays[timeframe]?.filter(ind => ind.id !== indicatorId) || []
    }
  })),
  
  setSyncSettings: (settings) => set((state) => ({
    syncSettings: { ...state.syncSettings, ...settings }
  })),
}));

// Algorithm and backtest state
interface AlgorithmStore {
  algorithm: AlgorithmInfo | null;
  backtestConfig: BacktestConfig;
  runStatus: 'idle' | 'running' | 'paused' | 'completed' | 'error';
  progress: BacktestProgress;
  results: BacktestResults | null;
  logs: LogEntry[];
  
  loadAlgorithm: (code: string) => Promise<void>;
  startBacktest: () => Promise<void>;
  stopBacktest: () => Promise<void>;
  pauseBacktest: () => Promise<void>;
  updateConfig: (config: Partial<BacktestConfig>) => void;
}

export const useAlgorithmStore = create<AlgorithmStore>((set, get) => ({
  algorithm: null,
  backtestConfig: {
    symbol: 'EURUSD',
    startDate: '2024-01-01T00:00:00',
    endDate: '2024-12-31T23:59:59',
    initialBalance: 10000,
    commission: 5,
    slippage: 1,
  },
  runStatus: 'idle',
  progress: { current: 0, total: 0, percentage: 0 },
  results: null,
  logs: [],
  
  loadAlgorithm: async (code) => {
    try {
      const algorithm = await window.electronAPI.loadAlgorithm(code);
      set({ algorithm });
    } catch (error) {
      console.error('Failed to load algorithm:', error);
      throw error;
    }
  },
  
  startBacktest: async () => {
    const { algorithm, backtestConfig } = get();
    if (!algorithm) throw new Error('No algorithm loaded');
    
    set({ runStatus: 'running', progress: { current: 0, total: 0, percentage: 0 } });
    
    try {
      await window.electronAPI.startBacktest({
        algorithm: algorithm.code,
        config: backtestConfig,
      });
    } catch (error) {
      set({ runStatus: 'error' });
      throw error;
    }
  },
  
  stopBacktest: async () => {
    try {
      await window.electronAPI.stopBacktest();
      set({ runStatus: 'idle' });
    } catch (error) {
      console.error('Failed to stop backtest:', error);
    }
  },
  
  pauseBacktest: async () => {
    try {
      await window.electronAPI.pauseBacktest();
      set({ runStatus: 'paused' });
    } catch (error) {
      console.error('Failed to pause backtest:', error);
    }
  },
  
  updateConfig: (config) => set((state) => ({
    backtestConfig: { ...state.backtestConfig, ...config }
  })),
}));
```

**Persistent State**: Local storage integration for user preferences.

```typescript
// Persistent state configuration
interface PersistentState {
  theme: 'dark' | 'light';
  windowSize: { width: number; height: number };
  chartConfigs: Record<string, ChartConfig>;
  panelLayout: PanelLayoutConfig;
  recentAlgorithms: AlgorithmReference[];
}

// State persistence middleware
const persistentStorage = (config: StateCreator<PersistentState>) => (
  set: SetState<PersistentState>,
  get: GetState<PersistentState>,
  api: StoreApi<PersistentState>
) => {
  // Load initial state from localStorage
  const savedState = localStorage.getItem('backtestr-state');
  const initialState = savedState ? JSON.parse(savedState) : {};
  
  const store = config(set, get, api);
  
  // Save state to localStorage on changes
  api.subscribe((state) => {
    localStorage.setItem('backtestr-state', JSON.stringify(state));
  });
  
  return {
    ...store,
    ...initialState,
  };
};

// Apply persistence to stores
export const usePersistentAppStore = create<AppStore>(
  persistentStorage((set, get) => ({
    // ... store implementation
  }))
);
```

### 2. React Query Integration

**Server State Management**: React Query for efficient data fetching and caching.

```typescript
// React Query configuration
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      cacheTime: 10 * 60 * 1000, // 10 minutes
      retry: 3,
      refetchOnWindowFocus: false,
    },
    mutations: {
      retry: 1,
    },
  },
});

// Market data queries
export const useMarketData = (symbol: string, timeframe: string, dateRange: DateRange) => {
  return useQuery({
    queryKey: ['marketData', symbol, timeframe, dateRange],
    queryFn: async () => {
      const response = await window.electronAPI.getMarketData({
        symbol,
        timeframe,
        startDate: dateRange.start,
        endDate: dateRange.end,
      });
      return response.data;
    },
    enabled: Boolean(symbol && timeframe && dateRange.start && dateRange.end),
    keepPreviousData: true,
    staleTime: 30 * 1000, // 30 seconds for market data
  });
};

// Backtest results query
export const useBacktestResults = (backtestId: string) => {
  return useQuery({
    queryKey: ['backtestResults', backtestId],
    queryFn: async () => {
      const response = await window.electronAPI.getBacktestResults(backtestId);
      return response.data;
    },
    enabled: Boolean(backtestId),
    refetchInterval: (data) => {
      // Refetch every 5 seconds if backtest is running
      return data?.status === 'running' ? 5000 : false;
    },
  });
};

// Performance metrics query
export const usePerformanceMetrics = (backtestId: string) => {
  return useQuery({
    queryKey: ['performanceMetrics', backtestId],
    queryFn: async () => {
      const response = await window.electronAPI.getPerformanceMetrics(backtestId);
      return response.data;
    },
    enabled: Boolean(backtestId),
    select: (data) => ({
      ...data,
      // Transform data for chart components
      equityCurve: data.equity_history.map((point: any) => ({
        time: point.timestamp,
        value: point.equity,
      })),
      drawdownCurve: data.drawdown_history.map((point: any) => ({
        time: point.timestamp,
        value: point.drawdown,
      })),
    }),
  });
};

// Mutations for algorithm operations
export const useLoadAlgorithmMutation = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async (algorithmCode: string) => {
      return await window.electronAPI.loadAlgorithm(algorithmCode);
    },
    onSuccess: (data) => {
      // Update algorithm store
      useAlgorithmStore.getState().setAlgorithm(data);
      
      // Invalidate related queries
      queryClient.invalidateQueries(['algorithms']);
    },
    onError: (error) => {
      console.error('Failed to load algorithm:', error);
    },
  });
};

export const useStartBacktestMutation = () => {
  return useMutation({
    mutationFn: async (config: BacktestConfig) => {
      return await window.electronAPI.startBacktest(config);
    },
    onMutate: () => {
      // Optimistic update
      useAlgorithmStore.getState().setRunStatus('running');
    },
    onError: () => {
      // Revert optimistic update
      useAlgorithmStore.getState().setRunStatus('idle');
    },
  });
};
```

## Real-time Data Handling

Efficient real-time data processing and UI updates to handle high-frequency market data without blocking the interface.

### 1. WebSocket/IPC Communication

**Message Handling**: Structured approach to real-time data updates from the backend.

```typescript
// IPC message types
interface IPCMessage {
  type: string;
  payload: any;
  timestamp: number;
}

interface MarketDataUpdate extends IPCMessage {
  type: 'market_data_update';
  payload: {
    symbol: string;
    timeframe: string;
    data: OHLCData[];
    partial?: Partial<OHLCData>;
  };
}

interface BacktestProgress extends IPCMessage {
  type: 'backtest_progress';
  payload: {
    current: number;
    total: number;
    percentage: number;
    currentTime: string;
    trades: number;
  };
}

interface PositionUpdate extends IPCMessage {
  type: 'position_update';
  payload: {
    positions: Position[];
    totalPnL: number;
    unrealizedPnL: number;
  };
}

// IPC communication manager
class IPCManager {
  private eventListeners: Map<string, Set<(data: any) => void>> = new Map();
  private messageQueue: IPCMessage[] = [];
  private isProcessing = false;
  
  constructor() {
    this.setupIPC();
  }
  
  private setupIPC() {
    window.electronAPI.onMessage((message: IPCMessage) => {
      this.messageQueue.push(message);
      this.processQueue();
    });
  }
  
  private async processQueue() {
    if (this.isProcessing) return;
    this.isProcessing = true;
    
    while (this.messageQueue.length > 0) {
      const message = this.messageQueue.shift()!;
      await this.handleMessage(message);
      
      // Yield to prevent blocking UI
      await new Promise(resolve => setTimeout(resolve, 0));
    }
    
    this.isProcessing = false;
  }
  
  private async handleMessage(message: IPCMessage) {
    const listeners = this.eventListeners.get(message.type);
    if (listeners) {
      listeners.forEach(listener => {
        try {
          listener(message.payload);
        } catch (error) {
          console.error(`Error in IPC listener for ${message.type}:`, error);
        }
      });
    }
  }
  
  subscribe(messageType: string, listener: (data: any) => void) {
    if (!this.eventListeners.has(messageType)) {
      this.eventListeners.set(messageType, new Set());
    }
    this.eventListeners.get(messageType)!.add(listener);
    
    return () => {
      const listeners = this.eventListeners.get(messageType);
      if (listeners) {
        listeners.delete(listener);
        if (listeners.size === 0) {
          this.eventListeners.delete(messageType);
        }
      }
    };
  }
  
  async sendMessage(message: IPCMessage) {
    return await window.electronAPI.sendMessage(message);
  }
}

export const ipcManager = new IPCManager();

// Real-time data hooks
export const useRealTimeMarketData = (symbol: string, timeframes: string[]) => {
  const [marketData, setMarketData] = useState<Record<string, OHLCData[]>>({});
  const [partialBars, setPartialBars] = useState<Record<string, Partial<OHLCData>>>({});
  
  useEffect(() => {
    const unsubscribes = timeframes.map(timeframe => {
      return ipcManager.subscribe('market_data_update', (update: MarketDataUpdate['payload']) => {
        if (update.symbol === symbol && timeframes.includes(update.timeframe)) {
          setMarketData(prev => ({
            ...prev,
            [update.timeframe]: update.data
          }));
          
          if (update.partial) {
            setPartialBars(prev => ({
              ...prev,
              [update.timeframe]: update.partial!
            }));
          }
        }
      });
    });
    
    return () => {
      unsubscribes.forEach(unsub => unsub());
    };
  }, [symbol, timeframes]);
  
  return { marketData, partialBars };
};

export const useBacktestProgress = () => {
  const [progress, setProgress] = useState<BacktestProgress['payload']>({
    current: 0,
    total: 0,
    percentage: 0,
    currentTime: '',
    trades: 0,
  });
  
  useEffect(() => {
    return ipcManager.subscribe('backtest_progress', setProgress);
  }, []);
  
  return progress;
};

export const usePositionUpdates = () => {
  const [positions, setPositions] = useState<Position[]>([]);
  const [pnlData, setPnlData] = useState({ total: 0, unrealized: 0 });
  
  useEffect(() => {
    return ipcManager.subscribe('position_update', (update: PositionUpdate['payload']) => {
      setPositions(update.positions);
      setPnlData({
        total: update.totalPnL,
        unrealized: update.unrealizedPnL,
      });
    });
  }, []);
  
  return { positions, pnlData };
};
```

### 2. Data Streaming

**Efficient Update Batching**: Batch updates to prevent UI thrashing during high-frequency data streams.

```typescript
// Update batching utility
class UpdateBatcher<T> {
  private updates: T[] = [];
  private timeout: NodeJS.Timeout | null = null;
  private readonly maxBatchSize: number;
  private readonly maxWaitTime: number;
  private readonly onFlush: (updates: T[]) => void;
  
  constructor(
    onFlush: (updates: T[]) => void,
    maxBatchSize = 100,
    maxWaitTime = 16 // ~60 FPS
  ) {
    this.onFlush = onFlush;
    this.maxBatchSize = maxBatchSize;
    this.maxWaitTime = maxWaitTime;
  }
  
  add(update: T) {
    this.updates.push(update);
    
    if (this.updates.length >= this.maxBatchSize) {
      this.flush();
    } else if (!this.timeout) {
      this.timeout = setTimeout(() => this.flush(), this.maxWaitTime);
    }
  }
  
  private flush() {
    if (this.timeout) {
      clearTimeout(this.timeout);
      this.timeout = null;
    }
    
    if (this.updates.length > 0) {
      const updates = [...this.updates];
      this.updates = [];
      this.onFlush(updates);
    }
  }
}

// Market data streaming with batching
export const useStreamingMarketData = (symbol: string, timeframes: string[]) => {
  const [data, setData] = useState<Record<string, StreamingData>>({});
  const batcherRef = useRef<UpdateBatcher<MarketDataUpdate> | null>(null);
  
  useEffect(() => {
    // Create batcher for market data updates
    batcherRef.current = new UpdateBatcher<MarketDataUpdate>(
      (updates) => {
        // Process batched updates efficiently
        const groupedUpdates = updates.reduce((acc, update) => {
          const key = `${update.symbol}_${update.timeframe}`;
          if (!acc[key]) acc[key] = [];
          acc[key].push(update);
          return acc;
        }, {} as Record<string, MarketDataUpdate[]>);
        
        setData(prevData => {
          const newData = { ...prevData };
          
          Object.entries(groupedUpdates).forEach(([key, updateList]) => {
            const latestUpdate = updateList[updateList.length - 1];
            const timeframe = latestUpdate.timeframe;
            
            if (timeframes.includes(timeframe)) {
              newData[timeframe] = {
                bars: latestUpdate.data,
                partial: latestUpdate.partial,
                lastUpdate: Date.now(),
              };
            }
          });
          
          return newData;
        });
      },
      50, // Max batch size
      16  // Max wait time (60 FPS)
    );
    
    const unsubscribe = ipcManager.subscribe('market_data_update', (update) => {
      if (update.symbol === symbol) {
        batcherRef.current?.add(update);
      }
    });
    
    return () => {
      unsubscribe();
      batcherRef.current = null;
    };
  }, [symbol, timeframes]);
  
  return data;
};

// Chart update optimization
export const useOptimizedChartUpdates = (
  chartRef: React.RefObject<IChartApi>,
  data: StreamingData,
  timeframe: string
) => {
  const lastUpdateTime = useRef(0);
  const pendingUpdate = useRef<StreamingData | null>(null);
  
  useEffect(() => {
    if (!data) return;
    
    pendingUpdate.current = data;
    
    // Throttle chart updates to 60 FPS
    const now = performance.now();
    const timeSinceLastUpdate = now - lastUpdateTime.current;
    
    if (timeSinceLastUpdate >= 16) { // ~60 FPS
      performChartUpdate();
    } else {
      setTimeout(performChartUpdate, 16 - timeSinceLastUpdate);
    }
  }, [data]);
  
  const performChartUpdate = useCallback(() => {
    const chart = chartRef.current;
    const updateData = pendingUpdate.current;
    
    if (!chart || !updateData) return;
    
    // Update chart data efficiently
    const series = chart.getVisibleRange(); // Get existing series
    if (series) {
      // Update existing bars
      updateData.bars.forEach(bar => {
        // Only update if bar is in visible range for performance
        if (bar.time >= series.from && bar.time <= series.to) {
          chart.update(bar);
        }
      });
      
      // Update partial bar if present
      if (updateData.partial) {
        chart.update(updateData.partial);
      }
    }
    
    lastUpdateTime.current = performance.now();
    pendingUpdate.current = null;
  }, [chartRef]);
};
```

## UI/UX Design Patterns

Professional trading interface design patterns that prioritize usability, accessibility, and visual clarity for financial data.

### 1. Responsive Layout System

**Adaptive Panel System**: Flexible layout that adapts to different screen sizes and user preferences.

```tsx
// Responsive layout hook
export const useResponsiveLayout = () => {
  const [layout, setLayout] = useState<LayoutConfig>(() => getDefaultLayout());
  const [screenSize, setScreenSize] = useState<ScreenSize>('desktop');
  
  useEffect(() => {
    const updateScreenSize = () => {
      const width = window.innerWidth;
      if (width < 768) setScreenSize('mobile');
      else if (width < 1024) setScreenSize('tablet');
      else if (width < 1440) setScreenSize('desktop');
      else setScreenSize('wide');
    };
    
    updateScreenSize();
    window.addEventListener('resize', updateScreenSize);
    return () => window.removeEventListener('resize', updateScreenSize);
  }, []);
  
  // Adapt layout based on screen size
  useEffect(() => {
    const adaptedLayout = adaptLayoutForScreenSize(layout, screenSize);
    if (JSON.stringify(adaptedLayout) !== JSON.stringify(layout)) {
      setLayout(adaptedLayout);
    }
  }, [screenSize]);
  
  return { layout, setLayout, screenSize };
};

// Layout adaptation function
const adaptLayoutForScreenSize = (layout: LayoutConfig, screenSize: ScreenSize): LayoutConfig => {
  switch (screenSize) {
    case 'mobile':
      return {
        ...layout,
        chartGrid: { columns: 1, rows: 6 }, // Stack charts vertically
        sidebarWidth: 0, // Hide sidebar
        bottomPanelHeight: '40%',
      };
    case 'tablet':
      return {
        ...layout,
        chartGrid: { columns: 2, rows: 3 }, // 2x3 grid
        sidebarWidth: 200,
        bottomPanelHeight: '35%',
      };
    case 'desktop':
      return {
        ...layout,
        chartGrid: { columns: 3, rows: 2 }, // 3x2 grid
        sidebarWidth: 280,
        bottomPanelHeight: '30%',
      };
    case 'wide':
      return {
        ...layout,
        chartGrid: { columns: 6, rows: 1 }, // Single row, all timeframes
        sidebarWidth: 320,
        bottomPanelHeight: '25%',
      };
    default:
      return layout;
  }
};

// Responsive chart grid component
export const ResponsiveChartGrid: React.FC = () => {
  const { layout, screenSize } = useResponsiveLayout();
  const timeframes = ['1m', '5m', '15m', '1H', '4H', 'Daily'];
  
  return (
    <div 
      className={`chart-grid chart-grid-${screenSize}`}
      style={{
        gridTemplateColumns: `repeat(${layout.chartGrid.columns}, 1fr)`,
        gridTemplateRows: `repeat(${layout.chartGrid.rows}, 1fr)`,
      }}
    >
      {timeframes.map((timeframe, index) => (
        <div
          key={timeframe}
          className={`chart-cell ${screenSize === 'mobile' ? 'chart-cell-mobile' : ''}`}
          style={{
            gridColumn: screenSize === 'mobile' ? '1' : undefined,
            gridRow: screenSize === 'mobile' ? `${index + 1}` : undefined,
          }}
        >
          <ChartPanel timeframe={timeframe} />
        </div>
      ))}
    </div>
  );
};
```

### 2. Theming System

**Professional Trading Theme**: Dark-optimized color scheme designed for extended use and visual clarity.

```scss
// CSS Variables for theming
:root {
  // Dark theme (default)
  --bg-primary: #0f0f0f;
  --bg-secondary: #1a1a1a;
  --bg-tertiary: #2a2a2a;
  --bg-accent: #3a3a3a;
  
  --text-primary: #ffffff;
  --text-secondary: #d1d5db;
  --text-muted: #9ca3af;
  --text-disabled: #6b7280;
  
  --border-primary: #374151;
  --border-secondary: #4b5563;
  --border-accent: #6b7280;
  
  // Trading colors
  --color-bullish: #22c55e;
  --color-bearish: #ef4444;
  --color-warning: #f59e0b;
  --color-info: #3b82f6;
  
  // Chart colors
  --chart-bg: #1a1a1a;
  --chart-grid: #374151;
  --chart-text: #d1d5db;
  --chart-crosshair: #3b82f6;
  
  // UI component colors
  --button-primary: #3b82f6;
  --button-secondary: #6b7280;
  --button-success: #22c55e;
  --button-danger: #ef4444;
  
  // Shadows and effects
  --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
}

// Light theme variant
[data-theme="light"] {
  --bg-primary: #ffffff;
  --bg-secondary: #f8fafc;
  --bg-tertiary: #e2e8f0;
  --bg-accent: #cbd5e1;
  
  --text-primary: #1f2937;
  --text-secondary: #374151;
  --text-muted: #6b7280;
  --text-disabled: #9ca3af;
  
  --border-primary: #e5e7eb;
  --border-secondary: #d1d5db;
  --border-accent: #9ca3af;
  
  --chart-bg: #ffffff;
  --chart-grid: #e5e7eb;
  --chart-text: #374151;
}

// Component styles using theme variables
.chart-panel {
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  color: var(--text-primary);
  
  .chart-toolbar {
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border-secondary);
    padding: 0.5rem 1rem;
    
    .toolbar-button {
      background: transparent;
      border: 1px solid var(--border-secondary);
      color: var(--text-secondary);
      transition: all 0.2s ease;
      
      &:hover {
        background: var(--bg-accent);
        border-color: var(--border-accent);
        color: var(--text-primary);
      }
      
      &.active {
        background: var(--button-primary);
        border-color: var(--button-primary);
        color: white;
      }
    }
  }
  
  .chart-container {
    background: var(--chart-bg);
    height: calc(100% - 3rem); // Account for toolbar height
  }
}

// Trading-specific color utilities
.pnl-positive {
  color: var(--color-bullish);
}

.pnl-negative {
  color: var(--color-bearish);
}

.price-up {
  background: var(--color-bullish);
  color: white;
}

.price-down {
  background: var(--color-bearish);
  color: white;
}

// Responsive design utilities
@media (max-width: 768px) {
  .chart-grid {
    grid-template-columns: 1fr !important;
    grid-template-rows: repeat(6, 200px) !important;
    overflow-y: auto;
  }
  
  .sidebar {
    position: fixed;
    left: -100%;
    transition: left 0.3s ease;
    z-index: 1000;
    
    &.open {
      left: 0;
    }
  }
  
  .chart-toolbar {
    .toolbar-button {
      padding: 0.25rem 0.5rem;
      font-size: 0.875rem;
    }
  }
}

@media (max-width: 480px) {
  .chart-grid {
    grid-template-rows: repeat(6, 150px) !important;
  }
  
  .bottom-panel {
    height: 50vh !important;
  }
}
```

### 3. Accessibility Features

**WCAG 2.1 Compliance**: Comprehensive accessibility support for professional trading environments.

```tsx
// Accessible component patterns
export const AccessibleButton: React.FC<AccessibleButtonProps> = ({
  children,
  onClick,
  disabled = false,
  variant = 'primary',
  size = 'medium',
  ariaLabel,
  ...props
}) => {
  return (
    <button
      className={`btn btn-${variant} btn-${size}`}
      onClick={onClick}
      disabled={disabled}
      aria-label={ariaLabel}
      aria-disabled={disabled}
      {...props}
    >
      {children}
    </button>
  );
};

// Keyboard navigation hook
export const useKeyboardNavigation = (items: NavigationItem[]) => {
  const [activeIndex, setActiveIndex] = useState(0);
  
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      switch (event.key) {
        case 'ArrowUp':
          event.preventDefault();
          setActiveIndex(prev => (prev > 0 ? prev - 1 : items.length - 1));
          break;
        case 'ArrowDown':
          event.preventDefault();
          setActiveIndex(prev => (prev < items.length - 1 ? prev + 1 : 0));
          break;
        case 'Enter':
        case ' ':
          event.preventDefault();
          items[activeIndex]?.action();
          break;
        case 'Escape':
          setActiveIndex(0);
          break;
      }
    };
    
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [items, activeIndex]);
  
  return { activeIndex, setActiveIndex };
};

// Screen reader announcements
export const useScreenReader = () => {
  const announce = useCallback((message: string, priority: 'polite' | 'assertive' = 'polite') => {
    const announcement = document.createElement('div');
    announcement.setAttribute('aria-live', priority);
    announcement.setAttribute('aria-atomic', 'true');
    announcement.className = 'sr-only';
    announcement.textContent = message;
    
    document.body.appendChild(announcement);
    
    setTimeout(() => {
      document.body.removeChild(announcement);
    }, 1000);
  }, []);
  
  return { announce };
};

// Accessible chart component
export const AccessibleChart: React.FC<AccessibleChartProps> = ({
  data,
  timeframe,
  onDataChange
}) => {
  const { announce } = useScreenReader();
  const chartRef = useRef<IChartApi>(null);
  const [isDataLoading, setIsDataLoading] = useState(false);
  
  // Announce data updates to screen readers
  useEffect(() => {
    if (data && data.length > 0) {
      const latestBar = data[data.length - 1];
      announce(
        `${timeframe} chart updated. Latest price: ${latestBar.close}, ` +
        `change: ${latestBar.close - latestBar.open > 0 ? 'up' : 'down'}`,
        'polite'
      );
    }
  }, [data, timeframe, announce]);
  
  // Keyboard chart navigation
  useEffect(() => {
    const handleChartKeyboard = (event: KeyboardEvent) => {
      if (!chartRef.current) return;
      
      const chart = chartRef.current;
      const timeScale = chart.timeScale();
      
      switch (event.key) {
        case 'ArrowLeft':
          event.preventDefault();
          timeScale.scrollPosition(-10);
          announce('Chart scrolled left', 'polite');
          break;
        case 'ArrowRight':
          event.preventDefault();
          timeScale.scrollPosition(10);
          announce('Chart scrolled right', 'polite');
          break;
        case '+':
        case '=':
          event.preventDefault();
          timeScale.zoomIn();
          announce('Chart zoomed in', 'polite');
          break;
        case '-':
          event.preventDefault();
          timeScale.zoomOut();
          announce('Chart zoomed out', 'polite');
          break;
      }
    };
    
    window.addEventListener('keydown', handleChartKeyboard);
    return () => window.removeEventListener('keydown', handleChartKeyboard);
  }, [announce]);
  
  return (
    <div 
      className="accessible-chart"
      role="img"
      aria-label={`${timeframe} price chart`}
      aria-describedby={`chart-description-${timeframe}`}
      tabIndex={0}
    >
      <div 
        id={`chart-description-${timeframe}`}
        className="sr-only"
      >
        Price chart showing {timeframe} candlestick data. 
        Use arrow keys to navigate, plus and minus to zoom.
      </div>
      
      {isDataLoading && (
        <div 
          role="status" 
          aria-live="polite"
          className="loading-indicator"
        >
          Loading chart data...
        </div>
      )}
      
      <LightweightChart
        ref={chartRef}
        data={data}
        timeframe={timeframe}
        onDataChange={onDataChange}
      />
    </div>
  );
};

// High contrast mode support
export const useHighContrast = () => {
  const [isHighContrast, setIsHighContrast] = useState(false);
  
  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-contrast: high)');
    setIsHighContrast(mediaQuery.matches);
    
    const handleChange = (e: MediaQueryListEvent) => {
      setIsHighContrast(e.matches);
    };
    
    mediaQuery.addEventListener('change', handleChange);
    return () => mediaQuery.removeEventListener('change', handleChange);
  }, []);
  
  useEffect(() => {
    if (isHighContrast) {
      document.documentElement.classList.add('high-contrast');
    } else {
      document.documentElement.classList.remove('high-contrast');
    }
  }, [isHighContrast]);
  
  return { isHighContrast, setIsHighContrast };
};
```

## Performance Optimizations

Critical frontend optimizations to maintain 60 FPS performance during intensive chart rendering and real-time data updates.

### 1. React Memoization

**Selective Re-rendering**: Prevent unnecessary component updates during high-frequency data streams.

```tsx
// Memoized chart component
export const ChartPanel = React.memo<ChartPanelProps>(({
  timeframe,
  data,
  config,
  onConfigChange
}) => {
  // Component implementation
}, (prevProps, nextProps) => {
  // Custom comparison for shallow equality on data arrays
  if (prevProps.timeframe !== nextProps.timeframe) return false;
  if (prevProps.config !== nextProps.config) return false;
  
  // Deep comparison for data array (only check length and last item for performance)
  if (prevProps.data.length !== nextProps.data.length) return false;
  if (prevProps.data.length > 0 && nextProps.data.length > 0) {
    const prevLast = prevProps.data[prevProps.data.length - 1];
    const nextLast = nextProps.data[nextProps.data.length - 1];
    if (prevLast.time !== nextLast.time || prevLast.close !== nextLast.close) return false;
  }
  
  return true;
});

// Memoized data processing hooks
export const useProcessedMarketData = (rawData: RawMarketData[]) => {
  return useMemo(() => {
    return rawData.map(item => ({
      time: item.timestamp / 1000, // Convert to seconds for Lightweight Charts
      open: item.open,
      high: item.high,
      low: item.low,
      close: item.close,
      volume: item.volume,
    }));
  }, [rawData]);
};

export const useCalculatedIndicators = (data: OHLCData[], indicators: IndicatorConfig[]) => {
  return useMemo(() => {
    const results: Record<string, IndicatorResult> = {};
    
    indicators.forEach(indicator => {
      switch (indicator.type) {
        case 'sma':
          results[indicator.id] = calculateSMA(data, indicator.params.period);
          break;
        case 'rsi':
          results[indicator.id] = calculateRSI(data, indicator.params.period);
          break;
        // ... other indicators
      }
    });
    
    return results;
  }, [data, indicators]);
};

// Debounced resize handler
export const useDebouncedResize = (callback: () => void, delay = 100) => {
  const timeoutRef = useRef<NodeJS.Timeout>();
  
  useEffect(() => {
    const handleResize = () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
      timeoutRef.current = setTimeout(callback, delay);
    };
    
    window.addEventListener('resize', handleResize);
    return () => {
      window.removeEventListener('resize', handleResize);
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, [callback, delay]);
};

// Throttled scroll handler
export const useThrottledScroll = (callback: (scrollY: number) => void, delay = 16) => {
  const lastExecution = useRef(0);
  
  useEffect(() => {
    const handleScroll = () => {
      const now = Date.now();
      if (now - lastExecution.current >= delay) {
        callback(window.scrollY);
        lastExecution.current = now;
      }
    };
    
    window.addEventListener('scroll', handleScroll, { passive: true });
    return () => window.removeEventListener('scroll', handleScroll);
  }, [callback, delay]);
};
```

### 2. Virtual Scrolling

**Efficient List Rendering**: Handle large datasets without performance degradation.

```tsx
// Virtual scrolling implementation for trade tables
export const VirtualTradeTable: React.FC<VirtualTradeTableProps> = ({
  trades,
  height = 400,
  itemHeight = 40
}) => {
  const [scrollTop, setScrollTop] = useState(0);
  const containerRef = useRef<HTMLDivElement>(null);
  
  const visibleItemCount = Math.ceil(height / itemHeight);
  const totalHeight = trades.length * itemHeight;
  const startIndex = Math.floor(scrollTop / itemHeight);
  const endIndex = Math.min(startIndex + visibleItemCount + 1, trades.length);
  
  const visibleTrades = trades.slice(startIndex, endIndex);
  const offsetY = startIndex * itemHeight;
  
  const handleScroll = useCallback((e: React.UIEvent<HTMLDivElement>) => {
    setScrollTop(e.currentTarget.scrollTop);
  }, []);
  
  return (
    <div
      ref={containerRef}
      className="virtual-trade-table"
      style={{ height, overflow: 'auto' }}
      onScroll={handleScroll}
    >
      <div style={{ height: totalHeight, position: 'relative' }}>
        <div
          style={{
            transform: `translateY(${offsetY}px)`,
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
          }}
        >
          {visibleTrades.map((trade, index) => (
            <TradeRow
              key={trade.id}
              trade={trade}
              style={{ height: itemHeight }}
              index={startIndex + index}
            />
          ))}
        </div>
      </div>
    </div>
  );
};

// Optimized trade row component
const TradeRow = React.memo<TradeRowProps>(({ trade, style, index }) => {
  const pnlClass = trade.pnl >= 0 ? 'pnl-positive' : 'pnl-negative';
  
  return (
    <div className={`trade-row ${index % 2 === 0 ? 'even' : 'odd'}`} style={style}>
      <div className="trade-cell">{trade.symbol}</div>
      <div className="trade-cell">{trade.side}</div>
      <div className="trade-cell">{trade.size}</div>
      <div className="trade-cell">{trade.entryPrice}</div>
      <div className="trade-cell">{trade.exitPrice}</div>
      <div className={`trade-cell ${pnlClass}`}>{formatCurrency(trade.pnl)}</div>
      <div className="trade-cell">{formatDateTime(trade.timestamp)}</div>
    </div>
  );
});

// Intersection observer for lazy loading
export const useLazyLoading = <T,>(
  items: T[],
  loadMore: () => Promise<void>,
  threshold = 0.1
) => {
  const [isLoading, setIsLoading] = useState(false);
  const sentinelRef = useRef<HTMLDivElement>(null);
  
  useEffect(() => {
    const sentinel = sentinelRef.current;
    if (!sentinel) return;
    
    const observer = new IntersectionObserver(
      async (entries) => {
        if (entries[0].isIntersecting && !isLoading) {
          setIsLoading(true);
          await loadMore();
          setIsLoading(false);
        }
      },
      { threshold }
    );
    
    observer.observe(sentinel);
    return () => observer.disconnect();
  }, [loadMore, isLoading, threshold]);
  
  return { sentinelRef, isLoading };
};
```

### 3. Canvas Rendering

**High-Performance Chart Rendering**: Direct canvas manipulation for performance-critical visualizations.

```tsx
// Canvas-based heat map component
export const PerformanceHeatmap: React.FC<PerformanceHeatmapProps> = ({
  data,
  width = 800,
  height = 400
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationRef = useRef<number>();
  
  const drawHeatmap = useCallback((ctx: CanvasRenderingContext2D) => {
    ctx.clearRect(0, 0, width, height);
    
    const cellWidth = width / data.columns;
    const cellHeight = height / data.rows;
    
    // Use RequestAnimationFrame for smooth rendering
    const renderCell = (x: number, y: number, value: number) => {
      const color = getHeatmapColor(value, data.min, data.max);
      ctx.fillStyle = color;
      ctx.fillRect(x * cellWidth, y * cellHeight, cellWidth, cellHeight);
      
      // Add text if cell is large enough
      if (cellWidth > 40 && cellHeight > 20) {
        ctx.fillStyle = getContrastColor(color);
        ctx.font = '12px monospace';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(
          value.toFixed(2),
          x * cellWidth + cellWidth / 2,
          y * cellHeight + cellHeight / 2
        );
      }
    };
    
    // Batch render cells for performance
    let cellIndex = 0;
    const renderBatch = () => {
      const batchSize = 100; // Render 100 cells per frame
      const endIndex = Math.min(cellIndex + batchSize, data.values.length);
      
      for (let i = cellIndex; i < endIndex; i++) {
        const x = i % data.columns;
        const y = Math.floor(i / data.columns);
        renderCell(x, y, data.values[i]);
      }
      
      cellIndex = endIndex;
      
      if (cellIndex < data.values.length) {
        animationRef.current = requestAnimationFrame(renderBatch);
      }
    };
    
    renderBatch();
  }, [data, width, height]);
  
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    
    // Set canvas size for high DPI displays
    const dpr = window.devicePixelRatio || 1;
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
    ctx.scale(dpr, dpr);
    
    drawHeatmap(ctx);
    
    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [drawHeatmap]);
  
  return (
    <canvas
      ref={canvasRef}
      className="performance-heatmap"
      style={{ width, height }}
    />
  );
};

// WebGL-accelerated chart rendering (for extremely high-frequency data)
export const WebGLChart: React.FC<WebGLChartProps> = ({ data, width, height }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const glRef = useRef<WebGLRenderingContext | null>(null);
  const programRef = useRef<WebGLProgram | null>(null);
  
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    
    const gl = canvas.getContext('webgl');
    if (!gl) {
      console.warn('WebGL not supported, falling back to canvas');
      return;
    }
    
    glRef.current = gl;
    
    // Vertex shader for line rendering
    const vertexShaderSource = `
      attribute vec2 a_position;
      uniform vec2 u_resolution;
      uniform vec2 u_scale;
      uniform vec2 u_offset;
      
      void main() {
        vec2 position = (a_position * u_scale + u_offset) / u_resolution * 2.0 - 1.0;
        gl_Position = vec4(position * vec2(1, -1), 0, 1);
      }
    `;
    
    const fragmentShaderSource = `
      precision mediump float;
      uniform vec3 u_color;
      
      void main() {
        gl_FragColor = vec4(u_color, 1.0);
      }
    `;
    
    const program = createShaderProgram(gl, vertexShaderSource, fragmentShaderSource);
    if (!program) return;
    
    programRef.current = program;
    
    return () => {
      if (gl && program) {
        gl.deleteProgram(program);
      }
    };
  }, []);
  
  // Render chart data using WebGL
  useEffect(() => {
    const gl = glRef.current;
    const program = programRef.current;
    if (!gl || !program || !data.length) return;
    
    gl.useProgram(program);
    
    // Create vertex buffer
    const positions = new Float32Array(data.length * 2);
    data.forEach((point, i) => {
      positions[i * 2] = point.x;
      positions[i * 2 + 1] = point.y;
    });
    
    const positionBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STATIC_DRAW);
    
    // Set up attributes and uniforms
    const positionLocation = gl.getAttribLocation(program, 'a_position');
    gl.enableVertexAttribArray(positionLocation);
    gl.vertexAttribPointer(positionLocation, 2, gl.FLOAT, false, 0, 0);
    
    const resolutionLocation = gl.getUniformLocation(program, 'u_resolution');
    gl.uniform2f(resolutionLocation, width, height);
    
    const colorLocation = gl.getUniformLocation(program, 'u_color');
    gl.uniform3f(colorLocation, 0.2, 0.8, 1.0); // Blue color
    
    // Clear and draw
    gl.viewport(0, 0, width, height);
    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.LINE_STRIP, 0, data.length);
    
    return () => {
      if (positionBuffer) {
        gl.deleteBuffer(positionBuffer);
      }
    };
  }, [data, width, height]);
  
  return (
    <canvas
      ref={canvasRef}
      width={width}
      height={height}
      className="webgl-chart"
    />
  );
};

// Utility functions for WebGL
const createShaderProgram = (
  gl: WebGLRenderingContext,
  vertexSource: string,
  fragmentSource: string
): WebGLProgram | null => {
  const vertexShader = createShader(gl, gl.VERTEX_SHADER, vertexSource);
  const fragmentShader = createShader(gl, gl.FRAGMENT_SHADER, fragmentSource);
  
  if (!vertexShader || !fragmentShader) return null;
  
  const program = gl.createProgram();
  if (!program) return null;
  
  gl.attachShader(program, vertexShader);
  gl.attachShader(program, fragmentShader);
  gl.linkProgram(program);
  
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    console.error('Program link error:', gl.getProgramInfoLog(program));
    gl.deleteProgram(program);
    return null;
  }
  
  return program;
};

const createShader = (
  gl: WebGLRenderingContext,
  type: number,
  source: string
): WebGLShader | null => {
  const shader = gl.createShader(type);
  if (!shader) return null;
  
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    console.error('Shader compile error:', gl.getShaderInfoLog(shader));
    gl.deleteShader(shader);
    return null;
  }
  
  return shader;
};
```

## Electron Integration

Seamless integration between the React frontend and Electron's main process, with secure IPC communication and native OS features.

### 1. Main/Renderer Process Architecture

**Process Separation**: Clean separation of concerns between main process (system integration) and renderer process (UI).

```typescript
// Main process (main.ts)
import { app, BrowserWindow, ipcMain, Menu, dialog } from 'electron';
import { createProtocol } from 'vue-cli-plugin-electron-builder/lib';
import path from 'path';

class MainProcess {
  private mainWindow: BrowserWindow | null = null;
  private backtestEngine: BacktestEngine | null = null;
  
  constructor() {
    this.setupApp();
    this.registerIPCHandlers();
  }
  
  private setupApp() {
    app.whenReady().then(() => {
      this.createWindow();
      this.setupMenu();
    });
    
    app.on('window-all-closed', () => {
      if (process.platform !== 'darwin') {
        app.quit();
      }
    });
    
    app.on('activate', () => {
      if (BrowserWindow.getAllWindows().length === 0) {
        this.createWindow();
      }
    });
  }
  
  private createWindow() {
    this.mainWindow = new BrowserWindow({
      width: 1400,
      height: 900,
      minWidth: 800,
      minHeight: 600,
      frame: false, // Custom title bar
      titleBarStyle: 'hidden',
      webPreferences: {
        nodeIntegration: false,
        contextIsolation: true,
        enableRemoteModule: false,
        preload: path.join(__dirname, 'preload.js'),
        webSecurity: true,
      },
      icon: path.join(__dirname, '../assets/icon.png'),
    });
    
    // Load the app
    if (process.env.NODE_ENV === 'development') {
      this.mainWindow.loadURL('http://localhost:3000');
      this.mainWindow.webContents.openDevTools();
    } else {
      createProtocol('app');
      this.mainWindow.loadURL('app://./index.html');
    }
    
    // Window event handlers
    this.mainWindow.on('closed', () => {
      this.mainWindow = null;
    });
    
    this.mainWindow.on('maximize', () => {
      this.sendToRenderer('window-maximized', true);
    });
    
    this.mainWindow.on('unmaximize', () => {
      this.sendToRenderer('window-maximized', false);
    });
  }
  
  private setupMenu() {
    const template: Electron.MenuItemConstructorOptions[] = [
      {
        label: 'File',
        submenu: [
          {
            label: 'New Algorithm',
            accelerator: 'CmdOrCtrl+N',
            click: () => this.sendToRenderer('menu-new-algorithm'),
          },
          {
            label: 'Open Algorithm',
            accelerator: 'CmdOrCtrl+O',
            click: () => this.handleOpenAlgorithm(),
          },
          {
            label: 'Save Algorithm',
            accelerator: 'CmdOrCtrl+S',
            click: () => this.sendToRenderer('menu-save-algorithm'),
          },
          { type: 'separator' },
          {
            label: 'Export Results',
            submenu: [
              {
                label: 'PDF Report',
                click: () => this.handleExportPDF(),
              },
              {
                label: 'CSV Data',
                click: () => this.handleExportCSV(),
              },
            ],
          },
          { type: 'separator' },
          { role: 'quit' },
        ],
      },
      {
        label: 'View',
        submenu: [
          { role: 'reload' },
          { role: 'forceReload' },
          { role: 'toggleDevTools' },
          { type: 'separator' },
          { role: 'resetZoom' },
          { role: 'zoomIn' },
          { role: 'zoomOut' },
          { type: 'separator' },
          { role: 'togglefullscreen' },
        ],
      },
      {
        label: 'Backtest',
        submenu: [
          {
            label: 'Start',
            accelerator: 'CmdOrCtrl+R',
            click: () => this.sendToRenderer('menu-start-backtest'),
          },
          {
            label: 'Stop',
            accelerator: 'CmdOrCtrl+.',
            click: () => this.sendToRenderer('menu-stop-backtest'),
          },
          {
            label: 'Pause',
            accelerator: 'CmdOrCtrl+P',
            click: () => this.sendToRenderer('menu-pause-backtest'),
          },
        ],
      },
    ];
    
    const menu = Menu.buildFromTemplate(template);
    Menu.setApplicationMenu(menu);
  }
  
  private registerIPCHandlers() {
    // Window controls
    ipcMain.handle('window-minimize', () => {
      this.mainWindow?.minimize();
    });
    
    ipcMain.handle('window-maximize', () => {
      if (this.mainWindow?.isMaximized()) {
        this.mainWindow.unmaximize();
      } else {
        this.mainWindow?.maximize();
      }
    });
    
    ipcMain.handle('window-close', () => {
      this.mainWindow?.close();
    });
    
    // File operations
    ipcMain.handle('open-file-dialog', async (_, options) => {
      const result = await dialog.showOpenDialog(this.mainWindow!, options);
      return result;
    });
    
    ipcMain.handle('save-file-dialog', async (_, options) => {
      const result = await dialog.showSaveDialog(this.mainWindow!, options);
      return result;
    });
    
    ipcMain.handle('read-file', async (_, filePath) => {
      const fs = await import('fs/promises');
      const content = await fs.readFile(filePath, 'utf-8');
      return content;
    });
    
    ipcMain.handle('write-file', async (_, filePath, content) => {
      const fs = await import('fs/promises');
      await fs.writeFile(filePath, content, 'utf-8');
    });
    
    // Backtest operations
    ipcMain.handle('load-algorithm', async (_, code) => {
      try {
        this.backtestEngine = new BacktestEngine();
        const result = await this.backtestEngine.loadAlgorithm(code);
        return { success: true, data: result };
      } catch (error) {
        return { success: false, error: error.message };
      }
    });
    
    ipcMain.handle('start-backtest', async (_, config) => {
      try {
        if (!this.backtestEngine) {
          throw new Error('No algorithm loaded');
        }
        
        const result = await this.backtestEngine.startBacktest(config);
        return { success: true, data: result };
      } catch (error) {
        return { success: false, error: error.message };
      }
    });
    
    ipcMain.handle('stop-backtest', async () => {
      try {
        await this.backtestEngine?.stopBacktest();
        return { success: true };
      } catch (error) {
        return { success: false, error: error.message };
      }
    });
    
    // Market data operations
    ipcMain.handle('get-market-data', async (_, request) => {
      try {
        const data = await this.getMarketDataFromBackend(request);
        return { success: true, data };
      } catch (error) {
        return { success: false, error: error.message };
      }
    });
  }
  
  private sendToRenderer(channel: string, ...args: any[]) {
    this.mainWindow?.webContents.send(channel, ...args);
  }
  
  private async handleOpenAlgorithm() {
    const result = await dialog.showOpenDialog(this.mainWindow!, {
      filters: [{ name: 'Python Files', extensions: ['py'] }],
      properties: ['openFile'],
    });
    
    if (!result.canceled && result.filePaths.length > 0) {
      const fs = await import('fs/promises');
      const content = await fs.readFile(result.filePaths[0], 'utf-8');
      this.sendToRenderer('algorithm-loaded', content);
    }
  }
  
  private async handleExportPDF() {
    const result = await dialog.showSaveDialog(this.mainWindow!, {
      filters: [{ name: 'PDF Files', extensions: ['pdf'] }],
      defaultPath: 'backtest-report.pdf',
    });
    
    if (!result.canceled) {
      this.sendToRenderer('export-pdf', result.filePath);
    }
  }
  
  private async handleExportCSV() {
    const result = await dialog.showSaveDialog(this.mainWindow!, {
      filters: [{ name: 'CSV Files', extensions: ['csv'] }],
      defaultPath: 'backtest-results.csv',
    });
    
    if (!result.canceled) {
      this.sendToRenderer('export-csv', result.filePath);
    }
  }
  
  private async getMarketDataFromBackend(request: MarketDataRequest): Promise<MarketData> {
    // Interface with Rust backend for market data
    // This would use the MessagePack IPC protocol
    return {} as MarketData; // Placeholder
  }
}

// Initialize main process
new MainProcess();
```

### 2. Preload Script

**Secure IPC Bridge**: Context-isolated bridge between main and renderer processes.

```typescript
// preload.ts
import { contextBridge, ipcRenderer } from 'electron';

// Expose protected methods that allow the renderer process to use
// the ipcRenderer without exposing the entire object
contextBridge.exposeInMainWorld('electronAPI', {
  // Window controls
  minimize: () => ipcRenderer.invoke('window-minimize'),
  maximize: () => ipcRenderer.invoke('window-maximize'),
  close: () => ipcRenderer.invoke('window-close'),
  
  // File operations
  openFile: (options: Electron.OpenDialogOptions) => 
    ipcRenderer.invoke('open-file-dialog', options),
  saveFile: (options: Electron.SaveDialogOptions) =>
    ipcRenderer.invoke('save-file-dialog', options),
  readFile: (filePath: string) =>
    ipcRenderer.invoke('read-file', filePath),
  writeFile: (filePath: string, content: string) =>
    ipcRenderer.invoke('write-file', filePath, content),
  
  // Algorithm operations
  loadAlgorithm: (code: string) =>
    ipcRenderer.invoke('load-algorithm', code),
  startBacktest: (config: BacktestConfig) =>
    ipcRenderer.invoke('start-backtest', config),
  stopBacktest: () =>
    ipcRenderer.invoke('stop-backtest'),
  pauseBacktest: () =>
    ipcRenderer.invoke('pause-backtest'),
  
  // Market data
  getMarketData: (request: MarketDataRequest) =>
    ipcRenderer.invoke('get-market-data', request),
  getBacktestResults: (backtestId: string) =>
    ipcRenderer.invoke('get-backtest-results', backtestId),
  getPerformanceMetrics: (backtestId: string) =>
    ipcRenderer.invoke('get-performance-metrics', backtestId),
  
  // Real-time updates
  onMessage: (callback: (message: any) => void) => {
    ipcRenderer.on('ipc-message', (_, message) => callback(message));
    return () => ipcRenderer.removeAllListeners('ipc-message');
  },
  
  sendMessage: (message: any) =>
    ipcRenderer.invoke('send-message', message),
  
  // Menu events
  onMenuAction: (callback: (action: string) => void) => {
    const actions = [
      'menu-new-algorithm',
      'menu-save-algorithm',
      'menu-start-backtest',
      'menu-stop-backtest',
      'menu-pause-backtest',
    ];
    
    actions.forEach(action => {
      ipcRenderer.on(action, () => callback(action));
    });
    
    return () => {
      actions.forEach(action => {
        ipcRenderer.removeAllListeners(action);
      });
    };
  },
  
  // Algorithm loading events
  onAlgorithmLoaded: (callback: (content: string) => void) => {
    ipcRenderer.on('algorithm-loaded', (_, content) => callback(content));
    return () => ipcRenderer.removeAllListeners('algorithm-loaded');
  },
  
  // Export events
  onExportRequest: (callback: (type: string, filePath: string) => void) => {
    ipcRenderer.on('export-pdf', (_, filePath) => callback('pdf', filePath));
    ipcRenderer.on('export-csv', (_, filePath) => callback('csv', filePath));
    
    return () => {
      ipcRenderer.removeAllListeners('export-pdf');
      ipcRenderer.removeAllListeners('export-csv');
    };
  },
  
  // Window state events
  onWindowStateChange: (callback: (isMaximized: boolean) => void) => {
    ipcRenderer.on('window-maximized', (_, isMaximized) => callback(isMaximized));
    return () => ipcRenderer.removeAllListeners('window-maximized');
  },
});

// Type definitions for the exposed API
declare global {
  interface Window {
    electronAPI: {
      minimize: () => Promise<void>;
      maximize: () => Promise<void>;
      close: () => Promise<void>;
      openFile: (options: Electron.OpenDialogOptions) => Promise<Electron.OpenDialogReturnValue>;
      saveFile: (options: Electron.SaveDialogOptions) => Promise<Electron.SaveDialogReturnValue>;
      readFile: (filePath: string) => Promise<string>;
      writeFile: (filePath: string, content: string) => Promise<void>;
      loadAlgorithm: (code: string) => Promise<{ success: boolean; data?: any; error?: string }>;
      startBacktest: (config: BacktestConfig) => Promise<{ success: boolean; data?: any; error?: string }>;
      stopBacktest: () => Promise<{ success: boolean; error?: string }>;
      pauseBacktest: () => Promise<{ success: boolean; error?: string }>;
      getMarketData: (request: MarketDataRequest) => Promise<{ success: boolean; data?: any; error?: string }>;
      getBacktestResults: (backtestId: string) => Promise<{ success: boolean; data?: any; error?: string }>;
      getPerformanceMetrics: (backtestId: string) => Promise<{ success: boolean; data?: any; error?: string }>;
      onMessage: (callback: (message: any) => void) => () => void;
      sendMessage: (message: any) => Promise<any>;
      onMenuAction: (callback: (action: string) => void) => () => void;
      onAlgorithmLoaded: (callback: (content: string) => void) => () => void;
      onExportRequest: (callback: (type: string, filePath: string) => void) => () => void;
      onWindowStateChange: (callback: (isMaximized: boolean) => void) => () => void;
    };
  }
}
```

This comprehensive Frontend Architecture section completes the technical foundation for BackTestr_ai's desktop application, providing a professional trading interface with institutional-grade performance and user experience. The architecture leverages modern web technologies within Electron to deliver real-time data visualization, responsive controls, and seamless integration with the high-performance Rust backend.
