# Adds "Turbo Delete" to the Explorer context menu of folders and files.
# Writes to HKEY_CURRENT_USER only, so no admin rights are needed.
#
# Run it from a regular PowerShell window. Apps installed as MSIX package (and
# every process they start) write HKCU\Software\Classes into a private copy of
# the registry that Explorer never sees.
param(
    # path to turbo-delete.exe, found on the PATH by default
    [string]$Path = (Get-Command turbo-delete -ErrorAction SilentlyContinue).Source,
    [string]$Label = 'Turbo Delete'
)

$ErrorActionPreference = 'Stop'

if (-not $Path -or -not (Test-Path -LiteralPath $Path)) {
    throw 'turbo-delete.exe not found. Install it with "cargo install --path . --locked" or pass -Path.'
}

$Path = (Resolve-Path -LiteralPath $Path).Path

foreach ($class in 'Directory', '*') {
    $key = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey("Software\Classes\$class\shell\TurboDelete")
    $key.SetValue('', $Label)
    $key.SetValue('Icon', 'shell32.dll,31')
    # keep the entry visible when more than 15 items are selected
    $key.SetValue('MultiSelectModel', 'Player')

    # --confirm merges the processes Explorer starts for a multi-selection into one confirmation
    $command = $key.CreateSubKey('command')
    $command.SetValue('', "`"$Path`" --confirm `"%1`"")
    $command.Close()
    $key.Close()
}

Write-Host "Registered the context menu entry for $Path"
