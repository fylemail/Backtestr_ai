# Coding Standards

## Development Standards Overview

BackTestr_ai maintains strict coding standards across all technology stack components to ensure code quality, performance, and maintainability in a multi-language financial application.

## Rust Code Standards

**Formatting and Style**
- Use `rustfmt` with default configuration for consistent code formatting
- Line length limit: 100 characters maximum
- Use 4-space indentation (automatic with rustfmt)
- Prefer explicit lifetimes in public APIs
- Use descriptive variable names: `tick_processor` not `tp`

**Documentation Requirements**
- All public functions must have comprehensive doc comments with examples
- Include performance characteristics in doc comments for critical path code
- Document safety invariants for unsafe code blocks
- Use `cargo doc` to generate and verify documentation

**Error Handling**
- Use `Result<T, E>` for all fallible operations
- Create custom error types with `thiserror` for domain-specific errors
- Never use `unwrap()` or `expect()` in production code paths
- Implement `From` traits for error conversion chains

**Performance Considerations**
- Profile all hot paths with `criterion` benchmarks
- Avoid allocations in tick processing loops
- Use `&str` instead of `String` for temporary string handling
- Implement `Clone` judiciously - prefer borrowing
- Use `#[inline]` for small, frequently called functions

**Testing Requirements**
- Minimum 80% code coverage for core engine modules
- Property-based tests with `proptest` for mathematical functions
- Integration tests in separate `tests/` directory
- Benchmark regression tests for performance-critical components

## Python Code Standards

**Type Hints and Annotations**
- All function signatures must include complete type hints
- Use `typing` module for complex types: `List[Dict[str, float]]`
- Document return types even for simple functions
- Use `@dataclass` for structured data with type annotations

**Performance Annotations**
- Use `@lru_cache` for expensive pure functions
- Prefer NumPy operations over pure Python loops for numerical calculations
- Document time complexity for algorithm implementations
- Profile Python code with `cProfile` for performance bottlenecks

**Integration with Rust**
- Follow PyO3 naming conventions for Rust-exposed functions
- Handle Python exceptions gracefully in Rust integration layer
- Use type validation at Python/Rust boundary
- Document memory ownership patterns in hybrid functions

**Testing Procedures**
- Use `pytest` for all Python testing
- Test algorithm implementations with historical data samples
- Validate numerical accuracy against known reference implementations
- Integration tests must cover Python ↔ Rust data exchange

## TypeScript/React Standards

**Component Architecture Patterns**
- Use functional components with hooks exclusively
- Implement proper component composition over inheritance
- Create reusable components in `components/shared/`
- Use TypeScript strict mode with no `any` types

**State Management Conventions**
- Use Zustand for global application state
- Local component state only for UI-specific data
- Implement proper state normalization for complex data
- Document state update patterns and side effects

**Performance Guidelines**
- Use `React.memo()` for expensive rendering components
- Implement proper dependency arrays for `useEffect` hooks
- Use `useMemo()` and `useCallback()` for expensive calculations
- Profile render performance with React DevTools

**File Organization**
- One React component per file with matching filename
- Co-locate component-specific styles and tests
- Use barrel exports (`index.ts`) for clean imports
- Separate business logic into custom hooks

## Quality Assurance Standards

**Code Review Process**
- All code must pass automated linting and formatting checks
- Manual review required for changes to critical path components
- Performance impact assessment for engine modifications
- Cross-language integration points require extra scrutiny

**Automated Quality Gates**
- Pre-commit hooks enforce formatting and basic linting
- CI pipeline runs full test suite on all pull requests
- Benchmark regression detection for performance-critical changes
- Documentation generation must succeed without warnings

**Security Considerations**
- No hardcoded credentials or API keys in source code
- Input validation at all external data boundaries
- Regular dependency security audits with `cargo audit`
- Secure coding practices for Python algorithm execution sandboxing
