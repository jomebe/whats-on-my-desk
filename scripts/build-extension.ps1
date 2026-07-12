$root = Split-Path $PSScriptRoot -Parent
$out = Join-Path $root 'release\whats-on-my-desk-extension.zip'
New-Item -ItemType Directory -Force (Split-Path $out) | Out-Null
Compress-Archive -Path (Join-Path $root 'extension\*') -DestinationPath $out -Force
