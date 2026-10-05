# 下载与安装

从[官方 GitHub Releases](https://github.com/poprako-dev/poprako-native/releases) 下载对应平台的安装包。

## macOS

选择文件名包含 `macos-arm64` 的 `.dmg`，适用于 Apple Silicon Mac。打开磁盘映像，将应用拖入「应用程序」文件夹，再打开应用。

当前版本使用 ad-hoc 签名，未经过 Apple 公证。如果首次打开提示无法验证开发者，在确认安装包来自上述官方发布页后：

1. 打开「系统设置 → 隐私与安全性」。
2. 找到该应用的拦截提示，选择「仍要打开」。
3. 按系统提示确认打开。

操作入口会在尝试打开应用后出现；具体提示以系统版本为准。详细说明见 [Apple 官方帮助](https://support.apple.com/zh-cn/102445)。如果系统提示文件损坏或包含恶意软件，请重新下载并核对发布页的校验值，仍有问题时提交反馈。

## Windows

选择文件名包含 `windows-x64-setup` 的 `.exe`，运行安装程序并按提示安装。当前版本暂未使用代码签名证书，系统可能显示未知发布者或 SmartScreen 提示。

应用使用电脑已有的 Microsoft Edge WebView2 Runtime；安装包不包含、不下载运行库。启动需要系统已安装 WebView2 Runtime。

遇到无法安装或启动的问题，请在 [GitHub Issues](https://github.com/poprako-dev/poprako-native/issues) 附上应用版本、系统版本和提示截图。
