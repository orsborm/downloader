Write-Host "Stopping all Claude processes..." -ForegroundColor Red

# Kill claude
Get-Process -Name "claude" -ErrorAction SilentlyContinue | Stop-Process -Force

# Kill PowerShell jobs
Get-Job | Remove-Job -Force

Write-Host "Done!" -ForegroundColor Green
