# 阅读器与替换方向

更新日期：2026-11-09。本文为 M2，说明现有自研阅读器、分析坐标、选择和进度，并保留阅读器替换的集成关注点。EPUB 导入和书库见 [M1](library.md)，说明交互见 [M3](explanation_layout.md)。

## 正文与坐标

`App.vue` 将书库 Markdown 或文本输入交给 `compileReaderDocumentAsync`。每次编译创建一个 `document.worker.ts` Worker，完成后终止，返回 `ReaderDocument`；书籍位置用于后续打开文档会话。

| `ReaderDocument` 字段 | 当前含义 |
| --- | --- |
| `metadata`、`markdown` | frontmatter 的标题、作者、日期、语言，以及清理后的 Markdown |
| `analysisText` | 提交文档会话的正文，保留供准备阶段识别的作者注音，文字块之间使用两个换行 |
| `blocks` | heading、paragraph、image；文字块有 `charRange`，图片有 `charOffset` |
| `chapters` | 从标题生成的 ID、名称、层级和字符偏移 |
| `images`、`cleanup` | 图片块集合，以及锚点、属性、HTML、目录和提示清理计数 |

[document.ts](../src/reader/document.ts) 清理 frontmatter、展示属性、目录残片、设备提示和行内标记，提取标题、段落及图片。图片占据阅读布局空间，字符锚点位于相邻正文位置；图片自身不增加分析字符数。

文字块范围按准备后的 Unicode scalar 计数，使用半开区间。`analysisText` 中的注音标记经 Rust `prepare_text` 处理后形成 `plan.prepared.text`；编译器的 `preparedCharacterLength` 采用相同注音有效性规则计算块偏移。原始 Markdown、提交文本、准备后正文与 DOM 文本位置各有坐标用途。

