$ErrorActionPreference = "Stop"
$workspace = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

Push-Location $workspace
try {
  npm run build
  Push-Location src-tauri
  try { cargo build --release }
  finally { Pop-Location }
  $release = Join-Path $workspace "release"
  New-Item -ItemType Directory -Path $release -Force | Out-Null
  Copy-Item "src-tauri\target\release\WhatsOnMyDeskAgent.exe" $release -Force
}
finally {
  Pop-Location
}
