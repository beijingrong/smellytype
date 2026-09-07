# Install SmellyType

Supported environment for this preview: Omarchy with Quickshell and Hyprland Lua config,
standard `~/.config`, `~/.local/share`, `~/.local/state` directories. Rust/Cargo,
Clang, CMake, pkgconf, alsa-lib, Python 3.11+, Quickshell, wtype and systemd user
services are required. No Voxtype package or local speech model is required.
Inherited Whisper build dependencies remain in this initial fork.

## Get the source

```sh
git clone https://github.com/beijingrong/smellytype.git
cd smellytype
```

## Build

```sh
cargo build --release --bin smellytype --bin smellytype-osd --bin smellytype-osd-quickshell --bin smellytype-audio-bridge
cargo test --lib
python3 -m unittest discover -s plugin/tests -v
```

For an optimized dev build use `cargo build` without `--release`, then pass
`--artifacts target/debug` to the installer. Optional OpenVINO git patches can
still be fetched by Cargo; there is no need to install an OpenVINO runtime.

## Fresh activation

Privately create `~/.config/smellytype/doubao.env` with mode 0600, containing
`DOUBAO_API_KEY="..."`. Use a hidden prompt in a user-visible terminal; never put
the real key in chat, arguments or source. Activate streaming ASR 2.0 in the
speech console first. A general chat-model key is not proof of speech access.

```sh
python3 scripts/install.py --activate
```

The installer creates its own service and config. It takes over F9 and
Super+Ctrl+X with a clearly marked block in the user's Hyprland bindings.
If Voxtype was running it is stopped, because both must not listen to the same
shortcuts. Existing files and service state are saved for rollback.

## Migrate from the earlier Voxtype + Doubao fork

```sh
python3 scripts/install.py --activate --migrate-voxtype
```

This additionally copies existing settings, the private key and cumulative
minutes into SmellyType's own directories. It disables the old companion plugin,
removes only the old custom Voxtype desktop override if it is ours, and removes
the old `90-doubao.conf` service override (backed up). Stock Voxtype remains
installed and disabled; its own Configuration entry becomes visible again.
The original key and usage file are preserved.

SmellyType Configuration is a distinct menu entry opening the SmellyType panel.
All current speech still uses Doubao; a local-model workflow should use the
original Voxtype application and explicitly switch shortcuts/services back.

## Verify and roll back

Check `systemctl --user is-active smellytype.service`, F9 bindings, settings-menu
launch, preview, cancellation and exactly one final insertion. Automated tests
and service readiness alone do not verify microphone input or cloud access.
For automated recognition use only a public fixture or otherwise authorized
audio using `smellytype transcribe <fixture.wav>`, which prints its result instead
of typing into another application.

```sh
python3 scripts/install.py --rollback
```

Rollback restores snapshotted config, desktop entry, bar layout, bindings and old
service state. It leaves staged SmellyType binaries for inspection. Review any
changes since activation before rolling back; restoring old snapshots would
replace those later edits. Do not delete SmellyType's config/history manually
before deciding whether to preserve recordings made after migration.

For updates, wait for idle, stop `smellytype.service`, stage using the installer
without `--activate`, update the plugin via `python3 plugin/install.py` if needed,
then restart the service. Preserve credentials, migration.json and backups.

## Rename an existing YunType installation

Run `python3 scripts/install.py --activate --migrate-yuntype` (add `--artifacts`
for a dev build). This copies the current YunType settings, private key and
usage, replaces its marked F9 binding block, and disables the old service and
plugin. SmellyType Configuration replaces the old YunType menu entry. Original
YunType files/binaries remain for rollback; Voxtype is unaffected by this path.
