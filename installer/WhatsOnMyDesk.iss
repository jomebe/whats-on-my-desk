#define AppName "What’s on My Desk?"
#define AppVersion "0.1.0-alpha.1"
#define AppExeName "WhatsOnMyDesk.exe"

[Setup]
AppId={{7B2C60CD-C3C3-4FC1-95EF-09AFA6D7C457}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=jomebe
DefaultDirName={localappdata}\Programs\WhatsOnMyDesk
DefaultGroupName={#AppName}
OutputDir=..\release
OutputBaseFilename=WhatsOnMyDeskSetup-x64
Compression=lzma2
SolidCompression=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\{#AppExeName}

[Files]
Source: "..\src-tauri\target\release\{#AppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExeName}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "WhatsOnMyDesk"; ValueData: """{app}\{#AppExeName}"""; Tasks: autostart; Flags: uninsdeletevalue

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"
Name: "autostart"; Description: "Start with Windows"

[Run]
Filename: "{app}\{#AppExeName}"; Description: "Launch {#AppName}"; Flags: nowait postinstall skipifsilent
