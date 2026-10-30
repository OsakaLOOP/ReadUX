# 开发与桌面交付

## 运行组成

交付物包括 Windows 桌面程序、前端、现代语 UniDic 资源、词典源包、规则与讲解目录，以及随包交付的最小 GiNZA Python 运行环境。开发渠道仍可使用本机 Python 环境，发行渠道使用包内解释器和模型。

本机配置记录解释器绝对路径、适配器入口、模型与词典位置、版本及任务选项。资源检查从应用启动入口执行，成功后复用已加载服务。配置完成的环境支持离线分析。

## 当前开发入口

仓库使用 Rust、Node.js、npm 和 Tauri 2，Windows 编译依赖相应的桌面工具链。已有依赖时直接运行：

```powershell
npx tauri dev
```

前端动画开发使用 `npm run frontend:dev`，浏览器直接显示带占位书籍的书架。点击书籍或右上角的打开、关闭按钮，即可预览完整 header 动画，包括播放期间反向操作。该入口通过 Vite 热更新，独立于 Rust、书库和 NLP 服务。

`npm run acceptance` 在 1420 端口提供正式应用和 Rust 分析 bridge；纯前端预览使用 5173 端口。Vite 的依赖扫描入口固定为 `index.html`，监听范围排除模型、构建产物和浏览器测试目录；各模式使用独立的依赖预构建缓存。气泡测试页可通过查询参数 `?view=grammar-guide&lesson=grammar-stage11` 直接打开。

指南例句读取预生成的分析快照。Vite 热更新会加载接续配置中的图示文案，实际命中路径和前项范围由快照保存。修改 Rust 接续规则后，须使用重新编译的服务生成对应快照，再刷新浏览器页面。开发 bridge 占用默认服务时，可生成独立的指南服务；以下命令更新复杂句组：

```powershell
cargo rustc -p readux-core --bin readux-nlp -- -o target/debug/readux-nlp-guide.exe
python -X utf8 scripts/generate_grammar_guide_analysis.py --lesson stage11 --group complex --service target/debug/readux-nlp-guide.exe
```

header 的统一动画资源为 `src/branding/readux-animated.svg`，由 `python -X utf8 scripts/generate_header_mark.py` 从两种品牌姿态生成。`ReadingMark.vue` 挂载单个 SVG，`markPlayback.ts` 统一控制 2.2 秒时间轴。眼睛与刀身在前 45%（约 1 秒）完成变换，使用较弱的缓动；书本保持前 42% 进入、后 58% 展开的节奏。各部分同时触发，由速度差形成先后顺序。路径或书本设计调整后执行生成脚本。

`npm run test:header` 验证初始姿态、路径插值、打开与关闭、重复操作、减少动态效果设置以及桌面和窄屏截图。需要可用的 Playwright，沿用 `PLAYWRIGHT_MODULE` 指定本机模块路径，`READUX_TEST_URL` 可覆盖预览地址。

当前 CLI 支持如下命令：

```powershell
cargo run -p readux-core --bin readux-nlp -- inspect cwj "七日は警察署へ向かった。"
cargo run -p readux-core --bin readux-nlp -- repl
cargo run -p readux-core --bin readux-nlp -- stdio
```

`stdio` 接收逐行 JSON 请求，使用与桌面端相同的 `AnalysisService`。网页开发由 Vite 提供，通过开发 bridge 调用 Rust；Tauri 窗口承担桌面 IPC 与资源定位验收。

本机模型入口为 `scripts/nlp_provider.py`，由 Rust 使用 `-X utf8 -u` 启动。桌面先显示 UniDic 结果，再以 `enrich` 请求追加 GiNZA 结构。设置位于“结构分析 → 本机模型设置”，开发环境可指定 GiNZA 解释器、模型和 Sudachi 词典文件；发行包默认使用 `python/Scripts/python.exe`。词典路径留空时使用随 Python 包安装的资源。配置写入数据目录的 `providers.local.json`，日志位于同目录 `provider-logs`。旧配置中的 KWJA 字段可读取，但不进入当前配置对象、状态响应或分析任务；发行版运行时不依赖系统 Python。

