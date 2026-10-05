# 首期离线 Translator 实施方案

本文件集中定义首期协议、IPC、偏好、验证与实施顺序。工程和领域约束以 [REQUIREMENTS.md](./REQUIREMENTS.md)、[MODEL.md](./MODEL.md) 及关联契约为准。

界面依据与复核见[界面来源对照](./contract/ui-reference.md)。模型存储、保存协调、图片资源及迁移验收分别见 [docs/contract](./contract)。

## 1. 实施边界

- 只做 home、comic detail、translator，Windows x64／macOS arm64，完全本地运行；保留已经确认的翻校、搜索替换、交换和图片管理能力。
- 业务归路由，多个页面的共享业务提升至共同父路由；Rust 承担 I/O，前端只通过 bridge 使用生成契约，不保留重复的 API/model 示例。
- 完成范围以源码和实际检查记录为准，不能用规格替代运行门禁。提交与发布属于独立交付步骤。
- 技术失败优先在既定规范内解决，不自动升级 Tailwind 4.1、取消 Orchestra、关闭类型检查或扩大支持平台。确实无法满足已有要求时报告具体冲突，而非再拆一轮偏好问卷。

## 2. 文件交换

### 2.1 原生 PRK v1

本地导出使用带显式格式与版本的 PRK JSON；旧 server/Web PRK 为输入适配格式。原生文件不冒充当前 server API 的 chapter DTO，也不承诺可直接上传旧 server。LP 是面向既有单文本工具的兼容输出。

原生 JSON 使用 UTF-8 和 snake_case，完整形状如下。示例是协议说明，不是手写 IPC DTO。

```json
{
  "format": "poprako-native",
  "format_version": 1,
  "coordinate_space": "display-oriented",
  "comic": { "title": "示例项目", "subtitle": "", "author": "" },
  "pages": [
    {
      "index": 0,
      "image": {
        "path": "images/001.png",
        "original_name": "scan-01.png",
        "format": "png",
        "width": 1200,
        "height": 1800
      },
      "units": [
        {
          "index": 0,
          "x_coord": 0.5,
          "y_coord": 0.25,
          "is_bubble": true,
          "is_flagged": false,
          "translated_text": "",
          "proofread_text": "",
          "is_proofread": false
        }
      ]
    }
  ]
}
```

- 所示字段均必需、类型严格。可留空文本用空字符串；pages/units 可为空。索引必须唯一且连续，接收数组可以无序，校验后按 index 排列。布尔字段不接受 0/1 或字符串替代。
- 不导出本机 ID、时间、unit_revision、工作位置、应用偏好或绝对路径；导入按既定规则重新生成身份和本次创建时间。
- 图片 path 始终使用按当前页序生成的 images/编号.实际格式扩展名；译文 ZIP 同样保留此逻辑引用，实际图片由外部目录按顺序配齐。original_name 独立保留原始名称，不用于匹配或生成受管路径。
- width/height 是方向校正后的显示尺寸；实际配图解码后必须相符，原图格式也须相符。旧格式没有这些声明时，从配图得到，不伪造原始元数据。
- 仅接受已支持的格式版本与坐标空间。未知版本拒绝；已知版本中的未知附加字段可忽略但在导入预览集中提示，不允许用缺失必需字段掩盖拼写错误。重复 JSON 对象键拒绝，不采用最后一个覆盖。
- PRK 保留完整双阶段正文、空行、标记和确认状态。导入使用已定空白规范化，含非空白内容的正文不裁剪、不做 Unicode 正规化。

### 2.2 历史 PRK 输入

输入依据为 `poprako-server@7cb534e4b9103a0893b64bc49008ccc46f766497` 的翻校 DTO 与导入／导出器，以及 `poprako-web@ca909afce66049e4d877a3958695ca43094d3eae` 的 chapter 导出与 ZIP 解析。建立三个独立识别分支：原生 v1、server snake_case、Web camelCase。识别成功后由对应适配器严格解析，不给所有字段散加 alias，不把坏的新版本降级成旧格式。参考来源不是构建所需的相邻目录。

