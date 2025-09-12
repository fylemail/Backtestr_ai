# Technology Stack

## Frontend Technologies

### Primary UI Framework
- **Electron 28+**: Cross-platform desktop application framework
  - Main process for system integration and native OS features
  - Renderer processes for UI with Node.js integration disabled for security
  - Custom protocol handlers for secure inter-process communication
  - Native menu systems and file dialogs for professional desktop UX

- **React 18+**: Component-based UI library
  - Functional components with hooks for state management
  - Strict mode enabled for development quality assurance
  - React.memo for chart component optimization
  - Concurrent features for non-blocking UI updates during heavy processing

- **TypeScript 5+**: Static type checking and enhanced developer experience
  - Strict mode configuration with null checks and strict function types
  - Custom type definitions for financial data structures and chart interfaces
  - Interface definitions for IPC message contracts between processes
  - Generic types for timeframe-agnostic algorithm implementations

### Visualization and Charts
- **Lightweight Charts 4+**: TradingView's professional charting library
  - Six synchronized chart panels (1m, 5m, 15m, 1H, 4H, Daily)
  - Custom time scale synchronization across all panels
  - Real-time data streaming with efficient update batching
  - Custom series types for algorithm entry/exit markers
  - Overlay support for indicators and drawing tools

- **D3.js 7+**: Data visualization for analysis dashboards
  - Heatmap visualizations for parameter optimization results
  - P&L curve rendering with custom axis formatting
  - Interactive scatter plots for risk/return analysis
  - Custom SVG overlays for trade distribution analysis

### State Management and Data Flow
- **Zustand**: Lightweight state management
  - Minimal boilerplate compared to Redux for rapid development
  - TypeScript-first design with excellent IDE support
  - Persistent stores for user preferences and chart configurations
  - Real-time state synchronization with main process data

- **React Query**: Server state management and caching
  - Automatic background refetching of backtest results
  - Optimistic updates for improved perceived performance
  - Error boundary integration with automatic retry logic
  - Background synchronization with main process statistics

### Styling and UI Components
- **Tailwind CSS 3+**: Utility-first CSS framework
  - Custom design system with financial application color palette
  - Dark/light theme support with CSS variables
  - Responsive design for various screen sizes and resolutions
  - Custom components for charts, tables, and control panels

- **Headless UI**: Unstyled, accessible UI components
  - Modal dialogs for algorithm configuration and settings
  - Dropdown menus for timeframe and symbol selection
  - Tab panels for organizing analysis views
  - Popover components for contextual help and tooltips

## Backend Technologies

### Core Engine
- **Rust 1.75+**: High-performance systems programming language
  - Memory safety without garbage collection overhead
  - Zero-cost abstractions for financial calculations
  - Excellent concurrency primitives for multi-threaded processing
  - Windows-focused compilation with MSVC toolchain optimization

- **PyO3 0.20+**: Python-Rust language bridge
  - Embedded Python 3.11+ interpreter within Rust process
  - Zero-copy data sharing between Rust and Python for tick data
  - GIL management for optimal multi-threading performance
  - Custom Python modules exposed from Rust for data access

- **Tokio**: Asynchronous runtime for Rust
  - Non-blocking I/O for file operations and IPC communication
  - Task scheduling for concurrent algorithm execution
  - Timer utilities for precise backtesting timeline control
  - Channel-based communication between Rust subsystems

### Algorithm Execution Environment
- **Python 3.11+**: Embedded interpreter for algorithm development
  - NumPy 1.24+ for vectorized indicator calculations
  - Pandas 2.0+ for time series data manipulation and analysis
  - SciPy 1.10+ for advanced statistical functions and optimization
  - TA-Lib bindings for technical analysis indicators

- **Custom Python Extensions**: Rust-implemented modules
  - High-performance OHLC bar manipulation functions
  - Direct access to MTF state without serialization overhead
  - Position management APIs with realistic slippage modeling
  - Performance-critical indicator calculations in Rust

