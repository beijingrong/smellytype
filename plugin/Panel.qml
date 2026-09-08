import QtQuick
import QtQuick.Controls
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
    property bool editingWords: false
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
        contentWidth: fittedContentWidth(Style.space(380))
        contentHeight: surface.fittedContentHeight(body.implicitHeight)
        PanelKeyCatcher {
            id: catcher
            anchors.fill: parent
            onCloseRequested: root.close()
            onTabRequested: function(direction) { root.switchPanel(direction) }
        }
        ScrollView {
            id: scroll
            anchors.fill: parent
            clip: true
            ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
            Column {
                id: body
                width: scroll.availableWidth
                spacing: Style.space(14)
                Text { text: "SmellyType"; color: Color.foreground; font.family: root.bar ? root.bar.fontFamily : Style.font.family; font.pixelSize: Style.font.title; font.bold: true }
                Text {
                    text: !root.info.installed || !root.info.configured ? "请先安装并配置 SmellyType 主程序" : root.info.recording ? "● 正在录音 · " + Math.floor(root.info.elapsed_ms / 1000) + " 秒"
                        : root.info.state === "offline" ? "语音服务未运行" : root.info.state === "idle" ? "就绪 · 按住 F9 说话" : "等待识别结果"
                    color: Color.foreground; font.family: root.bar ? root.bar.fontFamily : Style.font.family; font.pixelSize: Style.font.body
                }
                Text {
                    visible: !root.info.installed || !root.info.configured
                    width: parent.width
                    text: "github.com/beijingrong/smellytype"
                    color: Color.foreground; font.family: root.bar ? root.bar.fontFamily : Style.font.family; font.pixelSize: Style.font.caption; wrapMode: Text.Wrap
                }
                Text {
                    text: ((Number(root.info.total_ms || 0) + Number(root.info.elapsed_ms || 0)) / 60000).toFixed(1) + " 分钟"
                    color: Color.foreground; font.family: root.bar ? root.bar.fontFamily : Style.font.family; font.pixelSize: Style.font.display
                }
                Text {
                    width: parent.width
                    text: "本机累计录音 · " + (root.info.sessions || 0) + " 次\n包含取消的录音；不代表火山账单用量。"
                    color: Color.foreground; opacity: 0.65; font.family: root.bar ? root.bar.fontFamily : Style.font.family; font.pixelSize: Style.font.caption; wrapMode: Text.Wrap
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
                            Text { anchors.centerIn: parent; text: modelData.label; color: Color.background; font.family: root.bar ? root.bar.fontFamily : Style.font.family; font.pixelSize: Style.font.body }
                            MouseArea { id: buttons; anchors.fill: parent; hoverEnabled: true; onClicked: {
                                // Close the keyboard panel first so dictation returns to the prior input.
                                const action = modelData.action; root.close()
                                Qt.callLater(function() { root.hostWidget.act([action]) })
                            } }
                        }
                    }
                }
                Text { text: "单次录音上限 · 当前 " + (root.info.limit || 180) + " 秒"; color: Color.foreground; font.family: root.bar ? root.bar.fontFamily : Style.font.family; font.pixelSize: Style.font.body }
                Row {
                    spacing: 8
                    Repeater {
                        model: [60, 180, 300, 600]
                        Rectangle {
                            required property int modelData
                            width: 66; height: 34; radius: 6
                            color: Color.foreground
                            opacity: root.info.state === "idle" ? (root.info.limit === modelData ? 1 : 0.55) : 0.2
                            Text { anchors.centerIn: parent; text: (modelData / 60) + " 分钟"; color: Color.background; font.family: root.bar ? root.bar.fontFamily : Style.font.family; font.pixelSize: Style.font.bodySmall }
                            MouseArea { anchors.fill: parent; enabled: root.info.state === "idle"; onClicked: root.hostWidget.act(["limit", String(modelData)]) }
                        }
                    }
                }
                Text { text: "豆包 · 流式识别 2.0"; color: Color.foreground; font.family: root.bar ? root.bar.fontFamily : Style.font.family; font.pixelSize: Style.font.body }
                Text { text: "结束录音后等待 · " + (root.info.final_timeout || 60) + " 秒"; color: Color.foreground; font.family: root.bar ? root.bar.fontFamily : Style.font.family; font.pixelSize: Style.font.body }
                Row {
                    spacing: 8
                    Repeater {
                        model: [30, 60, 120, 300]
                        Rectangle {
                            required property int modelData
                            width: 66; height: 34; radius: 6; color: Color.foreground
                            opacity: root.info.state === "idle" ? (root.info.final_timeout === modelData ? 1 : 0.55) : 0.2
                            Text { anchors.centerIn: parent; text: modelData + " 秒"; color: Color.background; font.family: root.bar ? root.bar.fontFamily : Style.font.family; font.pixelSize: Style.font.bodySmall }
                            MouseArea { anchors.fill: parent; enabled: root.info.state === "idle"; onClicked: root.hostWidget.act(["timeout", String(modelData)]) }
                        }
                    }
                }
                Row {
                    spacing: Style.space(8)
                    Rectangle {
                        width: Style.space(140); height: Style.space(34); radius: 6
                        color: Color.foreground
                        opacity: root.info.state === "idle" ? 0.85 : 0.3
                        Text { anchors.centerIn: parent; text: "口语整理 · " + (root.info.enable_ddc ? "开" : "关"); color: Color.background; font.family: Style.font.family; font.pixelSize: Style.font.bodySmall }
                        MouseArea { anchors.fill: parent; enabled: root.info.state === "idle"; onClicked: root.hostWidget.act(["cleanup", root.info.enable_ddc ? "false" : "true"]) }
                    }
                    Rectangle {
                        width: Style.space(140); height: Style.space(34); radius: 6
                        color: Color.foreground; opacity: 0.85
                        Text { anchors.centerIn: parent; text: "个人词库 · 编辑"; color: Color.background; font.family: Style.font.family; font.pixelSize: Style.font.bodySmall }
                        MouseArea { anchors.fill: parent; onClicked: { if (!root.editingWords) words.text = root.info.hotwords || ""; root.editingWords = !root.editingWords } }
                    }
                }
                Text { width: parent.width; text: "整理会减少结巴和语气词，也可能删去表达语气。"; color: Color.foreground; opacity: 0.65; font.family: Style.font.family; font.pixelSize: Style.font.caption; wrapMode: Text.Wrap }
                Column {
                    visible: root.editingWords
                    width: parent.width
                    spacing: Style.space(8)
                    Text { width: parent.width; text: "每行一个词，重要的放前面；最多 50 个。\n每次识别会将词库发送给豆包，热词不保证命中。"; color: Color.foreground; font.family: Style.font.family; font.pixelSize: Style.font.caption; wrapMode: Text.Wrap }
                    TextArea {
                        id: words
                        width: parent.width
                        wrapMode: TextEdit.Wrap
                        placeholderText: "SmellyType\nOmarchy"
                        color: Color.foreground
                        selectionColor: Color.foreground
                        selectedTextColor: Color.background
                        font.family: Style.font.family
                        font.pixelSize: Style.font.body
                        background: Rectangle { color: Color.background; border.color: Color.foreground; opacity: 0.5; radius: 4 }
                    }
                    Rectangle {
                        width: Style.space(140); height: Style.space(34); radius: 6
                        color: Color.foreground; opacity: root.info.state === "idle" ? 0.85 : 0.3
                        Text { anchors.centerIn: parent; text: "保存词库"; color: Color.background; font.family: Style.font.family; font.pixelSize: Style.font.bodySmall }
                        MouseArea { anchors.fill: parent; enabled: root.info.state === "idle"; onClicked: root.hostWidget.act(["hotwords"], words.text) }
                    }
                }
                Text { text: root.info.key_configured ? "密钥文件已配置" : "尚未配置密钥"; color: Color.foreground; font.family: root.bar ? root.bar.fontFamily : Style.font.family; font.pixelSize: Style.font.bodySmall }
                Row {
                    spacing: 10
                    Repeater {
                        model: [{label: "设置 / 更换密钥", action: "credentials"}, {label: "重启语音服务", action: "restart"}]
                        Rectangle {
                            required property var modelData
                            width: 140; height: 34; radius: 6; color: Color.foreground
                            opacity: root.info.state === "idle" || root.info.state === "offline" ? 0.85 : 0.2
                            Text { anchors.centerIn: parent; text: modelData.label; color: Color.background; font.family: root.bar ? root.bar.fontFamily : Style.font.family; font.pixelSize: Style.font.bodySmall }
                            MouseArea { anchors.fill: parent; enabled: root.info.state === "idle" || root.info.state === "offline"; onClicked: {
                                const action = modelData.action
                                if (action === "credentials") root.close()
                                Qt.callLater(function() { root.hostWidget.act([action]) })
                            } }
                        }
                    }
                }
                Text { width: parent.width; text: "空闲时修改并应用。松开后等待最终识别结果。\n长会话可用性取决于服务端与网络。"; color: Color.foreground; opacity: 0.65; font.family: root.bar ? root.bar.fontFamily : Style.font.family; font.pixelSize: Style.font.caption; wrapMode: Text.Wrap }
                Text { width: parent.width; visible: text.length > 0; text: root.hostWidget ? root.hostWidget.message : ""; color: Color.foreground; font.family: root.bar ? root.bar.fontFamily : Style.font.family; font.pixelSize: Style.font.caption; wrapMode: Text.Wrap }
            }
    }
    }
}
