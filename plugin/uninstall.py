#!/usr/bin/env python3
"""Remove the panel and its optional desktop entry; leave speech data alone."""
import os
from pathlib import Path
import subprocess
import sys


def main():
    data = Path(os.environ.get('XDG_DATA_HOME', str(Path.home()/'.local/share')))
    config = Path(os.environ.get('XDG_CONFIG_HOME', str(Path.home()/'.config')))
    desktop = data/'applications/smellytype-configure.desktop'
    target = config/'omarchy/plugins/beijingrong.smellytype/configure.py'
    # The native remover provides the user's confirmation and removes only the
    # selected plugin. Do not remove the menu if that operation is cancelled.
    subprocess.run(['omarchy', 'plugin', 'remove', 'beijingrong.smellytype'] +
                   (['--yes'] if '--yes' in sys.argv else []), check=True)
    if desktop.is_file() and str(target) in desktop.read_text():
        desktop.unlink()

if __name__ == '__main__': main()
