$ErrorActionPreference = "Stop"

$workspace = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$buildRoot = Join-Path $env:TEMP "whats-on-my-desk-tauri-build"
$releaseRoot = Join-Path $workspace "release"

if (Test-Path -LiteralPath $buildRoot) {
  Remove-Item -LiteralPath $buildRoot -Recurse -Force
}

New-Item -ItemType Directory -Path $buildRoot | Out-Null
Get-ChildItem -LiteralPath $workspace -Force |
  Where-Object { $_.Name -notin @(".git", "node_modules", "dist", "release") } |
  Copy-Item -Destination $buildRoot -Recurse -Force

Push-Location $buildRoot
try {
  npm ci
  npm run tauri build
  New-Item -ItemType Directory -Path $releaseRoot -Force | Out-Null
  Copy-Item "src-tauri\target\release\WhatsOnMyDesk.exe" $releaseRoot -Force
  Get-ChildItem "src-tauri\target\release\bundle" -Recurse -Include "*.msi", "*-setup.exe" |
    Copy-Item -Destination $releaseRoot -Force
}
finally {
  Pop-Location
}