| 输入 | 本地映射 |
| --- | --- |
| server 根 comic_title、chapter_subtitle | 项目标题、副标题；作者缺失为空，导入预览可修改资料 |
| Web 根 comicTitle、chapterSubtitle | 同上，先统一命名；exportedAt 等导出元信息不写入本地内容时间 |
| server/Web 页和单元序号 | 页序须连续完整；旧单元序号允许有间隔，唯一非负后按序重编为连续序号 |
| 双阶段文本和校对确认 | 分别映射，不按当前模式过滤；合法的缺失／null 文本转为空字符串 |
| 缺少关注字段 | 初始化 false，并在预览说明来源未携带关注信息，不能宣称恢复了丢失值 |
| 外部身份、贡献者、comment 等非本地字段 | 不生成本地实体，不用于合并；不可保留的信息集中提示 |

- 不套用旧 server 的 200 页／每页 100 单元硬上限，仍执行本地资源和数值校验。
- 混用两套命名、矛盾的归属／序号、越界或非有限坐标、缺失必需的校对确认等都明确失败，不能裁剪坐标来接受无效输入。
- 旧 Web exportedImagePath 用于在包内定位对应页图片，不据此生成任意本机路径。旧包有缺图时不能创建半项目，必须在预览中配齐所有图片。
- 旧格式没有坐标空间声明时，按方向校正后的显示坐标解释并在预览提示核对；不凭外部 ID 或元数据猜测旋转标记。参考项目的现行导出／导入缺口不伪报为已验证的互操作。

### 2.3 LP 与 ZIP

- LP 沿用参考 LabelPlus 的页标记、单元标记和框内／外分组语法；导入允许 UTF-8 BOM，接受 LF/CRLF。非法 UTF-8 明确报错，不猜测系统编码。
- 原生导出的 LP 头固定为 `1,0`、`-`、`框内`、`框外`、`-` 和生产者说明六行；页面按当前顺序使用一基编号文件名，单元编号每页从 1 开始，组 1 为框内、2 为框外。导出不带 BOM，统一 LF；例如：

```text
1,0
-
框内
框外
-
Exported by PopRaKo Native


>>>>>>>>[001.png]<<<<<<<<
----------------[1]----------------[0.5000,0.2500,1]
最终文本

```

- 导入以文件中页块顺序为准；页内正整数单元编号须唯一，允许间隔并按序重编。兼容结构行末尾的空格／Tab；保留参考解析器丢弃正文完全空行、统一换行为 LF 的已知限制，并在导入预览提示 LP 损失。
- LP 输出预检拒绝 NUL 和会被页／单元解析器识别为结构或畸形结构的正文行；不发明私有转义让既有工具误读。原生 PRK 继续用 JSON 转义保留其能够表达的正文。语法依据为前述固定 server 版本的 `src/complex/chapter_port/export_translation.rs` 与 `src/complex/chapter_port/import_translation/label_plus.rs`。
- LP 导出只写最终文本和四位小数坐标，是有损兼容格式；空行、精度、双阶段文本、关注与校对确认不能靠 LP 完整往返。PRK 始终是本地完整内容来源。
- LP 的结构行与正文存在歧义，按参考解析器的识别规则预检；无法无歧义表示的正文返回具体页面／单元错误，不输出会被当成其他页面或单元的损坏 LP，也不静默改写正文。两文件 ZIP 的这项限制需要在导出预检结果中说明。
- 两种 ZIP 均含 translation.prk.json、translation.lp.txt；含图包再含全部 images/ 原图。没有页面的 comic 可以输出资料、空页面数组和合法的空 LP 文档；已有页没有 unit 仍保留其页块。
- 只识别一个明确的根译文入口。PRK 优先，存在但损坏就失败；不能自动退回 LP。旧 assignments.txt 可忽略，不转为成员或 workflow。
- 归一化条目名称后拒绝重复／大小写或 NFC 冲突、绝对路径、..、驱动器／UNC 路径、符号链接及加密条目。逻辑路径用于匹配，实际解压写入生成的 staging 文件名，不把条目路径直接拼到文件系统。
- 只解压被选定译文引用的图片；未知附加文件不执行、不解压。核对 CRC、实际展开字节、声明长度和可用磁盘；限制解析元数据的工作空间，不仅按压缩包体积或压缩率推断安全。
- 初始文本元数据预算 64 MiB，超限明确拒绝，作为资源安全限制而非页数上限；ZIP 目录解析、累积文件大小及单条目解压使用受检查计数。若实际合法样例需要更高预算，在内存验收内调整，不靠无界分配兼容。
- 导入预览绑定解析后的输入副本、配图、目标版本与覆盖方式；用户确认后复核目标版本，再整体提交。来源或目标变化不能让旧预览确认写入新内容。
- 导出使用保存后的内容快照及不可变图片引用，流式写到目标目录内的唯一临时文件；完成并校验后再发布为成品。目标已存在时明确覆盖确认，取消或失败不覆盖旧成品。图片缺失使含图导出整体失败，译文导出独立可用。

