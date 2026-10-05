# PopRaKo Native

PopRaKo Native 是完全离线的本地漫画翻校工具，使用本地图片组织独立翻校项目。

## Language

**Comic（翻校项目）**：一组有序页面及其翻校内容组成的独立工作对象。本地 comic 对应 PopRaKo 服务端的 chapter，不包含另一层章节。
_Avoid_: Project、Chapter 作为本地域内的独立层级；将本地 Comic 等同于服务端 Comic

**Page（页面）**：Comic 内的一张有序图片及其翻校单元。页面身份与页码、图片文件名相互独立。

**Unit（单元）**：Page 内一个有序、带位置的翻校标记，包含框内外类别、文本和校对确认状态。
_Avoid_: 将 Unit 限定为气泡或矩形文本框

**Translation（翻译文本）**：Unit 的翻译阶段文本，与校对阶段文本分别保留。

**Proofread Text（校对文本）**：Unit 的校对阶段文本，可以在没有翻译文本时独立存在。具有校对文本不代表已经确认校对。

**Proofread Confirmation（校对确认）**：对 Unit 作出的明确校对确认，与校对文本是否存在相互独立。
_Avoid_: 将“有校对文本”直接等同于“已校对”

**Final Text（最终文本）**：Unit 用于结果展示和单文本交付的内容，取非空校对文本，否则取翻译文本。最终文本不表示该单元已经确认校对。

**Flag（标记）**：为后续关注而给 Unit 添加的标识，与翻译内容和校对确认相互独立。

**Blank Page（空白页）**：尚无任何 Unit 的 Page。已有定位标记但尚未填写文本的页面不属于空白页。

**Empty Comic（空项目）**：没有 Page 的 Comic，仍保留项目资料，能够继续添加图片。

**Editor Mode（编辑模式）**：翻译、校对和只读三种操作方式，使用者能够自由切换。模式不表达成员权限或项目工作流阶段。

**Work Position（工作位置）**：使用者在某个 Comic 中上次停留的页面、单元和编辑模式，用于继续工作。

**Translation Archive（译文包）**：包含 PRK JSON 与 LP TXT 两种译文的 ZIP，不包含页面图片。

**Image-inclusive Archive（含图片包）**：包含两种译文及全部页面图片的 ZIP，用于离线交接和继续翻校。
_Avoid_: 将项目包理解为包含应用偏好、编辑历史和本机工作位置的整机备份
