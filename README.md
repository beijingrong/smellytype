# SmellyType

面向 Omarchy 的豆包云端语音输入：按住 F9 说话，松开后输入最终文字。
支持实时预览、时长设置和累计录音统计，无需安装原版 Voxtype 或下载本地模型。

把下面这句话发给你的 Agent：

> 请帮我安装 https://github.com/beijingrong/smellytype ，先阅读仓库的 AGENTS.md。

首次使用仍需你开通豆包流式语音服务，并在本机填写 API Key。


Independent cloud dictation for Omarchy / Hyprland, initially using Doubao
streaming ASR 2.0. Version 0.1.0 is an early preview.

Hold F9 to speak, release for final text. Preview revisions are displayed, not
typed into the application. The Omarchy panel provides recording controls,
limits, final-response timeout, local usage, service controls and private key entry.

Give an Agent this repository and say **“Install SmellyType for me.”** Start at
[AGENTS.md](AGENTS.md) and [docs/INSTALL.md](docs/INSTALL.md).

- No installed Voxtype package or local speech model is required.
- Separate `smellytype` command, `smellytype.service`, `~/.config/smellytype/`, runtime
  sockets and usage history.
- Separate **SmellyType Configuration** menu entry; no Voxtype menu override.
- Optional [SmellyType for Omarchy](https://github.com/beijingrong/omarchy-smellytype)
  plugin: `beijingrong.smellytype`. Its canonical source remains in `plugin/`;
  the standalone repository supports native Omarchy installation and updates.
- MIT licensed; derived from Voxtype, with original license and credit retained.

This first version is a standalone product fork, not a complete rewrite or a
fully minimized core. Reliable audio capture, dictation lifecycle, output drivers
and OSD infrastructure are inherited from Voxtype. Local-model implementation
and build dependencies still exist internally; removing them is future work.
The supported SmellyType runtime selects Doubao, and does not offer local-model
installation, meeting transcription or upstream self-update workflows.

Recording limits and final-response waits are independent. Local cumulative
minutes count microphone time including cancelled/failed sessions, not provider
billing. Earlier usage can be copied during migration. A crash can lose the
ongoing session. Never infer remaining free allowance from the local counter.

Repository: [beijingrong/smellytype](https://github.com/beijingrong/smellytype).
The Omarchy plugin-directory listing has not been submitted yet.

Renamed from the initial YunType trial. To migrate that installation, use
`python3 scripts/install.py --activate --migrate-yuntype`. Settings, key and
usage are copied; the old service/plugin/menu are disabled or removed, with
a rollback snapshot. The earlier Voxtype migration is still supported.
