$ErrorActionPreference = "Stop"
$workspace = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Push-Location $workspace
try {
  & "$PSScriptRoot\validate-installer-names.ps1"
  npm run build
  Push-Location src-tauri
  try { cargo build --release } finally { Pop-Location }
  $release = Join-Path $workspace "release"
  New-Item -ItemType Directory -Force -Path $release | Out-Null
  Copy-Item "src-tauri\target\release\WhatsOnMyDesk.exe" "$release\WhatsOnMyDesk.exe" -Force
  $portable = Join-Path $release "portable"
  Remove-Item $portable -Recurse -Force -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Force -Path $portable | Out-Null
  Copy-Item "$release\WhatsOnMyDesk.exe" $portable
  Set-Content "$portable\README.txt" "What’s on My Desk? portable`r`nRun WhatsOnMyDesk.exe. WebView2 Evergreen Runtime is required."
  Compress-Archive -Path "$portable\*" -DestinationPath "$release\WhatsOnMyDesk-0.1.0-alpha.2-portable-x64.zip" -Force
  $iscc = (Get-Command iscc -ErrorAction SilentlyContinue).Source
  if (-not $iscc) { $iscc = Join-Path $env:LOCALAPPDATA "Programs\Inno Setup 6\ISCC.exe" }
  & $iscc "installer\WhatsOnMyDesk.iss"
  Get-FileHash "$release\WhatsOnMyDeskSetup-0.1.0-alpha.2-x64.exe", "$release\WhatsOnMyDesk-0.1.0-alpha.2-portable-x64.zip" -Algorithm SHA256 | ForEach-Object { "{0}  {1}" -f $_.Hash.ToLower(), $_.Path.Split('\')[-1] } | Set-Content "$release\SHA256SUMS.txt"
} finally { Pop-Location }
