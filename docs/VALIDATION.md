# Initial trial validation (2026-09-07)

- Full Rust test suite: 1126 passed, 2 ignored.
- Clippy all targets: clean.
- Plugin bridge/private key helper: 8 mocked tests passed.
- Installer migration and rollback: isolated-home test passed.
- Public 5.5-second Mandarin fixture recognized through the real Doubao service
  by the independently named SmellyType binary; no text injected into an app.
- Live Omarchy migration: independent service ready, original Voxtype disabled,
  F9 press/release and toggle bound to SmellyType, desktop config errors empty.
- Recording limit, final wait, key and cumulative minutes preserved.
- Distinct SmellyType Configuration desktop entry and panel rendering checked.
- Source scan found no occurrence of the user's private API key.

The first migration attempt was automatically rolled back because the quoted
EnvironmentFile path was rejected by systemd. The installer now writes the
absolute path correctly and verifies the unit before switching services.

The migrated microphone-to-application flow still needs user trial. The current
local installation uses the optimized dev profile, not a release build. This is a source preview release; an Omarchy plugin-directory listing remains
pending.

## Rename to SmellyType

The user confirmed the YunType microphone-to-application flow worked smoothly
before requesting the new name. After renaming, the full Rust suite (1126 pass,
2 ignored), Clippy, 8 plugin tests, and both Voxtype/YunType migration rollback
tests passed. The migration replaces the old YunType F9 block instead of
accumulating duplicate bindings.

## Security maintenance (0.1.1, 2026-09-07)

A static review of the supported Doubao path, output drivers, runtime files and
installer/plugin identified four issues: default full-transcript logging,
dotool command-stream injection, unverified shared dotool FIFO discovery, and
an unsafe shared runtime fallback. These were addressed in 0.1.1 (plugin 0.1.2).
The review mapped the repository but did not exhaustively audit all inherited
optional engines, platforms or dependencies; it is not a guarantee of no bugs.

- Full Rust suite after fixes: 1130 passed, 2 ignored.
- Clippy all targets with warnings denied: passed.
- Plugin tests: 10 passed; isolated installer migration/rollback tests: 2 passed.
- Added regression cases for control-character rejection, runtime permissions
  and symlink rejection. No real desktop key-injection payload was executed.
- Local mock WebSocket tests exercise final-only output, cancellation, timeout
  and compressed-response limits without sending recordings to a cloud service.
- Checked tracked files in both public repositories against the locally stored
  API key and high-confidence secret patterns; no matches found.
