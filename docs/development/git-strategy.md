# Git Strategy & CI/CD Pipeline Architecture

## Overview

This document defines the git branching strategy, CI/CD pipeline configuration, and development workflow for BackTestr AI. The strategy is designed to support progressive development across our multi-epic roadmap while maintaining code quality and preventing production disruptions.

## Project Context

BackTestr AI is a Windows-only desktop application with:
- **Rust Core Engine** (Epic 1-3): High-performance backtesting engine
- **Python Integration** (Epic 4): Trading algorithm support via PyO3
- **Electron/React Frontend** (Epic 5-6): Advanced charting and UI
- **Statistical Analysis** (Epic 7): Reporting and analytics

## Branch Strategy

### Core Branches

```
main (protected)
├── Purpose: Production-ready code, release candidates
├── Protection: Requires PR reviews, all CI checks must pass
├── Deploy: Triggers release builds and signing
└── Merges: Only from release/* or hotfix/* branches

develop (protected)  
├── Purpose: Integration branch for completed features
├── Protection: Requires CI checks to pass
├── Deploy: Triggers nightly/beta builds
└── Merges: From feature/* and bugfix/* branches
```

### Working Branches

```
story/STORY-{n.m}-{description}  [BMad-aligned naming]
├── Example: story/STORY-1.2-data-ingestion
├── Created from: develop (or epic branch if exists)
├── Merges to: develop (or epic branch) via PR
└── Lifetime: Duration of story implementation

epic/epic-{n}-{name} (optional for multi-story coordination)
├── Example: epic/epic-2-data-pipeline
├── Created from: develop
├── Merges to: develop via PR when epic complete
└── Purpose: Coordinate multiple related stories

bugfix/{issue-id}-{description}
├── Example: bugfix/123-fix-data-parsing
├── Created from: develop  
├── Merges to: develop via PR
└── Lifetime: Until bug is resolved

release/v{major}.{minor}.{patch}
├── Example: release/v1.0.0
├── Created from: develop when features complete
├── Merges to: main AND develop
└── Purpose: Final testing and version preparation

hotfix/{critical-issue}
├── Example: hotfix/critical-data-corruption
├── Created from: main
├── Merges to: main AND develop via PR
└── Purpose: Emergency production fixes only
```

## Feature Flag Strategy

### Managing Incomplete Epic Functionality

Feature flags enable safe integration of partial epic implementations:

```rust
// src/features.rs
pub struct Features {
    pub epic_2_data_pipeline: bool,
    pub epic_3_mtf_engine: bool,
    pub epic_4_python_bridge: bool,
    pub epic_5_frontend: bool,
    pub epic_6_charting: bool,
    pub epic_7_analytics: bool,
}

impl Features {
    pub fn from_env() -> Self {
        Self {
            epic_2_data_pipeline: env::var("FEATURE_EPIC_2").is_ok(),
            epic_3_mtf_engine: env::var("FEATURE_EPIC_3").is_ok(),
            epic_4_python_bridge: env::var("FEATURE_EPIC_4").is_ok(),
            epic_5_frontend: env::var("FEATURE_EPIC_5").is_ok(),
            epic_6_charting: env::var("FEATURE_EPIC_6").is_ok(),
            epic_7_analytics: env::var("FEATURE_EPIC_7").is_ok(),
        }
    }
}
```

### Environment-Based Feature Control

```bash
# .env.development
FEATURE_EPIC_2=true  # Enable data pipeline in dev
FEATURE_EPIC_3=false # MTF engine not ready
FEATURE_EPIC_4=false # Python bridge disabled

# .env.production
# Only enable completed epics in production
FEATURE_EPIC_2=true
```

### Usage in Code

```rust
// Only initialize Python bridge if Epic 4 is complete
if features.epic_4_python_bridge {
    python_engine::initialize()?;
}

// Conditionally enable UI features
if features.epic_5_frontend && features.epic_6_charting {
    enable_advanced_charts();
}
```

## Progressive CI/CD Configuration

### Philosophy: Build What Exists

Our CI/CD pipeline uses **progressive validation** - only testing components that have been built. This prevents blocking progress on incomplete epics.

