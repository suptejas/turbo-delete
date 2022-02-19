Import-Module BitsTransfer

$ErrorActionPreference = "Stop"
Start-BitsTransfer 'https://cdn.xtremedevx.com/turbo-delete/Turbo-Delete.exe' "${Env:\TEMP}\Turbo-Delete.exe" -Description 'Downloading Turbo Delete v0.0.1 from CDN' -DisplayName 'Downloading Turbo Delete' -TransferType Download

Write-Host 'Installing Turbo Delete' -ForegroundColor cyan
& "${Env:\TEMP}\Turbo-Delete.exe" /VERYSILENT | Out-Null

Write-Host 'Successfully Installed Turbo Delete' -ForegroundColor green