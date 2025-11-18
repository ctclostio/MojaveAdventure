@echo off
echo ╔══════════════════════════════════════════════════════════╗
echo ║       FALLOUT DND - LAUNCHING WITH AI DM                 ║
echo ╚══════════════════════════════════════════════════════════╝
echo.
echo Checking llama-server...
timeout /t 2 >nul

echo.
echo Starting game...
echo.

cd /d "%~dp0"
set PATH=%USERPROFILE%\.cargo\bin;%PATH%
cargo run --release

pause
