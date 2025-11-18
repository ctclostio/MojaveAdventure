@echo off
echo Starting Hermes-2-Pro-8B Extraction AI Server...
echo This model will extract entities from AI narratives for the worldbook.
echo.
echo Server will run on http://localhost:8081
echo.

llama-server.exe -m models\Hermes-2-Pro-Llama-3-8B-Q4_K_M.gguf --port 8081 -c 4096 --threads 6 --ctx-size 4096

pause
