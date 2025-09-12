# Epic 1: Foundation & Core Data Pipeline (Progressive)

**Goal:** Establish a minimal, working foundation using progressive development principles. Focus on Rust core engine and basic data pipeline only. No Python, no frontend, no advanced features.

**Progressive Development Context:** This epic has been restructured following CLAUDE.md principles. Only includes what's absolutely necessary for a working foundation. Advanced features are properly deferred to later epics.

## Epic Overview

### What We're Building
- ✅ Rust workspace structure
- ✅ Basic DuckDB integration  
- ✅ Simple CSV import
- ✅ Basic CLI for queries
- ✅ Performance validation

### What We're NOT Building (Deferred)
- ❌ Python integration → Epic 4
- ❌ Frontend/Electron → Epic 5
- ❌ IPC architecture → Epic 5
- ❌ Advanced credentials → Epic 2
- ❌ MTF state engine → Epic 3
- ❌ Service architecture → Epic 3
- ❌ Complex optimizations → Epic 2

## Stories

## Story 1.1: Project Infrastructure Setup ✅ COMPLETE

**As a** developer,  
**I want** a Rust-focused monorepo with basic tooling,  
**So that** I can build the core engine without complexity.

### Acceptance Criteria (COMPLETED)
1. ✅ Rust workspace structure created (crates/)
2. ✅ Cargo workspace configured with shared dependencies
3. ✅ Basic environment configuration (.env support)
4. ✅ Simple CI/CD pipeline (Rust only)
5. ✅ Git repository with proper .gitignore
6. ✅ Development scripts (cargo build, test, clippy)
7. ✅ README.md with setup instructions
8. ✅ Rust toolchain installed and working

**Status:** Merged to develop (commit f5d28a6)

---

## Story 1.2: Basic DuckDB Integration 📝 NEXT

**As a** developer,  
**I want** basic DuckDB embedded database integration,  
**So that** I can store and query tick data locally.

### Acceptance Criteria
1. [ ] Resolve DuckDB arrow-arith compatibility issue
2. [ ] Add DuckDB dependency to workspace
3. [ ] Create simple tick table schema (symbol, timestamp, bid, ask)
4. [ ] Implement basic CRUD operations
5. [ ] Support in-memory and file-based databases
6. [ ] Add unit and integration tests

### Technical Details
- Use `duckdb = { version = "0.9", features = ["bundled"] }`
- Simple schema only - no partitioning or optimization
- Basic connection management - no pooling needed yet

**Dependencies:** Story 1.1  
**Branch:** `story/STORY-1.2-basic-duckdb-integration`

---

## Story 1.3: Simple CSV Tick Import 📝 PLANNED

**As a** developer,  
**I want** basic CSV import functionality,  
**So that** I can load historical tick data.

### Acceptance Criteria
1. [ ] Parse standard CSV format (symbol, timestamp, bid, ask)
2. [ ] Validate required fields present
3. [ ] Batch insert into DuckDB (1000 rows/batch)
4. [ ] Handle invalid rows with error logging
5. [ ] Return import summary (processed, errors, inserted)
6. [ ] Create sample test data files

### Technical Details
- Support CSV files up to 100MB
- Basic validation only
- No progress reporting or streaming

**Dependencies:** Story 1.2  
**Branch:** `story/STORY-1.3-simple-csv-import`

---

## Story 1.4: Basic CLI Query Interface 📝 PLANNED

**As a** user,  
**I want** a simple command-line interface,  
**So that** I can import and query tick data.

### Acceptance Criteria
1. [ ] Create `backtestr` CLI binary
2. [ ] Implement `import` command for CSV files
3. [ ] Implement `query` command with symbol/date filters
4. [ ] Implement `stats` command for database overview
5. [ ] Implement `delete` command for data cleanup
6. [ ] Add table, CSV, and JSON output formats

### Commands
```bash
backtestr import --file data.csv
backtestr query --symbol EURUSD --from 2024-01-01 --to 2024-01-02
backtestr stats
backtestr delete --symbol EURUSD --confirm
```

**Dependencies:** Story 1.3  
**Branch:** `story/STORY-1.4-basic-cli-queries`

---

## Story 1.5: Basic Performance Validation 📝 PLANNED

**As a** developer,  
**I want** to validate basic performance targets,  
**So that** I know the foundation is solid.

### Acceptance Criteria
1. [ ] Create Criterion benchmark suite
2. [ ] Validate tick insertion: ≥10K ticks/second
3. [ ] Validate memory usage: <500MB for 1M ticks
4. [ ] Validate query response: <100ms for basic queries
5. [ ] Document performance baseline
6. [ ] Identify optimization opportunities for Epic 2

### Technical Details
- Use Criterion.rs for benchmarking
- Basic profiling only
- Document but don't optimize unless targets missed

**Dependencies:** Story 1.4  
**Branch:** `story/STORY-1.5-performance-validation`

---

## Epic Completion Criteria

### Must Have
- [x] Rust workspace building and tested
- [ ] DuckDB storing and querying ticks
- [ ] CSV import working
- [ ] CLI providing basic access
- [ ] Performance targets met

### Nice to Have (But Not Required)
- [ ] Better error messages
- [ ] More CSV formats
- [ ] Additional CLI commands

### Explicitly Excluded
- No Python code
- No JavaScript/TypeScript
- No IPC/messaging
- No complex schemas
- No optimization beyond basics
- No real-time data
- No MTF processing
- No service architecture

## Performance Targets

| Metric | Target | Priority |
|--------|--------|----------|
| Tick insertion | ≥10K/second | Required |
| Memory (1M ticks) | <500MB | Required |
| Query response | <100ms | Required |
| CSV import | >10K rows/second | Nice to have |

## Risk Mitigation

### Known Issues
1. **DuckDB compatibility** - Arrow-arith version conflict needs resolution
2. **Scope creep** - Strong temptation to add "just one more feature"
3. **Documentation drift** - Multiple docs with conflicting information

### Mitigation Strategy
1. Fix DuckDB issue before starting Story 1.2
2. Strict adherence to acceptance criteria
3. CLAUDE.md as single source of truth

## Success Metrics

### Epic Success
- [ ] All 5 stories complete
- [ ] Performance targets met
- [ ] Zero dependencies on future epics
- [ ] Clean, maintainable codebase
- [ ] Clear path to Epic 2

### Warning Signs
- Adding Python "just for testing"
- Creating IPC "for future use"
- Complex optimization attempts
- Feature additions beyond scope

## Notes for Developers

### Do's
- Keep it simple
- Focus on correctness
- Test thoroughly
- Document clearly
- Ask when unsure

### Don'ts
- Don't add Python
- Don't add frontend code
- Don't over-engineer
- Don't optimize prematurely
- Don't add unused features

## Related Documentation
- [CLAUDE.md](../../CLAUDE.md) - Progressive development guidelines
- [Feature Preservation Matrix](../development/feature-preservation-matrix.md) - Where features moved
- [Progressive Development Audit](../development/progressive-development-audit.md) - Why we restructured
- Individual story files in `docs/stories/epic-1/`

---

**Epic Status:** IN PROGRESS  
**Current Story:** 1.2 - Basic DuckDB Integration  
**Last Updated:** Current Session  
**Epic Owner:** Development Team