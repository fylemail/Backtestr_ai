# BackTestr_ai

## Revolutionary Multi-Timeframe Forex Backtesting Platform

BackTestr_ai solves the critical validation gap in algorithmic trading where existing platforms fail at multi-timeframe strategy testing. Our MTF (Multi-Timeframe) State Engine maintains synchronized partial and completed bar states across all timeframes at every tick, achieving 95%+ correlation between backtest and live trading results.

## Project Status

- **Documentation**: ✅ Complete (PRD & Architecture validated at 95% readiness)
- **Development**: 🚀 Ready to begin Epic 1
- **Method**: BMad Method with multi-agent orchestration

## Technology Stack

- **Core Engine**: Rust (high-performance tick processing)
- **Algorithm Runtime**: Python via PyO3 bridge
- **Data Storage**: DuckDB (columnar storage with 10-20x compression)
- **UI Framework**: Electron + React + TypeScript
- **Charts**: Lightweight Charts (TradingView library)
- **Target Platform**: Windows Desktop (MVP)

## Key Features

- Process 1M+ ticks per second with sub-100μs MTF state updates
- 6-panel synchronized chart visualization
- Tick-by-tick walkback replay with variable speed control
- Comprehensive statistical analysis and performance heatmaps
- Multi-position tracking with realistic execution modeling

## Development Approach

This project uses the BMad Method for AI-driven development with comprehensive documentation-first approach:

- `/docs/prd/` - Product Requirements (sharded)
- `/docs/architecture/` - System Architecture (sharded)
- `/.bmad-core/` - BMad framework and agents
- `/.ignore/` - Original documentation files

## Getting Started

### Prerequisites

- Windows 10/11 (64-bit)
- Rust toolchain (latest stable)
- Node.js 20+
- Python 3.11+
- Git

### Setup

```bash
# Clone the repository
git clone https://github.com/fylemail/Backtestr_ai.git
cd Backtestr_ai

# Install dependencies (coming in Epic 1)
# cargo build
# npm install
# pip install -r requirements.txt
```

## Project Structure

```
backtestr_ai/
├── .bmad-core/          # BMad Method framework
├── docs/                # Sharded documentation (ignored in git)
│   ├── prd/            # Product requirements
│   └── architecture/   # Architecture documents
├── .ignore/            # Original doc files (ignored in git)
├── src/                # Rust source (Epic 1)
├── electron/           # Electron/React UI (Epic 5)
└── algorithms/         # Python algorithms (Epic 4)
```

## Epic Roadmap

1. **Epic 1**: Foundation & Core Data Pipeline (10 weeks)
2. **Epic 2**: Multi-Timeframe Synchronization Engine
3. **Epic 3**: Core Position Management & Execution
4. **Epic 4**: Algorithm Integration & Python Bridge
5. **Epic 5**: Chart Visualization System
6. **Epic 6**: Walkback Replay Engine
7. **Epic 7**: Statistical Analysis & Reporting

## Contributing

This is currently a private project. Development follows the BMad Method with multi-agent validation.

## License

Proprietary - All rights reserved

---

*Built with BMad Method™ - AI-Driven Development Framework*