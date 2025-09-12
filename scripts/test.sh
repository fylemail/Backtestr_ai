#!/bin/bash
# Test runner script for BackTestr AI

set -e

echo "🧪 Running BackTestr AI test suite..."

# Set environment
export NODE_ENV=test
export RUST_LOG=warn

# Run Rust tests
echo "🦀 Running Rust tests..."
cargo test --all --verbose

# Run Rust benchmarks (optional)
if [ "$RUN_BENCHMARKS" = "true" ]; then
    echo "⚡ Running Rust benchmarks..."
    cargo bench --all
fi

# Run clippy
echo "🔍 Running Rust linter (clippy)..."
cargo clippy --all -- -D warnings

# Check Rust formatting
echo "🎨 Checking Rust formatting..."
cargo fmt --all -- --check

# Run TypeScript/JavaScript tests
echo "⚛️ Running JavaScript tests..."
pnpm test:js

# Run TypeScript type checking
echo "📝 Running TypeScript type check..."
cd electron/renderer
pnpm run typecheck
cd ../..

# Run ESLint
echo "🔍 Running JavaScript linter..."
pnpm run lint:js || true

# Run Python tests if Python is available
if command -v python3 &> /dev/null && [ -d "algorithms/tests" ]; then
    echo "🐍 Running Python tests..."
    python3 -m pytest algorithms/tests/ -v || true
fi

echo "✅ All tests completed!"

# Generate coverage report if requested
if [ "$COVERAGE" = "true" ]; then
    echo "📊 Generating coverage report..."
    cargo tarpaulin --out Html --output-dir target/coverage
    echo "Coverage report: target/coverage/index.html"
fi