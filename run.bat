@echo off
echo ╔════════════════════════════════════════════════════════════╗
echo ║          FALLOUT: WASTELAND ADVENTURES                     ║
echo ║          Building and launching...                         ║
echo ╚════════════════════════════════════════════════════════════╝
echo.

REM Check if Rust is installed
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Rust is not installed!
    echo Please install from: https://rustup.rs/
    echo.
    pause
    exit /b 1
)

REM Create saves directory if it doesn't exist
if not exist "saves" mkdir saves

REM Build and run
echo Building game...
cargo build --release

if %ERRORLEVEL% EQU 0 (
    echo.
    echo Build successful! Starting game...
    echo.
    cargo run --release
) else (
    echo.
    echo Build failed! Check errors above.
    pause
)
