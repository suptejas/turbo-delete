
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

### Build from source

Requires Rust 1.89 or newer:

```powershell
cargo install --path . --locked
```

`--locked` is needed, without it cargo picks newer dependency versions that do not compile.

<br>

### Context menu

Run this in a regular PowerShell window to add a "Turbo Delete" entry for folders and files (current user only, no admin rights needed):

```powershell
.\dist\register-context-menu.ps1
```

On Windows 11 the entry is in the classic menu ("Show more options" or Shift+F10). If several items are selected, they are deleted after a single confirmation. `.\dist\unregister-context-menu.ps1` removes the entry again.

Do not run the script from a terminal inside an app installed as MSIX package: those apps write `HKEY_CURRENT_USER\Software\Classes` into a private copy of the registry, so Explorer never sees the entry.

<br>

### Install Using Powershell

The installer below downloads the prebuilt v0.0.1, which does not contain the changes described in [Usage](#usage).

First you'll need to set your execution policy to RemoteSigned if not done so:

```powershell
Set-ExecutionPolicy RemoteSigned
```

Run the following command on your powershell window:

```powershell
iwr -useb cdn.xtremedevx.com/turbo-delete/install.ps1 | iex
```

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

## :clap: Supporters

![Stargazers repo roster for @suptejas/turbo-delete](https://reporoster.com/stars/suptejas/turbo-delete)

<br>


## Authors

[Tejas](https://www.github.com/suptejas) 

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/suptejas/turbo-delete/tags).

## License

This project is licensed under the Apache 2.0 License - see the [LICENSE](LICENSE.txt) file for details.
