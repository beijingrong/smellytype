# SmellyType

**简体中文** | [English](README.en.md)

面向 Omarchy / Hyprland 的豆包云端语音输入。按住 **F9** 说话，松开后将识别结果输入当前应用。

**SmellyType 基于 [Voxtype](https://github.com/peteonrails/voxtype) 分叉开发，直接沿用了大量核心代码。** 感谢 **Peter Jackson（peteonrails）和所有 Voxtype 贡献者**：音频采集、录音与识别流程、文字输出和屏幕提示（OSD）等基础能力都来自他们的工作。SmellyType 在此基础上加入豆包云端识别、录音用量统计和 Omarchy 控制面板。

## 功能

- **豆包流式识别 2.0**：说话时显示实时预览，结束后一次性输入最终文字。
- **快捷键输入**：按住 F9 录音，或用 Super+Ctrl+X 切换录音状态。
- **状态栏面板**：开始、停止或取消录音，调整单次录音上限和识别等待时间。
- **累计录音统计**：查看本机录音分钟数和次数。

## 安装

把下面这句话发给你的 Agent：

> 请帮我安装 https://github.com/beijingrong/smellytype ，先阅读仓库的 AGENTS.md 和 docs/INSTALL.md。

需要支持 Quickshell 的 Omarchy / Hyprland 环境、网络连接，以及已开通豆包流式语音识别服务的火山引擎 API Key。密钥在本机填写，请勿发到聊天或提交到仓库。

目前为 **0.1.3 预览版**，从源码构建安装。完整依赖、安装步骤、更新和迁移说明见 [安装文档](docs/INSTALL.md)。无需另行安装 Voxtype 或下载本地语音模型。

安装程序包含控制面板。已有 SmellyType 的用户也可通过 [SmellyType for Omarchy](https://github.com/beijingrong/omarchy-smellytype) 独立安装和更新插件。点击状态栏小猫按钮或打开 **SmellyType Configuration** 即可进入面板。

## 个人词库与口语整理

点击小猫面板中的 **个人词库 · 编辑**，每行填写一个常用名称，保存后生效；清空并保存可移除词库。最多 50 个词、每词 64 字、总计 4096 UTF-8 字节（客户端限制）。请精简词表，重要的词放前面。词库会随每次录音发送给豆包；热词是提示，不保证正确识别。手动修改转写结果不会自动写入词库。

**口语整理**调用豆包的语义顺滑，减少结巴、语气词和无意重复，但也可能删去你想保留的表达语气。新安装默认关闭，可以在面板中比较开关效果。两项设置仅在空闲时应用。

## 使用说明

- 录音会发送到豆包云端识别；服务开通、额度和费用由火山引擎管理。
- 单次录音上限与结束录音后的识别等待时间分别设置。
- 累计分钟数记录本机麦克风使用时间，包含取消或失败的录音，不等同于云端账单或剩余免费额度。

## 致谢与许可

Voxtype 是 SmellyType 的代码基础。我们保留其原始 MIT 版权声明，并继续以 **MIT** 许可开源。详见 [LICENSE](LICENSE)、[来源与修改说明](NOTICE) 和 [第三方声明](THIRD_PARTY.md)。本项目由独立维护者开发，与 Voxtype、Omarchy 或火山引擎无官方隶属关系。

如果你希望使用本地模型，欢迎了解和支持上游项目 **[Voxtype](https://github.com/peteonrails/voxtype)**。
