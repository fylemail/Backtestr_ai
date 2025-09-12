@echo off
REM Development server script for BackTestr AI (Windows)

echo Starting BackTestr AI in development mode...

REM Set environment
set NODE_ENV=development
set RUST_LOG=debug

REM Build Rust in development mode
echo Building Rust engine (debug mode)...
cargo build --all
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%

REM Install dependencies if needed
if not exist "node_modules" (
    echo Installing Node dependencies...
    call pnpm install
    if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%
)

REM Start all processes
echo Starting development servers...

REM Start Rust engine
start "Rust Engine" cmd /c "cargo run"

REM Wait a moment for Rust to start
timeout /t 2 /nobreak >nul

REM Start React dev server
start "React Dev Server" cmd /c "cd electron\renderer && pnpm run dev"

REM Wait for React dev server
timeout /t 3 /nobreak >nul

REM Start Electron
start "Electron" cmd /c "cd electron && pnpm run start"

echo Development servers started!
echo Close this window to keep servers running.
echo Close individual command windows to stop specific servers.
pause