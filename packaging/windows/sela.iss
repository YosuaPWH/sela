#ifndef AppVersion
  #error AppVersion must be supplied by the packaging script.
#endif
#ifndef FileVersion
  #error FileVersion must be supplied by the packaging script.
#endif
#ifndef StageDir
  #error StageDir must be supplied by the packaging script.
#endif
#ifndef OutputDir
  #error OutputDir must be supplied by the packaging script.
#endif

[Setup]
AppId={{8981B9F1-B013-4470-828C-FBE4182EDCEA}
AppName=Sela
AppVersion={#AppVersion}
AppPublisher=Sela contributors
AppPublisherURL=https://github.com/YosuaPWH/sela
AppSupportURL=https://github.com/YosuaPWH/sela/issues
AppUpdatesURL=https://github.com/YosuaPWH/sela/releases/latest
DefaultDirName={localappdata}\Programs\Sela
DefaultGroupName=Sela
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.17763
LicenseFile={#StageDir}\LICENSE
UninstallDisplayIcon={app}\sela-notch.exe
UninstallDisplayName=Sela
OutputDir={#OutputDir}
OutputBaseFilename=Sela-{#AppVersion}-setup-x64
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no
VersionInfoVersion={#FileVersion}
VersionInfoDescription=Sela installer
VersionInfoProductName=Sela

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop shortcut"; GroupDescription: "Additional shortcuts:"; Flags: unchecked

[Files]
Source: "{#StageDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\Sela"; Filename: "{app}\sela-notch.exe"
Name: "{autodesktop}\Sela"; Filename: "{app}\sela-notch.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\sela-notch.exe"; Description: "Launch Sela"; Flags: nowait postinstall skipifsilent
