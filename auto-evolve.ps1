cd E:\project

$prompt = Get-Content -Raw -Encoding UTF8 prompt.txt

for ($i = 1; $i -le 10; $i++) {
    Write-Host "==========================================" -ForegroundColor Cyan
    Write-Host " Evolution Round $i / 10 - $(Get-Date)" -ForegroundColor Yellow
    Write-Host "==========================================" -ForegroundColor Cyan
    
    claude --dangerously-skip-permissions --max-turns 150 --max-budget-usd 20 -p "$prompt"
    
    Write-Host "=== Round $i finished - $(Get-Date) ===" -ForegroundColor Green
    Start-Sleep -Seconds 10
}

Write-Host "=== All 10 rounds completed ===" -ForegroundColor Magenta
