# 书架、导入与书库

更新日期：2026-11-09。本文为 M1，说明当前书目、EPUB 导入和书架操作。阅读呈现见 [M2](reader.md)，桌面运行见 [E1](desktop.md)，浏览器发布书目见 [E2](static_library.md)。

## 对象与职责

| 对象 | 内容与消费 |
| --- | --- |
| `LibraryBookSummary` | 书籍 ID、书名、作者、语言、来源名、封面、章节数、总字符数、位置与进度、时间、颜色和标签；用于书架及打开过渡 |
| `LibraryBook` | 在摘要上增加 Markdown、章节标题、资源列表、warnings 和书库路径；交给正文编译和图片解析 |
| `LibraryResource` | 原始 href、本机或网页路径、media type、宽高；用于正文图片和封面 |
| `ReaderDocument` | 前端编译后的正文、阅读块、章节及图片锚点；定义见 [M2](reader.md) |

Rust 类型位于 [library.rs](../crates/readux-core/src/library.rs)，前端类型与资源索引位于 [src/reader/library.ts](../src/reader/library.ts)。书库负责导入及保存书目，语言分析由文档会话服务执行。

## EPUB 导入

[import/epub.rs](../crates/readux-core/src/import/epub.rs) 按 container 定位 OPF，读取 metadata、manifest 和 spine，优先采用 EPUB3 nav，再尝试 NCX，并结合正文内目录形成章节信息。目录不可用时从正文结构标题选择章节，记录相应 warning。

导入器依 spine 和章节信息读取 XHTML，区分正文、封面、目录及插图等文档，清理设备提示与展示残片，提取标题、段落、ruby 和图片。正文输出为规范 Markdown，作者读法使用可供正文准备识别的注音表示；引用图片另存资源并附带可用尺寸。缺失文件、解析失败及目录问题通过错误或 warnings 表达，具体处理按所属阶段执行。

输入与输出关系为：

```mermaid
flowchart LR
    EPUB[EPUB 原文件] --> Import[container、OPF、导航与 spine]
    Import --> Text[规范 Markdown 与章节标题]
    Import --> Assets[图片与导入诊断]
    Text --> Store[ReaderLibrary 保存]
    Assets --> Store
    Store --> Compile[前端 Worker 编译正文]
    Compile --> Session[文档会话与分析]
```

EPUB 原书的排版与结构经过导入转换进入当前阅读模型。阅读器替换时，原书资源、原书定位与分析正文之间的映射需要结合新方案确定，关注点见 [M2](reader.md#阅读器替换方向)。

## 保存结构与身份

桌面默认书库位于系统文档目录 `ReadUX Library/`，Windows 常见位置为用户的 `Documents/ReadUX Library`，实际位置由 Tauri `document_dir()` 确定。

```text
ReadUX Library/
  library.sqlite
  books/<book-id>/
    source.epub
    content.md
    assets/0000.<扩展名>
```

书籍 ID 根据 EPUB 文件内容计算。相同文件重复导入会写入同一目录，更新书目、章节及资源，保留原有进度、阅读时间、颜色和标签。内容发生变化的文件会取得新的书籍身份。

SQLite 的 `books`、`chapters`、`resources`、`book_tags` 保存摘要、章节顺序、资源相对路径和分类。`source.epub` 保留原文件，`content.md` 保存转换结果，assets 使用顺序文件名，资源记录保留原 href。

当前导入先创建目录并写入文件，再在事务中更新索引；文件写入与数据库事务分阶段完成。排查失败导入时同时检查目录与索引。导入返回的 warnings 附在本次 `LibraryBook` 中，重新打开书籍时 warnings 初始化为空，当前主页面没有独立的 warnings 列表。

导入时总字符数初值来自 Markdown 字符计数；阅读进度更新时使用阅读正文的准备后字符数，百分比据此计算。字符锚点的正式含义见 [M2](reader.md#正文与坐标)。

## 书架交互

[LibraryHome.vue](../src/components/reader/LibraryHome.vue) 显示最近阅读和全部书籍，采用分页展示。搜索覆盖书名、作者、当前章节、来源文件名与标签；排列支持最近阅读、书名、阅读进度、导入时间。最近阅读依据 `lastOpenedAt`。

书籍菜单提供详情、颜色标记、逗号分隔的标签编辑及重置进度。桌面支持删除、打开书库目录及定位封面所在目录；这些操作通过 `App.vue` 进入 reader 命令。删除前应用显示确认框。

| 操作 | 当前调用及结果 |
| --- | --- |
| 加载书架 | `reader_library` 返回摘要列表和书库路径 |
| 导入 | `reader_import {path}` 返回完整书籍，应用刷新书架并打开 |
| 打开 | 先 `reader_book_summary {id}`，再 `reader_open_book {id}`；后者读取正文、资源并更新打开时间 |
| 分类 | `reader_organize {id,color,tags}` 更新颜色与标签 |
| 进度 | `reader_progress {id,offset,total,chapter,seconds}` 更新字符位置、总数、章节及累计时间 |
| 重置 | `reader_reset {id}` 由书库存储重置进度字段 |
| 删除 | `reader_remove {id}` 删除书库记录和对应文件目录 |

删除时先将书籍目录移至书库内 `.removing-<id>`，再提交数据库删除；提交失败分支尝试恢复目录，成功后删除暂存目录。该接口管理书库数据，摘录及词汇状态保存在独立 `reader-state.sqlite` 中。

打开书籍后，应用编译正文、恢复位置、创建分析会话并加载对应版本的收藏。返回书架时关闭前端会话并刷新书目。生命周期由 [App.vue](../src/App.vue)、[useDocumentSession.ts](../src/composables/useDocumentSession.ts) 与 [ReaderEngine](../src-tauri/src/reader_engine.rs) 协作。

## 桌面与静态书架

[readerRequest](../src/services/reader.ts) 在 Tauri 内调用桌面命令，在浏览器内始终调用 `staticLibrary.ts`。静态书架从 `catalog.json` 取得发布书目，浏览器本地覆盖阅读进度、颜色和标签；导入、文本输入和删除等操作按静态能力限制，目录路径为空。

静态发布保留桌面书籍 ID，资源路径改为版本目录 URL。浏览器个人状态与桌面数据库分别保存。发布选择、资源生成和版本清理见 [E2](static_library.md)。

## 维护与后续工作

导入相关问题可依次核对 OPF 与导航、spine 文档选择、规范 Markdown、资源 href 和最终阅读块。当前已有 EPUB 与正文坐标样例可供专项核验；纯文档维护集中检查接口、路径和引用。

后续阅读器选型需要确定 EPUB 原文保存、书籍身份、目录与资源定位怎样继续服务阅读与分析。书目管理沿用本页的现有数据职责，具体迁移步骤在阅读器方案确定后设计。
