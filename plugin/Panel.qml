import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.Commons
import qs.Ui

Panel {
    id: root
    moduleName: "beijingrong.smellytype"
    manageIpc: false
    property var anchorItem: null
    property var hostWidget: null
    readonly property var info: hostWidget ? hostWidget.info : ({})
    function switchPanel(direction) {
        return root.bar && root.bar.switchPanelFrom ? root.bar.switchPanelFrom(root.hostWidget || root, direction) : false
    }
    KeyboardPanel {
        id: surface
        anchorItem: root.anchorItem
        owner: root.hostWidget || root
        bar: root.bar
        open: root.opened
        focusTarget: catcher
        contentWidth: fittedContentWidth(Style.space(370))
        contentHeight: body.implicitHeight + Style.space(40)
        PanelKeyCatcher {
            id: catcher
            anchors.fill: parent
            onCloseRequested: root.close()
            onTabRequested: function(direction) { root.switchPanel(direction) }
        }
        Column {
            id: body
            x: Style.space(20)
            y: Style.space(20)
            width: parent.width - Style.space(40)
            spacing: Style.space(14)
            Text { text: "SmellyType"; color: Color.foreground; font.pixelSize: 22; font.bold: true }
            Text {
                text: !root.info.installed || !root.info.configured ? "请先安装并配置 SmellyType 主程序" : root.info.recording ? "● 正在录音 · " + Math.floor(root.info.elapsed_ms / 1000) + " 秒"
                    : root.info.state === "offline" ? "语音服务未运行" : root.info.state === "idle" ? "就绪 · 按住 F9 说话" : "等待识别结果"
                color: Color.foreground; font.pixelSize: 14
            }
            Text {
                visible: !root.info.installed || !root.info.configured
                width: parent.width
                text: "github.com/beijingrong/smellytype"
                color: Color.foreground; font.pixelSize: 12; wrapMode: Text.Wrap
            }
            Text {
                text: ((Number(root.info.total_ms || 0) + Number(root.info.elapsed_ms || 0)) / 60000).toFixed(1) + " 分钟"
                color: Color.foreground; font.pixelSize: 32
            }
            Text {
                width: parent.width
                text: "本机累计录音 · " + (root.info.sessions || 0) + " 次\n包含取消的录音；不代表火山账单用量。"
                color: Color.foreground; opacity: 0.65; font.pixelSize: 12; wrapMode: Text.Wrap
            }
            Row {
                spacing: 10
                Repeater {
                    model: [{label: root.info.recording ? "停止并识别" : "开始录音", action: "toggle"}, {label: "取消", action: "cancel"}]
                    Rectangle {
                        required property var modelData
                        width: 140; height: 38; radius: 7
                        color: Color.foreground
                        opacity: buttons.containsMouse ? 0.85 : 1
                        Text { anchors.centerIn: parent; text: modelData.label; color: Color.background; font.pixelSize: 14 }
                        MouseArea { id: buttons; anchors.fill: parent; hoverEnabled: true; onClicked: {
                            // Close the keyboard panel first so dictation returns to the prior input.
                            const action = modelData.action; root.close()
                            Qt.callLater(function() { root.hostWidget.act([action]) })
                        } }
                    }
                }
            }
            Text { text: "单次录音上限 · 当前 " + (root.info.limit || 180) + " 秒"; color: Color.foreground; font.pixelSize: 14 }
            Row {
                spacing: 8
                Repeater {
                    model: [60, 180, 300, 600]
                    Rectangle {
                        required property int modelData
                        width: 66; height: 34; radius: 6
                        color: Color.foreground
                        opacity: root.info.state === "offline" ? "语音服务未运行" : root.info.state === "idle" ? (root.info.limit === modelData ? 1 : 0.55) : 0.2
                        Text { anchors.centerIn: parent; text: (modelData / 60) + " 分钟"; color: Color.background; font.pixelSize: 13 }
                        MouseArea { anchors.fill: parent; enabled: root.info.state === "idle"; onClicked: root.hostWidget.act(["limit", String(modelData)]) }
                    }
                }
            }
            Text { text: "豆包 · 流式识别 2.0"; color: Color.foreground; font.pixelSize: 14 }
            Text { text: "结束录音后等待 · " + (root.info.final_timeout || 60) + " 秒"; color: Color.foreground; font.pixelSize: 14 }
            Row {
                spacing: 8
                Repeater {
                    model: [30, 60, 120, 300]
                    Rectangle {
                        required property int modelData
                        width: 66; height: 34; radius: 6; color: Color.foreground
                        opacity: root.info.state === "idle" ? (root.info.final_timeout === modelData ? 1 : 0.55) : 0.2
                        Text { anchors.centerIn: parent; text: modelData + " 秒"; color: Color.background; font.pixelSize: 13 }
                        MouseArea { anchors.fill: parent; enabled: root.info.state === "idle"; onClicked: root.hostWidget.act(["timeout", String(modelData)]) }
                    }
                }
            }
            Text { text: root.info.key_configured ? "密钥文件已配置" : "尚未配置密钥"; color: Color.foreground; font.pixelSize: 13 }
            Row {
                spacing: 10
                Repeater {
                    model: [{label: "设置 / 更换密钥", action: "credentials"}, {label: "重启语音服务", action: "restart"}]
                    Rectangle {
                        required property var modelData
                        width: 140; height: 34; radius: 6; color: Color.foreground
                        opacity: root.info.state === "idle" || root.info.state === "offline" ? 0.85 : 0.2
                        Text { anchors.centerIn: parent; text: modelData.label; color: Color.background; font.pixelSize: 13 }
                        MouseArea { anchors.fill: parent; enabled: root.info.state === "idle" || root.info.state === "offline"; onClicked: {
                            const action = modelData.action
                            if (action === "credentials") root.close()
                            Qt.callLater(function() { root.hostWidget.act([action]) })
                        } }
                    }
                }
            }
            Text { width: parent.width; text: "空闲时修改并应用。松开后等待最终识别结果。\n长会话可用性取决于服务端与网络。"; color: Color.foreground; opacity: 0.65; font.pixelSize: 12; wrapMode: Text.Wrap }
            Text { width: parent.width; visible: text.length > 0; text: root.hostWidget ? root.hostWidget.message : ""; color: Color.foreground; font.pixelSize: 12; wrapMode: Text.Wrap }
        }
    }
}