## 3. 用例和 IPC

下表是用例契约清单。Rust DTO 与 command 实现后单向生成 TypeScript；不把本文转换成另一套手写前端 DTO。命令仅接受当前用例所需字段，事务在 usecase 内，I/O 在 implementation 内。

| 用例组 | 输入 | 输出与原子性 |
| --- | --- | --- |
| list_comic_infos | 已枚举排序、分页游标、页大小 | 资料摘要、派生进度、封面句柄、最近位置；默认最近打开、其次最近修改、ID 打破平局，50 项一页 |
| get_comic_detail | comic ID | 资料、全部页面摘要及派生计数；不返回全项目 unit 正文或原图字节 |
| create_comic / update_comic_metadata / delete_comic | 明确的资料或目标 ID | 空项目允许；资料规范化；删除内容及工作位置原子完成，文件清理按资源契约后置 |
| reorder_pages / remove_pages | comic ID、完整顺序或选中 ID 集合、读到的页序基线 | 校验实际集合和基线，事务重排／删除并修复工作位置；不信任只显示出来的列表 |
| get_page_editor | page ID | 完整单元集合、unit_revision、图片句柄和显示尺寸；完成前不允许提交快照 |
| save_page_units | page ID、expected_revision、完整有序单元快照 | 规范化后的已提交单元、版本、时间；差异写入及父级时间同一事务 |
| get_work_position / update_work_position | comic ID；明确的页／单元定位和模式 | 校验归属，真实缺失可为 null；浏览不改变内容时间 |
| search_units | comic ID、文本阶段、查询串 | 先保存当前草稿，返回带页面版本的命中；大小写敏感字面子串、超过 100 条报错 |
| replace_unit_texts | 选中 ID、阶段、规则、预览涉及的页版本 | 整批事务提交，返回实际变更数及受影响页版本；统计来自实际变更，不把请求数当结果 |
| prepare_import / confirm_import | 来源授权句柄、目标、新建资料或覆盖方式；确认用预览句柄 | Rust 持有已解析快照与受管副本；确认后整体应用，过期预览不写入 |
| prepare_page_images / confirm_page_images | 目标 comic/page、增页或换图意图、来源句柄、保留／清空选择 | 完整准备与预览后提交；换图保留 page 身份，清空单元时推进 unit_revision |
| export_comic | comic ID、译文／含图枚举、目标授权句柄 | 后台任务；返回可查询 task ID，不直接返回成功下载 |
| get_preference / update_preference | 完整强类型偏好及本会话读到的偏好基线 | 事务比较基线再写单行 JSON，返回规范化完整偏好；不另加偏好版本表，不暴露任意 key/value API |
| choose_import_source / choose_export_destination | 有限用途枚举 | Rust 打开原生对话框；返回 cancelled 或 selected 判别联合，selected 为短期授权句柄 |
| get_task / cancel_task | 当前进程的 task ID | 权威阶段与最终结果；取消是请求，不等于回滚；事件可丢失，查询可恢复状态 |
| get_image_resource / get_image_tile | page ID、图片引用代次、预览规格或图块坐标 | 有限范围资源句柄；验证归属／预算，不接受任意路径，不回传 Base64 大图 |

### 3.1 共同约束

