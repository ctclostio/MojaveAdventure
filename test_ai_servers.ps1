# Test script to check if AI servers are running and responding

Write-Host "Testing AI Server Connections..." -ForegroundColor Cyan
Write-Host ""

# Test Narrative AI (port 8080)
Write-Host "1. Testing Narrative AI (http://localhost:8080)..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "http://localhost:8080/health" -TimeoutSec 5 -UseBasicParsing
    Write-Host "   ✓ Narrative AI is responding (Status: $($response.StatusCode))" -ForegroundColor Green
} catch {
    Write-Host "   ✗ Narrative AI is NOT responding: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host ""

# Test Extraction AI (port 8081)
Write-Host "2. Testing Extraction AI (http://localhost:8081)..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "http://localhost:8081/health" -TimeoutSec 5 -UseBasicParsing
    Write-Host "   ✓ Extraction AI is responding (Status: $($response.StatusCode))" -ForegroundColor Green
} catch {
    Write-Host "   ✗ Extraction AI is NOT responding: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host ""
Write-Host "Checking for llama-server processes..." -ForegroundColor Yellow
$llamaProcesses = Get-Process -Name "llama-server" -ErrorAction SilentlyContinue
if ($llamaProcesses) {
    Write-Host "   ✓ Found $($llamaProcesses.Count) llama-server process(es) running" -ForegroundColor Green
    foreach ($proc in $llamaProcesses) {
        Write-Host "     - PID: $($proc.Id), Memory: $([math]::Round($proc.WS / 1MB, 2)) MB" -ForegroundColor Gray
    }
} else {
    Write-Host "   ✗ No llama-server processes found" -ForegroundColor Red
}

Write-Host ""
Write-Host "Done!" -ForegroundColor Cyan