GiNZA 模型支持已安装模型名或 spaCy 模型目录。“保存并检查”执行 `check_providers`，重新初始化已启用模型，报告资源缺失、加载错误及完整清单；检查成功后复用模型进行分析。开发默认路径对应下表已准备的环境；实际任务清单由加载的模型返回。

`scripts/nlp_resources.py` 记录实际资源内容的 SHA-256、路径和大小。GiNZA 清单包含模型及 Sudachi 资源，并结合依赖版本、执行参数形成 `resource_digest`，随完整来源结果传输。`provider_status.configured` 表示解释器与适配器路径存在，`available` 表示模型已经成功初始化。

`npm run dev` 使用开发资源配置启动桌面应用；`npm run insider` 准备独立运行资源、以 release 配置生成 `target/release/ReadUX.exe`，再组织 Windows 便携包，产物位于 `packages/ReadUX-insider-portable-win64.zip`。具体行为见 [run_channel.ps1](../scripts/run_channel.ps1)。

## EdgeOne Makers 静态部署

在仓库根目录运行 `npm ci`、`npm run build`，输出目录为 `dist`。EdgeOne Makers 的框架选择 Vue 或 Vite，根目录为 `./`，构建命令为 `npm run build`，输出目录为 `dist`，Node.js 可选择 22.21.1。直接上传时选择整个 `dist` 文件夹，ZIP 上传须保证 `index.html` 位于压缩包根目录。

正式构建自动将三课指南的分析、词典快照压缩为 `.json.gz`，前端通过浏览器原生 `DecompressionStream` 解压后读取。开发服务继续使用 JSON。静态托管按普通 gzip 文件提供这些资源，浏览器负责解压文件内容。

浏览器首页直接打开语法指南，三课共 812 条例句采用预生成分析与词典内容。课节直达链接使用 `/?view=grammar-guide&lesson=grammar-1.1`、`grammar-2.1` 或 `grammar-stage11`。指南正文、注音、译文、词典与活用接续图均可通过静态资源提供；书库导入和即时分析依赖桌面宿主。

平台规定单文件最大 25 MB，项目最多 20000 个文件；构建产物应按这两个条件核对。本次官方文档正文和完整索引暂存于 `.agents/analysis/edgeone-docs/`。

2026-10-30 构建产物共 23 个文件、13.77 MiB，最大压缩快照为 6.90 MiB；六份 gzip 文件解压后均与源 JSON 一致。纯静态 HTTP 服务已验证三课切换、词典、注音与译文开关、活用接续图、悬浮气泡及 390 px 抽屉布局，页面无脚本错误、失败资源请求或分析 API 请求。气泡测试课有 7 条孤立助动词样例的原始快照缺少查询目标，按纯文本显示：`M-synthetic-001` 至 `M-synthetic-004`、`M-E051` 至 `M-E053`。验收报告和截图位于 `.agents/analysis/edgeone-static/`。

## 当前资源定位

| 资源 | 开发位置或覆盖项 |
| --- | --- |
| CWJ | `experiments/unidic-source/unidic-cwj-202512.vibrato.dic`；`READUX_UNIDIC_CWJ` |
| CSJ | `experiments/unidic-source/unidic-csj-202512.vibrato.dic`；`READUX_UNIDIC_CSJ` |
| 词典源包 | `data/dict-sources/*.kdict` |
| 词典查询缓存 | `data/dicts` |
| 词典数据根目录 | `READUX_DATA_DIR` |
| GiNZA 实验环境 | `experiments/ginza311`，由本机配置选择解释器 |
| GiNZA 最小环境 | `experiments/ginza-minimal`，由 `scripts/build_ginza_minimal_env.py` 生成 |

桌面 release 从安装目录读取 `nlp/cwj.dic`、`nlp/csj.dic`、`dict-sources`、`python` 和 `scripts`，查询缓存使用应用数据目录。`scripts/prepare_installer_resources.py` 将本地 GiNZA 环境转换为独立 Python 解释器、标准库与模型包；安装资源统一写入 `release-resources`，Windows 解释器为 `python/python.exe`。

