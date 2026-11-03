; Windows installer of Débarras. Built by .github/workflows/release-binaries.yml:
;   DEBARRAS_VERSION=0.1.0 iscc packaging/windows/debarras.iss   (makes dist/debarras-windows-x86_64-setup.exe)
#define Version GetEnv("DEBARRAS_VERSION")

[Setup]
AppId={{9FA8036E-3A44-44D2-A116-7D24CAE2160F}
AppName=Débarras
AppVersion={#Version}
AppPublisher=Andrea Larboullet Marin
AppPublisherURL=https://github.com/alarboulletmarin/debarras
DefaultDirName={autopf}\Debarras
DefaultGroupName=Débarras
; Installed for the current user only (%LOCALAPPDATA%\Programs\Debarras): no administrator prompt.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
MinVersion=10.0
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
SetupIconFile=debarras.ico
UninstallDisplayIcon={app}\debarras.exe
LicenseFile=..\..\LICENSE
OutputDir=..\..\dist
OutputBaseFilename=debarras-windows-x86_64-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
DisableProgramGroupPage=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; Flags: unchecked

[Files]
Source: "..\..\target\release\debarras.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Débarras"; Filename: "{app}\debarras.exe"
Name: "{autodesktop}\Débarras"; Filename: "{app}\debarras.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\debarras.exe"; Description: "{cm:LaunchProgram,Débarras}"; Flags: nowait postinstall skipifsilent
