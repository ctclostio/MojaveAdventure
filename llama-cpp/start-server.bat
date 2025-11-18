@echo off
echo ╔══════════════════════════════════════════════════════════╗
echo ║    FALLOUT DND - AI DUNGEON MASTER SERVER               ║
echo ║    Starting llama.cpp with Mistral 7B...                ║
echo ╚══════════════════════════════════════════════════════════╝
echo.

REM Get the directory where this script is located
set SCRIPT_DIR=%~dp0

echo Starting llama-server...
echo Model: Mistral 7B Instruct v0.2 (Q4_K_M)
echo Server URL: http://localhost:8080
echo.
echo Press Ctrl+C to stop the server when done.
echo.

REM Start the server with optimal settings for DM
"%SCRIPT_DIR%llama-server.exe" ^
  -m "%SCRIPT_DIR%models\mistral-7b-instruct-v0.2.Q4_K_M.gguf" ^
  --port 8080 ^
  -c 4096 ^
  --threads 6 ^
  --batch-size 512 ^
  --n-predict 512 ^
  --ctx-size 4096

pause
