#!/bin/bash
# Build script for BackTestr AI

set -e

echo "🚀 Building BackTestr AI..."

# Set environment
export NODE_ENV=${NODE_ENV:-production}

# Build Rust components
echo "📦 Building Rust engine..."
cargo build --release --all

# Install Node dependencies if needed
echo "📦 Installing Node dependencies..."
if ! command -v pnpm &> /dev/null; then
    echo "pnpm not found. Please install pnpm first: npm install -g pnpm"
    exit 1
fi

pnpm install --frozen-lockfile

# Build Electron renderer
echo "📦 Building React frontend..."
cd electron/renderer
pnpm run build
cd ../..

# Build Electron app
echo "📦 Packaging Electron app..."
pnpm run electron:build

echo "✅ Build complete!"
echo "Output: dist/"