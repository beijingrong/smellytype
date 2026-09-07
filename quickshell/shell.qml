// SmellyType Quickshell shell entry point.
//
// Run standalone for testing:
//   qs -p quickshell
//
// Quickshell treats this file as the entry of a config directory. The
// shared theme/state/audio modules live at `smellytype-shared/` (sibling
// dir).
//
// This file is intentionally thin: each Wave 2 feature is its own QML
// component (OsdSurface, EnginePicker, MeetingControls) and `ShellRoot`
// is just the composition root that wires the shared state and audio
// bridges into each widget that needs them.
//
// All three Wave 2 widgets now live in the ShellRoot:
//   - OsdSurface: state-driven recording HUD. Reads daemonState +
//     audio bridge to render the waveform.
//   - EnginePicker: floating engine switcher, toggled via the
//     `$XDG_RUNTIME_DIR/smellytype/engine-picker.flag` file. Self-sources
//     its state from config.toml; doesn't need the daemon state or
//     audio bridge.
//   - MeetingControls: meeting status + start/stop/pause/resume,
//     toggled via the `$XDG_RUNTIME_DIR/smellytype/meeting-controls.flag`
//     file. Self-sources state from
//     `$XDG_RUNTIME_DIR/smellytype/meeting_state` and `smellytype meeting
//     show`; doesn't need the daemon state or audio bridge either.
//
// State is sourced from the daemon's state file at
// $XDG_RUNTIME_DIR/smellytype/state. Audio frames are sourced from the
// `smellytype-audio-bridge` sidecar which reads
// $XDG_RUNTIME_DIR/smellytype/audio.sock and emits NDJSON on stdout.

import QtQuick
import Quickshell
import "smellytype-shared" as VT

ShellRoot {
    id: shell

    VT.StateReader {
        id: stateReader
    }

    VT.AudioBridge {
        id: audio
    }

    VT.StyleLoader {
        id: osdStyle
    }

    OsdSurface {
        id: osd
        daemonState: stateReader.state
        osdSuppressed: stateReader.osdSuppressed
        audio: audio
        style: osdStyle
    }

    TranscriptPreview {
        daemonState: stateReader.state
        osdSuppressed: stateReader.osdSuppressed
        runtimeDir: stateReader.statePath.substring(0, stateReader.statePath.lastIndexOf("/"))
    }

    EnginePicker {
        id: enginePicker
    }

    MeetingControls {
        id: meetingControls
    }
}
