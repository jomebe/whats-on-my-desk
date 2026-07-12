#define MyAppDisplayName "What's on My Desk?"
#define MyAppSafeName "WhatsOnMyDesk"
#define MyAppMenuName "Whats on My Desk"
#define MyAppExeName "WhatsOnMyDesk.exe"
#define MyAppVersion "0.1.0-alpha.2"
#define MyAppPublisher "jomebe"
#define MyAppURL "https://whats-on-my-desk.pages.dev"

[Setup]
AppId={{7B2C60CD-C3C3-4FC1-95EF-09AFA6D7C457}
AppName={#MyAppDisplayName}
AppVerName={#MyAppDisplayName} {#MyAppVersion}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={localappdata}\Programs\{#MyAppSafeName}
DefaultGroupName={#MyAppMenuName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=..\release
OutputBaseFilename=WhatsOnMyDeskSetup-{#MyAppVersion}-x64
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayName={#MyAppDisplayName}
UninstallDisplayIcon={app}\{#MyAppExeName}

[Files]
Source: "..\src-tauri\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{userprograms}\{#MyAppMenuName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; IconFilename: "{app}\{#MyAppExeName}"
Name: "{userdesktop}\{#MyAppMenuName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "WhatsOnMyDesk"; ValueData: """{app}\{#MyAppExeName}"" --startup"; Tasks: autostart; Flags: uninsdeletevalue

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"
Name: "autostart"; Description: "Start What's on My Desk? with Windows"

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Launch {#MyAppDisplayName}"; Flags: nowait postinstall skipifsilent
