# 静态 Web 与内容部署

更新日期：2026-11-09。本文为 E2，说明浏览器入口、预生成资源、版本与部署。书架及正文复用 [M1](library.md)、[M2](reader.md) 的组件；词典静态查询见 [M5](dictionary.md)。

## 网页入口与能力

浏览器默认进入已发布书架，从 `library/catalog.json` 读取书目。打开图书后按阅读范围加载分析分片和查询内容；阅读位置、摘录、笔记、词汇状态、颜色与标签保存于当前浏览器 localStorage。

静态页面提供已发布内容的阅读、查询、活用与接续图、收藏和 JSON 导出。导入 EPUB 和文本即时分析入口禁用，删除本机书籍与目录操作属于桌面接口。浏览器搜索读取当前书籍随包发布的词条索引。

语法指南由书架右上角进入，也可使用以下页面参数：

| URL 参数 | 内容 |
| --- | --- |
| `?view=grammar-guide&lesson=grammar-1.1` | 《句法入门》 |
| `?view=grammar-guide&lesson=grammar-2.1` | 《认识实词、虚词、助词》 |
| `?view=grammar-guide&lesson=grammar-stage11` | 活用与接续气泡测试 |

指南具有独立内容、分析与词典快照，复用阅读器的内容投影及说明组件。制作入口见[开发说明](development.md#更新指南快照)。

## 资源结构

源资源位于 `public/library/`，Vite 构建后复制到 `dist/library/`。

```text
library/
  catalog.json
  books/<book-id>/<version>/
    manifest.json
    book.json.gz
    plan.json.gz
    lookup-index.json.gz
    analysis/*.json.gz
    queries/*.json.gz
    contents/*.json.gz
    images/*
```

| 资源 | 协议与职责 |
| --- | --- |
| catalog | `readux.static-library.v1`；书籍摘要及 manifest 路径，决定发布范围 |
| manifest | `readux.static-book.v1`；书籍 ID、资源版本、正文、计划、索引和单元到分片的路径映射 |
| book | Markdown、章节标题、书籍摘要及图片资源；浏览器编译正文 |
| plan | 桌面服务生成的 `DocumentPlan`，含准备后全文、语域和单元锚点 |
| analysis | 每片最多六个分析单元，保存前端消费的文档字段、lookup 及目标查询索引 |
| lookup-index | 查询 ID → queries 文件、内容 ID → contents 文件、词／entry key → 内容 ID |
| queries | 默认与各表记的查询快照，共享词条内容引用 |
| contents | 按记录身份去重的词典正文和按内容去重的语法解说 |

生成器按约 1 MiB 的原始 JSON 大小组织 queries 和 contents 分包，再使用 gzip 写出。静态文档保留前端所需的活用、词汇、范围和来源 `cType/cForm` 等字段；发布资源的精简字段集合由生成器维护。

## 范围加载与查询

[staticAnalysis.ts](../src/services/staticAnalysis.ts) 用静态快照实现文档会话接口。`open_document` 按书籍 ID 查 manifest 与 plan，从恢复偏移加载首片；`request_range` 找出与全文范围相交的单元并读取对应文件。内存保留最近分片，容量下限为四片，单次可见范围跨更多分片时容纳该范围所需文件。

适配器返回 `DocumentUpdate` 快照及已加载单元；`continue_document`、`poll_document` 和 `sync_document` 返回现有快照。`retry_document` 重新读取指定单元或当前范围，`close_document` 移除内存会话。静态适配中的完成进度表示发布内容已经预生成。

`query_lookup_document` 由单元目标 ID 查查询快照，按选择的表记读取正文引用；`search` 在当前书籍 `words` 索引精确查找词条。语法解说中的 `$content` 引用在单元读取时恢复。前端继续用 [L11](language_analysis/presentation.md) 投影词形及接续图。

[staticLibrary.ts](../src/services/staticLibrary.ts) 的 JSON 加载器按 URL 复用 Promise，最多保留八项；响应头两字节为 gzip 标志时使用 `DecompressionStream` 解压，否则直接解析 JSON。平台提前解压的响应同样适用。

## 浏览器状态与宿主选择

书库操作在浏览器始终交给 `staticLibrary.ts`；NLP 在非 Tauri 且 production、frontend 或 static 模式下交给 `staticAnalysis.ts`。`acceptance` 开发模式中的 NLP 请求经 bridge，书库请求仍使用静态书架；模式差异见 [E3](cli.md)。

`readux.static-reader.v1` 保存书籍本地覆盖字段、selections 及以基本形／读音为键的词汇状态。摘录按书籍 ID、文本版本和起止位置保存，发布资源更新后由对应文本版本决定可读取的摘录。排版另用 `readux.reader.appearance`，阅读器词典顺序另用 `readux.reader.dictionaryOrder`。

浏览器状态属于当前站点来源及浏览器配置，桌面书库和个人数据库独立保存。发布目录中的书目、正文与分析作为静态资源读取，浏览器修改颜色或标签时保存本地覆盖值。

## 生成单本书籍

本机具备 EPUB 书库、词典与 GiNZA 资源后执行：

```powershell
npm run morphology:build
cargo build --release -p readux-core --bin readux-nlp
python -X utf8 scripts/generate_static_library.py --book-id <书籍ID> --version <资源版本>
npm run build
```

[generate_static_library.py](../scripts/generate_static_library.py) 默认读取 `Path.home()/Documents/ReadUX Library`，默认服务为 `target/release/readux-nlp.exe`。`--library` 指定书库，`--service` 指定服务，`--book-id` 精确选择图书；未指定 ID 时使用 `--title` 的书名前缀匹配，要求唯一结果。`--version` 默认当天日期，`--work-dir` 默认 `data/edgeone-cache`。

生成器通过 [static_library_document.mjs](../scripts/static_library_document.mjs) 使用前端正文编译，调用正式服务取得计划、完整单元及各表记的查询结果，加载所需词典正文，再写出资源。过程数据保存于 `<work-dir>/<book-id>/<version>/generation.sqlite`，同一输入版本可续接；`report.json` 记录数量与体积。

文本、服务或规则更新后采用新的资源版本以对应新的过程数据。同版本再次生成会先删除该 public 输出目录，再利用该版本过程数据重新输出。生成成功后更新 catalog 并清理未被 catalog 引用的版本。

## 完整构建、清理与部署

`npm run build` 使用现有资源构建前端，prebuild 依据 `public/library/catalog.json` 清理未引用版本，然后生成活用配置、检查 Vue 类型并构建 `dist/`。单独清理入口为：

```powershell
python -X utf8 scripts/generate_static_library.py --clean
```

需要重新制作指南及当前发布书目时执行 `npm run edgeone:build`。[build_edgeone.py](../scripts/build_edgeone.py) 编译 release NLP 服务，更新三课指南快照，再为 catalog 中能在本机书库找到的书籍生成资源；其版本依据正文、服务、配置和相关资源身份生成，随后执行前端构建并创建 `dist.zip`。

开发预览 `npm run frontend:dev` 启动 5173，静态书籍 URL 指向 `dist/library/`，指南读取源码内容与快照。已有有效 dist 时可以直接预览；更新 public 资源后先完成前端构建。`npm run preview` 检查整个生产 dist。

生产指南的分析和词典资源由 [grammar_guide_assets.mjs](../scripts/grammar_guide_assets.mjs) 压缩为 `.json.gz`，由 `src/grammar-guide/loadJson.ts` 加载。完整部署目录为 `dist/`，ZIP 的 `index.html` 位于根目录，library 与页面一同上传。EdgeOne Pages 使用构建命令 `npm run build` 和输出目录 `dist`。

完整发布资源核验使用 `node scripts/verify_static_library.mjs`，覆盖查询引用、全文分片、正式胶囊投影及单文件限制，报告位于 `.agents/analysis/static-library/verification.json`。纯文档更新只核对入口与协议；资源生成后的核验在对应制作流程最终执行。
