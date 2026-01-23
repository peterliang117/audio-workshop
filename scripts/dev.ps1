$ErrorActionPreference = "Stop"

# Repo root = scripts/ 的上一级
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$audiomassDir = Join-Path $repoRoot "vendor\audiomass\src"
$port = 5055
$url  = "http://localhost:$port/"

Write-Host "Repo: $repoRoot"
Write-Host "AudioMass dir: $audiomassDir"
Write-Host "Starting AudioMass server on $url ..."

# 1) 开新窗口跑静态服务（方便看日志）
Start-Process -FilePath "powershell.exe" -ArgumentList @(
  "-NoExit",
  "-Command",
  "cd `"$audiomassDir`"; py -m http.server $port"
) | Out-Null

# 2) 等待服务起来（最多等 10 秒）
$deadline = (Get-Date).AddSeconds(10)
do {
  try {
    Invoke-WebRequest -Uri $url -UseBasicParsing -TimeoutSec 1 | Out-Null
    break
  } catch {
    Start-Sleep -Milliseconds 300
  }
} while ((Get-Date) -lt $deadline)

try {
  Invoke-WebRequest -Uri $url -UseBasicParsing -TimeoutSec 2 | Out-Null
  Write-Host "AudioMass is up: $url"
} catch {
  Write-Warning "AudioMass server may not be reachable yet. You can still continue, but the app may show blank until $url responds."
}

# 3) 启动 Tauri dev（在当前窗口，方便看编译输出）
Write-Host "Starting Tauri dev..."
cd $repoRoot
npx.cmd tauri dev
