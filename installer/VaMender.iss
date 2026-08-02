; SPDX-License-Identifier: MIT

#define MyAppName "VaMender"
#define MyAppPublisher "AgenticCreator"
#define MyAppURL "https://github.com/TheAgenticCreator/vamender"
#define MyAppExeName "vamender.exe"
#define MyAppVersion "0.1.1"

[Setup]
AppId={{79C5997D-6D92-44F3-844F-6B9EA0477145}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}/issues
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={localappdata}\VaMender
DisableProgramGroupPage=yes
LicenseFile=..\LICENSE
InfoBeforeFile=..\DISCLAIMER.md
OutputDir=..\dist
OutputBaseFilename=VaMender-Setup-{#MyAppVersion}
SetupIconFile=..\assets\vamender.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma2/ultra64
SolidCompression=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
CloseApplications=yes
CloseApplicationsFilter=vamender.exe
RestartApplications=no

[Files]
Source: "..\target\release\vamender.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\dist\AgenticCreator.VaMender.2.var"; DestDir: "{tmp}"; Flags: deleteafterinstall

[Icons]
Name: "{autoprograms}\VaMender"; Filename: "{app}\vamender.exe"; WorkingDir: "{app}"; IconFilename: "{app}\vamender.exe"; Flags: runminimized

[Run]
Filename: "{app}\vamender.exe"; Parameters: "install-host ""{code:GetVaMRoot}"" --backup ""{code:GetBackupRoot}"" --plugin-var ""{tmp}\AgenticCreator.VaMender.2.var"""; StatusMsg: "Installing the VaM integration and tray host..."; Flags: runhidden waituntilterminated

[UninstallRun]
Filename: "{app}\vamender.exe"; Parameters: "uninstall-host --purge"; Flags: runhidden waituntilterminated; RunOnceId: "RemoveVaMenderHost"

[Code]
var
  VaMPage: TInputDirWizardPage;
  BackupPage: TInputDirWizardPage;

function NormalizeDirectory(Value: String): String;
begin
  Result := RemoveBackslashUnlessRoot(ExpandFileName(Value));
end;

function GetVaMRoot(Param: String): String;
begin
  Result := NormalizeDirectory(VaMPage.Values[0]);
end;

function GetBackupRoot(Param: String): String;
begin
  Result := NormalizeDirectory(BackupPage.Values[0]);
end;

function IsWithin(ChildPath: String; ParentPath: String): Boolean;
var
  ChildWithSlash: String;
  ParentWithSlash: String;
begin
  ChildWithSlash := AddBackslash(NormalizeDirectory(ChildPath));
  ParentWithSlash := AddBackslash(NormalizeDirectory(ParentPath));
  Result := Pos(Uppercase(ParentWithSlash), Uppercase(ChildWithSlash)) = 1;
end;

procedure InitializeWizard;
var
  SuggestedRoot: String;
begin
  SuggestedRoot := ExpandConstant('{param:VAMROOT|}');
  if SuggestedRoot = '' then
    SuggestedRoot := ExpandConstant('{sd}\VaM');
  VaMPage := CreateInputDirPage(
    wpSelectDir,
    'Locate Virt-a-Mate',
    'Choose the folder containing VaM.exe.',
    'VaMender installs its Session Plugin into this VaM installation. ' +
      'VaM may remain open, but no VAR may be loaded while a repair is applied.',
    False,
    ''
  );
  VaMPage.Add('VaM installation folder:');
  VaMPage.Values[0] := SuggestedRoot;

  BackupPage := CreateInputDirPage(
    VaMPage.ID,
    'Choose the VaMender backup folder',
    'This must be outside AddonPackages.',
    'Every changed or archived VAR is copied here first. Keep this folder ' +
      'on reliable storage and do not treat VaMender as a substitute for a ' +
      'separate, tested backup of your library.',
    False,
    ''
  );
  BackupPage.Add('Durable backup folder:');
  BackupPage.Values[0] := ExpandConstant('{param:BACKUP|}');
  if BackupPage.Values[0] = '' then
    BackupPage.Values[0] := SuggestedRoot + '-VaMender-Backup';
end;

function NextButtonClick(CurPageID: Integer): Boolean;
var
  VaMRoot: String;
  Packages: String;
  Backup: String;
begin
  Result := True;
  if CurPageID = VaMPage.ID then
  begin
    VaMRoot := GetVaMRoot('');
    if not FileExists(AddBackslash(VaMRoot) + 'VaM.exe') then
    begin
      MsgBox('The selected folder does not contain VaM.exe.', mbError, MB_OK);
      Result := False;
      Exit;
    end;
    if not DirExists(AddBackslash(VaMRoot) + 'AddonPackages') then
    begin
      MsgBox('The selected folder does not contain AddonPackages.', mbError, MB_OK);
      Result := False;
      Exit;
    end;
  end;
  if CurPageID = BackupPage.ID then
  begin
    VaMRoot := GetVaMRoot('');
    Packages := AddBackslash(VaMRoot) + 'AddonPackages';
    Backup := GetBackupRoot('');
    if CompareText(NormalizeDirectory(Backup), NormalizeDirectory(Packages)) = 0 then
    begin
      MsgBox('The backup folder cannot be AddonPackages.', mbError, MB_OK);
      Result := False;
      Exit;
    end;
    if IsWithin(Backup, Packages) then
    begin
      MsgBox('The backup folder must be outside AddonPackages.', mbError, MB_OK);
      Result := False;
      Exit;
    end;
  end;
end;

function EngineIsBusy: Boolean;
var
  StatusLines: TArrayOfString;
  StatusPath: String;
  RequestPath: String;
begin
  StatusPath := AddBackslash(GetVaMRoot('')) +
    'Saves\PluginData\VaMender\Bridge\status.txt';
  RequestPath := AddBackslash(GetVaMRoot('')) +
    'Saves\PluginData\VaMender\Bridge\request.json';
  Result := FileExists(RequestPath);
  if (not Result) and LoadStringsFromFile(StatusPath, StatusLines) and
     (GetArrayLength(StatusLines) > 0) then
    Result := Pos('RUNNING:', Trim(StatusLines[0])) = 1;
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  Engine: String;
  ExitCode: Integer;
begin
  Result := '';
  Engine := ExpandConstant('{app}\{#MyAppExeName}');
  if not FileExists(Engine) then
    Exit;

  if EngineIsBusy then
  begin
    Result :=
      'VaMender is currently running or has a queued operation. ' +
      'Wait for it to finish, then run Setup again. No files were changed.';
    Exit;
  end;

  { Current builds support stop-host. The uninstall-host fallback safely }
  { upgrades older builds that predate cooperative external shutdown. }
  if Exec(Engine, 'stop-host', '', SW_HIDE, ewWaitUntilTerminated, ExitCode) and
     (ExitCode = 0) then
    Exit;
  if (not Exec(Engine, 'uninstall-host', '', SW_HIDE,
      ewWaitUntilTerminated, ExitCode)) or (ExitCode <> 0) then
    Result :=
      'Setup could not stop the existing VaMender tray engine safely. ' +
      'Exit VaMender from its notification-area menu and run Setup again.';
end;