- 纯数据生成 type，纯函数能力接口若确有消费者才用 interface，Props 始终为 type。Props 的无对象场景由视图分支或判别联合表达，不用伪造实体补齐。
- 输入 schema 严格校验 ID、枚举、有限坐标、安全整数及完整集合；所有 SQL 值绑定。模型字段不因为前端隐藏控件而免校验。
- 列表摘要由查询聚合得到，不加可独立写入的计数；页面内部虚拟渲染不等于只加载部分 unit 后提交完整快照。
- 每个编辑会话拥有基线、草稿、固定在途快照和代次；保存状态由它们推导。Zustand 可承担当前会话，React 管理局部弹层；首期不用 Query 再复制一份可编辑内容。
- 读请求在页面切换后用代次丢弃旧响应。保存与批量动作成功后按回执精确推进基线／失效摘要，不能用失败重试重放已经提交的操作。
- 任务由 Rust 持有。阶段为 preparing、awaiting_confirmation、committing、cleaning、completed、failed、cancelled；completed 可附清理警告，失败包含提交状态。未知／过期 task ID 与任务失败区分。
- 同一进程中最终任务结果保留至消费确认或应用退出，避免仅靠一次事件通知判断提交。重启后的文件恢复由数据库引用和 staging 扫描完成，不把内存任务表当持久化事实。
- 错误统一携带可识别 code、安全 message、可选页面／单元定位和 recovery 枚举；详细诊断只在 Rust 错误源记录一次，不输出内部 SQL 或完整用户路径。
- 基本错误区分 validation、not_found、conflict、resource_missing、resource_limit、storage_full、storage_busy、unsupported_format、commit_unknown、cancelled、internal；是否可重试依据实际阶段，不仅依赖 code。

### 3.2 偏好与默认值

- 固定浅色；旧偏好中的 system／light／dark 经完整校验后移除 theme，其余设置保留，损坏数据不重置。编辑模式属于每个 comic 工作位置，不混进全局偏好。
- 快捷键由有限 action → binding 映射组成，binding 是 unbound 或 chord 判别联合；提供恢复默认和冲突检查。默认键位、顺序和动作直接复刻 `poprako-web@46dd776` 的 `src/route/_authenticated/translator/business/preference/use-shortcuts.ts`，Windows 与 macOS 均使用明确的 Ctrl，不转换为 Cmd。
- Ctrl＋M 切换模式，Ctrl＋L 切换重定位，Ctrl＋X 切换标记显示／淡化，Tab／Shift＋Tab 循环选择下一个／上一个标记，Ctrl＋U／D 上一／下一页，Ctrl＋Q 输入最近一次符号（未使用过时取字符表首项），Alt＋1／2／3 输入前三个优选符号，Ctrl＋S 保存。标记导航在首尾循环，切页停在边界；Escape 取消单元选择。Ctrl＋X 与工具栏眼睛按钮共享暂态，不改写 marker_opacity 偏好。
- 单元输入框中快捷键正常生效，选择标记后滚动并聚焦输入末尾；符号插入替换选区，在 DOM 提交时恢复光标并消费请求，返回页面不会重放，也支持从画布向已选单元插入。最近符号、标记显示／淡化等工作台状态跨页保留。其他输入框、组合输入、AltGr、已处理事件、弹窗、只读统计展开、切页锁和提交待核实状态不拦截工作台键位。WKWebView 会把正常 Option 数字键报为 keyCode 229；此时按 code 匹配，并以实际 composition 状态保护正在组合的输入。只读保留标记导航、切页、模式和透明度切换，屏蔽编辑操作。
- 旧默认键位通过一次性偏好数据升级改为 Web 键位，与旧默认不同的自定义键位／解绑保留；旧默认未绑定的符号和显示操作补齐 Web 绑定，新默认与自定义冲突时自定义优先。旧 canvas_navigation 范围升级为 editor；旧 cycle_marker_opacity action 兼容读取并序列化为 toggle_proofread_preview。升级标记与偏好写入在同一事务中，重开不会再次覆盖用户配置。
- 特殊字符为有序的强类型条目：稳定 ID、非空字符文本、收藏布尔值；顺序即数组顺序。初始顺序为 `♪`、`「」`、`『』`、`❤`、`●`、`★`、`☆`、`♡`、`○`、`※`，全部收藏，沿用前述固定 Web 版本的 `src/hook/useSpecialChars.ts`。内置条目使用固定 ID，自定义条目使用 UUID v4；空列表合法，不把字符内容当稳定身份。
- 重定位为 `relocation_enabled: false`，启用后选中单元时自动将画布定位到该标记；手动定位入口始终可用。另设 `marker_opacity`，有限值 `(0,1]`，默认 1。方向、缩放等暂态不是此偏好隐含字段。参考前述固定 Web 版本的 `src/route/_authenticated/translator/business/preference/use-relocation-preference.ts`。
- 偏好升级纳入数据库迁移；错误数据保留且可诊断，不静默覆盖为默认。完整字段和默认值在 Rust 结构中只有一份，生成结果向前端提供。

