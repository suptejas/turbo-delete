import winreg

winreg.DeleteKey(winreg.HKEY_CLASSES_ROOT,
                 r'Directory\shell\turbo-delete\command')
winreg.DeleteKey(winreg.HKEY_CLASSES_ROOT, r'Directory\shell\turbo-delete')
