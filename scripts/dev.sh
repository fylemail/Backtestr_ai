#!/bin/bash
# Development server script for BackTestr AI

set -e

echo "🚀 Starting BackTestr AI in development mode..."

# Set environment
export NODE_ENV=development
export RUST_LOG=debug

# Function to kill all processes on exit
cleanup() {
    echo "🛑 Shutting down development servers..."
    kill $(jobs -p) 2>/dev/null || true
    exit
}

trap cleanup EXIT INT TERM

# Build Rust in development mode
echo "📦 Building Rust engine (debug mode)..."
cargo build --all

# Start Rust engine in background
echo "🦀 Starting Rust engine..."
cargo run &
RUST_PID=$!

# Wait for Rust engine to start
sleep 2

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing Node dependencies..."
    pnpm install
fi

# Start Electron renderer dev server
echo "⚛️ Starting React dev server..."
cd electron/renderer
pnpm run dev &
REACT_PID=$!
cd ../..

# Wait for React dev server
sleep 3

# Start Electron
echo "🖥️ Starting Electron..."
cd electron
pnpm run start &
ELECTRON_PID=$!
cd ..

echo "✅ Development servers started!"
echo "   Rust Engine PID: $RUST_PID"
echo "   React Dev Server PID: $REACT_PID"
echo "   Electron PID: $ELECTRON_PID"
echo ""
echo "Press Ctrl+C to stop all servers"

# Wait for any process to exit
wait