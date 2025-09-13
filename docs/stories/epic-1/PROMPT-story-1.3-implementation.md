# Developer Agent Prompt: Implement Story 1.3 - Simple CSV Tick Import

## 🎯 Task Overview
Implement Story 1.3: Simple CSV Tick Import for Epic 1 (Foundation & Core Data Pipeline). This story adds basic CSV import functionality to our SQLite-based tick data storage system.

## 📋 Key Context
- **Current State**: Story 1.2 is complete with SQLite integration working
- **Branch Strategy**: Create `story/STORY-1.3-simple-csv-import` from `develop`
- **Focus**: BASIC CSV import only - no optimization or advanced features
- **Database**: SQLite only (DuckDB is deferred to Epic 2)

## ⚠️ Critical Constraints (MUST FOLLOW)

### Progressive Development Rules
1. **DO NOT** add Python code - deferred to Epic 4
2. **DO NOT** add frontend/UI code - deferred to Epic 5
3. **DO NOT** implement streaming for large files - deferred to Epic 2
4. **DO NOT** add progress reporting - deferred to Epic 2
5. **DO NOT** implement parallel processing - deferred to Epic 2
6. **DO NOT** add binary formats (Parquet/Arrow) - deferred to Epic 2

### What TO Build
✅ Simple CSV parser using Rust csv crate
✅ Basic validation (required fields, positive numbers)
✅ Batch inserts (1000 rows per batch)
✅ Error handling with line numbers
✅ Import summary reporting
✅ File size limit: 100MB maximum

## 🔧 Implementation Steps

### Step 1: Git Setup
```bash
# Start from develop branch
git checkout develop
git pull origin develop

# Create story branch
git checkout -b story/STORY-1.3-simple-csv-import

# Verify you're on correct branch
git branch --show-current
```

### Step 2: Add CSV Dependencies
Update `crates/backtestr-data/Cargo.toml`:
```toml
[dependencies]
# ... existing dependencies ...
csv = "1.3"  # For CSV parsing
```

### Step 3: Create Module Structure
Create these files in `crates/backtestr-data/src/`:
```
import/
├── mod.rs       # Module exports
├── csv.rs       # CSV parsing logic
└── validator.rs # Data validation
```

### Step 4: Implement CSV Parser
Key requirements:
- Parse standard CSV with headers
- Required fields: symbol, timestamp, bid, ask
- Optional fields: bid_size, ask_size
- Support ISO 8601 and Unix timestamps
- Skip invalid rows with error logging

### Step 5: Implement Database Import
- Use existing `Database` struct from Story 1.2
- Batch inserts (1000 rows per transaction)
- Handle duplicates (skip or update based on primary key)
- Return `ImportSummary` with statistics

### Step 6: Create Test Data
Create `test-data/` directory with sample CSV files:
- `valid_small.csv` (100 rows)
- `valid_medium.csv` (10,000 rows)
- `invalid_mixed.csv` (mix of valid/invalid rows)
- `malformed.csv` (broken CSV structure)

### Step 7: Write Tests
Required tests:
```rust
// Unit tests in csv.rs
#[test]
fn test_parse_valid_csv() { }
#[test]
fn test_parse_invalid_rows() { }
#[test]
fn test_timestamp_formats() { }

// Integration tests
#[test]
fn test_import_small_file() { }
#[test]
fn test_import_with_errors() { }
#[test]
fn test_duplicate_handling() { }
```

### Step 8: Performance Validation
Verify targets:
- Import rate: >10K ticks/second
- Memory usage: <50MB for 100K row file
- Use Criterion benchmarks if needed

### Step 9: Code Quality Checks
```bash
# Format code
cargo fmt

# Run clippy
cargo clippy --all-targets --all-features

# Run tests
cargo test --workspace

# Check no warnings
cargo build --release
```

### Step 10: Create PR
```bash
# Commit your changes
git add -A
git commit -m "feat(data): implement Story 1.3 - Simple CSV tick import

- Add CSV parsing with csv crate
- Implement basic validation for tick data
- Support batch inserts (1000 rows per batch)
- Add error handling with line numbers
- Create test data and comprehensive tests
- Meet performance targets (>10K ticks/sec)

Closes: Story 1.3"

# Push to remote
git push -u origin story/STORY-1.3-simple-csv-import

# Create PR via GitHub CLI (or manually on GitHub)
gh pr create --title "[Story 1.3] Simple CSV Tick Import" \
  --body "## Summary
- Implements basic CSV import for tick data
- Adds validation and error handling
- Includes comprehensive test coverage

## Testing
- [ ] Unit tests passing
- [ ] Integration tests passing
- [ ] Performance targets met
- [ ] No clippy warnings

## Checklist
- [ ] Follows Epic 1 constraints (no Python/frontend)
- [ ] Uses SQLite only
- [ ] Batch size: 1000 rows
- [ ] File size limit: 100MB" \
  --base develop
```

## 📝 Acceptance Criteria Checklist

### CSV Parser Implementation
- [ ] Parse standard CSV format with headers
- [ ] Support required fields: symbol, timestamp, bid, ask
- [ ] Support optional fields: bid_size, ask_size
- [ ] Handle both ISO 8601 and Unix timestamps

### File Reading
- [ ] Read CSV files from filesystem
- [ ] Handle files up to 100MB
- [ ] Validate file existence and readability

### Data Validation
- [ ] Validate required fields are present
- [ ] Validate bid/ask are positive numbers
- [ ] Validate timestamps are parseable
- [ ] Skip invalid rows with error logging

### Database Import
- [ ] Batch insert into SQLite (1000 rows per batch)
- [ ] Handle duplicate timestamps
- [ ] Basic transaction support

### Error Handling
- [ ] Report parsing errors with line numbers
- [ ] Continue on individual row errors
- [ ] Return summary with statistics

## 🚫 Common Mistakes to Avoid

1. **Over-engineering**: Keep it simple! No async, no channels, no parallelism
2. **Wrong dependencies**: Use `csv` crate, not Arrow/Parquet
3. **Performance optimization**: Don't optimize beyond basic batching
4. **Scope creep**: Don't add progress bars, callbacks, or UI
5. **Complex validation**: Keep validation basic (required fields, positive numbers)

## 📊 Expected Output Structure

```rust
pub struct ImportSummary {
    pub file_path: PathBuf,
    pub total_rows: usize,
    pub rows_imported: usize,
    pub rows_skipped: usize,
    pub errors: Vec<ImportError>,
    pub duration: Duration,
}

pub struct ImportError {
    pub line_number: usize,
    pub error_type: ErrorType,
    pub details: String,
}
```

## 🎯 Definition of Done

- [ ] All acceptance criteria met
- [ ] Unit tests passing with >80% coverage
- [ ] Integration tests passing
- [ ] Performance targets met (>10K ticks/sec)
- [ ] No clippy warnings
- [ ] Code reviewed and approved
- [ ] PR merged to develop branch
- [ ] Story marked complete in tracking

## 💡 Tips for Success

1. Start with the simplest implementation that works
2. Use existing `Database` and `Tick` structs from Story 1.2
3. Test with small files first, then scale up
4. Keep error messages clear and actionable
5. Document any assumptions in code comments

## 🔗 References

- Story 1.3 Spec: `docs/stories/epic-1/story-1.3.md`
- Git Strategy: `docs/development/git-strategy.md`
- Epic 1 Context: `docs/prd/epic-1-foundation-core-data-pipeline.md`
- CLAUDE.md: Progressive development guidelines

---

**Remember**: This is Epic 1 - keep it simple, make it work, avoid scope creep. We can optimize and add features in later epics!