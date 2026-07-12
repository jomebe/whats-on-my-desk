param([Parameter(Mandatory=$true)][string]$ExtensionId)
$root = Split-Path $PSScriptRoot -Parent
$install = Join-Path $env:LOCALAPPDATA 'WhatsOnMyDesk'
New-Item -ItemType Directory -Force $install | Out-Null
Copy-Item (Join-Path $root 'release\WhatsOnMyDeskAgent.exe') $install -Force
$manifestPath = Join-Path $install 'com.whats_on_my_desk.agent.json'
@{ name='com.whats_on_my_desk.agent'; description='What’s on My Desk Windows device agent'; path=(Join-Path $install 'WhatsOnMyDeskAgent.exe'); type='stdio'; allowed_origins=@("chrome-extension://$ExtensionId/") } | ConvertTo-Json | Set-Content $manifestPath
foreach($browser in @('HKCU:\Software\Google\Chrome\NativeMessagingHosts','HKCU:\Software\Microsoft\Edge\NativeMessagingHosts')) { New-Item -Path "$browser\com.whats_on_my_desk.agent" -Force | Out-Null; Set-ItemProperty -Path "$browser\com.whats_on_my_desk.agent" -Name '(Default)' -Value $manifestPath }
