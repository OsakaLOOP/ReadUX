# 开发说明

更新日期：2026-11-09。先阅读[开发目标](goals.md)和[总体架构](architecture.md)。本页集中说明当前入口、资源及维护流程；模块文档的后续整理与交接要求见[总计划](documentation_update_plan.md)。

## 环境与资源准备

开发以仓库根目录为工作目录，文件、配置和进程协议采用 UTF-8。需要 Rust、Node.js/npm、Python 及 Tauri 2 对应的平台工具链；Windows 桌面编译使用 MSVC 和 WebView2。发布 CI 当前使用 Node.js 20、Python 3.11，以及 Windows x64 和 macOS arm64 构建环境。

安装前端依赖：

```powershell
npm ci
```

大体积词典、模型和解释器通过本机环境或发行资源准备。开发默认路径由 `ResourcePaths::development` 和 `ProviderSettings::development` 定义：

| 资源 | 默认位置或配置 |
| --- | --- |
| CWJ Vibrato 词典 | `experiments/unidic-source/unidic-cwj-202512.vibrato.dic`，环境变量 `READUX_UNIDIC_CWJ` 可覆盖 |
| CSJ Vibrato 词典 | `experiments/unidic-source/unidic-csj-202512.vibrato.dic`，环境变量 `READUX_UNIDIC_CSJ` 可覆盖 |
| 词典源包 | 数据目录的 `dict-sources/*.kdict` |
| 词典查询数据库 | 数据目录的 `dicts/` |
| GiNZA 解释器与模型 | `experiments/ginza311/Scripts/python.exe`、模型 `ja_ginza` |
| provider 配置 | 数据目录的 `providers.local.json` |
| 活用运行配置 | `configs/unidic-nlp/morphology.json`，由 source 编译生成 |
| 指南快照 | `src/grammar-guide/lesson-*-analysis.json` 与 `lesson-*-dictionary.json` |
| 静态书籍 | `public/library/`；纯前端开发预览读取构建后的 `dist/library/` |

开发 CLI 的数据目录默认为仓库 `data/`，`READUX_DATA_DIR` 可覆盖。桌面 debug 的分析服务沿用开发资源定位；书库和阅读状态由 Tauri 使用文档目录及应用数据目录定位。桌面 release 从便携目录或 Tauri 资源目录查找资源，应用数据目录保存配置和缓存。详细分工见[总体架构](architecture.md)。

## 运行入口

| 目的 | 入口 | 条件及行为 |
| --- | --- | --- |
| 静态前端开发 | `npm run frontend:dev` | 5173 端口，frontend 模式；浏览器书籍从 `dist/library` 读取，指南使用源码中的内容和快照 |
| 已构建站点预览 | `npm run preview` | 读取 `dist/`，直接检查生产资源 |
| NLP 开发 bridge | `npm run acceptance` | 编译活用配置和 debug CLI，启动 1420 端口 Vite；`POST /api/nlp` 转发至 Rust stdio |
| 桌面开发渠道 | `npm run dev` | 调用 `scripts/run_channel.ps1 -Channel dev`，使用 Tauri 开发资源配置；当前端口配置差异见下文 |
| 基础文本检查 | `cargo run -p readux-core --bin readux-nlp -- inspect cwj "七日は警察署へ向かった。"` | 调用基础 Analyze 请求，检查 UniDic 与基础活用结果 |
| 持续请求 | `cargo run -p readux-core --bin readux-nlp -- stdio` | 逐行 JSON，可调用追加 GiNZA、文档会话、查询及配置请求 |

### 桌面开发的端口配置

当前 `src-tauri/tauri.conf.json` 的 `beforeDevCommand` 执行 `npm run frontend:dev`，启动 5173；`devUrl` 为 `http://127.0.0.1:1420`。开发渠道覆盖文件只调整打包资源。单独执行渠道命令时，Tauri 等待的地址与前端启动地址不同。

沿用 5173 前端进行桌面开发时，可以通过 Tauri 的 JSON 配置覆盖 URL：

```powershell
$desktopConfig = '{"build":{"devUrl":"http://127.0.0.1:5173"}}'
node node_modules/@tauri-apps/cli/tauri.js dev --config src-tauri/tauri.dev.conf.json --config $desktopConfig
```

Tauri CLI 支持按顺序合并多个配置参数；此命令继续使用原有前端启动命令。桌面窗口内 `isTauri()` 为真，阅读和分析使用桌面接口。后续若修正渠道配置，应同步更新本节。

`run_channel.ps1` 还会检查 `target/` 的大小，超过 4 GiB 且无构建进程时清理该构建目录。选择渠道脚本时应了解这一现有行为。

### 前端与 bridge 的范围

`frontend` 模式提供静态阅读适配，适合正文显示、说明视图、图组件及指南调试。浏览器书库接口始终由 `staticLibrary.ts` 承接。`acceptance` 模式额外提供 NLP 请求 bridge，适合分析请求调试；该 bridge 的接口范围为 `/api/nlp`。