### Data Storage and Management
- **DuckDB 0.9+**: Embedded analytical database
  - Columnar storage with 10-20x compression for tick data
  - Vectorized query execution for high-speed analytics
  - Memory-mapped files for zero-copy data access
  - Built-in time series functions optimized for financial data

- **Parquet Integration**: Efficient data interchange format
  - Import/export compatibility with external data providers
  - Columnar compression for archival storage
  - Schema evolution support for data format updates
  - Integration with pandas for seamless data pipeline

### Inter-Process Communication
- **MessagePack**: High-performance binary serialization
  - 5-10x faster than JSON for numerical data arrays
  - Schema-less format for flexible data structure evolution
  - Cross-language compatibility between Rust and TypeScript
  - Compact binary representation minimizing IPC overhead

- **Custom IPC Protocol**: Application-specific communication layer
  - Batched updates to prevent UI flooding during high-frequency processing
  - Priority queuing for critical vs. informational messages
  - Compression for large data transfers (historical query results)
  - Message deduplication for efficient state synchronization

## Development Tools

### Build and Package Management
- **Cargo**: Rust package manager and build system
  - Workspace configuration for multi-crate project structure
  - Custom build scripts for Python extension compilation
  - Cross-compilation support for multiple target platforms
  - Dependency management with version pinning for reproducible builds

- **pnpm**: Fast, disk space efficient package manager (RECOMMENDED)
  - Lockfile-first approach for deterministic dependency resolution
  - Workspace configuration linking frontend and Electron main process
  - Custom scripts for development, building, and packaging workflows
  - Security auditing for dependency vulnerability management
  - Symbolic linking reduces node_modules duplication and improves Windows performance

- **Electron Builder**: Windows application packaging and distribution
  - Code signing for Windows distribution with Authenticode certificates
  - Auto-updater integration for seamless application updates
  - NSIS installer for Windows with custom installation options and MSI support
  - Windows Store packaging support for enterprise distribution

### Testing Frameworks
- **Rust Testing**: Built-in test framework and additional tools
  - Unit tests with `#[cfg(test)]` modules for core engine components
  - Integration tests for MTF state engine accuracy validation
  - Property-based testing with `proptest` for algorithm edge cases
  - Benchmark tests with `criterion` for performance regression detection

- **Jest**: JavaScript testing framework for frontend components
  - Unit tests for React components with React Testing Library
  - Mock implementations for IPC communication during testing
  - Snapshot testing for UI component regression detection
  - Code coverage reporting integrated with CI/CD pipeline

- **Python Testing**: Algorithm validation and verification
  - pytest for algorithm unit tests with financial data fixtures
  - Hypothesis for property-based testing of trading strategies
  - Custom test harnesses for backtesting accuracy validation
  - Performance profiling tools for algorithm optimization

### Code Quality and Analysis
- **Clippy**: Rust linter for code quality and best practices
  - Custom lint configuration for financial application requirements
  - Integration with CI/CD for automated code review
  - Performance lint rules for high-frequency trading optimizations
  - Security-focused lints for memory safety and data handling

- **ESLint + Prettier**: JavaScript/TypeScript code formatting and linting
  - TypeScript-specific rules for type safety enforcement
  - React-specific rules for component best practices
  - Import sorting and unused variable detection
  - Consistent code formatting across development team

- **Black + isort**: Python code formatting and import organization
  - Consistent code style for embedded Python algorithms
  - Integration with PyO3 build process for automatic formatting
  - Custom configuration for financial domain naming conventions
  - Pre-commit hooks for automated code quality enforcement

### Development Environment

- **Visual Studio Code**: Primary IDE with extensions
  - Rust Analyzer for intelligent code completion and error detection
  - Python extension with embedded interpreter debugging support
  - TypeScript/React extensions for frontend development
  - Custom debug configurations for multi-process application debugging

- **Git**: Version control with project-specific configuration
  - Large File Storage (LFS) for test data and sample tick files
  - Conventional commit messages for automated changelog generation
  - Branch protection rules for code review enforcement
  - Custom hooks for automated testing and formatting

### Windows Development Container Strategy

**Container-Based Development**: Streamlined setup using Windows containers and WSL2 for consistent development environments.

