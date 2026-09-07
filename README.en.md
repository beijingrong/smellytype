# SmellyType

[简体中文](README.md) | **English**

Doubao cloud dictation for Omarchy / Hyprland. Hold **F9** to speak, then release to insert the recognized text into the current application.

**SmellyType is a fork of [Voxtype](https://github.com/peteonrails/voxtype) and directly reuses substantial upstream code.** Thank you to **Peter Jackson (peteonrails) and all Voxtype contributors**: audio capture, the recording and recognition lifecycle, text output, and on-screen display (OSD) are built on their work. SmellyType adds Doubao cloud recognition, recording usage tracking, and an Omarchy control panel.

## Features

- **Doubao streaming ASR 2.0**: see live previews while speaking; only the final text is inserted, once recording ends.
- **Keyboard shortcuts**: hold F9 to record, or use Super+Ctrl+X to toggle recording.
- **Status bar panel**: start, stop or cancel recording, and adjust the recording limit and recognition timeout.
- **Recording statistics**: view cumulative microphone minutes and session counts on this machine.

## Install

Give your Agent this prompt:

> Please install https://github.com/beijingrong/smellytype for me. Read AGENTS.md and docs/INSTALL.md in the repository first.

You need an Omarchy / Hyprland environment with Quickshell, an internet connection, and a Volcengine API key with Doubao streaming speech recognition enabled. Enter the key locally; do not share it in chat or commit it to the repository.

The current release is **0.1.1 preview**, installed by building from source. See the [installation guide](docs/INSTALL.md) for dependencies, installation, updates and migration. You do not need to install Voxtype separately or download a local speech model.

The installer includes the control panel. Existing SmellyType users can also install and update the plugin separately through [SmellyType for Omarchy](https://github.com/beijingrong/omarchy-smellytype). Click the **S** button in the status bar or open **SmellyType Configuration** to access the panel.

## Usage notes

- Recordings are sent to Doubao for cloud recognition. Service activation, quotas and billing are managed by Volcengine.
- The recording limit and the recognition timeout after recording ends are separate settings.
- Cumulative minutes measure local microphone use, including cancelled or failed recordings. They do not represent cloud billing or your remaining free allowance.

## Acknowledgements and license

Voxtype is the code foundation of SmellyType. We retain its original MIT copyright notice and continue to publish under the **MIT** license. See [LICENSE](LICENSE), [source and modification notices](NOTICE), and [third-party notices](THIRD_PARTY.md). This project is independently maintained and is not officially affiliated with Voxtype, Omarchy or Volcengine.

If you prefer local speech models, please explore and support the upstream project, **[Voxtype](https://github.com/peteonrails/voxtype)**.