可用页面参数包括：

| 参数 | 页面 |
| --- | --- |
| `/?view=header-animation` | frontend 模式下的 header 动画预览 |
| `/?view=grammar-guide&lesson=grammar-1.1` | 《句法入门》 |
| `/?view=grammar-guide&lesson=grammar-2.1` | 《认识实词、虚词、助词》 |
| `/?view=grammar-guide&lesson=grammar-stage11` | 活用与接续气泡测试 |

## CLI 与 GiNZA 配置

CLI 提供 `inspect`、`repl` 和 `stdio`。前两者调用基础分析，`stdio` 暴露 `AnalysisService` 请求协议；每行接收一个 JSON 请求，返回包含 `result` 或 `error` 的响应。

```powershell
cargo build -p readux-core --bin readux-nlp
.\target\debug\readux-nlp.exe stdio
```

进入 stdio 后可逐行输入：

```json
{"command":"status"}
{"command":"provider_status"}
{"command":"check_providers"}
{"command":"analyze","text":"本を読んでいる。","register":"cwj"}
```

分析响应中的 `id` 用于后续 `enrich` 请求的 `analysis_id`。整书阅读使用文档会话请求，完整分析单元会执行 UniDic、GiNZA、统一分析和查询目标构建。请求字段以 `crates/readux-core/src/analysis.rs` 的 `Request` 枚举为准。

provider 设置的当前结构为：

```json
{
  "ginza": {
    "python": "D:/path/to/python.exe",
    "model": "ja_ginza",
    "enabled": true,
    "timeout_seconds": 120,
    "dictionary": ""
  },
  "analysis_timing_enabled": true
}
```

通过 `configure_providers` 的 `settings` 字段提交设置，再调用 `check_providers` 检查加载。`model` 接受安装模型名或模型目录，`dictionary` 为空时采用 Python 环境中的默认词典。配置写入 `providers.local.json`，日志保存在相邻的 `provider-logs/`。现有 `ProviderPanel.vue` 保留设置 UI，主应用的可访问入口以实际组件引用为准；CLI 可以直接调用对应请求。

Rust 通过 `scripts/nlp_provider.py` 启动 Python 子进程，采用 UTF-8、标准输入输出协议并复用模型。Windows 通过 Job Object 管理所属子进程。模型缺失、解释器错误和完整单元分析失败可从 provider 响应、日志及会话错误定位。

## 活用规则、词条与界面维护

活用维护涉及执行规则、解说和显示三个相互关联的部分。

完整规则维护契约见 [L5](language_analysis/morphology_configuration.md)，词条及解说的读取顺序见 [L6](language_analysis/entries.md)，输入和分析对象从 [L0 总览](language_analysis/README.md)进入。

| 修改对象 | 维护入口 | 后续操作 |
| --- | --- | --- |
| 接续、状态和执行条件 | `configs/unidic-nlp/morphology/source/` | 生成配置，重新编译使用该配置的 Rust 服务，更新受影响快照 |
| 概念解说及例句 | `crates/readux-core/resources/grammar/source/` | 生成 compiled 内容，核对活用概念引用及前端显示 |
| 内置助词、助动词 | `configs/grammar/particles.json`、`auxiliaries.json` | 核对阅读器和指南调用的词条与例句 |
| 说明内容和图布局 | `src/reader/morphologyBubble.ts`、`morphologyGraph.ts`、`lookupPresentation.ts` | 在相应例句中检查词条、图、正文范围及高亮 |
| 气泡、侧栏及抽屉 | `src/explanation/`、`src/composables/useExplanation*.ts`、`src/components/explanation/` | 检查宽窄视口、布局切换、滚动及点击或触摸行为 |

规则生成与检查：

```powershell
npm run morphology:build
npm run morphology:check
```

编译器生成 `configs/unidic-nlp/morphology.json`，同时检查规则、路径、连接器、词条解说和例句引用。Rust 编译时嵌入该产物，前端导入相同配置。单独调用 Cargo 前应先生成最新配置；前端开发和构建命令已经包含生成步骤。

grammar 解说源内容发生变化时，使用：

```powershell
python -X utf8 scripts/build_grammar_catalog.py
```

`configs/grammar/` 仍参与当前阅读及活用解说。原 grammar 整体分析流程的过时状态见[总体架构](architecture.md)，现用内容的维护应根据实际引用进行。

## 指南及书籍静态资源

### 更新指南快照

修改 Rust 规则或影响分析的配置后，重新编译服务，再生成相关课节或例句组：

```powershell
npm run morphology:build
cargo build -p readux-core --bin readux-nlp
python -X utf8 scripts/generate_grammar_guide_analysis.py --lesson stage11 --group complex
```

