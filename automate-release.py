"""Builds the release files that are committed to the repository.

    bin/turbo-delete.exe            the program, from target/release-optimized
    dist/Turbo-Delete-Install.exe   the installer, built by Inno Setup from dist/turbo-delete.iss

Pre-requisites: Rust (cargo) and Inno Setup 6 (iscc.exe), e.g.
    winget install JRSoftware.InnoSetup
"""

import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
EXE = 'turbo-delete.exe'


def find_iscc() -> str:
    found = shutil.which('iscc')
    if found:
        return found

    candidates = [
        Path(os.environ.get('ProgramFiles(x86)', r'C:\Program Files (x86)')) / 'Inno Setup 6' / 'ISCC.exe',
        Path(os.environ.get('ProgramFiles', r'C:\Program Files')) / 'Inno Setup 6' / 'ISCC.exe',
        Path(os.environ.get('LOCALAPPDATA', '')) / 'Programs' / 'Inno Setup 6' / 'ISCC.exe',
    ]
    for candidate in candidates:
        if candidate.is_file():
            return str(candidate)

    sys.exit('iscc.exe not found. Install Inno Setup 6, e.g. with: winget install JRSoftware.InnoSetup')


def cargo_version() -> str:
    cargo_toml = (ROOT / 'Cargo.toml').read_text(encoding='utf-8')
    return re.search(r'^version\s*=\s*"([^"]+)"', cargo_toml, re.MULTILINE).group(1)


def run(*args: str) -> None:
    print('>', ' '.join(args), flush=True)
    subprocess.run(args, cwd=ROOT, check=True)


def main() -> None:
    iscc = find_iscc()
    version = cargo_version()

    # --locked: newer versions of the dependencies do not compile
    run('cargo', 'build', '--profile', 'release-optimized', '--locked')

    (ROOT / 'bin').mkdir(exist_ok=True)
    shutil.copyfile(ROOT / 'target' / 'release-optimized' / EXE, ROOT / 'bin' / EXE)

    run(iscc, '/Q', f'/DMyAppVersion={version}', str(ROOT / 'dist' / 'turbo-delete.iss'))

    print(f'Built Turbo Delete {version}:')
    for path in (ROOT / 'bin' / EXE, ROOT / 'dist' / 'Turbo-Delete-Install.exe'):
        print(f'  {path.relative_to(ROOT)} ({path.stat().st_size / 1024 / 1024:.1f} MB)')


if __name__ == '__main__':
    main()