## 4. 第一阶段技术门禁

以下表格定义工程门禁。实际工具链与依赖由 `.deno-version`、`rust-toolchain.toml`、包配置及锁文件固定；最低系统、Windows 和完整性能验收仍需目标环境实测。

| 门禁 | 实现约束 | 通过证据／失败处理 |
| --- | --- | --- |
| Deno 与前端 | 固定 Deno 2.9.6、Vite 8.0.16、React 插件 6.0.2、TS 6.0.3；Tailwind 4.1.18 走 PostCSS | 安装、HMR、构建和严格类型通过；不用 peer 不覆盖 Vite 8 的旧 @tailwindcss/vite，不能偷升 Tailwind 4.2 |
| 路由生成 | src/route，business 子树忽略，生成文件在外部 | 嵌套路由正确、排除范围精确、生成检查只读；解决生成器抑制头，不手改生成物 |
| IPC | 精确固定 tauri-specta/specta 2.0.0-rc.24 与 specta-typescript 0.0.11 | Rust/TS 实编译，Result/Option/枚举与 i64 安全整数生成一致，无重复 Specta 版本破坏接口 |
| Orchestra / SQLx | 最小 command → usecase → Context 事务，临时文件库 | 实际 Future 满足 Tauri Send／生命周期，提交／回滚／取消边界可验证；不取消架构来掩盖泛型约束 |
| SQLite 运行时 | SQLx 0.9.0；验证实际 SQLite 含 WAL 修复并记录来源及版本 | STRICT、逐连接外键、受检查查询、真实锁竞争；不能只看 Cargo 包版本，必须验证实际绑定的运行库 |
| 图片管线 | 固定支持四种静态格式及动画检测的解码器 | 格式／方向／高位深／畸形输入与预算覆盖明确；Limits 的尽力限制不能当硬隔离 |
| 质量任务 | 落实统一 deno task 入口和既定专项规则 | 一次完整 check 通过，负样例能拦截错误 type/interface、未处理 Promise、400 行及禁用 Rust 写法 |

依赖精确版本以实际通过结果入锁。上表保留门禁依据，实际解析版本以锁文件为准。缺少 Windows 环境不阻止宿主验证，但 Windows 部分阻塞该平台交付；不伪报双平台通过。

## 5. 实施顺序与完成条件

| 阶段 | 实现内容 | 退出条件 |
| --- | --- | --- |
| 0：工程基础 | 上述技术门禁、任务入口、路由、生成 IPC、数据库与迁移协调 | 已选路径有运行证据，记录平台缺口；依赖未过的模块不进入后续阶段 |
| 1：最小离线闭环 | 五表／迁移、图片文件夹导入、home/detail、单页 unit 编辑与保存、重启恢复 | 创建→进入页→标记／翻校→15 秒保存→退出重开保持内容；失败保留草稿 |
| 2：完整页与资源管理 | 重排、增删、换图、工作位置、预览／图块、取消／清理、单实例与正常退出 | 数据库和文件阶段故障不误删引用；快速切页有界；正常退出及异常重启符合契约 |
| 3：翻校能力 | 三模式、关注／页提示、差异、统计、特殊字符、快捷键、搜索替换 | 参考能力矩阵逐项通过；确认和文本独立，批量原子，无旧回执覆盖 |
| 4：文件交换 | 原生／历史 PRK、LP、两种 ZIP、预览与整体导入、快照导出 | 普通、空、边界与历史夹具通过；原图字节保留，坏 PRK 不降级，失败不交付半包 |
| 5：双平台交付 | 性能、升级／恢复、最低系统安装与离线运行、ad-hoc 签名及首次打开验证 | 所支持平台逐项有记录；未满足条件的平台不得作为已支持发布 |