- **Docker Desktop for Windows**: Container runtime with WSL2 backend
  - Windows container support for native Windows development
  - WSL2 integration for Linux-based tools and cross-compilation testing
  - Volume mounting for hot-reload development workflows
  - Network isolation for testing different deployment scenarios

- **Development Container Configuration** (.devcontainer/):
```json
{
  "name": "BackTestr AI Development",
  "image": "mcr.microsoft.com/windows/servercore:ltsc2022",
  "features": {
    "rust": "latest",
    "node": "20",
    "python": "3.11",
    "git": "latest"
  },
  "extensions": [
    "rust-lang.rust-analyzer",
    "ms-python.python",
    "bradlc.vscode-tailwindcss",
    "ms-vscode.vscode-typescript-next"
  ],
  "postCreateCommand": "pnpm install && cargo build",
  "mounts": [
    "source=${localWorkspaceFolder}/target,target=/workspace/target,type=bind",
    "source=backtestr-cargo-cache,target=/usr/local/cargo/registry,type=volume"
  ],
  "containerEnv": {
    "RUST_LOG": "debug",
    "NODE_ENV": "development"
  }
}
```

- **WSL2 Development Workflow**:
  - **Primary**: Windows native development with MSVC toolchain
  - **Secondary**: WSL2 Ubuntu for Linux toolchain testing and verification
  - **CI/CD**: Container-based testing that mirrors production Windows environments
  - **Debugging**: Cross-platform debugging setup for multi-environment testing

- **Windows-Specific Optimizations**:
  - Windows Defender exclusions configured automatically
  - Windows Terminal integration with PowerShell and WSL2
  - Windows SDK and MSVC Build Tools containerized installation
  - Named Pipes development and testing in Windows containers

## Infrastructure & Deployment

### Desktop Application Packaging
- **Windows-Only Distribution**: Focused native Windows packaging
  - Windows: NSIS installer with registry integration and start menu shortcuts
  - MSI packages for enterprise deployment and Group Policy management
  - Windows Store (MSIX) packaging for modern deployment scenarios
  - Automatic Windows version detection and feature compatibility

- **Code Signing and Security**: Windows trust establishment for enterprise deployment
  - Extended Validation (EV) certificates for Windows SmartScreen compatibility
  - Authenticode signing for all executable components
  - Windows Defender exclusion recommendations for performance
  - Checksum verification for download integrity validation

### Update Management
- **Electron Auto-Updater**: Seamless application updates
  - Delta updates to minimize download size for frequent releases
  - Staged rollout capability for gradual deployment and issue detection
  - Rollback functionality for quick recovery from problematic updates
  - User notification system with optional/mandatory update controls

- **Update Server Infrastructure**: Self-hosted update distribution
  - Static file hosting with CDN distribution for global availability
  - Version manifest with digital signatures for security verification
  - Analytics integration for update adoption tracking and issue detection
  - Bandwidth optimization with compressed delta packages

### Performance Monitoring and Analytics
- **Application Performance Monitoring**: Real-time performance tracking
  - Custom metrics for tick processing throughput and MTF state update latency
  - Memory usage monitoring for leak detection and optimization
  - Crash reporting with stack traces and system information
  - Performance regression detection across application versions

- **User Analytics**: Usage patterns and feature adoption
  - Privacy-first analytics with local data aggregation
  - Feature usage tracking for development prioritization
  - Error rate monitoring for quality assurance
  - Performance benchmarks across different hardware configurations

### Backup and Data Management
- **Local Data Backup**: User data protection and recovery
  - Automatic backup of user algorithms and test configurations
  - Incremental backup system for efficient storage utilization
  - Cross-platform backup location standardization
  - Import/export functionality for data portability

- **Configuration Management**: Settings and preferences synchronization
  - JSON-based configuration with schema validation
  - Migration scripts for configuration format updates
  - Default configuration templates for new installations
  - Environment-specific configuration overrides for development vs. production

This technology stack provides a solid foundation for building a high-performance, professional-grade financial backtesting application while maintaining developer productivity and code quality throughout the development lifecycle.
