# Data Storage Architecture Strategy

## Epic 1 Implementation (Current)
- **Database**: SQLite only
- **Scope**: Basic tick storage for foundation
- **Performance**: Meets targets (10K ticks/sec, <500MB for 1M ticks)
- **Status**: ✅ Implemented in Story 1.2

## Future Context (Epic 2+)
- **Requirement**: Store and query 7 years of forex tick data (AUDUSD)
- **Volume**: ~2.3 billion ticks (~147GB uncompressed, ~30GB compressed)
- **Use Case**: Historical backtesting (read-heavy, write-once)
- **Challenge**: DuckDB compilation takes 20+ minutes in CI (deferred to Epic 2)

## Option 1: SQLite Development / DuckDB Production (Epic 2+)

### Architecture
```
Development Environment:
├── SQLite Database
│   └── Sample data (1-7 days, ~10M ticks)
│   └── Fast iteration, quick tests
│   └── CI/CD uses this

Production Environment:
├── DuckDB Database
│   └── Full 7 years data
│   └── Optimized for analytics
│   └── Columnar storage
```

### Pros
- ✅ Fast CI/CD (2 minutes vs 20+)
- ✅ Quick local development
- ✅ Simple architecture
- ✅ No complex abstractions

### Cons
- ❌ **Different SQL dialects** - Code that works in dev might fail in prod
- ❌ **Different performance characteristics** - Can't optimize queries properly
- ❌ **Limited testing** - Can't test with realistic data volumes
- ❌ **False confidence** - "Works on my machine" problem

### Risk Assessment
| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| SQL incompatibility | High | High | Use only common SQL subset |
| Performance regression | High | Medium | Benchmark critical queries |
| Type mismatches | Medium | High | Strict type mapping layer |
| Missing bugs | High | Medium | Production staging environment |

### Implementation Strategy
```rust
// Use trait to abstract database operations
trait TickDatabase {
    fn insert_tick(&self, tick: &Tick) -> Result<()>;
    fn query_range(&self, symbol: &str, start: DateTime, end: DateTime) -> Result<Vec<Tick>>;
}

#[cfg(feature = "development")]
type DatabaseImpl = SqliteDatabase;

#[cfg(not(feature = "development"))]
type DatabaseImpl = DuckDBDatabase;
```

### Verdict: ⚠️ **Risky but Workable**
- Requires discipline to use only common SQL features
- Need comprehensive integration tests
- Must test with DuckDB before releases

---

## Option 2: Three-Tier Storage Architecture

### Architecture
```
┌─────────────────────────────────────────┐
│ Tier 1: HOT (Last 7 days)               │
│ SQLite/DuckDB - Fast writes & queries   │ ← Real-time ingestion
└────────────────┬────────────────────────┘
                 ▼
┌─────────────────────────────────────────┐
│ Tier 2: WARM (7 days - 1 year)          │
│ DuckDB tables - Balanced performance    │ ← Active backtesting
└────────────────┬────────────────────────┘
                 ▼
┌─────────────────────────────────────────┐
│ Tier 3: COLD (1+ years)                 │
│ Parquet files - Compressed storage      │ ← Historical analysis
└─────────────────────────────────────────┘
```

### Pros
- ✅ Optimized for access patterns
- ✅ Best tool for each use case
- ✅ Excellent compression (30GB for 7 years)
- ✅ Industry standard approach

### Cons
- ❌ Complex implementation
- ❌ Multiple storage engines to maintain
- ❌ Data migration between tiers
- ❌ Overkill for backtesting-only use case

### Implementation Strategy
```rust
pub struct TieredStorage {
    hot: Box<dyn TickDatabase>,   // SQLite
    warm: Box<dyn TickDatabase>,  // DuckDB
    cold: ParquetStore,           // Parquet files
}

impl TieredStorage {
    pub fn query_range(&self, start: DateTime, end: DateTime) -> Result<TickIterator> {
        let age = Utc::now() - start;
        match age {
            d if d < Duration::days(7) => self.hot.query(start, end),
            d if d < Duration::days(365) => self.warm.query(start, end),
            _ => self.cold.query(start, end),
        }
    }
}
```

### Verdict: ✅ **Best for Production**
- Handles all scales efficiently
- Standard in financial industry
- Complex but robust

---

## Option 3: Parquet-Only for Historical Data

### Architecture
```
Historical Data (Static):
├── data/
│   ├── 2017/
│   │   ├── AUDUSD_2017_Q1.parquet
│   │   ├── AUDUSD_2017_Q2.parquet
│   │   └── ...
│   └── 2024/
│       └── AUDUSD_2024_Q1.parquet

Query Layer:
├── DuckDB (reads Parquet directly)
└── Polars (for data science workflows)
```

### Pros
- ✅ No database needed for historical data
- ✅ Excellent compression (5-10x)
- ✅ Industry standard format
- ✅ Works with multiple tools
- ✅ No CI compilation issues

### Cons
- ❌ Immutable (corrections are complex)
- ❌ Not suitable for real-time updates
- ❌ Requires batch processing

### Verdict: ✅ **Best for Backtesting-Only**
- Perfect for read-only historical data
- Simple and efficient
- Avoids database complexity

---

## Recommendation for Epic 1-2

### Immediate (Epic 1):
Use **SQLite everywhere** with small sample data (1 month)
- Fast development
- Fast CI/CD
- Sufficient for POC

### Epic 1 (Complete):
Implement **SQLite Only**
- Basic embedded database ✅
- Simple tick storage ✅
- No abstraction needed yet ✅

### Epic 2 (Future):
Expand to **Option 1** (SQLite dev / DuckDB prod)
- Add feature flags for database selection
- Create abstraction layer
- Test with full dataset locally

### Long-term (Epic 3+):
Migrate to **Option 3** (Parquet for historical)
- Download historical data as Parquet
- Use DuckDB to query Parquet files
- Add SQLite cache for frequently accessed data

---

## Decision Matrix

| Criteria | SQLite/DuckDB | Three-Tier | Parquet-Only |
|----------|---------------|------------|--------------|
| Complexity | Low | High | Medium |
| CI Speed | Fast | Fast | Fast |
| Prod Performance | Good | Excellent | Excellent |
| Storage Efficiency | Poor | Excellent | Excellent |
| Real-time Updates | Yes | Yes | No |
| Dev Experience | Good | Complex | Good |
| **Best For** | **MVP/POC** | **Full Platform** | **Backtesting** |

---

## Next Steps

1. **For Story 1.2**: Keep SQLite for now
2. **For Story 1.3-1.5**: Continue with SQLite
3. **For Epic 2**: Implement database abstraction layer
4. **For Epic 3**: Evaluate Parquet migration based on actual usage patterns