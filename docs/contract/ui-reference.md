# 界面迁移来源

生产布局直接来自以下固定参考版本。界面固定浅色，保留 Web 的深石灰画布和深色文本浮层。

## Translator

参考来源：`poprako-web@46dd776`，根目录 `src/route/_authenticated/translator/business/`。这是源码迁移依据，不是构建所需的相邻目录。

| 来源 | 生产实现保持的结构与交互 |
| --- | --- |
| `editor/BaseTranslatorLayout.tsx` | 左画布、右栏占 1/3 且最小 380px；portrait 上下排列，小宽度时右栏 2/5，sm 时 200px |
| `editor/EditorCanvas.tsx` | 左上 32px 图标工具箱和独立退出；右上 96×32px 翻页器；没有业务顶栏、缩略图常驻栏或操作教程 |
| `editor/EditorToolbar.tsx` | 模式、重定位、保存、质量、预览同序；只读隐藏保存；粗指针创建开关；保存中旋转反馈 |
| `preference/use-shortcuts.ts`、`editor/use-editor-keyboard.ts`、`editor/keyboard-scope.ts` | 同序的全部 12 个默认键位；Ctrl 明确保留、Tab 首尾循环、文本输入中导航、符号选区插入、Escape 取消选择；弹窗／输入法隔离 |
| `unit-list/BaseUnitItem.tsx` | 连续行、4px 粉／黄类别边、32px 序号、选中石灰底色；不使用独立卡片 |
| `unit-list/TranslateModeUnitItem.tsx`、`AutoResizeTextarea.tsx` | 模式切换和宽度变化保持文本完整显示，自适应无边框输入、星标、完成点；选中时显示优选符号 |
| `unit-list/ProofreadModeUnitItem.tsx` | 保留原译文，选中或已有校对时显示输入；复制、确认分开；整页确认固定在列表底部且作用于完整草稿 |
| `unit-list/use-unit-reorder.ts` | 点序号切换框内外，拖动重排，预览与提交分开，Escape／取消／丢失捕获回滚；过滤只移动可见位置 |
| `preference/use-detachable-special-chars-bar.ts` | Grip 拖出／移动、Undo2 放回、视口约束；临时位置不持久化 |
| `unit-list/ReadOnlyDiffUnitItem.tsx`、`LineBreakOverlay.tsx` | 删除／被替换／替换／新增四种柔和差异与换行标记；只读无编辑星标 |
| `canvas/Marker.tsx` | 圆 32px、点 8px、锚偏移 38px；框内粉、框外黄、选中蓝边、完成叶绿边；关闭预览时淡化，拖动缩放 1.1 |
| `canvas/Canvas.tsx`、`use-canvas-interaction.ts` | 石灰 600 画布、图片最大 90%×95%；滚轮缩放 .5–5、鼠标平移与标记拖动阈值分开；24px 键盘平移、200ms 重定位；悬停最终文本，校对聚焦预览 |
| `src/shared/component/paginator/PagePicker.tsx` | 224px 浮动页列表，当前项、关注、未保存图标及总数／翻译／确认计数 |
| `page-statistic/PageUnitStatsChart.tsx` | 只读右下统计、304px 浮层、28px 行、翻译／修改／追加；只查找后续修改页，不循环 |

固定浅色变量集中在 `src/application/appearance.css`，页面外观通过 `Appearance` 上下文传入 Radix portal；弹层与页面使用同一外观。优先图标、title、aria-label 与键盘焦点，危险操作、有损交换和恢复信息保留必要文字。

本地差异：自由切换三模式；移除成员、在线阶段提交；术语按下述 issue #1 扩展；图块对应本地资源。整页确认在编辑列表底部，不表达线上阶段完成。保存仍使用完整草稿、15 秒计时器和版本快照；筛选、拖动、确认不会缩减持久化快照。

统计由 Rust 页面聚合和当前页实时草稿派生，不加载全项目正文。校对修改要求双文本非空且不同，追加要求原译文为空、校对非空，确认状态独立。偏好迁移先严格校验旧 JSON，再移除 theme 并一次性升级旧默认快捷键；自定义键位与明确解绑保留，新默认冲突时自定义优先。非法枚举、重复键、未知字段或损坏数据不重置。

## 快捷键

快捷键与 Web 保持一致。

| 操作 | Web 与 Native 默认键位 |
| --- | --- |
| 切换模式／重定位／标记显示与淡化 | Ctrl+M／Ctrl+L／Ctrl+X |
| 下一个／上一个标记（首尾循环） | Tab／Shift+Tab |
| 上一／下一页（边界不循环） | Ctrl+U／Ctrl+D |
| 最近符号／前三个优选符号 | Ctrl+Q／Alt+1、Alt+2、Alt+3 |
| 保存／取消选择 | Ctrl+S／Escape |