### CI Configuration Phases

#### Phase 1: Foundation (Epic 1 - Current)
```yaml
# .github/workflows/ci.yml
on:
  pull_request:
    branches: [main, develop]
  push:
    branches: [develop]

jobs:
  foundation:
    name: Foundation Checks
    runs-on: windows-latest
    steps:
      - name: Structure Validation
        run: |
          # Check project structure exists
          test -d crates && test -d docs
      
      - name: Rust Core (if exists)
        run: |
          if [ -f "Cargo.toml" ]; then
            cargo fmt --check
            cargo clippy -- -D warnings
            cargo test --workspace
          fi
        continue-on-error: false  # Required for core
```

#### Phase 2: Data Pipeline (Epic 2-3)
```yaml
      - name: Data Components (if exists)
        run: |
          if [ -f "crates/backtestr-data/Cargo.toml" ]; then
            cargo test -p backtestr-data
            # Run data integrity tests
          fi
        continue-on-error: false  # Required once implemented
```

#### Phase 3: Python Integration (Epic 4)
```yaml
      - name: Python Bridge (if exists)
        run: |
          if [ -d "algorithms" ] && [ -f "algorithms/requirements.txt" ]; then
            pip install -r algorithms/requirements.txt
            pytest algorithms/tests/ || true  # Non-blocking initially
          fi
        continue-on-error: true  # Optional until Epic 4 complete
```

#### Phase 4: Frontend (Epic 5-6)
```yaml
      - name: Frontend Build (if exists)
        run: |
          if [ -f "electron/package.json" ]; then
            cd electron
            pnpm install
            pnpm run typecheck || true  # Non-blocking initially
            pnpm run build || true
          fi
        continue-on-error: true  # Optional until Epic 5 complete
```

### Progressive Test Requirements

| Epic | Component | Required Checks | When to Enable |
|------|-----------|-----------------|----------------|
| 1 | Core + Basic DuckDB | Rust fmt/clippy/test | **ACTIVE NOW** |
| 2 | Advanced Data | DuckDB performance | When Epic 2 starts |
| 3 | MTF Engine | State consistency | When Epic 3 starts |
| 4 | Python Bridge | PyO3 + pytest | When Epic 4 starts |
| 5 | Frontend | TypeScript + Jest | When Epic 5 starts |
| 6 | Charting | Canvas tests | When Epic 6 starts |
| 7 | Analytics | Statistical tests | When Epic 7 starts |

## Story Completion Criteria (BMad Method)

### Definition of Done
A story is considered complete when:

1. **All Acceptance Criteria Met** - Every AC in the story file is satisfied
2. **Code Quality Standards** - Passes all linting, formatting, and static analysis
3. **Tests Implemented** - Minimum 80% coverage for new code
4. **Documentation Complete**:
   - Story file updated with implementation notes
   - CLAUDE.md updated with context for future agents
   - API docs generated (if applicable)
5. **QA Gate Passed** - Gate validation shows PASS status
6. **Performance Validated** - Meets NFR targets:
   - Sub-100ms response for data operations
   - < 500MB memory for typical workload
   - 60 FPS UI rendering maintained

### Story Lifecycle
```
1. Ready for Dev → Create story branch
2. In Progress → Regular commits with progress
3. Code Complete → All ACs implemented
4. In Review → PR created with BMad checklist
5. QA Review → Gate validation executed
6. Done → Merged to develop/epic branch
```

## Commit Conventions

### Format
```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types
- **feat**: New feature (Epic implementation)
- **fix**: Bug fix
- **perf**: Performance improvement
- **refactor**: Code restructuring
- **test**: Test additions/modifications
- **docs**: Documentation updates
- **ci**: CI/CD configuration changes
- **build**: Build system changes

### Examples
```bash
feat(epic-1): implement Rust workspace structure

- Created three crates: core, data, ipc
- Configured shared dependencies
- Added Windows-specific optimizations

Implements: Story 1.1
```

```bash
fix(data): correct DuckDB connection pooling issue

Prevents connection exhaustion under high load
by implementing proper connection lifecycle

