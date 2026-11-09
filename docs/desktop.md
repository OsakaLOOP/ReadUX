# 桌面端使用与运行

更新日期：2026-11-09。本文为 E1，说明桌面端可访问的功能、运行资源与状态恢复。开发环境及构建命令见[开发说明](development.md)，书库结构见[书架与导入](library.md)。

## 启动与资源

Windows 便携包为 `packages/ReadUX-insider-portable-win64.zip`，解压后运行 `ReadUX.exe`。发行目录同时保存 `nlp/`、`dict-sources/`、`python/` 及 provider 脚本；程序依赖这些资源完成本机分析和查询。当前发行 CI 覆盖 Windows x64 便携 ZIP 与 macOS arm64 DMG，详细资源组织见[桌面发行资料](standalone_distribution.md)。

| 资源 | 当前作用与定位 |
| --- | --- |
| UniDic/Vibrato CWJ、CSJ | 提供书面语及口语分词，release 优先查找程序旁 `nlp/cwj.dic`、`nlp/csj.dic`，再查找打包资源；`READUX_UNIDIC_CWJ`、`READUX_UNIDIC_CSJ` 可覆盖 |
| `.kdict` 词典源包 | release 依次检查数据目录、程序目录和打包资源中的 `dict-sources/`，建立本机 `dicts/` 查询数据库 |
| GiNZA | 通过 Python provider 生成正式词元和 UD 结构；完整阅读单元依赖可用的 provider 结果 |
| 活用与解说 | 活用配置嵌入 Rust 服务，前端读取同源配置与词条解说，随程序版本交付 |

资源定位入口为 [lib.rs](../src-tauri/src/lib.rs) 与 `ResourcePaths`。debug 使用仓库开发资源；release 使用便携或打包资源及应用数据目录。`READUX_DATA_DIR` 可指定桌面个人数据目录；书库由系统文档目录独立定位。

开发者可按[桌面开发端口说明](development.md#桌面开发的端口配置)启动。当前前端启动端口为 5173，Tauri 配置中的 `devUrl` 为 1420，该节提供 5173 的覆盖命令。

## 书架与输入

应用启动后加载书架。书架显示最近阅读与全部书籍，可搜索书名、作者、章节、来源文件名和标签，按最近阅读、书名、进度或导入时间排列。书籍菜单提供详情、颜色、标签、重置进度及删除，桌面端提供本机目录操作。

点击导入按钮选择一份 EPUB。导入完成后应用刷新书架并打开该书；目录、正文、注音及图片由导入器处理。重复导入相同文件沿用书籍身份及进度。导入结果、目录和存储约定见 [M1](library.md)。

“打开文本”接受正文及当前编译器支持的 Markdown。输入进入独立阅读会话，收藏使用 `__text__` 作为书籍身份，并按文本版本区分。文本输入正文随本次会话使用；长期书目管理以 EPUB 书库为入口。

## 阅读与分析

打开书籍时，应用先读取书籍摘要及正文，在 Worker 中编译阅读块，再按已保存位置打开分析会话。恢复位置所属单元完成后显示阅读器，后续单元继续处理。分析单元包含 UniDic、GiNZA、统一分析及查询目标，完整缓存可复用已有结果。

顶部进度区域说明正文准备、分析或加载状态，失败时提供重试；阅读器在暂停或失败状态下显示相应继续、重试按钮。耗时记录开关及 JSON 导出位于分析进度区域。具体会话协议见[会话与调度资料](reader_engine.md)，请求入口见 [CLI](cli.md)。

阅读器支持章节目录、全文查找、排版、墨水屏模式和选择导出。`Ctrl+F`／`Cmd+F` 打开全文查找，`Enter`／`F3` 转至下一项，配合 `Shift` 返回上一项。正文、胶囊、图片、注音和定位见 [M2](reader.md)。

## 查询与分析说明

默认固定布局通过点击词汇或内部成分激活；宽屏显示右侧栏，视口不超过 960 px 时显示底部抽屉。工具栏按钮切换为悬浮气泡后，悬浮或点击可打开说明。

词典提供整体词及内部词汇、表记选择、词典切换和正文参照导航；活用区显示当前词形、功能与解说；接续区显示完整句中组成和当前前接规则。图中句中节点支持切换当前解释对象。内容与范围见 [L11](language_analysis/presentation.md)，视图交互见 [M3](explanation_layout.md)，图控件见 [M4](analysis_graphs.md)。

阅读器支持词汇的已知状态和曝光计数。收藏保存当前选择的表记、基本形、读音及出处范围，选择面板可编辑笔记、删除、清空和导出 JSON。当前导出根对象为 `title`、`text_version`、`selections`；收藏记录字段见 [M2](reader.md#选择收藏与导出)。收藏统计与 Anki 的详细设计按总计划继续讨论。

书架右上角进入语法指南。三课使用制作好的内容、分析及词典快照，复用正文和说明组件；指南资源制作见[开发说明](development.md#指南及书籍静态资源)。

## 设置与状态恢复

| 状态 | 保存与恢复 |
| --- | --- |
| 书籍位置、章节与阅读时长 | 书库 `library.sqlite`；再次打开书籍时恢复字符位置 |
| 摘录、笔记与词汇状态 | 应用数据目录 `reader-state.sqlite`；摘录按书籍和文本版本读取 |
| 字号、行距、段距与版心 | WebView localStorage 的 `readux.reader.appearance` |
| 阅读器词典顺序 | WebView localStorage 的 `readux.reader.dictionaryOrder` |
| provider 与耗时配置 | 分析数据目录 `providers.local.json` |
| 说明布局及当前激活对象 | 组件状态，创建时默认固定布局；进入页面后选择查询对象 |

排版与词典设置可从阅读器工具栏打开。provider 配置通过 `configure_providers` 等本机请求访问，字段和示例见 [E3](cli.md)；`ProviderPanel.vue` 保留实现，主应用当前没有挂载该设置面板。

应用启动进入书架；继续阅读由打开书籍触发。滚动停止 700 ms 后提交位置及本次时间，组件卸载会清除等待中的进度计时器。保存时机依此理解，快速退出前尚未提交的位置可能没有写入。桌面数据库与 WebView localStorage 各有保存范围，备份书库和个人状态时应分别定位。

## 问题定位与实现入口

| 表现 | 核对入口 |
| --- | --- |
| 开发窗口等待页面 | 核对 Tauri `devUrl` 和前端端口，见开发说明 |
| EPUB 无法导入或内容不完整 | 查看书架错误，核对原文件、`import/epub.rs` 的导入阶段及 warnings |
| 完整单元失败 | 查看会话错误、`check_providers` 响应及数据目录 `provider-logs/` |
| 查词为空 | 检查词典源包、查询数据库、当前表记和读音；见 [M5](dictionary.md) |
| 规则修改未反映在界面 | 核对服务构建、分析缓存及指南／书籍快照版本，见 [L5](language_analysis/morphology_configuration.md) |

应用操作由 [App.vue](../src/App.vue) 协调，桌面命令在 [lib.rs](../src-tauri/src/lib.rs) 注册，书库和个人数据由 [ReaderEngine](../src-tauri/src/reader_engine.rs) 访问。阅读器替换、收藏管理和 LLM 的后续方向集中见[开发目标](goals.md)。