除下述万能回退使用 Command 外，macOS 既有操作同样使用 Control。单元输入框中正常触发，其他输入框、组合输入、AltGr、已处理事件、弹窗、只读统计展开、切页锁及提交待核实状态隔离。只读只保留导航、模式和标记显示操作。Ctrl+X 与眼睛按钮共享暂态，不递减持久化透明度；符号键与符号栏共用请求，替换选区，在 DOM 提交时恢复光标并确认消费，返回原页不重放。最近符号和标记淡化跨页保留。设置允许录制 Tab／Shift+Tab，冲突时保留原键位，并显示 Web 的固定鼠标操作表。

WKWebView 的 Option+3 事件形状：code=Digit3、alt=true、isComposing=false、keyCode=229。正常 Option 数字键按 code 处理，实际组合状态及其他 229 确认事件继续隔离。

一次性迁移兼容旧 macOS／Windows 默认、仍含主题与已移除主题的数据库；旧 action alias 兼容读取，升级标记与偏好写入同事务，重开不重复改写。自定义键位占用新默认时保留该自定义和被阻挡操作的原绑定。

## Home / Chapter Detail

参考来源：`porpako-native-sv@82c20b7baa5f2a732c4f09882258fb6f1620e209`，根目录 `src/routes/`。这是页面组织依据，不是构建依赖。

- `home/comps/biz/HomePage.svelte`、`QuickActions.svelte`：1024px 居中、16px 外边距、112px 高快捷区、左 2/3 继续／右 1/3 创建与导入。
- `home/comps/biz/ProjectItem.svelte`：紧凑行、活动点和图标统计；当前 Native 按用户修改使用四格阶段进度指示；设置与删除转为图标。
- `project-viewer/comps/biz/ViewerSidebar.svelte`：208px 侧栏，窄窗口 160px，封面最大 192px，资料、继续／导出／编辑。
- `project-viewer/comps/biz/PageList.svelte`：144px 最小列、3:4 缩略图、页码与进度；点击先预览。添加、管理、导入、删除为图标。

浅色配色直接取旧 Native：背景 `#fafaf7`、正文 `#414840`、主色 `#647c60`、边线 `#e4e6df`、柔和绿／黄／粉状态。页和项目进度遵守本地模型，确认与译文独立。

## 根路由窗口外框

用户新增示例作为窗口标题栏来源：窄淡绿色条、右侧三个窗口控制图标、圆润外轮廓。`WindowFrame` 位于根路由，36px 高，普通窗口 12px 圆角；透明底层及取消 decorations 去掉系统标题栏。Home／Detail 使用剩余容器高度，Translator 布局保持原有比例。最大化和全屏时取消圆角，恢复时重新显示圆角；关闭发出窗口关闭请求，继续进入现有退出保护。

macOS 透明底层启用 private-api，不用于 App Store 分发，详见 [IMPLEMENTATION.md](../IMPLEMENTATION.md)。

## 漫画与章节 Home 扩展（issue #3）

章节项与漫画项等宽，左右边界对齐，不给章节列表添加外部缩进。章节项默认背景透明，鼠标悬停和内部控件聚焦时整行使用浅色 accent 背景反馈，覆盖左侧圆点、章节内容及右侧操作区；通过行内的圆点与连线形成内容的视觉缩进，不额外移动圆点来扩大背景留白。行间距不着色，保留控件的键盘焦点轮廓。

Home 保留用户本地颜色与紧凑行。漫画行只显示标题、作者、章节数和展开状态，不展示活动状态点、进度或页面／单元／翻校统计。漫画行保留卡片背景、边框与悬停反馈；展开的章节区域为透明背景，无卡片底色与外框。章节左侧采用用户提供的时间线参考中的灰色空心圆与细连线，仅表达层级，不表示业务状态。

支持 Enter／Space 展开与局部加载／错误重试。章节名称后紧接日期与统计，同一行排列，空间不足时换行；日期仅显示月／日，不显示年份和时分。章节保留自身统计与四格进度，点击进入详情；行内仅提供编辑名称、移入漫画和删除，不提供继续翻校或上下调整章序按钮。漫画行右侧提供新增及导入章节，收起时也可操作；展开的章节列表底部不放这些入口。漫画资料编辑与删除独立于章节操作，删除漫画明确列出全部所属章节。新建弹窗分别填写漫画标题、作者和首章名称。首页快捷区的继续最近章节入口保留。Translator 的布局及快捷键不随层级变更重新设计。

