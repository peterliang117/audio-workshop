$ErrorActionPreference = "SilentlyContinue"

$port = 5055

Write-Host "Stopping Audio Workshop processes..." -ForegroundColor Cyan

# 1) Kill process that is listening on port 5055 (AudioMass static server)
try {
  $conns = Get-NetTCPConnection -LocalPort $port -State Listen
  foreach ($c in $conns) {
    $pid = $c.OwningProcess
    if ($pid -and $pid -ne 0) {
      Write-Host "Stopping PID $pid listening on $port..." -ForegroundColor Yellow
      Stop-Process -Id $pid -Force
    }
  }
} catch {
  Write-Host "Get-NetTCPConnection not available or no listener on $port." -ForegroundColor DarkYellow
}

# 2) Kill the Tauri app exe if it is still running
Write-Host "Stopping audio-workshop.exe (if running)..." -ForegroundColor Yellow
Stop-Process -Name "audio-workshop" -Force

Write-Host "Done." -ForegroundColor Green