Fixes: #234
```

## Pull Request Process

### PR Title Format
```
[Epic-{n}] Story {n.m}: {Description}
```

### PR Template (BMad-Enhanced)
```markdown
## Summary
Brief description of changes

## Epic/Story Reference
- Epic: {n} - {Epic Name}
- Story: {n.m} - {Story Name}
- Gate Status: {link to gate file}

## BMad Story Completion Checklist
### Required
- [ ] **Code Complete**: All acceptance criteria met
- [ ] **Tests Passing**: Unit tests for new functionality
- [ ] **Documentation Updated**: 
  - [ ] Story file updated with completion notes
  - [ ] CLAUDE.md updated with new commands/context
  - [ ] Architecture docs updated if design changed
- [ ] **QA Gate Validated**: Gate file shows PASS status
- [ ] **Performance Targets**: Meets NFR requirements

### Code Quality
- [ ] No hardcoded values (use config/environment)
- [ ] Error handling implemented
- [ ] Logging added for debugging
- [ ] No `unwrap()` in production code (Rust)
- [ ] No `any` types (TypeScript)

### Feature Flags
- [ ] New functionality behind feature flag (if incomplete epic)
- [ ] Flag documented in .env.example
- [ ] Graceful degradation when flag disabled

## Testing
- How to test these changes
- Expected outcomes
- Feature flag combinations tested

## Screenshots (if UI changes)

## Dev Agent Notes
- Model used: {claude-opus-4.1-20250805}
- Challenges faced:
- Solutions implemented:
```

### Review Requirements

| Target Branch | Required Reviewers | Auto-merge | Additional Checks |
|---------------|-------------------|------------|-------------------|
| develop | 1 dev agent or human | Yes (after checks) | CI must pass |
| main | 1 human + gate pass | No | All checks + gate validation |
| hotfix to main | 1 human (expedited) | No | Critical fix validation |

## Merge Strategies

### Feature → Develop
- **Strategy**: Squash and merge
- **Reason**: Clean linear history per feature
- **Commit message**: Use PR title and description

### Develop → Main (via Release)
- **Strategy**: Merge commit (no fast-forward)
- **Reason**: Preserve feature history
- **Tag**: Create version tag on merge

### Hotfix → Main
- **Strategy**: Merge commit
- **Reason**: Traceable emergency fixes
- **Follow-up**: Cherry-pick to develop

## Release Process

### Version Numbering
```
v{major}.{minor}.{patch}

major: Breaking changes or major feature sets (Epic completion)
minor: New features or significant improvements  
patch: Bug fixes and minor improvements
```

### Release Workflow

1. **Feature Freeze**
   ```bash
   git checkout develop
   git checkout -b release/v1.2.0
   ```

2. **Version Bump**
   - Update Cargo.toml versions
   - Update package.json versions
   - Update CHANGELOG.md

3. **Release Testing**
   - Run full test suite
   - Manual testing checklist
   - Performance validation

4. **Merge to Main**
   ```bash
   git checkout main
   git merge --no-ff release/v1.2.0
   git tag -a v1.2.0 -m "Release version 1.2.0"
   ```

5. **Back-merge to Develop**
   ```bash
   git checkout develop
   git merge --no-ff release/v1.2.0
   ```

## CI/CD Pipeline Details

### Build Matrix

```yaml
strategy:
  matrix:
    include:
      # Required builds
      - os: windows-latest
        rust: stable
        node: 20
        python: 3.11
        required: true
      
      # Optional builds (don't block)
      - os: windows-latest
        rust: 1.75.0  # MSRV
        required: false
      
      - os: windows-latest
        target: aarch64-pc-windows-msvc  # ARM64
        required: false
```

### Pipeline Stages

1. **Quick Validation** (< 2 min)
   - Syntax checking
   - Format validation
   - Structure verification

2. **Component Testing** (< 5 min)
   - Unit tests for implemented components
   - Skip tests for unbuilt components

3. **Integration Testing** (< 10 min)
   - Cross-component tests (if components exist)
   - IPC communication tests (when implemented)

4. **Build Artifacts** (< 15 min)
   - Release builds for existing components
   - Package for distribution (when ready)

### Conditional Execution

```yaml
# Smart detection of what to test
- name: Detect Components
  id: detect
  run: |
    echo "has_rust=$([[ -f Cargo.toml ]] && echo true || echo false)" >> $GITHUB_OUTPUT
    echo "has_frontend=$([[ -f electron/package.json ]] && echo true || echo false)" >> $GITHUB_OUTPUT
    echo "has_python=$([[ -f algorithms/requirements.txt ]] && echo true || echo false)" >> $GITHUB_OUTPUT

