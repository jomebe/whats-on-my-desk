$ErrorActionPreference = "Stop"
$iss = Get-Content -Raw (Join-Path $PSScriptRoot "..\installer\WhatsOnMyDesk.iss")
$forbidden = '[<>:"/\\|?*]'
$checks = @{
  "safe name" = "WhatsOnMyDesk"
  "menu name" = "Whats on My Desk"
  "installer output" = "WhatsOnMyDeskSetup-0.1.0-alpha.2-x64.exe"
  "portable output" = "WhatsOnMyDesk-0.1.0-alpha.2-portable-x64.zip"
}
foreach ($entry in $checks.GetEnumerator()) {
  if ($entry.Value -match $forbidden) { throw "Invalid filesystem name ($($entry.Key)): $($entry.Value)" }
}
if ($iss -match 'DefaultGroupName=.*\?') { throw "DefaultGroupName contains forbidden ?" }
if ($iss -match 'Name:\s*"\{(?:userprograms|group|userdesktop)[^"]*\?') { throw "Shortcut Name contains forbidden ?" }
if ($iss -match 'OutputBaseFilename=.*\?') { throw "OutputBaseFilename contains forbidden ?" }
if ($iss -notmatch 'DisableProgramGroupPage=yes') { throw "Start Menu folder page must be disabled" }
Write-Output "Installer naming validation passed"
