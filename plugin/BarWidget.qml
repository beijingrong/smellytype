import QtQuick
import QtQuick.Effects
import Quickshell
import Quickshell.Io
import qs.Ui

BarWidget {
    id: root
    moduleName: "beijingrong.smellytype"
    property var info: ({state: "offline", recording: false, limit: 180, total_ms: 0, elapsed_ms: 0, sessions: 0})
    property string message: ""
    readonly property string bridge: Qt.resolvedUrl("bridge.py").toString().replace(/^file:\/\//, "")
    readonly property bool opened: popup.item ? popup.item.opened : false
    readonly property bool popoutSwitchClosing: popup.item ? popup.item.popoutSwitchClosing : false
    function open() { if (popup.item) popup.item.open() }
    function close() { if (popup.item) popup.item.close() }
    function toggle() { if (popup.item) popup.item.toggle() }
    function closeForPopoutSwitch() { if (popup.item) popup.item.closeForPopoutSwitch() }
    function inject() {
        if (!popup.item) return
        popup.item.bar = root.bar
        popup.item.anchorItem = button
        popup.item.hostWidget = root
    }
    function refresh() { if (!poll.running) poll.running = true }
    function act(args) {
        if (action.running) return
        message = ""
        action.command = ["python3", bridge].concat(args)
        action.running = true
    }
    onBarChanged: inject()
    implicitWidth: button.implicitWidth
    implicitHeight: button.implicitHeight
    Process {
        id: poll
        command: ["python3", root.bridge, "status"]
        stdout: StdioCollector { onStreamFinished: {
            try { const value = JSON.parse(text); if (value.ok) root.info = value; else root.message = value.error }
            catch (_) { root.message = "无法读取语音输入状态" }
        } }
    }
    Process {
        id: action
        stdout: StdioCollector { onStreamFinished: {
            try { const value = JSON.parse(text); root.message = value.ok ? (value.message || "") : value.error }
            catch (_) { root.message = "操作未完成" }
            root.refresh()
        } }
    }
    Timer { interval: 2000; running: true; repeat: true; triggeredOnStart: true; onTriggered: root.refresh() }
    Loader {
        id: popup
        source: Qt.resolvedUrl("Panel.qml")
        active: true
        visible: false
        onLoaded: { root.inject(); Qt.callLater(root.inject) }
    }
    IpcHandler {
        target: "beijingrong.smellytype"
        function open(): void { root.open() }
        function close(): void { root.close() }
        function toggle(): void { root.toggle() }
    }
    WidgetButton {
        id: button
        anchors.fill: parent
        bar: root.bar
        hasVisualContent: true
        labelVisible: false
        fixedWidth: root.info.recording ? 44 : 34
        Image {
            id: cat
            anchors.centerIn: parent
            anchors.horizontalCenterOffset: root.info.recording ? 4 : 0
            width: Math.min(button.barSize - 4, 24)
            height: width
            source: Qt.resolvedUrl("cat-sitting.svg")
            sourceSize.width: 96
            sourceSize.height: 96
            fillMode: Image.PreserveAspectFit
            visible: false
            layer.enabled: true
        }
        MultiEffect {
            anchors.fill: cat
            source: cat
            colorization: 1.0
            colorizationColor: button.foreground
        }
        Rectangle {
            visible: root.info.recording
            width: 4; height: 4; radius: 2
            anchors.verticalCenter: parent.verticalCenter
            x: cat.x - 7
            color: button.foreground
        }
        tooltipText: "SmellyType · 语音输入 · 本机累计 " + ((root.info.total_ms + root.info.elapsed_ms) / 60000).toFixed(1) + " 分钟"
        onPressed: function(b) { if (b === Qt.LeftButton) root.toggle() }
    }
}
