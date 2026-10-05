# Engineering Requirements

本文定义 PopRaKo Native 的业务无关工程规范，供开发、代码审查和工程检查共同使用。除明确标为例外或按需引入的内容外，以下规则均为必须遵守的要求。

本文定义工程约束，不是实现完成报告。具体业务模型与页面设计不属于本文范围。

业务术语见 [CONTEXT.md](../CONTEXT.md)，本地 Translator 的模型与功能边界见 [MODEL.md](./MODEL.md)。本文继续只规定业务无关的工程约束。

## 1. Technology Stack

| 范围 | 选型与约束 |
| --- | --- |
| 包管理与任务 | Deno；统一通过 `deno task` 执行项目任务 |
| 前端构建 | Vite + TypeScript + React |
| 样式与组件 | Tailwind CSS 4.1、shadcn/ui、Lucide |
| 路由 | TanStack Router，采用文件路由 |
| 客户端状态 | React 局部状态；Zustand 跨组件状态 |
| 数据缓存 | 仅在需要缓存、失效和重取时引入 TanStack Query |
| 桌面容器 | Tauri 2 |
| Rust 编排 | `poprako-orchestra`，以参考项目采用的 0.6 接口为基线 |
| 持久化 | SQLx + SQLite |
| IPC 契约 | Tauri Specta，从 Rust 生成 TypeScript 类型与调用封装 |
| 前端质量工具 | ESLint、typescript-eslint、Prettier、Vitest、React Testing Library |
| Rust 质量工具 | rustfmt、Clippy、Cargo 测试及项目专项检查 |

