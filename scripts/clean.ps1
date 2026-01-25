$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $repoRoot

# Generated / heavy folders safe to recreate.
$paths = @(
  "node_modules",
  "src-tauri\\target",
  "src-tauri\\gen",
  "releases",
  "downloads"
)

Write-Host "Repo root: $repoRoot"

foreach ($p in $paths) {
  $full = Join-Path $repoRoot $p
  if (Test-Path $full) {
    Write-Host "Removing $p ..."
    Remove-Item -Recurse -Force $full
  } else {
    Write-Host "Skipping $p (not present)."
  }
}

Write-Host "Done. Reinstall deps with: npm install"
