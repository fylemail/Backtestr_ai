# Contributing to BackTestr AI

Thank you for your interest in contributing to BackTestr AI!

## Development Setup

### Prerequisites

1. **Install Required Tools**:
   - Rust 1.75+ with MSVC toolchain
   - Node.js 20+ and pnpm
   - Python 3.11+
   - Visual Studio Build Tools with "Desktop development with C++" workload

2. **Clone and Setup**:
   ```bash
   git clone https://github.com/backtestr-ai/backtestr.git
   cd backtestr
   pnpm install
   cargo build --all
   ```

## CI/CD Status

The project uses two CI workflows:

1. **Basic CI** (`ci-basic.yml`) - ✅ Required
   - Validates project structure
   - Checks configuration files
   - Ensures infrastructure is properly set up

2. **Full Build and Test** (`build.yml`) - ⚠️ Optional (during initial setup)
   - Complete Rust build and tests
   - Node.js build and tests
   - Python tests
   - Integration build

### Known CI Issues

- **Rust Build**: Requires Visual Studio Build Tools with C++ workload on Windows runners
- **Dependencies**: Some crates (arrow, zstd-sys) require native compilation
- **Initial Setup**: Full build may fail until all dependencies are properly configured

## Development Workflow

1. Create a feature branch
2. Make your changes
3. Run local tests: `cargo test && pnpm test`
4. Push to your fork
5. Create a Pull Request

## Code Standards

- **Rust**: Run `cargo fmt` before committing
- **TypeScript**: Follow ESLint rules
- **Python**: Use Black formatter

## Getting Help

- Check existing issues
- Join our Discord community
- Email: support@backtestr.ai