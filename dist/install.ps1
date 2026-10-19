# Installs Turbo Delete with:
#   iwr -useb https://github.tobse.eu/turbo-delete/install.ps1 | iex
#
# Downloads the installer to %TEMP%, runs it silently and deletes it again.
# The installer installs per user (no admin rights) to
# %LOCALAPPDATA%\Programs\Turbo Delete and adds it to the PATH and the
# Explorer context menu. It can be removed in Settings > Apps.

# a script block keeps the variables out of the session that runs iex
& {
    $ErrorActionPreference = 'Stop'
    # the progress bar slows Invoke-WebRequest down a lot
    $ProgressPreference = 'SilentlyContinue'

    $url = 'https://github.tobse.eu/turbo-delete/Turbo-Delete-Install.exe'
    $setup = Join-Path ([IO.Path]::GetTempPath()) 'Turbo-Delete-Install.exe'

    Write-Host 'Downloading Turbo Delete' -ForegroundColor Cyan
    Invoke-WebRequest -Uri $url -OutFile $setup -UseBasicParsing

    try {
        Write-Host 'Installing Turbo Delete' -ForegroundColor Cyan
        $process = Start-Process -FilePath $setup -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART' -Wait -PassThru

        if ($process.ExitCode -ne 0) {
            throw "The installer failed with exit code $($process.ExitCode)"
        }
    }
    finally {
        Remove-Item -LiteralPath $setup -ErrorAction SilentlyContinue
    }

    Write-Host 'Successfully installed Turbo Delete. Open a new terminal to use turbo-delete.' -ForegroundColor Green
}
