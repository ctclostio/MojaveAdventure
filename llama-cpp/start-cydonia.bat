@echo off
echo ╔══════════════════════════════════════════════════════════╗
echo ║    FALLOUT DND - AI DUNGEON MASTER SERVER               ║
echo ║    Starting with Cydonia 24B v4.2.0...                  ║
echo ╚══════════════════════════════════════════════════════════╝
echo.

REM Get the directory where this script is located
set SCRIPT_DIR=%~dp0

echo Starting llama-server with Cydonia 24B...
echo Model: TheDrummer_Cydonia-24B-v4.2.0 Q4_K_M
echo Server URL: http://localhost:8080
echo.
echo NOTE: First response will be SLOWER (20-30 sec) due to larger model.
echo Subsequent responses: 10-20 seconds each.
echo.
echo Press Ctrl+C to stop the server when done.
echo.

REM Start the server with optimal settings for 24B model
"%SCRIPT_DIR%llama-server.exe" ^
  -m "%SCRIPT_DIR%models\TheDrummer_Cydonia-24B-v4.2.0-Q4_K_M.gguf" ^
  --port 8080 ^
  -c 8192 ^
  --threads 8 ^
  --batch-size 512 ^
  --n-predict 512 ^
  --ctx-size 8192

pause
