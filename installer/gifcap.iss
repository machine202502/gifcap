#ifndef AppVersion
  #define AppVersion "1.2.2"
#endif
#ifndef SourceExe
  #define SourceExe "..\target\x86_64-pc-windows-msvc\release\gifcap.exe"
#endif
#ifndef ThirdParty
  #define ThirdParty "..\dist\THIRD-PARTY.txt"
#endif

[Setup]
AppId={{6F2A9C41-8B17-4E5D-A3C0-9D1E7B4F2A18}
AppName=gifcap
AppVersion={#AppVersion}
AppPublisher=gifcap
AppPublisherURL=https://github.com/machine202502/gifcap
AppSupportURL=https://github.com/machine202502/gifcap
AppUpdatesURL=https://github.com/machine202502/gifcap/releases
DefaultDirName={autopf}\gifcap
DefaultGroupName=gifcap
DisableProgramGroupPage=yes
OutputDir=..\dist
OutputBaseFilename=gifcap-{#AppVersion}-windows-x64-setup
UninstallDisplayIcon={app}\gifcap.exe
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=admin
CloseApplications=yes
InfoBeforeFile=..\NOTICE

[Tasks]
Name: desktopicon; Description: "Create a desktop shortcut"; GroupDescription: "Additional icons:"; Flags: unchecked

[Files]
Source: "{#SourceExe}"; DestDir: "{app}"; DestName: "gifcap.exe"; Flags: ignoreversion
Source: "..\COPYING"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\NOTICE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#ThirdParty}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\gifcap"; Filename: "{app}\gifcap.exe"
Name: "{autodesktop}\gifcap"; Filename: "{app}\gifcap.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\gifcap.exe"; Description: "Launch gifcap"; Flags: nowait postinstall skipifsilent
