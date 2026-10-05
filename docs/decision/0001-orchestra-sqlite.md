# 使用 Orchestra 编排与 SQLx/SQLite 本地持久化

桌面端采用 `poprako-orchestra` 的 `Oper / Run / Step / Context / Nucl::coord` 模型，与 `poprako-server` 当前的操作契约和事务编排方式保持一致，同时保留适合客户端的 SQLx/SQLite 持久化选型。项目在单个 Tauri crate 中按需建立用例、契约和实际适配，由 usecase 确定事务边界，SQLite 适配负责真实的提交、回滚及错误处理。

这一选择将事务协调集中到既有框架，代价是需要自行实现并验证 SQLite 的 Context/Nucl 适配、事务保证及 Tauri 异步调用兼容性；不引入服务器的数据库栈、行锁语义或部署政策。长期维护以 [工程规范](../REQUIREMENTS.md) 为准，依赖使用 registry 版本，参考项目的本地副本不成为构建前提。
