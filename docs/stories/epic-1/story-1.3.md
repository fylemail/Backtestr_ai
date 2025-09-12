# Story 1.3: Simple CSV Tick Import

## Overview
Implement basic CSV file import for tick data, focusing on simplicity and correctness over performance optimization.

## Story Details
- **Epic**: 1 - Foundation & Core Data Pipeline
- **Type**: Feature
- **Priority**: P0 (Critical Path)
- **Size**: S (2-3 days)
- **Dependencies**: Story 1.2 (Basic DuckDB Integration)

## Progressive Development Context
This story implements ONLY basic CSV parsing and import. Advanced features like binary formats, real-time feeds, and progress reporting are deferred to Epic 2.

## Acceptance Criteria

### 1. CSV Parser Implementation
- [ ] Parse standard CSV format with headers
- [ ] Support these required fields:
  - symbol (string)
  - timestamp (ISO 8601 or Unix timestamp)
  - bid (float)
  - ask (float)
- [ ] Support these optional fields:
  - bid_size (integer)
  - ask_size (integer)

### 2. File Reading
- [ ] Read CSV files from filesystem
- [ ] Handle files up to 100MB
- [ ] Basic validation of file existence and readability

### 3. Data Validation
- [ ] Validate required fields are present
- [ ] Validate bid/ask are positive numbers
- [ ] Validate timestamps are parseable
- [ ] Skip invalid rows with error logging

### 4. Database Import
- [ ] Batch insert into DuckDB (1000 rows per batch)
- [ ] Handle duplicate timestamps (skip or update)
- [ ] Basic transaction support (all-or-nothing per file)

### 5. Error Handling
- [ ] Report parsing errors with line numbers
- [ ] Continue on individual row errors
- [ ] Fail gracefully on critical errors
- [ ] Return summary (rows processed, errors, inserted)

## Non-Goals (Deferred to Later Epics)

### Deferred to Epic 2
- ❌ Binary format support (Parquet, Arrow)
- ❌ Streaming large files (>1GB)
- ❌ Progress reporting/callbacks
- ❌ Parallel processing
- ❌ Data normalization
- ❌ Real-time feed ingestion
- ❌ Complex validation rules
- ❌ Data quality scoring

## Technical Approach

### 1. CSV Format Example
```csv
symbol,timestamp,bid,ask,bid_size,ask_size
EURUSD,2024-01-01T00:00:00Z,1.0921,1.0923,1000000,1000000
EURUSD,2024-01-01T00:00:01Z,1.0922,1.0924,500000,750000
```

### 2. Module Structure
```
crates/backtestr-data/src/
├── import/
│   ├── mod.rs
│   ├── csv.rs       # CSV parsing logic
│   └── validator.rs # Basic validation
```

### 3. Simple API Design
```rust
pub struct CsvImporter {
    database: Database,
}

impl CsvImporter {
    pub fn new(database: Database) -> Self
    pub fn import_file(&self, path: &Path) -> Result<ImportSummary>
}

pub struct ImportSummary {
    pub total_rows: usize,
    pub imported: usize,
    pub errors: Vec<ImportError>,
}
```

## Testing Requirements

### Unit Tests
- [ ] CSV parsing with valid data
- [ ] CSV parsing with invalid rows
- [ ] Timestamp parsing (ISO 8601 and Unix)
- [ ] Validation logic

### Integration Tests
- [ ] Import small CSV file (100 rows)
- [ ] Import medium CSV file (10K rows)
- [ ] Handle malformed CSV
- [ ] Handle duplicate timestamps

### Test Data
- [ ] Create sample CSV files in `test-data/` directory
- [ ] Include valid and invalid examples

## Definition of Done
- [ ] All acceptance criteria met
- [ ] Unit tests passing (>80% coverage)
- [ ] Integration tests passing
- [ ] CSV import working end-to-end
- [ ] Code reviewed and approved
- [ ] Documentation updated
- [ ] No clippy warnings
- [ ] Merged to develop branch

## Performance Targets
- Import rate: >10K ticks/second
- Memory usage: <50MB for 100K row file
- File size limit: 100MB (for Epic 1)

## Story Points: 3
*Estimation based on: CSV parsing (1) + validation (1) + import logic (1)*

## Start Checklist
- [ ] Story 1.2 complete and merged
- [ ] DuckDB integration working
- [ ] Test data prepared
- [ ] Create story branch: `story/STORY-1.3-simple-csv-import`

## Completion Checklist
- [ ] All tests passing
- [ ] Sample imports successful
- [ ] Performance targets met
- [ ] Code review approved
- [ ] PR merged to develop
- [ ] Story marked as complete

---
*Story Status: Ready to Start*
*Last Updated: Current Session*