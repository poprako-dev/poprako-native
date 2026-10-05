# Project Instructions

本项目遵循计划 → 执行 → 复核，用户问题使用问卷工具。

开始相关工作前读取以下约定，避免从脚手架或相邻项目推断本项目规范：

- [REQUIREMENTS.md](docs/REQUIREMENTS.md)：工程架构、代码风格、质量与交付要求。
- [CONTEXT.md](CONTEXT.md)：领域术语。
- [MODEL.md](docs/MODEL.md)：首期模型及功能边界。
- [IMPLEMENTATION.md](docs/IMPLEMENTATION.md)：交换协议、用例、偏好与验收顺序。

具体行为契约位于 [docs/contract](docs/contract)：[模型持久化](docs/contract/model-representation.md)、[保存与离开](docs/contract/editor-save.md)、[图片生命周期](docs/contract/image-lifecycle.md)、[迁移验收](docs/contract/migration-acceptance.md)。契约定义应有行为，实际行为以源码和测试为准。

常规技术选择自行完成，不逐项问卷或等待“继续”；仅改变既定产品边界的分歧集中收取用户反馈。未实测的技术、平台与性能门禁不得标为通过。

UI 迁移直接以固定版本 Web Translator 组件为依据，不自行重新设计布局；Home 和 Comic Detail 参考旧 Native，保留用户已作出的本地调整。具体来源和本地化差异见[界面来源对照](docs/contract/ui-reference.md)。参考仓库不是构建依赖。
