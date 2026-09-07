# SmellyType for Omarchy

This is the control panel for https://github.com/beijingrong/smellytype.
Canonical source is that repository's plugin/ directory; standalone exports use
this same file at their root. Read README.md. Keep commits signed.

If core dictation is missing, follow the main repository's AGENTS.md and
installation guide. Do not try to install Voxtype or inject keys into QML.
Native `omarchy plugin add` installs only this panel, not the core service.

Use the installed Omarchy skill for desktop changes; never edit /usr/share/omarchy.
Preserve user edits and active dictation. Keep the stable ID beijingrong.smellytype.
When replacing an existing bundled copy with the git distribution, preserve
local edits before removal. The uninstall helper removes only its owned desktop
entry and invokes the native plugin remover; it leaves speech settings intact.
