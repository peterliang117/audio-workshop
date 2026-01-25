$ErrorActionPreference = 'Stop'

$root = git rev-parse --show-toplevel 2>$null
if (-not $root) {
  Write-Error "Not inside a git repo."
  exit 1
}

$cwd = (Get-Location).Path
Write-Host "Current directory: $cwd"
Write-Host "Repo top-level: $root"

if ($cwd -like "*\\vendor\\audiomass*") {
  Write-Error "Push blocked: you are inside vendor\\audiomass (submodule)."
  exit 1
}

git push