## 万能回退行为来源（Issue #2）

仅回退行为参考 poprako-web@e5022518b8a8dab619723440e8f3e69078bcf593（核心提交 98487427fc4f283c24c8416da2851f45519b135a）：unit/unit-history、unit-list/use-composed-text、persistence/unit-undo 测试及 editor/EditorUndoKeyboard。其余布局仍按本文原有参考和本地调整维护，不引入 Web 的额外功能。

快捷键设置首项增加「万能回退」：Windows Ctrl+.，macOS Cmd+.，可自定义或解绑，长按不重复。保存及原生输入撤销保持独立；单元删除提示说明当前会话历史有效时可回退，切页、离开或替换当前页内容后不可恢复。其他对象的删除不表示可以回退。

## 术语库页面与查询（issue #1）

全局管理页参考 `poprako-web@916b8abb6971f74221e4055293f7c9654ee3a58d` 的 `TeamTerminologyTool`、`TerminologyToolbar`、`TeamTermList`、`TermbaseRow`、`TermRow`。采用 Utilities 右侧 input + 列表的单列结构，Home 外观、16px 边距和 1024px 最大宽度；不引入 Web 团队与权限导航。

- 顶部返回 Home 与标题；40px 工具栏含 32px 搜索输入、32px 加号、8px 间距。库目录与库内术语为两层，同一库返回再进入保留查询，换库清空术语查询。新建库后直接进入；新建术语后清空原文过滤。
- 库行至少 48px，18px 文件夹、名称／数量／全局标识，描述靠右。库内 48px 文件夹标题行，术语行至少 44px、缩进圆点、原文在左、译名在右并换行，译名区最多一半宽。标题与工具栏固定，仅列表滚动；分页有失败重试。
- 点击库进入；长按 500ms 或焦点 F2 编辑，复制按钮不触发行编辑。库与术语编辑使用紧凑弹窗，译名独立增删和上下排序，删除二次确认。读刷新失败只重试读取，写失败保留字段。
- 层级立即切换，独立淡入 150ms；减少动态效果时关闭过渡。焦点、Escape 和弹窗隔离沿用应用约定。

Translator 查询器参考 `poprako-web@e5022518b8a8dab619723440e8f3e69078bcf593` 左下角术语组件，折叠 32px 高、展开显示库列表或词条列表，保持画布和右栏结构。全局库带标识且不显示编辑入口；漫画库可新建及长按／F2 编辑。查询面板交互期间隔离编辑快捷键，点击外部或 Escape 收起；复制首选译名不保存页面。只读模式保持原统计布局，不挂载术语编辑查询器。

### 术语导出

沿用 `poprako-web@916b8abb6971f74221e4055293f7c9654ee3a58d` 的 `TermbaseRow`、`TermbaseShareButton` 和 `use-termbase-export`：全局管理页与 Translator 的库行右侧提供 20px 分享按钮、12px 图标，导出中转圈并禁用同库重复操作。按钮独立于进入／长按／F2 编辑区域，支持键盘焦点，不改变当前库选择及搜索。只读全局库保留导出入口。

Native 差异：浏览器下载替换为 Rust 系统保存框，默认「库名.json」；取消无提示，实际保存成功后提示「已导出术语库」，失败提示重试或更换位置。Translator 反馈位于左上工具箱下方，不覆盖左下查询面板或拦截画布操作。过滤隐藏库行、折叠查询器、进入库详情及切换只读模式保留进行中的导出；原生保存启动后继续完成，页面离开后不显示旧反馈。不新增导出弹窗或改变 Translator 布局。

### 弹窗底部操作与术语导入

创建术语库弹窗同时参考同版本 Web 的 `TermbaseEditorDialog` 与 `TermbaseImportButton`：提供「导入」按钮，只在创建时显示，从 JSON 创建完整库。选择取消不关闭创建弹窗；导入失败保留输入，提交未知时只提供原请求核实。

所有弹窗底部操作使用 `ButtonGroup`：网格等宽分配、共同占满横向可用空间，单个按钮占整行，长文案换行。删除确认、保存、导入、任务恢复等操作遵守同一规则。已有右上角关闭叉号时移除重复的底部取消、关闭、完成及取消离开入口；叉号保留原有任务清理、结果确认与关闭保护。仅返回上一编辑步骤的按钮保留，并明确标为返回。

Home 顶部快捷区保留用户既定布局：左右宽度 2:1，左侧继续最近章节，右侧新建与导入上下排列；不得以统一按钮等宽为由改成三列。分页、设置分类及预览页操作栏等分宽度。表单行内图标与窗口控制保持统一尺寸，不作为文字操作按钮组分配。
