
<p align="center">
  <img src="https://github.com/XtremeDevX/turbo-delete/blob/master/assets/turbo-delete-icon-transparent-bg.png" />
</p>

<h1 align="center">Turbo Delete</h1>
<h4 align="center">A blazing fast alternative to the default Windows delete.</h1>
<br>

![comparison](https://user-images.githubusercontent.com/63039748/154819625-dce22227-0415-411f-8b53-708566ace569.gif)

Turbodelete is a blazing fast alternative to the default Windows delete function.

> TurboDelete integrates with your context menu as well as the command-line!

<br>

# :zap: Installation

### Installer (recommended)

Download **[Turbo-Delete-Install.exe](https://github.tobse.eu/turbo-delete/Turbo-Delete-Install.exe)** and double-click it.

- Installs for the current user to `%LOCALAPPDATA%\Programs\Turbo Delete`, no admin rights needed.
- Adds "Turbo Delete" to the Explorer context menu of folders and files. On Windows 11 it is in the classic menu ("Show more options" or Shift+F10).
- Adds `turbo-delete` to the PATH, so it works in every new terminal.
- Both can be deselected during setup. On the last page Explorer can be restarted, which is usually not needed.
- Uninstall it in Settings > Apps, which also removes the context menu entry and the PATH entry.

<br>

### Install using PowerShell

Runs the same installer silently with all options:

```powershell
iwr -useb https://github.tobse.eu/turbo-delete/install.ps1 | iex
```

<br>

### Build from source

Requires Rust 1.89 or newer:

```powershell
cargo install --path . --locked
```

`--locked` is needed, without it cargo picks newer dependency versions that do not compile.

To add the context menu entry for this build, run `.\dist\register-context-menu.ps1` in a regular PowerShell window. `.\dist\unregister-context-menu.ps1` removes it again. Do not run it from a terminal inside an app installed as MSIX package: those apps write `HKEY_CURRENT_USER\Software\Classes` into a private copy of the registry, so Explorer never sees the entry.

<br>

### Building a release

```powershell
winget install JRSoftware.InnoSetup
python automate-release.py
```

This builds `bin\turbo-delete.exe` and the installer `dist\Turbo-Delete-Install.exe` with the version from `Cargo.toml`.

<br>

# :wrench: Usage

```powershell
turbo-delete [--contents] [--confirm] <path>...
```

- Accepts several folders and files at once.
- `--contents` deletes only the contents of the folders and keeps the folders themselves, e.g. `turbo-delete --contents $env:TEMP`.
- `--confirm` asks before deleting and keeps the window open on errors. Calls started at the same time (like the context menu does for several selected items) are merged into one confirmation.
- Junctions and symlinks are removed as links, their targets are never touched.
- Files that are in use or not accessible are skipped. The number of remaining items is reported with exit code 1.
- Drive roots like `C:\` are refused.
- Everything is deleted permanently, nothing goes to the recycle bin.

<br>
<br>


## Authors

[Tejas](https://www.github.com/suptejas) 

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/suptejas/turbo-delete/tags).

## License

This project is licensed under the Apache 2.0 License - see the [LICENSE](LICENSE.txt) file for details.
