#!/usr/bin/env python3
"""Install plugin and user-owned configuration entry; back up replaced files."""
import os
from pathlib import Path
import shutil
import subprocess
import time

source = Path(__file__).resolve().parent
home = Path.home()
config = Path(os.environ.get('XDG_CONFIG_HOME', str(home/'.config')))
data = Path(os.environ.get('XDG_DATA_HOME', str(home/'.local/share')))
dest = config/'omarchy/plugins/beijingrong.smellytype'
backup = data/'omarchy-smellytype/backups'/time.strftime('%Y%m%d-%H%M%S')

def preserve(path):
    if path.exists():
        relative = str(path).lstrip('/')
        target = backup/relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, target)

preserve(config/'omarchy/shell.json')
dest.mkdir(parents=True, exist_ok=True)
for name in ('manifest.json','BarWidget.qml','Panel.qml','bridge.py','credentials.py','configure.py'):
    preserve(dest/name)
    shutil.copy2(source/name, dest/name)
(dest/'configure.py').chmod(0o755)
desktop = data/'applications/smellytype-configure.desktop'
desktop.parent.mkdir(parents=True, exist_ok=True)
preserve(desktop)
exec_path = str(dest/'configure.py').replace('\\','\\\\').replace('"','\\"').replace('`','\\`').replace('$','\\$').replace('%','%%')
desktop.write_text('[Desktop Entry]\nType=Application\nName=SmellyType Configuration\nComment=豆包语音输入：录音时长、识别等待、密钥和服务设置\nExec="' + exec_path + '"\nIcon=audio-input-microphone\nCategories=Settings;\nKeywords=smellytype;doubao;voice;语音;豆包;\nTerminal=false\n')
for args in (['omarchy','plugin','validate',str(dest)], ['omarchy-shell','shell','rescanPlugins'], ['omarchy','plugin','enable','beijingrong.smellytype','--section','right','--before','omarchy.audio']):
    subprocess.run(args, check=True, timeout=20)
if shutil.which('update-desktop-database'):
    subprocess.run(['update-desktop-database',str(desktop.parent)],check=True)
print('Installed configuration panel. Backups:', backup)
