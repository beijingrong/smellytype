#!/usr/bin/env python3
"""Move an existing SmellyType service to graphical-session startup.

Preserves the service body, other overrides, enablement, and running dictation.
The new startup timing takes effect at the next graphical login.
"""
from pathlib import Path
import shutil
import subprocess
import tempfile


def run(*args):
    return subprocess.run(args, check=True, capture_output=True, text=True, timeout=30)


def migrate(home=None):
    home = Path.home() if home is None else Path(home)
    units = home / '.config/systemd/user'
    service = units / 'smellytype.service'
    if not service.is_file():
        raise RuntimeError('SmellyType user service is missing; follow docs/INSTALL.md first')
    enabled = subprocess.run(
        ['systemctl', '--user', 'is-enabled', 'smellytype.service'],
        capture_output=True, text=True, timeout=30,
    ).stdout.strip()
    if enabled not in ('enabled', 'disabled'):
        raise RuntimeError(f'Unsupported service enablement: {enabled}; review it manually')
    override = units / 'smellytype.service.d/90-graphical-session.conf'
    content = ('# Start after the graphical session imports its display environment.\n'
               '[Unit]\nAfter=graphical-session.target\nPartOf=graphical-session.target\n'
               '[Install]\nWantedBy=\nWantedBy=graphical-session.target\n')
    backups = home / '.local/lib/smellytype/backups'
    backups.mkdir(parents=True, exist_ok=True)
    backup = Path(tempfile.mkdtemp(prefix='service-startup-', dir=backups))
    shutil.copy2(service, backup / 'smellytype.service')
    existed = override.exists()
    if existed:
        shutil.copy2(override, backup / override.name)
    (backup / 'enablement.txt').write_text(enabled + '\n')
    override.parent.mkdir(parents=True, exist_ok=True)
    try:
        override.write_text(content)
        run('systemd-analyze', '--user', 'verify', str(service))
        run('systemctl', '--user', 'daemon-reload')
        if enabled == 'enabled':
            # enable alone leaves the old default.target symlink in place.
            run('systemctl', '--user', 'reenable', 'smellytype.service')
    except Exception:
        if existed:
            shutil.copy2(backup / override.name, override)
        else:
            override.unlink(missing_ok=True)
        run('systemctl', '--user', 'daemon-reload')
        if enabled == 'enabled':
            run('systemctl', '--user', 'reenable', 'smellytype.service')
        raise
    print(f'Startup fixed for the next graphical login. Service not restarted. Backup: {backup}')


if __name__ == '__main__':
    migrate()
