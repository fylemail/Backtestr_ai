# Story 1.5: Basic Performance Validation

## Overview
Validate that the Epic 1 foundation meets basic performance targets through benchmarking and optimization of critical paths.

## Story Details
- **Epic**: 1 - Foundation & Core Data Pipeline
- **Type**: Technical Validation
- **Priority**: P0 (Critical Path)
- **Size**: S (2-3 days)
- **Dependencies**: Story 1.4 (Basic CLI)

## Progressive Development Context
This story validates that our foundation is solid before moving to Epic 2. We focus on meeting basic targets, not achieving optimal performance.

## Acceptance Criteria

### 1. Performance Benchmarks
- [ ] Create benchmark suite using Criterion.rs
- [ ] Measure tick insertion performance
- [ ] Measure query performance
- [ ] Measure memory usage
- [ ] Document baseline metrics

### 2. Performance Targets (from CLAUDE.md)
- [ ] Tick ingestion: ≥10K ticks/second
- [ ] Memory usage: <500MB for 1M ticks
- [ ] Query response: <100ms for basic queries
- [ ] CSV import: >10K rows/second

### 3. Critical Path Optimization
- [ ] Profile tick insertion path
- [ ] Profile query execution path
- [ ] Identify and fix any obvious bottlenecks
- [ ] Ensure batch operations are used effectively

### 4. Memory Management
- [ ] Validate no memory leaks
- [ ] Measure memory per tick
- [ ] Ensure efficient batch processing
- [ ] Document memory characteristics

### 5. Benchmark Report
- [ ] Create performance baseline document
- [ ] Include all benchmark results
- [ ] Document test methodology
- [ ] Note areas for future optimization (Epic 2)

## Non-Goals (Deferred to Later Epics)

### Deferred to Epic 2
- ❌ Advanced query optimization
- ❌ Database partitioning
- ❌ Compression strategies
- ❌ Parallel processing
- ❌ Cache optimization
- ❌ Index tuning

### Deferred to Epic 3
- ❌ Multi-timeframe performance
- ❌ State management optimization

## Technical Approach

### 1. Benchmark Structure
```
benches/
├── tick_insertion.rs
├── query_performance.rs
├── csv_import.rs
└── memory_usage.rs
```

### 2. Criterion Setup
```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "tick_insertion"
harness = false
```

### 3. Benchmark Examples
```rust
// Tick insertion benchmark
fn bench_tick_insertion(c: &mut Criterion) {
    let db = Database::new_memory().unwrap();
    let ticks = generate_ticks(10_000);
    
    c.bench_function("insert_10k_ticks", |b| {
        b.iter(|| {
            db.insert_ticks(&ticks).unwrap();
        });
    });
}

// Query benchmark
fn bench_query(c: &mut Criterion) {
    let db = setup_test_db_with_data();
    
    c.bench_function("query_date_range", |b| {
        b.iter(|| {
            db.query_ticks(
                "EURUSD",
                start_time,
                end_time
            ).unwrap();
        });
    });
}
```

### 4. Memory Profiling
```rust
// Use heaptrack or valgrind for memory profiling
#[test]
fn test_memory_usage() {
    let db = Database::new_memory().unwrap();
    let initial = get_memory_usage();
    
    // Insert 1M ticks
    for batch in tick_batches {
        db.insert_ticks(&batch).unwrap();
    }
    
    let final_mem = get_memory_usage();
    let used = final_mem - initial;
    
    assert!(used < 500_000_000); // <500MB
}
```

### 5. Performance Report Template
```markdown
# Epic 1 Performance Baseline

## Test Environment
- OS: [specify]
- CPU: [specify]
- RAM: [specify]
- Rust version: [specify]

## Results Summary
| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Tick Insert | 10K/s | [actual] | PASS/FAIL |
| Memory (1M ticks) | <500MB | [actual] | PASS/FAIL |
| Query Response | <100ms | [actual] | PASS/FAIL |

## Detailed Results
[Include Criterion HTML reports]

## Optimization Notes
[Document any optimizations made]

## Future Improvements (Epic 2)
[List identified optimization opportunities]
```

## Testing Requirements

### Benchmark Tests
- [ ] Tick insertion (single and batch)
- [ ] Query by symbol and date range
- [ ] CSV import of various sizes
- [ ] Memory usage under load

### Stress Tests
- [ ] Insert 1M ticks
- [ ] Query across large date ranges
- [ ] Import 100MB CSV file

### Profiling
- [ ] CPU profiling with flamegraph
- [ ] Memory profiling with heaptrack
- [ ] I/O profiling for database operations

## Simple Optimizations (If Needed)

Only implement if targets aren't met:
- [ ] Increase batch sizes
- [ ] Add simple prepared statements
- [ ] Tune DuckDB buffer size
- [ ] Pre-allocate collections

## Definition of Done
- [ ] All performance targets met
- [ ] Benchmark suite complete
- [ ] Performance report written
- [ ] No memory leaks detected
- [ ] Code reviewed and approved
- [ ] Documentation updated
- [ ] Merged to develop branch

## Risk Mitigation

If targets aren't met:
1. Document current performance
2. Identify specific bottlenecks
3. Create Epic 2 stories for optimization
4. Proceed if "good enough" for foundation

## Story Points: 3
*Estimation based on: benchmarks (1) + profiling (1) + optimization (1)*

## Start Checklist
- [ ] Stories 1.1-1.4 complete
- [ ] Test data generated
- [ ] Benchmark environment prepared
- [ ] Create story branch: `story/STORY-1.5-performance-validation`

## Completion Checklist
- [ ] All benchmarks run
- [ ] Performance targets validated
- [ ] Report documented
- [ ] Any critical issues fixed
- [ ] Code review approved
- [ ] PR merged to develop
- [ ] Epic 1 complete!

---
*Story Status: Ready to Start*
*Last Updated: Current Session*