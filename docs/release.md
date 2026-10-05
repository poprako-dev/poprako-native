# 版本发布

首发仓库只有 `main` 和一个根提交。推送触发 CI；创建指向同一提交的 `v1.0.0` 标签及草稿后，手动运行 Release 并填写标签，构建 macOS Apple Silicon DMG 与 Windows x64 NSIS。Release 不随 push 自动创建分支。后续需要 Release PR 时，手动运行 Release 并留空标签，使用 Release Please 汇总版本。

## 仓库设置

- Settings → Actions → General：允许 GitHub Actions 创建 Pull Request。
- 可选 Secret `RELEASE_TOKEN`：仅授权本仓库的 Contents、Pull requests 读写权限，用于自动触发机器人 PR 的 CI；未设置时使用 `GITHUB_TOKEN`。
- Variable `RELEASE_PUBLISH=true`：安装包齐备并通过校验后公开 Release；未设置时保留草稿。
- 首发 macOS 使用 ad-hoc 签名，Windows 不签名，`RELEASE_SIGNING` 保持未设置。安装提示见[安装说明](installation.md)。

## 版本与产物

- 使用 Conventional Commits：`fix` 对应 patch，`feat` 对应 minor，破坏性改动对应 major；一批提交汇总为一次发布。
- 首发 `1.0.0`。Release PR 同步 package、Tauri、Cargo、Cargo.lock 和发布 manifest；`deno task check` 检查版本一致。
- Cargo.lock 的 TOML 选择器使用 `@.name.value`，匹配固定 release-please 17.3.0 的节点结构；升级 Action 时需验证版本更新。
- 产物为 `PopRaKo-Native_X.Y.Z_macos-arm64.dmg`、`PopRaKo-Native_X.Y.Z_windows-x64-setup.exe`、两份平台元数据和 `SHA256SUMS`。
- 工作流验证固定 SHA、版本、平台、签名模式与文件摘要；任一平台失败或产物不完整均不公开。
- 每个平台安装包必须非空且不超过 10 MiB（10,485,760 字节）；收集和汇总均实际检查文件大小，超限禁止上传。
- Windows 使用系统已有的 WebView2 Runtime，配置为 `skip`，不内置、不下载运行库。

## 恢复草稿构建

Actions → Release → Run workflow：选择 `main`，填写已有草稿标签 `vX.Y.Z`。工作流重新构建相同提交，替换草稿的同名附件。已公开的版本禁止覆盖，应另发新版本。

## 可选发行签名

设置 `RELEASE_SIGNING=true` 时需要同时提供两平台签名材料：

- macOS Secrets：`APPLE_CERTIFICATE`（P12 Base64）、`APPLE_CERTIFICATE_PASSWORD`、`APPLE_SIGNING_IDENTITY`（Developer ID Application）、`APPLE_ID`、`APPLE_PASSWORD`（应用专用密码）、`APPLE_TEAM_ID`。
- Windows Secrets：`WINDOWS_CERTIFICATE`（PFX Base64）、`WINDOWS_CERTIFICATE_PASSWORD`；Variable `WINDOWS_TIMESTAMP_URL`。

工作流验证 macOS 签名与公证及 Windows Authenticode。证书和私钥不得进入仓库。
