[Setup]
AppName=Chronos Activity Tracker
AppVersion=1.0.0
AppPublisher=Chronos Team
AppPublisherURL=https://chronos-red-five.vercel.app
AppSupportURL=https://chronos-red-five.vercel.app
AppUpdatesURL=https://chronos-red-five.vercel.app
DefaultDirName={autopf}\Chronos
DefaultGroupName=Chronos
AllowNoIcons=yes
LicenseFile=
OutputDir=.
OutputBaseFilename=chronos-setup
SetupIconFile=
Compression=lzma
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "startup"; Description: "Start Chronos automatically with Windows"; GroupDescription: "Startup Options"; Flags: checkedonce

[Files]
Source: "..\rust-client\target\release\chronos.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Chronos Activity Tracker"; Filename: "{app}\chronos.exe"
Name: "{group}\{cm:UninstallProgram,Chronos Activity Tracker}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Chronos Activity Tracker"; Filename: "{app}\chronos.exe"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "Chronos"; ValueData: """{app}\chronos.exe"""; Flags: uninsdeletevalue; Tasks: startup

[Run]
Filename: "{app}\chronos.exe"; Description: "{cm:LaunchProgram,Chronos Activity Tracker}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "taskkill"; Parameters: "/f /im chronos.exe"; Flags: runhidden

[Code]
procedure CurStepChanged(CurStep: TSetupStep);
var
  ResultCode: Integer;
begin
  if CurStep = ssPostInstall then
  begin
    // Kill any existing chronos processes
    Exec('taskkill', '/f /im chronos.exe', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  end;
end;
