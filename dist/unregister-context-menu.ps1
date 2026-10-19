# Removes the "Turbo Delete" entry added by register-context-menu.ps1.

foreach ($class in 'Directory', '*') {
    [Microsoft.Win32.Registry]::CurrentUser.DeleteSubKeyTree("Software\Classes\$class\shell\TurboDelete", $false)
}

Write-Host 'Removed the context menu entry'
