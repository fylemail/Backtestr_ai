# Source Tree

## Project Structure Overview

BackTestr_ai follows a multi-crate Rust workspace architecture with embedded Electron frontend, organized for optimal development workflow and build performance:

```
backtestr_ai/
├── Cargo.toml                 # Workspace configuration
├── Cargo.lock                 # Dependency lock file
├── package.json               # Electron and Node.js dependencies
├── electron-builder.json      # Electron packaging configuration
├── .bmad-core/               # BMad development framework files
│   ├── core-config.yaml      # Project configuration
│   ├── agents/               # BMad agent definitions
│   ├── tasks/                # Reusable task workflows
│   └── templates/            # Document templates
│
├── src/                      # Main Rust application source
│   ├── main.rs               # Application entry point
│   ├── lib.rs                # Library definitions
│   └── modules/              # Core application modules
│
├── crates/                   # Rust workspace crates
│   ├── backtestr-core/       # Core engine (Rust)
│   │   ├── src/
│   │   │   ├── lib.rs        # Core library exports
│   │   │   ├── engine/       # MTF State Engine
│   │   │   ├── data/         # Data management
│   │   │   ├── positions/    # Position tracking
│   │   │   ├── indicators/   # Technical indicators
│   │   │   └── python/       # PyO3 bridge
│   │   └── Cargo.toml
│   │
│   ├── backtestr-data/       # DuckDB integration
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── storage/      # Database schemas
│   │   │   ├── query/        # Query optimization
│   │   │   └── migration/    # Schema migrations
│   │   └── Cargo.toml
│   │
│   └── backtestr-ipc/        # Inter-process communication
│       ├── src/
│       │   ├── lib.rs
│       │   ├── protocol/     # MessagePack protocol
│       │   ├── streaming/    # Real-time data streaming
│       │   └── handlers/     # Message handlers
│       └── Cargo.toml
│
├── electron/                 # Electron application
│   ├── main.js               # Electron main process
│   ├── preload.js            # Preload script for security
│   └── renderer/             # Renderer process (React frontend)
│       ├── src/
│       │   ├── index.tsx     # React application entry
│       │   ├── components/   # React components
│       │   │   ├── Charts/   # Chart visualization
│       │   │   ├── Controls/ # UI controls
│       │   │   ├── Analysis/ # Analysis dashboard
│       │   │   └── Layout/   # Layout components
│       │   ├── stores/       # Zustand state management
│       │   ├── hooks/        # React hooks
│       │   ├── utils/        # Frontend utilities
│       │   └── types/        # TypeScript definitions
│       ├── public/           # Static assets
│       ├── package.json      # Frontend dependencies
│       └── webpack.config.js # Build configuration
│
├── algorithms/               # User algorithm storage
│   ├── examples/             # Example algorithms
│   └── user/                 # User-created algorithms
│
├── data/                     # Data storage directory
│   ├── tick/                 # Raw tick data (DuckDB)
│   ├── bars/                 # Pre-computed OHLC bars
│   ├── results/              # Backtest results
│   └── cache/                # Performance cache
│
├── docs/                     # Documentation (BMad managed)
│   ├── prd/                  # Sharded PRD documents
│   ├── architecture/         # Sharded architecture documents
│   ├── stories/              # User stories
│   └── qa/                   # Quality assurance documents
│
├── target/                   # Rust build output
├── dist/                     # Electron build output
├── node_modules/             # Node.js dependencies
└── scripts/                  # Build and development scripts
    ├── build.sh              # Cross-platform build script
    ├── dev.sh                # Development server
    └── test.sh               # Test runner
```

## Key Directory Conventions

- **Multi-crate Architecture**: Each major component is a separate Rust crate for modularity and parallel compilation
- **Embedded Frontend**: React application lives within the Electron directory structure
- **Data Separation**: User data, algorithms, and results are kept separate from application code
- **BMad Integration**: `.bmad-core` directory contains all BMad framework files for AI-driven development
- **Cross-platform Scripts**: Shell scripts handle build processes across different environments
- **Workspace Optimization**: Cargo workspace enables efficient dependency sharing and compilation

## Build Artifacts Location

- **Rust Binaries**: `target/release/backtestr_ai.exe` (Windows)
- **Electron Package**: `dist/BackTestr_ai-1.0.0-win.exe` (installer)
- **Frontend Assets**: `electron/renderer/dist/` (webpack output)
- **Documentation**: Auto-generated in `docs/` via BMad workflows
