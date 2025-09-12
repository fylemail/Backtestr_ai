# Codebase Organization Standards

## Overview
This document defines the organizational standards for the BackTestr_ai codebase to maintain clarity and consistency as the project grows.

## Directory Structure

### Core Application Code

#### Rust Workspace (`/crates/`)
Each crate follows Rust conventions with clear separation of concerns:
```
crates/
├── backtestr-core/     # Core MTF engine and state management
│   ├── src/
│   │   ├── lib.rs      # Library entry point
│   │   ├── engine/     # MTF state engine modules
│   │   ├── models/     # Domain models
│   │   └── utils/      # Utility functions
│   └── Cargo.toml
├── backtestr-data/     # DuckDB integration and data management
│   ├── src/
│   │   ├── lib.rs
│   │   ├── storage/    # DuckDB interfaces
│   │   ├── models/     # Data models
│   │   └── loaders/    # Data loading utilities
│   └── Cargo.toml
└── backtestr-ipc/      # Inter-process communication
    ├── src/
    │   ├── lib.rs
    │   ├── messages/    # Message definitions
    │   └── transport/   # IPC transport layer
    └── Cargo.toml
```

#### Main Application (`/src/`)
The main Rust application that orchestrates all crates:
```
src/
├── main.rs             # Application entry point
├── config.rs           # Configuration management
└── cli.rs              # Command-line interface
```

#### Python Algorithms (`/algorithms/`)
Python trading algorithms with clear structure:
```
algorithms/
├── __init__.py         # Package initialization
├── base.py             # Base algorithm class
├── indicators/         # Technical indicators
├── strategies/         # Trading strategies
├── user/              # User-defined algorithms
└── tests/             # Algorithm tests
```

#### Electron UI (`/electron/`)
Frontend application following React/TypeScript conventions:
```
electron/
├── src/
│   ├── main/          # Electron main process
│   ├── renderer/      # React application
│   │   ├── components/ # React components
│   │   ├── hooks/     # Custom hooks
│   │   ├── stores/    # Zustand stores
│   │   └── utils/     # Utilities
│   └── preload/       # Preload scripts
├── package.json
└── tsconfig.json
```

### Documentation (`/docs/`)

Documentation follows BMad Method with clear separation:
```
docs/
├── architecture/       # Sharded architecture documents
│   └── *.md           # Architecture components
├── prd/               # Product requirements
│   ├── epic-*.md      # Epic definitions
│   └── epic-list.md   # Epic index
├── stories/           # User stories by epic
│   ├── epic-1/        # Epic 1 stories
│   │   └── story-*.md # Individual stories
│   └── README.md      # Story index
├── qa/                # Quality assurance
│   └── gates/         # Story gate files
└── development/       # Development guides
    ├── git-strategy.md
    └── codebase-organization.md
```

## File Naming Conventions

### Rust Files
- Use snake_case: `mtf_engine.rs`, `tick_processor.rs`
- Test modules: `#[cfg(test)] mod tests` in same file
- Benchmarks: `benches/` directory with `*_bench.rs`

### Python Files
- Use snake_case: `base_algorithm.py`, `sma_indicator.py`
- Test files: `test_*.py` in `tests/` directory
- Type stubs: `*.pyi` for external type hints

### TypeScript/JavaScript
- Components: PascalCase: `ChartPanel.tsx`, `TradeList.tsx`
- Utilities: camelCase: `formatCurrency.ts`, `parseTickData.ts`
- Types: `types/` directory with `*.types.ts`
- Tests: `*.test.ts` or `*.spec.ts`

### Documentation
- Epics: `epic-{n}-{kebab-case-name}.md`
- Stories: `story-{n.m}.md` within `epic-{n}/` directory
- Gates: `story-{n.m}-gate.yml`
- Guides: `{kebab-case-name}.md`

## Code Organization Rules

### 1. No Placeholder Files
- Only create files when implementing actual functionality
- Empty directories are fine, empty files are not
- Use `.gitkeep` only when absolutely necessary

### 2. Epic-Based Development
- Code for each epic should be clearly traceable
- Feature flags control epic activation
- No mixing of epic functionality

### 3. Test Organization
- Unit tests live with code (Rust: same file, Python/TS: tests/ directory)
- Integration tests in separate `tests/` at project root
- E2E tests in `e2e/` directory

### 4. Configuration Management
- Environment variables: `.env.*` files
- Application config: `config/` directory
- Feature flags: Centralized in features module

### 5. Data Files
```
data/
├── samples/           # Sample tick data for testing
├── fixtures/          # Test fixtures
└── cache/            # DuckDB cache (gitignored)
```

## Import Organization

### Rust
```rust
// 1. External crates
use tokio::runtime::Runtime;
use duckdb::Connection;

// 2. Workspace crates
use backtestr_core::engine::MTFEngine;
use backtestr_data::models::Tick;

// 3. Local modules
use crate::config::Config;
use crate::utils::helpers;
```

### Python
```python
# 1. Standard library
import os
from typing import List, Optional

# 2. Third-party
import numpy as np
import pandas as pd

# 3. Local packages
from algorithms.base import BaseAlgorithm
from algorithms.indicators import SMA
```

### TypeScript
```typescript
// 1. External packages
import React from 'react';
import { create } from 'zustand';

// 2. Internal packages
import { ChartPanel } from '@/components/ChartPanel';
import { useTicks } from '@/hooks/useTicks';

// 3. Types
import type { Tick, Bar } from '@/types';
```

## Version Control Patterns

### Branch Organization
- Stories: `story/STORY-{n.m}-{description}`
- Epics: `epic/epic-{n}-{name}`
- Bugfixes: `bugfix/{issue-id}-{description}`
- Hotfixes: `hotfix/{critical-issue}`

### Commit Message Structure
```
type(scope): subject

Story: {n.m}
Epic: {n}
```

## Build Artifacts

### Ignored Patterns
```
# Build outputs
target/          # Rust
dist/           # Electron
*.pyc           # Python
node_modules/   # Node.js

# IDE
.idea/
.vscode/
*.swp

# Data
data/cache/
*.db
```

## Migration Path

When adding new features:
1. Determine epic ownership
2. Create feature flag if incomplete
3. Follow directory structure above
4. Update this document if patterns change

## Enforcement

- CI checks enforce structure via `scripts/validate_structure.py`
- Pre-commit hooks validate file naming
- PR templates require structure compliance confirmation

---

*Last Updated: 2025-01-13*
*Version: 1.0.0*
*Status: Active*