实际实现遵守路由业务归属、生成类型和检查约束，不引入平行存储。

UI 必须直接对照固定版本参考组件迁移：Translator 以 Web 为准，Home 和详情以旧 Native 为准，并保留用户本地调整。不得增加全宽编辑顶栏、常驻左侧缩略图或单元卡片。组件、尺寸及本地化差异见[界面来源对照](./contract/ui-reference.md)。

## 6. 验收与交付

- 自动验证重点：新库／升级失败保数据、完整快照差异与版本、重排唯一约束、失败回滚、空白规范化、确认独立、输入法与在途保存、旧会话响应隔离、交换夹具及原图字节一致性。
- 资源故障覆盖复制、发布、提交前、提交结果未确认、提交后清理、磁盘不足、取消及重启恢复；使用临时文件库与目录，不以单连接内存库代替实际语义。
- 质量执行遵守 REQUIREMENTS 的格式／lint／类型／Rust／生成一致性／SQL 元数据／风险测试；不为简单展示写形式化测试，不用整目录豁免换取通过。
- 普通性能夹具：20 页、每页 30 单元、2048×3072 静态图片；压力夹具：200 页、每页 100 单元，混合常规及接近 32MP／16384 边界图片，并包含大于 50 MiB 的合法 BMP／PNG。文件大小来源和编码参数记录，不假定每张都 50 MiB。
- 在 8 GB＋SSD 机器记录 CPU、OS build、WebView、应用与依赖版本；普通交互和缓存切页各采样至少 30 次，记录中位／p95／最大值，以 p95 ≤100ms／300ms 验收。已有普通项目冷启动至可操作至少 10 次，以 p95 ≤2s 验收；首次解码等另列。
- 内存采样涵盖 Rust 与本应用 WebView 相关进程；记录平台口径与共享页重复计入影响，GPU 单列。连续翻完压力项目后稳定目标 ≤1 GiB，任务峰值 ≤1.5 GiB；不以 JS heap 代替总体占用。
- Windows 10 22H2 x64 和 macOS 13.3 arm64 分别验证安装、中文输入、焦点、保存退出、资源加载、导入导出和卸载后的数据策略。卸载默认不主动清空用户数据。
- Windows 使用用户电脑已有的 WebView2 Runtime，安装包不内置、不下载运行库（`webviewInstallMode: skip`）。安装与业务运行不发起运行库下载；目标电脑需已有满足前端能力的 WebView2，缺失环境须明确诊断。两平台安装包各不超过 10 MiB。
- macOS SDK／Xcode、Windows 构建环境是明确外部条件。首发采用 macOS ad-hoc 签名、Windows 无签名，须验证该模式下的首次打开与安装行为；Developer ID 证书和公证属于后续升级条件。
- 提交与发布须由用户明确授权；源码、生成文件与锁文件按既定检查管理变更。

## Release PR 与首发版本

- 首次正式版本为 `1.0.0`。应用的 package、Tauri、Cargo、锁文件与发布 manifest 保持同步。
- 日常 PR 与 main 提交运行双平台检查；首发保持单分支、单根提交，手动运行 Release 构建同一提交上的标签。后续手动启动 Release PR 汇总版本和日志。两平台产物齐备、每个平台安装包不超过 10 MiB、来源和校验值一致后统一上传到草稿。
- 首发采用 macOS ad-hoc 签名、Windows 无签名，允许在此模式下公开发布。默认保留草稿，实际签名和安装验收后开启 `RELEASE_PUBLISH`；可选的 Developer ID／Authenticode 模式由 `RELEASE_SIGNING` 控制。Release 正文自动附上当前模式及安装说明。工作流、Secrets、维护者操作及失败恢复见 [CI 与版本发布](./release.md)。

