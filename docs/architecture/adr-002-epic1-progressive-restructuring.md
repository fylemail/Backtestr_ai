# ADR-002: Epic 1 Progressive Restructuring

## Status
Accepted

## Context
The original Epic 1 definition contained 9 stories with features spanning multiple technology stacks (Rust, Python, TypeScript) and complex architectural components (IPC, service layers, MTF engine). This violated the progressive development principles outlined in CLAUDE.md and created significant risk of:
- Architectural debt from premature decisions
- Complex dependencies blocking progress
- Scope creep from over-engineering
- Testing complexity across multiple languages

## Decision
We have restructured Epic 1 from 9 complex stories to 5 focused stories that build only the Rust foundation and basic data pipeline. All advanced features have been redistributed to appropriate future epics.

### New Epic 1 Structure
1. **Story 1.1**: Project Infrastructure Setup (✅ Complete)
2. **Story 1.2**: Basic DuckDB Integration
3. **Story 1.3**: Simple CSV Import
4. **Story 1.4**: Basic CLI Query Interface
5. **Story 1.5**: Basic Performance Validation

### Feature Redistribution
- **Epic 2**: Advanced data pipeline features (9 features)
- **Epic 3**: MTF state engine and service architecture (5 features)
- **Epic 4**: Python integration and algorithms (4 features)
- **Epic 5**: Frontend, Electron, and IPC (10 features)

## Consequences

### Positive
- **Reduced complexity**: 44% reduction in story count for Epic 1
- **Clear boundaries**: No cross-language dependencies in foundation
- **Faster delivery**: Simpler scope enables quicker completion
- **Lower risk**: Foundation proven before complex features
- **Better testing**: Single-language testing is simpler
- **Progressive validation**: Each epic builds on proven foundation

### Negative
- **Documentation update burden**: Multiple documents needed updating
- **Feature deferral**: Some stakeholders may want features sooner
- **Potential rework**: Some early decisions might need adjustment

### Neutral
- **38 features preserved**: All functionality maintained, just resequenced
- **Performance targets unchanged**: Same 10K ticks/sec requirement
- **End goal unchanged**: Final product scope remains the same

## Implementation

### Completed Actions
1. ✅ Created Feature Preservation Matrix tracking all 38 features
2. ✅ Rewrote Epic 1 documentation with progressive focus
3. ✅ Created new Story 1.2-1.5 documents with simplified scope
4. ✅ Updated CLAUDE.md Epic 1 checklist
5. ✅ Resolved DuckDB dependency (upgraded to v1.3)

### Next Steps
1. Create story branch for Story 1.2
2. Implement basic DuckDB integration
3. Follow simplified story sequence

## Validation
The restructuring has been validated through:
- Feature Preservation Matrix confirms no features lost
- Dependency analysis shows logical progression
- Performance targets remain achievable
- Git strategy supports incremental development

## Technical Details

### Before (9 Stories)
- Mixed languages (Rust, Python, TypeScript)
- Complex IPC architecture
- Service layer abstractions
- Multi-stack testing framework
- Advanced credential management
- MTF state engine

### After (5 Stories)
- Pure Rust implementation
- Basic embedded database
- Simple CSV import
- Direct CLI interface
- Focused performance validation

### Dependency Resolution
- **DuckDB**: Upgraded from 0.9 to 1.3 (resolved arrow-arith conflict)
- **Python**: Deferred to Epic 4 (no pyo3 needed)
- **IPC**: Deferred to Epic 5 (no MessagePack needed)

## Metrics

| Metric | Original | Restructured | Improvement |
|--------|----------|--------------|-------------|
| Epic 1 Stories | 9 | 5 | 44% reduction |
| Language Stacks | 3 | 1 | 67% reduction |
| External Dependencies | Many | Few | Simplified |
| Completion Risk | High | Low | Reduced |

## References
- [CLAUDE.md](../../CLAUDE.md) - Progressive development guidelines
- [Feature Preservation Matrix](../development/feature-preservation-matrix.md) - Feature tracking
- [Progressive Development Audit](../development/progressive-development-audit.md) - Original analysis
- [Epic 1 Definition](../prd/epic-1-foundation-core-data-pipeline.md) - Updated scope
- Story files in `docs/stories/epic-1/`

## Decision Date
Current Session

## Decision Makers
- Development Team (following progressive development principles)
- Validated through systematic feature analysis

## Review Date
To be reviewed after Epic 1 completion

---
*This ADR documents the critical decision to restructure Epic 1 following progressive development principles, ensuring a solid foundation before adding complexity.*