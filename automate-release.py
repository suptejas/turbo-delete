import shutil
import os

# Pre-requisites: iscc.exe (Inno Setup), pyinstaller.exe (PyInstaller)

# Compile turbo-delete.exe
os.system('cargo build --profile release-optimized')

# Complile register-context-menu.py
os.system(r'pyinstaller --onefile dist\register-context-menu.py')

# Complile unregister-context-menu.py
os.system(r'pyinstaller --onefile dist\unregister-context-menu.py')

shutil.move(r'dist\register-context-menu.exe', r'bin\register-context-menu.exe')
shutil.move(r'dist\unregister-context-menu.exe', r'bin\unregister-context-menu.exe')

# Copy turbo-delete.exe from the target/release-optimized folder
shutil.copyfile(r'target\release-optimized\turbo-delete.exe', r'bin\td.exe')

os.system(r'iscc.exe dist\turbo-delete.iss')

# Cleanup
os.remove('register-context-menu.spec')
os.remove('unregister-context-menu.spec')
shutil.rmtree(r'build')
shutil.rmtree(r'dist\__pycache__')