## 根路由自定义窗口外框

- 根路由统一使用 `WindowFrame`，Home、Comic Detail、Translator 以及根路由加载／错误状态共享 36px 淡绿色标题栏。右侧图标提供最小化、最大化／还原和关闭；空白标题区拖动及双击最大化，边缘提供八方向缩放。
- 禁用系统 decorations，透明窗口背景与 12px 外框圆角配合，普通窗口四角透明；最大化／全屏移除圆角及缩放热区。页面使用标题栏下方容器高度，不再占整屏高度。
- 窗口调用集中在 `bridge/window.ts`，使用 Tauri 内置 Rust window command 和仅限 main 的细粒度 capability；关闭发出 CloseRequested，沿用已有草稿保存、任务核实和退出保护，不直接调用 requestExit。
- 标题栏的键盘焦点与 Translator 全局快捷键隔离；按钮保留可访问名称、提示和焦点样式。窗口状态监听处理异步卸载与旧回执；操作失败显示可重试提示。
- macOS 透明窗口启用 Tauri `macos-private-api` feature 与 `app.macOSPrivateApi`；此配置不适用于 Mac App Store 分发（[Tauri 配置说明](https://v2.tauri.app/reference/config/#windowconfig)）。当前项目采用独立桌面包，签名与公证仍按交付阶段验证。

## 单一桌面图标源

只维护 `src-tauri/icons/icon.png`（256×256 RGBA）。删除多尺寸 PNG、Square／StoreLogo、重复母版和已提交的 ICO／ICNS。Cargo build script 直接将相同 PNG 字节封装为单图层 ICO／ICNS，在已忽略的 `src-tauri/target/generated-icon/` 下生成，供 Tauri 打包与图标嵌入使用。无需额外工具、图像重采样或新增依赖，直接 cargo check／Tauri dev／package 均使用同一路径。生成器要求源图尺寸恰为 256×256，并只在派生字节变化时写文件。更新方式见 asset/README.md。

## 用户可见名称

- 前端标题栏和 HTML 标题固定为「白杨子 N」。系统默认产品名为 `PopRaKo Native`，简体中文显示为「白杨子 N」。保留 `com.poprako.native`、Cargo 包名、资源协议与交换格式标识，已有应用数据目录不因品牌名变化而迁移。
- macOS 包中包含英文、简体中文 `InfoPlist.strings`，声明开发语言、本地化语言与 Finder 本地化显示名称。启动时通过 Foundation 的安全接口读取系统解析后的 `CFBundleDisplayName`，同步 Tauri package name 与窗口标题，使默认应用菜单及关于窗口使用同一名称。Foundation 已在依赖树中，新增仅为 macOS 的显式依赖，不引入 unsafe 代码。[Apple 本地化 Info.plist 说明](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/AboutInformationPropertyListFiles.html)
- Linux 的 deb／rpm 使用统一 desktop 模板，包含 `Name=PopRaKo Native`、`Name[en]=PopRaKo Native`、`Name[zh_CN]=白杨子 N`；AppImage 通过 Tauri 的 Debian 数据目录生成流程复用该模板。[Desktop Entry 本地化规范](https://specifications.freedesktop.org/desktop-entry/latest/localized-keys.html)
- Windows 默认仅生成 NSIS 包，支持 English／SimpChinese，安装时按系统语言选择。安装完成后设置应用列表 DisplayName，并将本安装拥有的开始菜单／桌面快捷方式调整到相应名称；处理 Finish 页晚于 POSTINSTALL 创建的桌面快捷方式、重复安装和卸载清理，升级时保留快捷方式。注册表身份保持英文固定名称。系统语言切换后已有快捷方式／应用列表不会自动改名，需按新语言重新安装；前端名称始终保持中文。钩子依赖现有 `poprako-native.exe` 和默认开始菜单根目录，改变 default-run 或 startMenuFolder 时需同步修改。[Tauri NSIS 配置说明](https://v2.tauri.app/reference/config/#nsisconfig)
