# 应用图标

唯一图标源文件是 `src-tauri/icons/icon.png`，256×256 RGBA PNG，保留原有透明留白和圆润外轮廓。更新图标只替换这一个文件。

`src-tauri/build_icon.rs` 在 Cargo 构建时将同一份 PNG 字节封装为 macOS ICNS 和 Windows ICO，输出至已被 Git 忽略的 `src-tauri/target/generated-icon/`。不生成多尺寸 PNG、商店磁贴或移动端资源，不提交派生二进制。ICNS 使用单个 ic08 图层，ICO 使用单个 256px PNG 图层，不重新缩放或修改图像。

`src-tauri/build.rs` 追踪源目录，替换图标后重新构建开发程序或安装包即可更新；已运行的程序不会自动更新图标。

平台格式依据：[Tauri 图标说明](https://v2.tauri.app/develop/icons/)、[Chromium ICNS 格式记录](https://chromium.googlesource.com/experimental/chromium/src/+/refs/tags/85.0.4166.0/docs/mac/icons.md)、[Microsoft ICO 目录格式](https://devblogs.microsoft.com/oldnewthing/20101018-00/?p=12513)。
