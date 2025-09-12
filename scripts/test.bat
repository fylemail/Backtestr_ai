@echo off
REM Test runner script for BackTestr AI (Windows)

echo Running BackTestr AI test suite...

REM Set environment
set NODE_ENV=test
set RUST_LOG=warn

REM Run Rust tests
echo Running Rust tests...
cargo test --all --verbose
if %ERRORLEVEL% NEQ 0 (
    echo Rust tests failed!
    exit /b %ERRORLEVEL%
)

REM Run Rust benchmarks (optional)
if "%RUN_BENCHMARKS%"=="true" (
    echo Running Rust benchmarks...
    cargo bench --all
)

REM Run clippy
echo Running Rust linter (clippy)...
cargo clippy --all -- -D warnings
if %ERRORLEVEL% NEQ 0 (
    echo Clippy found issues!
    exit /b %ERRORLEVEL%
)

REM Check Rust formatting
echo Checking Rust formatting...
cargo fmt --all -- --check
if %ERRORLEVEL% NEQ 0 (
    echo Rust formatting issues found!
    exit /b %ERRORLEVEL%
)

REM Run TypeScript/JavaScript tests
echo Running JavaScript tests...
call pnpm test:js

REM Run TypeScript type checking
echo Running TypeScript type check...
cd electron\renderer
call pnpm run typecheck
cd ..\..

REM Run ESLint
echo Running JavaScript linter...
call pnpm run lint:js

REM Run Python tests if Python is available
where python >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    if exist "algorithms\tests" (
        echo Running Python tests...
        python -m pytest algorithms\tests\ -v
    )
)

echo All tests completed!

REM Generate coverage report if requested
if "%COVERAGE%"=="true" (
    echo Generating coverage report...
    cargo tarpaulin --out Html --output-dir target\coverage
    echo Coverage report: target\coverage\index.html
)