词典源目录按应用数据目录、便携目录和 Tauri 资源目录依次查找，候选目录必须包含 `.kdict` 源包。空目录会继续查找后续候选，包内源包用于核验并重建应用数据目录中的查询数据库。

## 资源与用户数据

只读词典、权重和语法知识内容具有资源版本及校验值；查询缓存按资源身份重建。画像、收藏、书库和阅读进度属于持久用户数据，升级采用独立迁移。

词典源包和查询数据库分别承担分发与本机访问。第三方程序、词典、权重和词典正文的许可分别登记。本机依赖清单注明来源、获取方式和验证命令，桌面产物说明相应的配置要求。

文件、进程协议和日志统一采用 UTF-8。程序启动使用配置路径和明确参数，后台 provider 以隐藏窗口运行。应用退出时关闭所属分析任务和进程。

## 检查与构建

按修改范围选择检查；以下命令是对应入口，实际结果记录在 TODO 或验收报告中：

```powershell
cargo test -p readux-nlp -p readux-core --lib
cargo check -p tauri-app
npm run test:ui
npm run build
python scripts/build_grammar_catalog.py --check
python scripts/test_dictionary_schema.py
```

核心测试覆盖已注册模块；模块恢复时将其测试纳入编译和执行。前端构建包含 TypeScript 检查与 Vite 打包。语法资源或词典 schema 改动时执行各自专项检查。

GitHub Actions 使用 `src-tauri/tauri.github-release.conf.json` 构建 Windows x64 便携包与 macOS arm64 DMG。`.kdict` 是发行包的词典源格式，四个源包已被 Git 跟踪；UniDic 与 GiNZA 资源由 `resources-v1` Release 提供，`scripts/fetch_release_resources.py` 下载资源并转换 Windows 虚拟环境。资源 ZIP 作为构建输入，应用发行产物为便携包。推送与 `package.json` 版本一致的 `v*` 标签后创建 GitHub Release；手动运行工作流生成构建产物。

## Windows 便携运行

便携包直接运行目录中的 `ReadUX.exe`，程序、UniDic、词典源包、Python 运行时与 provider 脚本使用固定相对路径。应用标识固定为 `com.readux.app`，版本从根目录 `package.json` 读取，Rust 包版本同步维护。Windows Job Object 管理分析进程，解释器使用 `-B`，避免在便携目录生成字节码缓存。

用户配置、阅读状态和查询缓存位于系统应用数据目录 `com.readux.app`，书库位于文档目录 `ReadUX Library`。升级保留这些目录；卸载默认保留应用数据，用户可在卸载界面选择清除应用数据，书库独立保留。`READUX_DATA_DIR` 指定的目录由使用者管理。

Windows EXE 使用 `src-tauri/icons/icon.ico`。`src-tauri/src/windows_icons.rs` 从 EXE 资源编号 32512 加载系统尺寸的大、小图标，分别用于任务栏和标题栏。

`scripts/verify_windows_icons.ps1` 核对 EXE 内全部 ICO 帧，并启动程序读取窗口大、小图标；Windows CI 在构建后执行同一检查，报告默认写入 `.agents/analysis/windows-icons.json`。

资源 Release 的资产名称固定为：

```text
ReadUX-insider-portable-nlp.zip
ReadUX-insider-portable-windows-x64-python.zip
ReadUX-insider-portable-macos-arm64-python.zip
```

## 桌面交付验收

生成可执行程序及资源包，保存程序版本、资源清单、运行时依赖清单和验收记录。应用从仓库以外的目录启动，直接调用发行包内的 UniDic 和 GiNZA 运行环境。

完整验收执行导入、继续阅读、结构展示、词典与语法查询、表达解释、个人状态、收藏、导出和质量审计，再重启验证持久状态。记录模型首次加载和复用效果，检查缺失依赖的提示与配置恢复。

桌面交付结果以 [TODO](TODO.md)中的功能矩阵和最终验收项为准。
