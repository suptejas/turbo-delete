; Installer for Turbo Delete, built by automate-release.py:
;   iscc /DMyAppVersion=x.y.z dist\turbo-delete.iss
;
; Installs per user (no admin rights) to %LOCALAPPDATA%\Programs\Turbo Delete,
; optionally adds it to the PATH of the user and to the Explorer context menu.
#include "environment.iss"

#define MyAppName "Turbo Delete"
#ifndef MyAppVersion
  #define MyAppVersion "0.2.0"
#endif
#define MyAppPublisher "XtremeDevX"
#define MyAppURL "https://github.com/suptejas/turbo-delete"
#define MyAppExeName "turbo-delete.exe"

[Setup]
; NOTE: The value of AppId uniquely identifies this application.
; Do not use the same AppId value in installers for other applications.
; (To generate a new GUID, click Tools | Generate GUID inside the IDE.)
AppId={{71679A1F-AE2C-4DC8-BB8E-03C0652B6527}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
LicenseFile=..\LICENSE.txt
OutputDir=.
OutputBaseFilename=Turbo-Delete-Install
UninstallDisplayName={#MyAppName}
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
ChangesEnvironment=yes
ChangesAssociations=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "german"; MessagesFile: "compiler:Languages\German.isl"

[CustomMessages]
english.ContextMenuLabel=Turbo Delete
german.ContextMenuLabel=Turbo Delete (endgültig löschen)
english.TaskContextMenu=Add "Turbo Delete" to the Explorer context menu of folders and files
german.TaskContextMenu=„Turbo Delete“ zum Explorer-Kontextmenü für Ordner und Dateien hinzufügen
english.TaskPath=Add turbo-delete to the PATH (command line)
german.TaskPath=turbo-delete zum PATH hinzufügen (Kommandozeile)
english.RestartExplorer=Restart Windows Explorer now (open Explorer windows will be closed)
german.RestartExplorer=Windows Explorer jetzt neu starten (offene Explorer-Fenster werden geschlossen)

[Tasks]
Name: "contextmenu"; Description: "{cm:TaskContextMenu}"
Name: "path"; Description: "{cm:TaskPath}"

[Files]
Source: "..\bin\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
; --confirm merges the processes Explorer starts for a multi-selection into one confirmation,
; MultiSelectModel=Player keeps the entry visible when more than 15 items are selected
Root: HKCU; Subkey: "Software\Classes\Directory\shell\TurboDelete"; ValueType: string; ValueName: ""; ValueData: "{cm:ContextMenuLabel}"; Flags: uninsdeletekey; Tasks: contextmenu
Root: HKCU; Subkey: "Software\Classes\Directory\shell\TurboDelete"; ValueType: string; ValueName: "Icon"; ValueData: "shell32.dll,31"; Tasks: contextmenu
Root: HKCU; Subkey: "Software\Classes\Directory\shell\TurboDelete"; ValueType: string; ValueName: "MultiSelectModel"; ValueData: "Player"; Tasks: contextmenu
Root: HKCU; Subkey: "Software\Classes\Directory\shell\TurboDelete\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" --confirm ""%1"""; Tasks: contextmenu
Root: HKCU; Subkey: "Software\Classes\*\shell\TurboDelete"; ValueType: string; ValueName: ""; ValueData: "{cm:ContextMenuLabel}"; Flags: uninsdeletekey; Tasks: contextmenu
Root: HKCU; Subkey: "Software\Classes\*\shell\TurboDelete"; ValueType: string; ValueName: "Icon"; ValueData: "shell32.dll,31"; Tasks: contextmenu
Root: HKCU; Subkey: "Software\Classes\*\shell\TurboDelete"; ValueType: string; ValueName: "MultiSelectModel"; ValueData: "Player"; Tasks: contextmenu
Root: HKCU; Subkey: "Software\Classes\*\shell\TurboDelete\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" --confirm ""%1"""; Tasks: contextmenu

[Run]
; Explorer reads new context menu entries right away, the restart is only a fallback.
; Windows restarts the shell on its own after it was stopped, it is only started if it did not.
Filename: "powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -Command ""Stop-Process -Name explorer -Force; Start-Sleep -Seconds 3; if (-not (Get-Process explorer -ErrorAction SilentlyContinue)) {{ Start-Process explorer.exe }"""; Description: "{cm:RestartExplorer}"; Flags: postinstall unchecked runhidden nowait skipifsilent

[Code]
procedure CurStepChanged(CurStep: TSetupStep);
begin
    if CurStep = ssPostInstall then
    begin
        { Also undo the choices of a previous installation that were deselected now }
        if WizardIsTaskSelected('path')
        then EnvAddPath(ExpandConstant('{app}'))
        else EnvRemovePath(ExpandConstant('{app}'));

        if not WizardIsTaskSelected('contextmenu') then
        begin
            RegDeleteKeyIncludingSubkeys(HKEY_CURRENT_USER, 'Software\Classes\Directory\shell\TurboDelete');
            RegDeleteKeyIncludingSubkeys(HKEY_CURRENT_USER, 'Software\Classes\*\shell\TurboDelete');
        end;
    end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
    if CurUninstallStep = usPostUninstall
    then EnvRemovePath(ExpandConstant('{app}'));
end;
