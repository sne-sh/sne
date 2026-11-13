; Inno Setup script for the sne Windows installer.
;
; Build from the repository root after `cargo build --release`:
;   iscc installer\sne.iss
; Options:  /DAppVersion=0.1.0  /DArch=x64|x86|arm64  /DSourceExe=path\to\sne.exe

#ifndef AppVersion
  #define AppVersion "0.1.0"
#endif
#ifndef Arch
  #define Arch "x64"
#endif
#ifndef SourceExe
  #define SourceExe "..\target\release\sne.exe"
#endif

[Setup]
AppId={{6E1B3B0C-6C1D-4E0B-9E55-5D2B6C0E7A41}
AppName=sne
AppVersion={#AppVersion}
AppVerName=sne {#AppVersion}
AppPublisher=sne contributors
AppPublisherURL=https://github.com/sne-sh/sne
AppSupportURL=https://github.com/sne-sh/sne/issues
DefaultDirName={localappdata}\Programs\sne
DisableProgramGroupPage=yes
DisableDirPage=auto
; Installs for the current user only, so no administrator prompt.
PrivilegesRequired=lowest
ChangesEnvironment=yes
LicenseFile=..\LICENSE
SetupIconFile=..\assets\sne.ico
UninstallDisplayIcon={app}\sne.ico
WizardStyle=modern
WizardImageFile=wizard-large.bmp
WizardSmallImageFile=wizard-small.bmp
OutputDir=..\target\installer
OutputBaseFilename=sne-{#AppVersion}-{#Arch}-setup
Compression=lzma2/max
SolidCompression=yes
#if Arch == "arm64"
ArchitecturesAllowed=arm64
ArchitecturesInstallIn64BitMode=arm64
#elif Arch == "x86"
; 32-bit build: runs on every Windows, so no architecture limits.
#else
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
#endif

[Tasks]
Name: "addtopath"; Description: "Add sne and Node.js to PATH (needed to run them from any terminal)"
Name: "psprofile"; Description: "Set up PowerShell: switch Node versions automatically (adds one line to your profile)"
Name: "notify"; Description: "Tell me when a new version of sne is available (checks once a day)"

[Files]
Source: "{#SourceExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\sne.ico"; DestDir: "{app}"
Source: "setup-profile.ps1"; DestDir: "{app}"
Source: "..\README.md"; DestDir: "{app}"
Source: "..\LICENSE"; DestDir: "{app}"; DestName: "LICENSE.txt"

[Run]
Filename: "powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\setup-profile.ps1"""; Flags: runhidden; Tasks: psprofile
Filename: "pwsh.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\setup-profile.ps1"""; Flags: runhidden; Tasks: psprofile; Check: HasPwsh
Filename: "{app}\sne.exe"; Parameters: "notify on"; Flags: runhidden; Tasks: notify

[UninstallRun]
; Removes the daily check, if it was turned on.
Filename: "{app}\sne.exe"; Parameters: "notify off"; Flags: runhidden; RunOnceId: "SneNotifyOff"

[Code]
const
  EnvKey = 'Environment';

function HasPwsh: Boolean;
begin
  Result := FileSearch('pwsh.exe', GetEnv('PATH')) <> '';
end;

{ Position of Dir inside a semicolon-separated list, ignoring case; 0 when absent. }
function PathPos(const Paths, Dir: string): Integer;
begin
  Result := Pos(';' + Uppercase(Dir) + ';', ';' + Uppercase(Paths) + ';');
end;

procedure AddToUserPath(const Dir: string);
var
  Paths: string;
begin
  if not RegQueryStringValue(HKEY_CURRENT_USER, EnvKey, 'Path', Paths) then
    Paths := '';
  if PathPos(Paths, Dir) > 0 then
    exit;
  if (Paths <> '') and (Paths[Length(Paths)] <> ';') then
    Paths := Paths + ';';
  RegWriteExpandStringValue(HKEY_CURRENT_USER, EnvKey, 'Path', Paths + Dir);
end;

procedure RemoveFromUserPath(const Dir: string);
var
  Paths: string;
  P: Integer;
begin
  if not RegQueryStringValue(HKEY_CURRENT_USER, EnvKey, 'Path', Paths) then
    exit;
  P := PathPos(Paths, Dir);
  if P = 0 then
    exit;
  { Removes the entry and the separator after it. }
  Delete(Paths, P, Length(Dir) + 1);
  if (Paths <> '') and (Paths[Length(Paths)] = ';') then
    Delete(Paths, Length(Paths), 1);
  RegWriteExpandStringValue(HKEY_CURRENT_USER, EnvKey, 'Path', Paths);
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if (CurStep = ssPostInstall) and WizardIsTaskSelected('addtopath') then
  begin
    AddToUserPath(ExpandConstant('{app}'));
    { The link to the default Node version: node then works everywhere with no shell setup. }
    AddToUserPath(ExpandConstant('{localappdata}\sne\aliases\default'));
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    RemoveFromUserPath(ExpandConstant('{app}'));
    RemoveFromUserPath(ExpandConstant('{localappdata}\sne\aliases\default'));
  end;
end;