`--lesson` 接受 `1.1`、`2.1` 或 `stage11`；省略 `--group` 时生成整课。生成器通过正式服务执行基础分析、GiNZA 追加和词典查询，写入课节的分析及词典快照。

默认 debug 服务正在被 bridge 占用时，可生成独立服务并显式指定：

```powershell
cargo rustc -p readux-core --bin readux-nlp -- -o target/debug/readux-nlp-guide.exe
python -X utf8 scripts/generate_grammar_guide_analysis.py --lesson stage11 --group complex --service target/debug/readux-nlp-guide.exe
```

配置文案的前端热更新与分析快照分别更新。真实命中路径及范围来自快照；修改执行条件后需要使用新服务生成相应快照。

### 构建静态站点

已有指南和书籍资源时执行：

```powershell
npm run build
npm run preview
```

`prebuild` 根据 `public/library/catalog.json` 清理未引用书籍版本，随后生成活用配置、执行 Vue 类型检查并构建 `dist/`。生产构建将指南的分析、词典 JSON 压缩为 gzip 资源。

需要从本机书库更新全部已发布书籍及三课指南时执行：

```powershell
npm run edgeone:build
```

该命令依赖文档目录中的 `ReadUX Library`、已登记的公开书目及本机分析资源。脚本构建 release NLP 服务、更新快照与书籍资源，随后构建站点并生成 `dist.zip`。发布范围取自公开书目。只更新书籍时可使用 `npm run generate:library`，具体选择、服务路径和版本参数见 `scripts/generate_static_library.py`。

静态部署产物为整个 `dist/`；ZIP 中 `index.html` 位于根目录。浏览器从静态文件加载内容，阅读状态保存在当前浏览器。资源协议、生成参数与部署步骤见[静态 Web 与内容部署](static_library.md)。

## 桌面构建与发行

Windows 本地便携发行入口：

```powershell
npm run insider
```

脚本准备 GiNZA 最小环境及 `release-resources/`，检查资源，使用 release 配置生成 `target/release/ReadUX.exe`，再生成 `packages/ReadUX-insider-portable-win64.zip`。本地资源准备入口为 `scripts/prepare_installer_resources.py`，便携目录由 `scripts/package_portable.py` 组织。

发行目录包括程序、`nlp/`、`dict-sources/`、`python/` 和 provider 脚本。Windows 独立解释器优先使用 `python/python.exe`。CI 通过 `scripts/fetch_release_resources.py` 获取预制资源，在 `.github/workflows/release.yml` 中构建 Windows x64 便携 ZIP 与 macOS arm64 DMG。应用版本来自根目录 `package.json`；标签发布与手动构建按 workflow 执行。

第三方来源及许可见 [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md)。资源组织与交付说明见[独立桌面发行版](standalone_distribution.md)，该模块正文按总计划继续整理。

## 验证与问题定位

根据修改范围选择现有验证入口，最终集中检查相关内容与提交差异。

| 修改范围 | 对应检查 |
| --- | --- |
| 活用规则或配置 | `npm run morphology:check`；需要执行证据时选择对应 Rust 用例与真实例句 |
| grammar 解说源 | `python -X utf8 scripts/build_grammar_catalog.py --check` |
| Rust 分析实现 | `cargo test -p readux-nlp -p readux-core --lib`，或对应专项用例 |
| 前端投影及图布局 | `npm run test:ui`，结合对应指南例句与目标视图检查 |
| 前端类型及生产资源 | `npm run build` |
| Tauri 接口 | `cargo check -p tauri-app`，结合实际桌面入口 |
| 词典 schema | `python -X utf8 scripts/test_dictionary_schema.py` |
| 静态书籍资源 | `node scripts/verify_static_library.mjs` |
| 桌面资源 | `npm run verify:release-resources -- --require-generated` |

基础分析问题从 `inspect`、来源字段和命中规则定位；完整阅读问题结合会话错误、provider 日志、单元结果及查询目标定位；显示问题核对同一范围在快照、内容投影、图节点和正文中的对应。生成后的资源有问题时，先核对生成服务及输入版本。

## 研究与文档交接

研究入口保留在 [research](research/)，包含来源行为、GiNZA GPU、资源与空间测量、Web 和云端推理资料。本轮通过总体文档简述研究范围，research 内部文档保持原状。

接替 agent 按[总计划](documentation_update_plan.md)完成现有文档的重写、整合与补充。待设计模块记录已确认方向、现有基础和待讨论问题，具体开发方案由后续讨论确定。每篇完成后更新计划状态及相关索引。

已整理的运行与模块入口为[桌面端](desktop.md)、[静态 Web](static_library.md)、[CLI](cli.md)、[书架与导入](library.md)、[阅读器](reader.md)、[说明视图](explanation_layout.md)、[D3 图](analysis_graphs.md)、[词典](dictionary.md)及 [L11 结果投影](language_analysis/presentation.md)。显示规范中的待实施修改继续依原清单推进。