# Only run relevant tests
- name: Rust Tests
  if: steps.detect.outputs.has_rust == 'true'
  run: cargo test

- name: Frontend Tests  
  if: steps.detect.outputs.has_frontend == 'true'
  run: pnpm test
```

## Migration Plan

### Immediate Actions (Do Now)

1. **Create develop branch**
   ```bash
   git checkout -b develop
   git push -u origin develop
   ```

2. **Update CI triggers**
   - Remove `master` from workflows
   - Add `develop` as primary integration branch

3. **Set branch protection** (GitHub Settings)
   - Protect `main` and `develop`
   - Require PR reviews for `main`

### Progressive Implementation

| Week | Action | Impact |
|------|--------|--------|
| 1 | Implement branch strategy | Safer development |
| 2 | Configure progressive CI | Non-blocking progress |
| 3 | Add component detection | Smart test execution |
| 4 | Implement PR templates | Better documentation |

## Troubleshooting

### Common Issues

**CI Failing on Unbuilt Components**
- Solution: Add `continue-on-error: true` for future epics
- Check component detection logic

**Branch Protection Blocking Emergency Fix**
- Solution: Use admin override for critical hotfixes
- Document override in PR description

**Merge Conflicts in develop**
- Solution: Rebase feature branches regularly
- Keep feature branches short-lived

## Best Practices

1. **Daily Sync**: Rebase feature branches from develop daily
2. **Small PRs**: Aim for < 500 lines changed per PR
3. **Test Locally**: Run relevant CI checks before pushing
4. **Clean History**: Use meaningful commit messages
5. **Document Breaks**: Note any breaking changes in commits

## Gate Integration

Each story completion requires gate validation:

```yaml
# .github/workflows/gate-check.yml
- name: Validate Story Gate
  run: |
    python scripts/validate_gate.py \
      --story ${{ github.event.pull_request.title }} \
      --gate-file docs/qa/gates/
```

## Epic Branch Coordination

### When to Use Epic Branches

Epic branches are optional but recommended when:
- Multiple developers working on related stories
- Epic spans multiple sprints
- Complex integration between stories needed
- Risk of destabilizing develop branch

### Epic Branch Workflow

```bash
# Create epic branch
git checkout develop
git checkout -b epic/epic-2-data-pipeline

# Stories branch from epic
git checkout epic/epic-2-data-pipeline
git checkout -b story/STORY-2.1-duckdb-setup

# Merge story to epic
git checkout epic/epic-2-data-pipeline
git merge --no-ff story/STORY-2.1-duckdb-setup

# When epic complete, merge to develop
git checkout develop
git merge --no-ff epic/epic-2-data-pipeline
```

### Epic Integration Testing

Epic branches enable isolated testing:
```yaml
# Run full epic test suite
on:
  push:
    branches: [epic/*]

jobs:
  epic-integration:
    name: Epic Integration Tests
    # Full test suite for epic functionality
```

## Appendix: Quick Commands

```bash
# Start new story (BMad-aligned)
git checkout develop
git pull origin develop
git checkout -b story/STORY-1.2-description

# Sync with develop
git fetch origin
git rebase origin/develop

# Prepare PR
git push -u origin story/STORY-1.2-description
# Create PR via GitHub UI with BMad template

# Release preparation
git checkout develop
git checkout -b release/v1.0.0
# ... version bumps ...
git push -u origin release/v1.0.0

# Hotfix
git checkout main
git checkout -b hotfix/critical-issue
# ... fix ...
git push -u origin hotfix/critical-issue
```

---

*Last Updated: 2025-01-13*
*Version: 1.1.0*
*Epic Status: Epic 1 In Progress*
*BMad Method: Fully Aligned*