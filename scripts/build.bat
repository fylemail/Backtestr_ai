@echo off
REM Build script for BackTestr AI (Windows)

echo Building BackTestr AI...

REM Set environment
if "%NODE_ENV%"=="" set NODE_ENV=production

REM Build Rust components
echo Building Rust engine...
cargo build --release --all
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%

REM Install Node dependencies if needed
echo Installing Node dependencies...
where pnpm >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo pnpm not found. Please install pnpm first: npm install -g pnpm
    exit /b 1
)

call pnpm install --frozen-lockfile
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%

REM Build Electron renderer
echo Building React frontend...
cd electron\renderer
call pnpm run build
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%
cd ..\..

REM Build Electron app
echo Packaging Electron app...
call pnpm run electron:build
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%

echo Build complete!
echo Output: dist\