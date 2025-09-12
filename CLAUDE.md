# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

BackTestr_ai is a revolutionary multi-timeframe forex backtesting platform that solves the critical validation gap in algorithmic trading. The project uses a hybrid Rust/Python architecture with Electron UI for high-performance tick processing and visualization.

## Development Commands

### Initial Setup (when implemented)
```bash
# Install Rust dependencies
cargo build

# Install Node.js dependencies for Electron UI
cd electron && npm install

# Install Python dependencies for algorithm runtime
pip install -r requirements.txt
```

### Running Tests
```bash
# Run Rust tests
cargo test

# Run Rust benchmarks
cargo bench

# Run Python algorithm tests
pytest algorithms/tests/
```

### Code Quality
```bash
# Format Rust code
cargo fmt

# Lint Rust code
cargo clippy

# Check TypeScript types
cd electron && npm run typecheck

# Run Python linter
ruff check algorithms/
```

## Architecture Overview

The system uses a **hybrid process architecture**:

1. **Main Process (Rust + Embedded Python)**
   - MTF (Multi-Timeframe) State Engine maintains synchronized bar states across 6 timeframes
   - Python algorithms embedded via PyO3 for zero-copy data access
   - DuckDB for columnar tick data storage with 10-20x compression
   - Processes up to 1M ticks/second with sub-100μs state updates

2. **UI Process (Electron + React + TypeScript)**
   - 6-panel synchronized chart visualization using Lightweight Charts
   - Zustand for state management
   - Tailwind CSS for styling
   - MessagePack IPC for high-performance data exchange

## Key Development Patterns

### Multi-Crate Rust Workspace
The project uses separate crates for modularity:
- `backtestr-core/` - Core MTF engine and tick processing
- `backtestr-data/` - DuckDB integration and data management
- `backtestr-ipc/` - Inter-process communication layer

### BMad Method Integration
The project follows the BMad Method for AI-driven development:
- Sharded documentation in `docs/prd/` and `docs/architecture/`
- Configuration in `.bmad-core/core-config.yaml`
- Use BMad slash commands prefixed with `/BMad` for specialized workflows

### Coding Standards
- **Rust**: Use `rustfmt`, no `unwrap()` in production, 80% test coverage minimum
- **Python**: Complete type hints, use `@dataclass`, profile with `cProfile`
- **TypeScript**: Strict mode, no `any` types, functional React components only
- **All languages**: Document performance characteristics for critical paths

## Project Structure

```
backtestr_ai/
├── src/                    # Main Rust application
├── crates/                 # Rust workspace crates
├── electron/               # Electron/React UI
├── algorithms/             # Python trading algorithms
├── data/                   # Data storage (DuckDB)
├── docs/                   # Sharded documentation
└── .bmad-core/            # BMad framework files
```

## Current Status

- **Phase**: Pre-development (documentation complete)
- **Next Step**: Epic 1 - Foundation & Core Data Pipeline
- **Main Branch**: `master`

## Performance Targets

- Process 1M+ ticks per second
- Sub-100μs MTF state updates
- 60 FPS chart rendering
- 95%+ correlation between backtest and live results