# SmellyType for Omarchy

An Omarchy Quattro bar plugin for [SmellyType](https://github.com/beijingrong/smellytype), the standalone Doubao cloud dictation client.

点击状态栏小猫按钮，控制语音输入、调整录音上限和识别等待时间、查看累计录音分钟数。

## Requirements

- Omarchy's Quickshell-based Shell (Quattro plugin API), Python 3.11+.
- SmellyType 0.1.2 or newer, installed and configured with an active user service.
- For key replacement: `xdg-terminal-exec` and a configured terminal.

**This plugin is the desktop control panel, not the speech engine.** If SmellyType is missing, the panel shows installation guidance. Give your Agent https://github.com/beijingrong/smellytype and ask it to follow `AGENTS.md`. Its installer builds the core and includes this panel, so you do not need to install both copies.

## Install the plugin separately

Review the source, then:

```sh
omarchy plugin add https://github.com/beijingrong/omarchy-smellytype.git --enable
```

If the bundled panel is already installed, keep using it; the ID is deliberately the same (`beijingrong.smellytype`) to prevent duplicate controls. To switch that copy to the standalone git distribution, back up any local edits, remove the existing plugin, then run the command above. The non-git folder is backed up by Omarchy's removal command. Removing the panel does not stop dictation.

Optional applications-menu entry (run from the plugin directory):

```sh
python3 install.py
```

This explicitly installs or updates **SmellyType Configuration** in the user applications directory, backs up existing managed files/bar layout, and enables the widget beside audio. It does not edit package-owned files or restart the speech service. The native `omarchy plugin add` command does not automatically execute this helper.

## Use and configure

Click the cat button to open the panel; Escape closes it. F9 still belongs to the core client's installation. The panel closes before recording starts/stops so final text can return to the previous application.

- Start, stop and cancel recording.
- Per-recording limit: 1, 3, 5 or 10 minutes.
- Final-response wait after audio ends: 30, 60, 120 or 300 seconds.
- Cumulative microphone time and session count, persisted by SmellyType.
- Service status/restart and private terminal API-key entry.
- Personal vocabulary (one term per line, sent to Doubao with each recording) and optional speech cleanup. Vocabulary hints are not guaranteed corrections; cleanup may remove expressive fillers.

Settings restart `smellytype.service` only while idle and restore prior settings on failure. Service command-line/environment overrides may take precedence over config values. Key replacement uses hidden terminal input and mode-0600 files; keys never enter QML. File presence does not verify cloud authentication.

Local minutes include cancelled/failed recordings and exclude final-response waiting; they are not provider billing or remaining free allowance. The plugin neither records audio itself nor stores transcripts.

```sh
omarchy bar move beijingrong.smellytype --section right
omarchy-shell beijingrong.smellytype open
omarchy-shell beijingrong.smellytype close
omarchy plugin update beijingrong.smellytype
```

Updates require a git-installed copy. If nested QML remains cached, `omarchy restart shell` reloads the panel without restarting speech.

## Remove

From the plugin directory, use the helper to remove its optional desktop entry and then remove the plugin:

```sh
python3 uninstall.py
```

Or remove only the bar plugin with `omarchy plugin remove beijingrong.smellytype`. If you installed the optional desktop entry, it will no longer work after removal; the helper removes it only when it still points at this plugin.

SmellyType's service, F9 bindings, config, credentials and usage history remain intact. Removal does not remove the speech client. Use the main repository's rollback instructions for that.

## Development and provenance

Canonical source: [smellytype/plugin](https://github.com/beijingrong/smellytype/tree/main/plugin). This repository is the standalone distribution exported from that directory; submit code changes to the main repository to avoid divergence. `SOURCE.md` records the exported revision.

```sh
python3 -m unittest discover -s tests -v
omarchy plugin validate .
```

MIT licensed. QML follows the Omarchy Quattro bar/panel contract; the desktop plugin runs inside the existing Shell process. It does not launch a second Shell. Its separate speech daemon manages its own recording OSD.
