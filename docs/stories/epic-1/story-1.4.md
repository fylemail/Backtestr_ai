# Story 1.4: Basic CLI Query Interface

## Overview
Create a simple command-line interface for querying tick data, providing basic data access without any IPC or frontend complexity.

## Story Details
- **Epic**: 1 - Foundation & Core Data Pipeline
- **Type**: Feature
- **Priority**: P0 (Critical Path)
- **Size**: S (2-3 days)
- **Dependencies**: Story 1.3 (Simple CSV Import)

## Progressive Development Context
This story implements ONLY a basic CLI using Rust's clap crate. No IPC, no frontend integration, no complex analytics - just simple data queries.

## Acceptance Criteria

### 1. CLI Command Structure
- [ ] Main binary: `backtestr`
- [ ] Subcommands:
  - `import` - Import CSV file
  - `query` - Query tick data
  - `stats` - Basic statistics
  - `delete` - Delete data

### 2. Import Command
```bash
backtestr import --file path/to/data.csv
# Output: Imported 10000 ticks for EURUSD
```
- [ ] Accept file path argument
- [ ] Display import summary
- [ ] Show errors if any

### 3. Query Command
```bash
backtestr query --symbol EURUSD --from 2024-01-01 --to 2024-01-02
# Output: Tabular display of ticks
```
- [ ] Filter by symbol (required)
- [ ] Filter by date range (optional)
- [ ] Limit results (--limit flag)
- [ ] Output formats: table (default), csv, json

### 4. Stats Command
```bash
backtestr stats
# Output: Database statistics
```
- [ ] Total tick count
- [ ] Symbols in database
- [ ] Date range per symbol
- [ ] Database file size

### 5. Delete Command
```bash
backtestr delete --symbol EURUSD --confirm
```
- [ ] Delete by symbol
- [ ] Delete by date range
- [ ] Require confirmation flag

### 6. Global Options
- [ ] `--db` flag for database path (default: ./backtest.sqlite)
- [ ] `--memory` flag for in-memory database
- [ ] `--verbose` flag for debug output

## Non-Goals (Deferred to Later Epics)

### Deferred to Epic 2
- ❌ Advanced query filters
- ❌ Data aggregation commands
- ❌ Export to multiple formats
- ❌ Streaming output

### Deferred to Epic 3
- ❌ Multi-timeframe queries
- ❌ State management commands

### Deferred to Epic 5
- ❌ IPC communication
- ❌ WebSocket server
- ❌ Frontend integration

## Technical Approach

### 1. Dependencies
```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
comfy-table = "7"  # For table output
serde_json = "1"   # For JSON output
```

### 2. CLI Structure
```rust
#[derive(Parser)]
struct Cli {
    #[arg(long, default_value = "./backtest.sqlite")]
    db: PathBuf,
    
    #[arg(long)]
    memory: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Import { file: PathBuf },
    Query { 
        symbol: String,
        from: Option<String>,
        to: Option<String>,
        #[arg(long, default_value = "100")]
        limit: usize,
    },
    Stats,
    Delete { symbol: String, confirm: bool },
}
```

### 3. Output Formatting
```
$ backtestr query --symbol EURUSD --limit 5
┌────────┬─────────────────────┬────────┬────────┐
│ Symbol │ Timestamp           │ Bid    │ Ask    │
├────────┼─────────────────────┼────────┼────────┤
│ EURUSD │ 2024-01-01 00:00:00 │ 1.0921 │ 1.0923 │
│ EURUSD │ 2024-01-01 00:00:01 │ 1.0922 │ 1.0924 │
└────────┴─────────────────────┴────────┴────────┘
```

## Testing Requirements

### Unit Tests
- [ ] CLI argument parsing
- [ ] Command validation
- [ ] Output formatting

### Integration Tests
- [ ] Full import -> query cycle
- [ ] All commands with various options
- [ ] Error handling for invalid inputs

### CLI Tests
- [ ] Test with real CSV files
- [ ] Test all output formats
- [ ] Test error conditions

## Definition of Done
- [ ] All acceptance criteria met
- [ ] All commands working end-to-end
- [ ] Help text clear and complete
- [ ] Tests passing
- [ ] Code reviewed and approved
- [ ] Documentation updated
- [ ] No clippy warnings
- [ ] Merged to develop branch

## User Documentation
```markdown
# Backtestr CLI Usage

## Import data
backtestr import --file data.csv

## Query ticks
backtestr query --symbol EURUSD --from 2024-01-01 --to 2024-01-02

## View statistics
backtestr stats

## Delete data
backtestr delete --symbol EURUSD --confirm
```

## Story Points: 3
*Estimation based on: CLI setup (1) + commands (1) + formatting (1)*

## Start Checklist
- [ ] Story 1.3 complete and merged
- [ ] Import functionality working
- [ ] Query functionality tested
- [ ] Create story branch: `story/STORY-1.4-basic-cli-queries`

## Completion Checklist
- [ ] All commands implemented
- [ ] Help text complete
- [ ] Integration tests passing
- [ ] User can import and query data
- [ ] Code review approved
- [ ] PR merged to develop
- [ ] Story marked as complete

---
*Story Status: Ready to Start*
*Last Updated: Current Session*