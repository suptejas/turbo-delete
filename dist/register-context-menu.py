import winreg
import os

appdata_dir = os.getenv('APPDATA') + r'\turbo-delete\bin\td.exe'
winreg.CreateKey(winreg.HKEY_CLASSES_ROOT, R'Directory\shell\turbo-delete')
winreg.CreateKey(winreg.HKEY_CLASSES_ROOT,
                 R'Directory\shell\turbo-delete\command')

delete_key = winreg.OpenKey(
    winreg.HKEY_CLASSES_ROOT, R'Directory\shell\turbo-delete', 0, winreg.KEY_ALL_ACCESS)
winreg.SetValueEx(delete_key, '', 0, winreg.REG_SZ, 'Turbo Delete')
delete_key.Close()


command_key = winreg.OpenKey(
    winreg.HKEY_CLASSES_ROOT, R'Directory\shell\turbo-delete\command', 0, winreg.KEY_ALL_ACCESS)
winreg.SetValueEx(command_key, '', 0, winreg.REG_SZ, Rf'"{appdata_dir}" "%1"')
command_key.Close()
