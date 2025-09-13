# CLAUDE.md - Agent Context for Progressive Development

## Current Development Focus

**Active Epic**: Epic 1 - Foundation & Core Data Pipeline  
**Current Story**: 1.1 Complete, 1.2 Starting  
**Development Philosophy**: Build ONLY what current epic needs

## Critical Boundaries - DO NOT BUILD YET

❌ **Python Integration** - Deferred to Epic 4  
❌ **Frontend/Electron** - Deferred to Epic 5  
❌ **Advanced Credentials** - Deferred to Epic 2  
❌ **MTF State Engine** - Deferred to Epic 3  
❌ **Charting/Visualization** - Deferred to Epic 6  
❌ **Statistical Analysis** - Deferred to Epic 7  

## What We're Building NOW (Epic 1)

✅ **Rust Core Engine** - Basic tick processing  
✅ **Simple SQLite** - Basic data storage (no optimization)  
✅ **Tick Data Ingestion** - CSV/simple formats only  
✅ **Basic CLI** - Simple command interface  

## Current Working Commands

```bash
# These work NOW:
cargo build              # Build Rust workspace
cargo test              # Run tests
cargo clippy            # Lint code
cargo fmt               # Format code

# These DON'T work yet (don't try):
# npm/pnpm commands     # No frontend yet
# python/pytest         # No Python yet
# electron commands     # No UI yet
```

## Project Structure (Current State)

```
backtestr_ai/
├── crates/           # Rust workspace (ACTIVE)
│   ├── backtestr-core/  # Core engine
│   ├── backtestr-data/  # Data layer
│   └── backtestr-ipc/   # IPC (minimal)
├── src/              # Main application
├── data/             # Data storage
├── docs/             # Documentation
└── scripts/          # Build scripts

# NOT YET CREATED (don't add):
# ├── algorithms/     # Epic 4
# ├── electron/       # Epic 5
# └── python/         # Epic 4
```

## Development Guidelines

### When Adding Code

1. **Ask First**: "Is this needed for Epic 1?"
2. **If No**: Add feature flag or defer entirely
3. **If Yes**: Implement minimally, no over-engineering

### Feature Flags

```rust
// Use feature flags for future epic code
#[cfg(feature = "epic_2")]
pub mod advanced_features;

// Current epic code doesn't need flags
pub mod core_features;  // Always built
```

### CI/CD

- Single workflow: `.github/workflows/ci.yml`
- Tests ONLY Rust code
- No Python/Node.js checks until their epics

### Git Workflow

```bash
# Branch naming for Epic 1
git checkout -b story/STORY-1.2-basic-sqlite

# Don't create branches for future epics yet
# ❌ story/STORY-5.1-frontend
```

## Common Mistakes to Avoid

1. **Building Python bridge** - Not until Epic 4
2. **Setting up Electron** - Not until Epic 5
3. **Complex CI workflows** - Keep it simple
4. **Unused dependencies** - Only add what's needed NOW
5. **Empty directories** - Don't create algorithms/, electron/ yet

## Performance Targets (Epic 1 Only)

- Tick ingestion: 10K ticks/second minimum
- Memory usage: < 500MB for 1M ticks
- Query response: < 100ms for basic queries

## Testing Requirements

### Epic 1 Tests
- Unit tests for data structures
- Integration tests for SQLite
- Basic performance benchmarks

### NOT Required Yet
- Python algorithm tests (Epic 4)
- UI component tests (Epic 5)
- Statistical validation (Epic 7)

## Epic 1 Completion Checklist

- [ ] Story 1.1: Project setup ✅
- [ ] Story 1.2: Basic SQLite integration
- [ ] Story 1.3: CSV tick data ingestion
- [ ] Story 1.4: Simple query interface
- [ ] Story 1.5: Basic performance optimization

## Questions to Ask Yourself

Before writing ANY code:
1. Is this needed for Epic 1?
2. Will this work without Python/Frontend?
3. Can this be simpler?
4. Am I over-engineering?

## Getting Help

- Review: `docs/development/progressive-development-audit.md`
- Epic details: `docs/prd/epic-1-foundation-core-data-pipeline.md`
- Git strategy: `docs/development/git-strategy.md`

---

**Remember**: We're building a foundation, not the entire building. Keep it simple, focused, and progressive.