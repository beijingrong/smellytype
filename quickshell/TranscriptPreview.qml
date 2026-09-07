// Cumulative hypotheses stay in a non-focusable panel until final commit.
import QtQuick
import Quickshell
import Quickshell.Io
import Quickshell.Wayland

PanelWindow {
    id: panel
    required property string daemonState
    required property bool osdSuppressed
    required property string runtimeDir
    property string transcript: ""
    visible: daemonState === "streaming" && !osdSuppressed && transcript.length > 0
    anchors { bottom: true }
    margins.bottom: 150
    implicitWidth: 640
    implicitHeight: Math.min(220, content.implicitHeight + 32)
    color: "transparent"
    exclusionMode: ExclusionMode.Ignore
    WlrLayershell.layer: WlrLayer.Overlay
    WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
    mask: Region {}

    FileView {
        path: panel.runtimeDir + "/preview.json"
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
        onLoaded: {
            try { panel.transcript = JSON.parse(text()).text || ""; }
            catch (_) { panel.transcript = ""; }
        }
        onLoadFailed: panel.transcript = ""
    }
    Rectangle {
        anchors.fill: parent
        color: "#ee20242b"
        radius: 12
        border.color: "#668ba9c9"
        Text {
            id: content
            anchors { left: parent.left; right: parent.right; bottom: parent.bottom; margins: 16 }
            text: panel.transcript
            textFormat: Text.PlainText
            color: "#f1f3f5"
            font.pixelSize: 22
            wrapMode: Text.Wrap
            maximumLineCount: 5
            elide: Text.ElideLeft
        }
    }
}