文档会话以全文字符位置划分单元，单元文档及查询目标使用局部坐标。阅读器保存单元 context 起点，将单元命中换算为全文位置，再与块、行、选区和收藏关联。作者 ruby 的原文映射、UTF-8 byte 和 JavaScript UTF-16 的转换见 [L2](language_analysis/lexical_input.md#正文坐标与来源引用)。

## 行、图片与虚拟滚动

`rows.ts` 将文字块和图片组织为异构阅读行，`virtualization.ts` 提供行估算及字符位置查找，`ReaderDocumentView.vue` 使用 `@tanstack/vue-virtual` 创建可见行。行 key 和正文字符锚点维持对象关联，虚拟行索引供当前布局使用。

图片由 `ReaderImageBlock.vue` 读取资源、宽高及加载状态。桌面通过 `convertFileSrc` 访问本机文件，静态书籍采用 URL；资源表通过原始 href 对应导入文件。图片完成或失败触发行测量，排版和分析更新同样影响行高。

可见行范围驱动 `request_range`，新单元分析和查询目标完成后更新对应正文。首次打开需恢复位置附近的单元完成；进入阅读后，`continue_document` 继续处理全文。静态浏览器按相同阅读范围读取已生成分片，见 [E2](static_library.md)。

## 胶囊、注音与查询

正文胶囊依据词典外层目标和活用父子链确定显示范围，内部保留词汇成分、活用命中及注音边界。`ReaderCapsule.vue` 与 `capsuleRuns.ts` 负责文字片段、ruby 和背景分段，`lookupPresentation.ts` 负责目标、词形和高亮的内容投影。

作者注音先采用已校验的范围，其余保留原注音；读法显示由 `readerRubyAnnotations`、`kanjiRubySegments` 等函数组织。注音边界和词汇目标相交时继续保存各自范围，具体关联见 [L11](language_analysis/presentation.md)。

固定布局点击后显示词典及当前分析说明，悬浮布局采用 60 ms 的悬浮意图延迟。整体词及当前位置内部词各有查询状态与历史；活用说明根据已生成的分析同步选择，词典请求异步完成。词典表记、记录和关联导航见 [M5](dictionary.md)。

## 选择、收藏与导出

鼠标普通拖动按成分合并查询范围，配合 `Shift` 使用字符位置进行自由范围选择；范围由当前单元、文字边界和排除区域约束。位移达到 5 px 后进入拖动状态，关闭当前查询并捕获指针。普通点击直接切换查询对象。触摸通过 `useTouchBrush` 处理成分选择和移动。

组件保留 DOM 选区到正文范围的转换，复制使用实际正文；保存操作由当前选择生成摘录。选择与导出面板显示已保存内容，可编辑笔记、删除单项、清空当前书籍版本和导出 JSON。

| `SavedSelection` 字段 | 含义 |
| --- | --- |
| `book_id`、`text_version` | 书籍及正文版本；文本输入使用 `__text__` |
| `start`、`end` | 准备后全文字符范围 |
| `surface`、`base_form`、`reading` | 选择表记、查询基本形和读音 |
| `note` | 用户笔记 |

桌面 `reader-state.sqlite` 和浏览器静态 localStorage 均按书籍、文本版本及起止位置保存摘录。`App.vue` 的 JSON 导出包含书名、文本版本及完整 `selections` 数组。当前摘录对象采用单段范围；后续收藏对象、统计和 Anki 字段在方案确定后设计。

## 章节与进度

章节面板根据 `ReaderDocument.chapters` 显示层级。点击目录后定位标题行，缺少对应标题行时按字符偏移寻找目标；可见范围变化继续触发分析范围请求。全文查找读取准备后正文，`Ctrl+F`／`Cmd+F` 打开，`Enter`／`F3` 和 `Shift` 切换匹配项。

滚动时以首个可见行的字符起点更新当前位置，700 ms 停止滚动后合并提交位置、当前章节和本次秒数。当前组件卸载会清除等待中的计时器，退出前最近一次尚未提交的滚动位置可能丢失。再次打开书籍按已保存偏移创建会话，再通过虚拟行定位恢复阅读区域。

百分比使用准备后字符偏移与总字符数。预计完成时间采用每分钟 400 字的固定估算，见 `reading.ts`；累计阅读时长由桌面书库记录。浏览器静态适配保存位置及百分比，当前未累计 `seconds`。

排版默认字号 19、行距 2.05、段距 20、版心 760 px，支持在排版面板调整，并保存至 `readux.reader.appearance`。墨水屏模式由阅读器组件切换。说明侧栏展开时，正文行宽由预留后的阅读区约束，见 [M3](explanation_layout.md)。

## 阅读器替换方向

用户已确定评估 epub.js 或同类方案。现阶段整理集成要求，阅读引擎选型、接口与迁移步骤留待讨论。

| 集成对象 | 待明确的对应关系 |
| --- | --- |
| 原书与分析正文 | spine／XHTML 结构、文本提取及准备后字符的稳定映射 |
| 阅读位置与目录 | 原书位置、章节锚点和现有字符偏移的互换；旧进度的迁移方式 |
| 选区与收藏 | DOM／引擎选区到全文范围、实际表记与版本身份的转换 |
| 注音及正文高亮 | 原书 ruby 与分析读法的协作，胶囊及高亮覆盖范围的呈现 |
| 分析调度 | 从可见阅读范围请求分析单元，接收更新及处理取消、跳转和错误 |
| 说明界面 | 提供查询目标、屏幕锚点和节点选择回调，继续使用三种视图及图组件 |
| 图片与资源 | 本机文件和静态 URL、封面及正文图片的定位 |

代表核验对象包括多章节跳转与重启恢复、连续 ruby、混合空白、补充平面字符、图片加载引起的重排，以及选区与接续图节点对应。具体替换阶段和方案确认后进入实施文档。

## 实现入口与维护

| 范围 | 入口 |
| --- | --- |
| 正文编译 | [document.ts](../src/reader/document.ts)、[document.worker.ts](../src/reader/document.worker.ts)、[reader_markdown.rs](../crates/readux-core/src/reader_markdown.rs) |
| 行与虚拟列表 | [rows.ts](../src/reader/rows.ts)、[virtualization.ts](../src/reader/virtualization.ts)、[ReaderDocumentView.vue](../src/components/reader/ReaderDocumentView.vue) |
| 胶囊及说明投影 | [capsuleProjection.ts](../src/reader/capsuleProjection.ts)、[lookupPresentation.ts](../src/reader/lookupPresentation.ts)、[L11](language_analysis/presentation.md) |
| 目录、图片与排版 | [ReaderNavigationPanel.vue](../src/components/reader/ReaderNavigationPanel.vue)、[ReaderImageBlock.vue](../src/components/reader/ReaderImageBlock.vue)、[ReaderAppearancePanel.vue](../src/components/reader/ReaderAppearancePanel.vue) |
| 范围选择与应用保存 | [mergeLookup.ts](../src/reader/mergeLookup.ts)、[App.vue](../src/App.vue)、[reader_state.rs](../crates/readux-core/src/reader_state.rs) |

现有 UI 检查集中于 `npm run test:ui`，正文坐标与虚拟列表具有对应样例。修改阅读行为时按实际范围选择验证，界面检查同时核对正文范围、当前目标和保存后位置。
