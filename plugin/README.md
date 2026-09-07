# SmellyType Omarchy panel

Bundled companion to SmellyType. This plugin is installed by the main installer;
its ID is `beijingrong.smellytype`. It controls `smellytype.service` and uses SmellyType's
own runtime/config/state directories. It needs no Voxtype installation.

SmellyType Configuration is a separate desktop entry and opens this same panel.
Controls: start/stop/cancel, recording cap, final-response wait, cumulative local
microphone minutes, restart, private terminal key replacement. No interim text
is typed. Key presence is not proof of authentication; local time is not billing.

For a plugin-only update, `python3 install.py` installs it into the user plugin
folder and validates/enables it. If nested QML remains cached, `omarchy restart
shell` refreshes UI without restarting speech. See ../docs/INSTALL.md for core
setup, migration, verification and rollback.