- Deno 可以消费 npm 生态包；禁止混用 npm、pnpm、Yarn 或 Bun 的安装流程与锁文件。
- 提交 `deno.lock` 和 `Cargo.lock`，固定开发工具链与生成工具版本；依赖升级必须同步验证类型、生成结果和构建。
- Orchestra 使用 registry 依赖；`poprako-server` 和 `poprako-orchestra` 的本地副本仅作为阅读参考，不得成为构建所需的相邻目录。
- Tauri Specta 的 Tauri 2 集成仍有预发布版本约束。精确固定兼容的 Specta、Tauri Specta 及相关生成依赖，受控升级，不自动跟随预发布版本。[版本与安装说明](https://github.com/specta-rs/tauri-specta)

## 2. Architecture

### 2.1 前端职责与状态

- 前端负责展示、交互、导航和客户端状态。业务实现归属具体路由，多个路由共享的业务实现放在最近的共同父路由下。
- `shared` 只容纳业务无关代码。禁止 FDD，禁止引入顶层 `features`、`entities` 等业务分层，也不得换一个目录名重新建立相同分层。
- 子路由不得依赖兄弟路由的内部实现；需要共享时提升到共同父路由。通用组件不得反向依赖路由业务。
- 组件临时状态使用 React；跨组件的客户端状态使用 Zustand；异步数据只有确实需要缓存、失效和重取时才使用 TanStack Query。适用的数据来源包括 Rust 调用，不限于远程服务。
- 同一份数据只维护一个权威来源。不得将 Query 缓存复制到 Zustand 再手工同步；能从现有状态推导的数据不另建状态。
- 数据库、文件系统、系统能力和远程 HTTP 请求统一由 Rust 处理。前端不得通过直接 HTTP 请求或插件调用绕过该边界。
- 前端原生调用集中在 `@/bridge`；组件、路由及状态容器不得直接调用 `invoke`。应用资源和开发服务器连接按环境配置，不属于业务 HTTP 调用。

### 2.2 Rust 职责与 Orchestra

调用链为：前端 → `bridge` → Tauri command → usecase → 操作契约 → Rust 适配。

Rust 先采用 `src-tauri` 内的单个 crate，按实际需要建立模块。参考 `poprako-server` 的职责与依赖方向，目录名称按本项目的单数、完整单词规则调整。

| 职责 | 约束 |
| --- | --- |
| `bridge` | Tauri command、输入转换、用例调用、输出与错误映射；保持薄，不写 SQL 或业务编排 |
| `usecase` | 自由函数，依赖需要的能力，负责用例编排及事务边界 |
| `model` / `value` | 持久化模型与纯值类型；不依赖传输、用例或实际适配 |
| `complex` | 按需出现的纯规则与转换，使用自由函数，不访问数据库或其他外部能力 |
| `data` | 按需出现的输入、输出 DTO；不承担事务或数据库访问 |
| `part` | 操作描述与能力契约，不包含实际 I/O 实现 |
| `implementation` | SQLx/SQLite、网络、文件与系统能力的实际适配，以及按需隔离的测试适配 |
| `harness.rs` | 保存已组装的长期资源；生命周期由启动入口管理，不承载业务逻辑 |
| `result.rs` | 应用错误分类与转换，不依赖具体页面 |

- Orchestra 使用 `Oper`、`Run`、`Step`、`Context`、`Level` 和 `Nucl::coord`。操作描述、执行契约和事务协调属于必要架构，不因只有一个数据库实现而移除。
- `Run` 用于独立操作；`Step` 使用调用方提供的上下文。二者独立，不要求每个操作都同时实现。
- usecase 决定哪些步骤需要原子提交，通过 `Nucl::coord` 协调。一个事务中的步骤必须共享同一 `Context`，禁止在其中通过另一连接执行本应属于该事务的独立 `Run`。
- SQLite 适配负责事务开启、提交、回滚和错误分类。`Level` 必须对应实际可提供的保证，不得仅靠类型名称宣称数据库隔离能力。
- 提交成功后才发出完成通知或执行后置副作用。提交后副作用失败不能伪装成数据库已回滚，也不能无条件重试整个写入用例。
- 不预建事件总线、outbox、调度器、第二套适配或多 crate 框架；确有需求时再设计相应能力。
- 不照搬服务器的 Diesel、PostgreSQL、HTTP 状态码、行锁 SQL 或部署政策。Orchestra 的 SQLite 适配由本项目实现，数据库驱动仍使用 SQLx。
- 异步调用必须验证具体 Future 的 `Send` 和生命周期约束，不能假设任意泛型 `Nucl::coord` 都可直接提交给 Tauri 任务执行器。阻塞或长时间计算不得阻塞界面线程。

相关选择及代价见 [Orchestra 与 SQLite 决策](./decision/0001-orchestra-sqlite.md)。

### 2.3 IPC 契约与错误

- Rust command 和 DTO 是 IPC 契约的唯一来源；生成的 TypeScript 类型及调用封装放在 `bridge/generated`，手写封装与生成文件分开。
- `bridge` 可以包含传输契约及调用封装，但不承载路由业务编排。UI 组合逻辑仍归路由所有。
- 禁止手工修改生成文件或维护一套重复的 TypeScript DTO。契约变化必须重新生成，并验证调用方兼容性。
- 类型生成不能替代边界校验。Rust 必须检查输入、资源范围和路径；不得信任前端校验已经执行。
- 错误需要稳定的可识别分类及安全的用户提示。SQL、凭据、完整内部路径和底层诊断信息不得直接作为界面错误展示。
- 前端必须处理调用失败，禁止空 `catch`、丢弃 Promise 或以类型断言掩盖契约错误。

### 2.4 本地持久化与升级

- SQLite 中的持久化内容默认按需要保留的用户数据保护；可丢弃缓存必须明确标识，不能将恢复策略混用于用户数据。
- 数据库存放在平台应用数据目录，不放在安装目录或依赖当前工作目录的位置。连接、迁移和事务策略由 Rust 集中管理。
- 应用启动时执行版本化迁移；完成前不得进入正常业务流程。已发布的迁移不可改写，后续变更通过新迁移表达。
- 迁移失败时保留原有数据，停止正常操作，并给出可理解的恢复提示；禁止自动删库、清空数据或创建空库掩盖失败。
- 迁移必须验证失败时的数据状态；可能损失用户数据的变更必须先验证备份与恢复流程。不得让失败后的重试从无法判断的部分升级状态继续执行。
- 数据库读写限于实际适配层。静态查询采用 SQLx 受检查宏，维护并验证 `.sqlx` 离线元数据。
- 确有动态查询需求时允许 `QueryBuilder`：值必须绑定参数，动态标识符与排序只能来自受控映射，并补充分支测试。禁止拼接用户输入形成 SQL。
- 不为规避编译期检查而将静态查询改写成运行时查询；迁移 SQL 与正常数据访问查询分别管理。

## 3. Project Structure

### 3.1 目录与命名边界

- 自有目录使用单数、完整单词，不使用复数或自造简写。采用 `route`、`component`、`bridge`、`repository`、`operation`、`implementation`，不采用 `routes`、`components`、`api`、`repo`、`oper`、`part_impl` 等目录名。
- 前端多词目录使用 `kebab-case`；Rust 模块目录使用 `snake_case`。目录规则不改写依赖包名或框架公开标识，如 `Oper`、`Nucl`。
- 工具约定目录 `src`、`src-tauri`、`node_modules`、`.sqlx`、`target`、`dist`，以及现有 `docs`，作为明确例外。其他例外须有工具要求作为依据，不得用“惯例”任意扩大范围。
- `@` 指向 `src/`。原生调用入口为 `@/bridge`，共享组件位于 `@/shared/component`，业务代码位于对应的 `@/route/<route>/business`。
- 以下是职责布局示意，不是需要一次性创建的空目录清单；`<route>` 表示实际路由名称。

```text
src/
├── Main.tsx
├── application/                 # 初始化、路由装配及 Provider
├── route/
│   ├── __root.tsx
│   ├── business/                # 根路由范围共享的业务实现
│   └── <route>/
│       ├── route.tsx            # 本路径路由或布局
│       ├── index.tsx            # 本路径的索引子路由
│       └── business/            # 组件、hook、状态、测试等路由业务
├── bridge/
│   └── generated/
├── shared/
│   ├── component/
│   ├── hook/
│   └── utility/
└── route-tree.gen.ts

src-tauri/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── bridge.rs
│   ├── bridge/
│   ├── usecase.rs
│   ├── usecase/
│   ├── part.rs
│   ├── part/
│   │   ├── repository.rs
│   │   └── repository/
│   │       ├── operation.rs
│   │       └── operation/
│   ├── implementation.rs
│   ├── implementation/
│   │   ├── repository.rs
│   │   ├── repository/
│   │   ├── coordinator.rs
│   │   └── coordinator/
│   ├── harness.rs
│   └── result.rs
└── migration/

script/                         # 检查、生成与构建辅助脚本
docs/
├── REQUIREMENTS.md
└── decision/
    └── 0001-orchestra-sqlite.md
```

### 3.2 文件路由与生成文件

- TanStack Router 的 `routesDirectory` 指向 `src/route`，`generatedRouteTree` 指向 `src/route-tree.gen.ts`。
- 配置 `routeFileIgnorePattern: "^business$"`，排除各级 `business` 目录及其子树。路由目录中的非路由实现与测试放入该目录，避免被识别为路由。
- `__root.tsx`、`route.tsx`、`index.tsx`、动态参数和布局标记遵守框架语义，列为文件命名例外。`route.tsx` 和 `index.tsx` 的职责不同，不可为了统一名称而互换。[文件约定](https://tanstack.com/router/latest/docs/routing/file-naming-conventions)、[生成器配置](https://tanstack.com/router/latest/docs/api/file-based-routing)
- 路由树放在扫描目录之外。生成的路由树、IPC 契约与 SQLx 离线元数据纳入版本管理，检查任务验证它们与输入一致。
- 生成文件必须可追溯到生成任务；不手改，不受手写文件命名与 400 行限制，但仍接受类型、契约和生成一致性检查。
- 可配置的工具目录使用本项目名称，例如 SQLx 迁移目录显式配置为 `migration`。shadcn/ui 生成后由本项目维护的组件属于自有源码，不享受持续自动生成文件的豁免。

## 4. Code Style

### 4.1 通用要求

- 手写 `.ts` 文件使用 `kebab-case`，例如 `user-settings.ts`；手写 `.tsx` 文件使用 `PascalCase`，例如 `SettingsPanel.tsx`；Rust 文件使用 `snake_case`。
- 测试文件保留对应源文件名称并添加 `.test.ts`、`.test.tsx` 等工具识别后缀。工具要求的配置名、类型声明后缀及第 3 节的路由/生成文件约定是明确例外。
- 手写 TS、TSX、Rust 源码和测试文件不得超过 **400 个物理行**，计入空行与注释；生成文件、锁文件、迁移 SQL 和文档不受此限制。
- 超限时按职责拆分，不通过压缩排版、删除必要注释或无意义切片满足行数检查。
- 优先函数与组合，按实际需求抽象；不为未来需求预建通用框架，不为单一实现机械增加接口、工厂或转发层。
- 格式以 Prettier 和 rustfmt 为准，不以个人排版绕过工具。项目特有风格通过专项检查落实。

### 4.2 TypeScript / React

- 使用具名导出。顶层函数与 React 组件使用普通 `function` 声明，回调使用箭头函数；工具强制的默认导出除外。
- 纯数据结构必须使用 `type` 声明；纯函数接口（仅描述函数签名或方法契约）必须使用 `interface` 声明，例如 `UnitApi`。
- React 组件的 Props 必须使用 `type` 声明，即使其中包含回调函数，也不改用 `interface`。
- Props 中尽可能不使用可选字段或可为 `null` / `undefined` 的字段；优先由调用方提供明确的值。只有缺省或空值具有明确语义且确有必要时才保留，不通过类型断言或无意义占位值规避约束。
- 纯类型导入使用 `import type`。导出的函数与调用边界明确返回类型。
- 输入未知时使用 `unknown` 并收窄；禁止显式 `any`、不安全的连续类型断言和非空断言掩盖问题。确有工具兼容例外时遵守第 6 节的定点规则。
- 遵守 React Hooks 规则；组件渲染保持纯净，副作用具有清理和错误处理。不要用 Effect 反复同步本可直接推导的数据。
- 禁止无理由 `@ts-ignore`、宽泛 lint 禁用以及无人处理的异步调用。已知类型问题优先修正配置或边界声明，不将抑制注释作为常规实现方式。
- Tailwind 负责常规样式，组件使用一致的主题变量。动态几何值可以使用必要的样式属性，不重复建立平行的主题或组件状态体系。

### 4.3 Rust

- 采用同名模块文件与目录并存的组织方式，禁止创建 `mod.rs`。
- 只使用 private 和 `pub`，禁止 `pub(crate)`、`pub(super)` 等受限可见性形式。
- `use` 的花括号只出现在路径末端，其中只放简单名称；当前 crate 内的跨模块导入使用 `crate::`。
- 类型的全部 inherent/trait `impl` 紧跟该类型定义，置于下一个类型之前。同一模块内，依赖项先于使用者定义。禁止循环引用，不得通过放宽检查器或移动到不同模块来规避。
- 使用提前返回的 guard、`match` 和 `let … else`，禁止普通 `else`、`else if` 分支链。
- 通道两端统一使用 `send` / `recv` 或带语义前缀的对应名称，不使用 `tx` / `rx` 等替代。
- 注释使用英文，解释意图与约束；相邻语句间保留空行，不强制在同一表达式的分支或字段之间插入语句分隔空行。
- 只有错误发生处直接记录错误；上层只传播时不重复添加日志事件，边界可通过 instrumentation 记录传播结果。
- 手写 Rust 禁止 `unsafe`。生产代码禁止 `unwrap`、`expect`，通过明确的错误处理表达失败；测试的定点例外见第 6 节。

## 5. UI / UX

- 首期界面使用简体中文，暂不引入国际化框架。界面文案表达用户任务和恢复方式，不暴露内部模块或异常栈。
- 固定浅色，不提供深色或跟随系统。Home／Comic Detail 沿用旧 Native 的柔和低饱和度配色，Translator 直接迁移 Web 现有布局与组件；使用统一外观变量，弹层继承当前页面外观。Web 的深石灰漫画画布与深色文本浮层保留。
- 常规操作优先 Lucide 图标，保留可访问名称、悬停提示和键盘焦点；减少常驻解释文案。删除、覆盖、失败恢复与有损交换的必要说明保留。
- 通用组件放在 `shared/component`，业务组件留在对应路由。shadcn/ui 组件按项目命名与检查规则维护，图标使用 Lucide。
- 核心操作支持键盘，交互元素有可辨识名称与清晰焦点；弹层正确管理焦点，关闭后恢复合理位置，不仅通过颜色表达状态。
- 快捷键考虑 Windows 与 macOS 的修饰键差异，避免抢占常用系统操作。
- 加载、空白、失败、重试和完成反馈保持一致。进行中的操作应有反馈，防止无意重复提交；失败后保留可恢复的用户输入。
- 错误提示需说明用户可以做什么；重试必须考虑操作是否可安全重复，不能将全部错误统一处理为自动重试。
- 长时间任务不得冻结交互；窗口缩放、内容溢出和滚动必须可用。具体视觉风格、窗口布局和业务交互由后续设计确定。

## 6. Engineering Quality

### 6.1 严格检查与例外

“最严格”指可共同执行的严格规则集、显式限制和零警告门禁，不是无差别启用相互矛盾的规则。源码、配置和测试均纳入相应检查范围。

| 检查 | 必须落实的基线 |
| --- | --- |
| TypeScript | `strict`、`noUncheckedIndexedAccess`、`exactOptionalPropertyTypes`、`noImplicitOverride`、`noPropertyAccessFromIndexSignature`、`noImplicitReturns`、未使用声明及 switch 穿透检查；禁止不可达代码和无用标签；`skipLibCheck: false` |
| ESLint | typescript-eslint 的 `strictTypeChecked`、`stylisticTypeChecked`，React/Hooks 检查；所有启用规则按 error 执行，`--max-warnings 0` |
| TS 显式限制 | 禁止显式 `any`、不安全赋值/调用/返回、非空断言、丢失 Promise 和误用异步回调；检查穷尽分支、类型导入、类型定义风格及 400 行上限 |
| 格式 | Prettier、rustfmt 的只读检查；ESLint 与 Prettier 冲突的纯排版规则交由 Prettier 负责，不借此关闭类型或正确性检查 |
| Clippy | `all`、`pedantic`、`cargo`，并显式启用 `unwrap_used`、`expect_used`、`panic`、`todo`、`unimplemented`、`dbg_macro` 等适用限制；警告视为失败 |
| Rust 专项 | 禁止手写 unsafe，检查物理 400 行、目录与模块命名、可见性、控制流、导入与定义顺序等 Clippy 未完整覆盖的项目规则 |
| 生成与 SQL | 检查路由树和 IPC 生成结果；验证 SQLx 离线元数据与实际迁移、查询一致 |

- `restriction` 和 `nursery` 按单项评估、集中记录后启用，不整组打开。Clippy 官方明确指出这些规则有适用范围与相互冲突问题。[Clippy 说明](https://doc.rust-lang.org/clippy/)、[规则组](https://doc.rust-lang.org/clippy/lints.html)
- 规则不得与本项目风格矛盾，例如不能启用要求补 `else` 的规则来检查禁止 `else` 的代码，也不能通过建议 `pub(crate)` 的规则推翻可见性约定。
- 新工具链或规则升级必须复核适用性。禁止只为通过检查而整体降低等级、排除手写源码或移除必要的检查范围。
- 工具误报、不可控依赖或必要兼容问题，仅允许针对精确规则、最小作用域的例外，必须说明原因。禁止整文件关闭检查、空泛理由及批量忽略。
- 测试可在对应测试范围对 `unwrap_used`、`expect_used` 定点放行，用于明确断言或夹具初始化，并说明原因；不对整个测试目录全面放宽检查。
- 不修改生成文件来消除生成器造成的问题。必要例外通过配置精确定位到生成路径和规则；类型、契约及生成一致性检查仍须通过。

### 6.2 测试策略

- 前端使用 Vitest + React Testing Library，从用户可观察的行为验证关键交互；Rust 使用 Cargo 测试，数据库集成使用独立临时 SQLite。
- 测试按风险选择，不设全局覆盖率数字，不为简单展示组件或实现细节编写形式化测试。
- 必须覆盖数据读写、首次建库与旧版本迁移、迁移失败保留数据、事务提交和失败回滚、取消清理、忙锁反馈、错误分类及关键界面状态。
- 涉及多连接、事务竞争或文件行为时使用临时文件数据库，不以单连接内存数据库的结果替代实际行为验证。
- 动态查询覆盖可选条件、组合及参数边界；IPC 契约变化检查生成结果和调用方。生成可通过不代表运行行为无需测试。
- 缺陷修复补充能够复现问题的回归测试。测试不得依赖真实用户数据库、凭据或不可控远程服务。
- 两个平台都需验证安装、启动、关键操作与退出；开发浏览器中的测试不能替代桌面容器验证。

### 6.3 统一任务与工作流

项目任务提供下列职责；实际入口由 `deno.json` 定义。

| 入口 | 职责 |
| --- | --- |
| `deno task dev` | 启动 Vite 前端开发服务器 |
| `deno task desktop` | 启动 Tauri 开发环境，允许 Tauri 调用前端 `dev` |
| `deno task build` | 前端类型检查与生产构建 |
| `deno task check` | 只读执行格式、类型、lint、项目专项、生成一致性、SQL 元数据检查及测试 |
| `deno task test` | 执行前端与 Rust 测试 |
| `deno task format` | 格式化手写源码 |
| `deno task generate` | 按依赖顺序更新 SQL 元数据、IPC 契约及路由树 |
| `deno task package` | 执行 Tauri 打包，由 Tauri 调用前端 `build` |

- Tauri 前置任务不能反向调用自身，避免 `desktop/dev` 或 `package/build` 形成递归。
- Rust 检查至少包含 `cargo fmt --all --check`、`cargo check --workspace --all-targets`、`cargo clippy --workspace --all-targets -- -D warnings` 及相应测试，在 `src-tauri` crate 上下文执行。
- `check` 不改写受版本管理的文件；生成检查在临时输出中比较差异，SQL 验证使用由迁移建立的临时数据库，不操作用户数据库。
- 辅助任务须可在 Windows 和 macOS 执行，不能将仅适用于某个开发者机器的路径或 shell 行为作为前提。
- 工作流遵循计划 → 实施 → 复核；说明变化、原因、验证结果与实际限制。新约束或架构变化同步更新规范，重要且有取舍的决策使用简短 ADR。

## 7. Security & Delivery

### 7.1 平台与许可证

| 平台 | 最低目标版本 | 首期架构 |
| --- | --- | --- |
| Windows | Windows 10 22H2，及后续兼容版本 | x64 |
| macOS | macOS 13.3，及后续兼容版本 | Apple Silicon / ARM64 |

- 首期不承诺 Linux、独立浏览器应用、移动端、Windows ARM64 或 Intel Mac 支持。
- 系统版本与 WebView 能力分别验证。Windows 使用满足前端栈能力要求的 WebView2，macOS 验证系统 WKWebView；最低目标系统必须实测，不以 Tauri 自身最低版本替代项目验证。[Tailwind 兼容性](https://tailwindcss.com/docs/compatibility)、[Tauri WebView 说明](https://v2.tauri.app/reference/webview-versions/)
- 项目许可证为 **AGPL-3.0-or-later**。后续 LICENSE、包元数据与发布说明使用同一标识，并保留适用的第三方许可证信息。
- 根目录 LICENSE 只保留 GNU 官方 AGPL v3 完整原文，项目授权声明位于 README 和包元数据；`deno task check` 校验正文摘要。修改许可证文件须运行 Licensee 实际核验识别结果。

### 7.2 权限、凭据与日志

- Tauri 必须显式配置 CSP，按实际资源来源和运行环境限制加载范围；生产配置不得沿用关闭 CSP 的脚手架状态。
- 窗口和 WebView 仅获得需要的能力与资源范围；权限配置、command 参数校验和路径校验分别落实。
- 自定义 command 注册到 `invoke_handler` 不等于自动受最小权限保护。通过 `AppManifest::commands` 等框架机制将应用命令纳入权限控制，并验证实际允许范围。[Tauri capabilities](https://v2.tauri.app/security/capabilities/)、[CSP](https://v2.tauri.app/security/csp/)
- Rust 校验外部输入和文件访问范围；网络请求明确超时和失败处理，重试考虑幂等性，不能以关闭证书验证解决连接问题。
- 凭据存入系统凭据存储，不写入普通 SQLite、前端持久化、源码或日志。普通本地应用数据不要求数据库整体加密。
- 日志保留在本地并脱敏，不记录凭据、完整敏感载荷或不必要的用户数据；错误发生处记录诊断，上层传播避免重复日志。
- 首期不建设遥测、崩溃报告或日志上传机制。将来引入此类能力必须先明确数据范围、用户选择与保留策略，并更新规范。

### 7.3 构建与发布

- 首期建立统一检查、两平台构建验证和可复现的打包流程；“可复现”指工具链、锁文件和步骤明确，不承诺签名后安装包逐字节一致。
- 发布构建使用受控工具链和锁文件，记录版本及对应源码状态。`bundle.targets: "all"` 不代表已经验证所有 CPU 架构。
- 首发 `1.0.0` 采用 macOS ad-hoc 签名、Windows 无签名分发；发布前验证实际安装包的安装、首次打开和运行行为，并在安装说明中写明系统提示。Apple Developer ID 签名与公证作为后续可选升级，不是当前首发的前置条件；签名密钥不进入仓库。
- 升级验证覆盖已有本地数据的保留与迁移。最低系统版本、架构和安装包的验证结果必须可追溯。
- 采用 GitHub Actions 管理版本发布；首发保持单分支、单根提交并手动构建标签，后续按需手动启动 Release PR。macOS DMG 与 Windows NSIS 安装包各不超过 10 MiB，收集和汇总均检查实际字节大小。流程、权限、签名配置和失败恢复见 [CI 与版本发布](./release.md)。正式发布开启前保留草稿；开启后由维护者授权该版本的构建与发布。应用内自动更新不在首期范围，发布流程不得依赖未记录的人工环境状态。
