@echo off
echo ========================================
echo Starting Dual AI Server Setup
echo ========================================
echo.
echo This will start TWO AI models:
echo   1. Cydonia 24B (Port 8080) - Narrative Generation
echo   2. Hermes-2-Pro 8B (Port 8081) - Entity Extraction
echo.
echo Both servers are required for worldbook functionality.
echo.
pause

echo.
echo Starting Cydonia 24B on port 8080...
start "Cydonia 24B - Narrative AI" cmd /k llama-server.exe -m models\TheDrummer_Cydonia-24B-v4.2.0-Q4_K_M.gguf --port 8080 -c 8192 --threads 8 --ctx-size 8192

echo Waiting 5 seconds before starting second server...
timeout /t 5 /nobreak >nul

echo.
echo Starting Hermes-2-Pro 8B on port 8081...
start "Hermes 8B - Extraction AI" cmd /k llama-server.exe -m models\Hermes-2-Pro-Llama-3-8B-Q4_K_M.gguf --port 8081 -c 4096 --threads 6 --ctx-size 4096

echo.
echo ========================================
echo Both servers are starting!
echo ========================================
echo.
echo Narrative AI: http://localhost:8080
echo Extraction AI: http://localhost:8081
echo.
echo Wait ~30 seconds for models to load, then start the game.
echo.
echo Press any key to close this window (servers will keep running)...
pause >nul
