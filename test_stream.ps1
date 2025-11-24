# Test if llama.cpp server supports streaming

Write-Host "Testing streaming endpoint..." -ForegroundColor Cyan

$body = @{
    prompt = "Test: Say hello"
    n_predict = 20
    temperature = 0.8
    stream = $true
} | ConvertTo-Json

Write-Host "Sending request with stream=true..." -ForegroundColor Yellow
Write-Host $body -ForegroundColor Gray

try {
    $response = Invoke-WebRequest -Uri "http://localhost:8080/completion" `
        -Method POST `
        -Body $body `
        -ContentType "application/json" `
        -UseBasicParsing

    Write-Host "`nResponse Status: $($response.StatusCode)" -ForegroundColor Green
    Write-Host "Content-Type: $($response.Headers['Content-Type'])" -ForegroundColor Yellow
    Write-Host "`nFirst 500 chars of response:" -ForegroundColor Yellow
    Write-Host $response.Content.Substring(0, [Math]::Min(500, $response.Content.Length)) -ForegroundColor Gray
} catch {
    Write-Host "`nError: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.Exception.Response) {
        $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
        $responseBody = $reader.ReadToEnd()
        Write-Host "Response body: $responseBody" -ForegroundColor Gray
    }
}
