# 文档更新总计划与追踪

日期：2026-11-08。依据提交：`34dc0f7`（清理和备份过时文档）。状态：目录与范围待审阅。

本计划组织 ReadUX 的文档重写，确定每篇文档的名称、职责、内容概要、代码入口和现有资料去向。目录与范围确认后，按本文顺序逐篇编写，并在本文记录进度。本次交付为本计划；后续正文在各自编写时进一步核实实现细节。

## 1. 依据与写作口径

设计依据为该提交后 `docs/` 保留的 23 篇顶层 Markdown 和 `docs/research/` 的 8 篇研究文档，以及当前工作区的入口、模块注册、服务适配、资源与脚本。根目录 README 和 memory 用于了解指引及定位文件；`docs_backup/`、其他工作目录和已经删除的文档均不作为正文恢复依据。

本轮核对覆盖 `package.json`、Cargo workspace、Vue 应用入口、Tauri 命令注册、Rust 模块注册与 CLI、浏览器静态宿主、指南资源、发布脚本和主要实验脚本。结论对应源码中可定位的调用关系；运行条件、命令有效性和细部行为在相关文档编写时验证。

文档采用以下口径：

- **当前实现**：说明已接入的入口、实际数据与调用关系，注明必要的运行条件。
- **开发目标**：说明希望达到的行为、现有差距和完成条件；待确认的产品选择明确标记。
- **研究记录**：说明实验问题、方法、环境、结果、适用范围和后续事项，保存测量日期及资源版本。

同一规则或数据协议由一篇模块文档负责，其他文档引用。总体文档描述目标、组成与协作关系，模块文档解释本领域设计，入口文档说明怎样使用和交付。文件路径均相对仓库根目录；尚待创建的目标文件使用代码格式，避免提前产生失效链接。

## 2. 已定位的入口与实际边界

| 入口或能力 | 已定位的当前基础 | 文档需要明确的范围 |
| --- | --- | --- |
| 桌面应用 | `src/main.ts` → `src/App.vue`；`src-tauri/src/lib.rs` 注册书库、导入、文本阅读、进度、词汇状态、摘录和分析命令；`src-tauri/src/reader_engine.rs` 协调本机资源 | 从书架导入 EPUB、粘贴 Markdown、进入阅读器、查询和保存状态；开发启动与便携发行分别说明 |
| 静态 Web | 同一 Vue 应用；`src/services/reader.ts` 分派至 `staticLibrary.ts`；`staticAnalysis.ts` 消费预生成书籍分析与查询资源 | 已发布书籍、按范围加载、浏览器状态、指南；新增文本的本机分析发生在资源制作阶段，网页的导入和即时分析操作有入口限制 |
| 语法指南 | `GrammarGuidePanel.vue` 注册 `grammar-1.1`、`grammar-2.1`、`grammar-stage11`；内容与快照位于 `src/grammar-guide/` | 正式课程、内容制作与审阅操作、课节直达链接；“活用与接续 · 气泡测试”的研究用途单独说明 |
| NLP CLI | `crates/readux-core/src/bin/readux-nlp.rs` 提供 `inspect`、`repl`、`stdio`；请求交给 `AnalysisService` | 短文本检查、持续 JSON 请求、文档会话、查询与规则管理；各命令实际触发的分析阶段分别核对 |
| 前端开发预览 | `npm run frontend:dev` 使用 5173 端口与 `frontend` 模式；浏览器书籍资源来自 `dist/library`；`?view=header-animation` 进入动画预览 | 静态资源准备、页面调试、header 预览与气泡调试；资源加载来源说明清楚 |
| 本机开发 bridge | `npm run acceptance` 构建 debug CLI 并启动 1420 端口 Vite；`scripts/nlp_bridge.mjs` 提供 `/api/nlp` → Rust `stdio` | bridge 提供的 NLP 请求范围；浏览器书库适配与 NLP 适配的组合需要单独核对 |
| 本机 NLP provider | `providers.rs` 管理配置与进程；`scripts/nlp_provider.py` 的 provider 参数为 GiNZA，复用 `nlp_adapters.py` | UniDic/Vibrato、GiNZA 的分工与资源；KWJA 适配器归离线研究 |
| 规则与知识维护 | `AnalysisService` 已有规则列举、预览、保存、启停及删除请求；活用配置、grammar catalog 和表达资源有各自入口 | 请求级能力、配置制作能力与应用中可访问的编辑界面分别核对 |
| 实验与评估 | `scripts/` 中保留来源比较、语料评估、训练、GPU、资源裁剪、空间测量、词典审查和活用验证脚本 | 按研究问题组织文档；`experiments/` 主要承载环境与结果，复现入口通常位于 `scripts/` |
| 尚待完整接入的功能 | `profile/`、`export/`、`llm/` 未在 `readux-core/src/lib.rs` 注册；质量审计 crate 被 workspace 排除，桌面注册未包含其命令 | 现有阅读器的词汇状态和 JSON 摘录导出有独立实现；高级画像、完整导出、LLM 助手和审计界面的目标需另行核实 |

入口说明以用户能完成的操作组织。开发预览和 bridge 收入开发说明，实验入口收入 research 索引，指南的使用入口与设计统一由指南文档承担。

## 3. 总体文档

三篇总体文档分别回答“准备做什么”“系统怎样组成”“怎样开发和交付”。

| 编号 | 文件与标题 | 内容概要 | 主要对应文件／目录 | 状态 |
| --- | --- | --- | --- | --- |
| G1 | `docs/goals.md` · 开发目标与功能范围 | 产品定位、使用者与主要任务；桌面、Web、指南的职责；当前功能范围与开发阶段；词汇、语法、阅读和学习功能的预期结果；后续方向及完成条件 | `src/App.vue`、`src/components/reader/`、`src/components/grammar/`、`src-tauri/src/lib.rs`、`docs/TODO.md`；现有 `docs/research/web_product_strategy.md`、`cloud_saas_roadmap.md` 提供待决策事项 | 待编写 |
| G2 | `docs/architecture.md` · 总体架构 | 应用与内容制作的组成；Vue、Tauri、Rust core、NLP 和 Python provider 的职责；桌面调用与静态读取两条路径；文档、分析、查询和个人状态的数据流；共用协议、资源制作及实验接入位置 | `Cargo.toml`、`src/main.ts`、`src/App.vue`、`src/services/`、`src-tauri/src/`、`crates/readux-core/src/lib.rs`、`crates/readux-nlp/src/lib.rs`、`scripts/nlp_provider.py` | 待编写 |
| G3 | `docs/development.md` · 开发说明 | 工具链和本机资源准备；桌面、静态前端、bridge、CLI 的启动方式；配置生成与热更新；指南及书籍快照更新；按修改范围选择检查；桌面和 Web 构建入口；目录、日志、排错与常用开发工具 | `package.json`、`vite.config.ts`、`src-tauri/tauri*.conf.json`、`scripts/run_channel.ps1`、`scripts/nlp_bridge.mjs`、`scripts/build_edgeone.py`、`.github/workflows/release.yml` | 待重写 |

G1 的目标需要覆盖已经保留的学习方向：语法与表达识别、词汇熟悉度及汉字知识、摘录与制卡、LLM 义项辅助、质量审计、云端新文本分析和可选同步。每个方向说明现有基础、预期价值和阶段归属。Web 主入口、私有阅读包、云端收费及具体供应商等选择，沿用研究资料中的讨论状态，待用户确认后进入正式目标。

G2 至少包含组件关系图、桌面阅读数据流、静态资源制作与读取数据流，以及数据所有权表。当前系统后端包含本机 Rust 服务与 Python 子进程；云端推理和账户服务的设计归研究文档，在总体架构中说明拟接入位置。

G3 以当前脚本和配置为准编写可执行命令。已发现 `tauri.conf.json` 的 `beforeDevCommand` 启动 5173，而 `devUrl` 指向 1420；开发入口章节须核实推荐启动组合和配置覆盖关系，再给出操作步骤。该问题在本轮登记，后续实现调整独立处理。

## 4. 面向入口的文档

| 编号 | 文件与标题 | 内容概要 | 主要对应文件／目录 | 状态 |
| --- | --- | --- | --- | --- |
| E1 | `docs/desktop.md` · 桌面端使用与运行 | 启动和资源要求、书架导入、文本输入、继续阅读、查询、摘录、设置及状态恢复；本机模型配置入口的实际可达性；Windows 便携运行和 macOS 交付范围；常见启动与资源问题 | `src/App.vue`、`src/components/reader/LibraryHome.vue`、`src-tauri/src/lib.rs`、`src-tauri/src/reader_engine.rs`、`src-tauri/tauri*.conf.json` | 待编写 |
| E2 | `docs/static_library.md` · 静态 Web 与内容部署 | 网页书架、指南链接与浏览器能力；发布书目和本机书库的关系；生成、增量制作、版本清理、预览和部署；catalog、manifest、分片、查询索引、共用词典与语法内容；gzip 加载；浏览器状态与平台限制 | `src/services/staticLibrary.ts`、`staticAnalysis.ts`、`public/library/`、`scripts/generate_static_library.py`、`scripts/build_edgeone.py`、`scripts/static_library_document.mjs`、`scripts/verify_static_library.mjs` | 待重写 |
| E3 | `docs/cli.md` · CLI 与本机请求协议 | `inspect`、`repl`、`stdio` 的用途及输入输出；资源路径与退出行为；逐行 JSON 请求示例；分析、会话、查询、provider 和规则请求索引；错误返回；UniDic/Vibrato 词典构建工具的用途 | `crates/readux-core/src/bin/readux-nlp.rs`、`unidic-vibrato-build.rs`、`crates/readux-core/src/analysis.rs`、`scripts/nlp_bridge.mjs` | 待编写 |

E2 需要说明两个构建入口的前提：`npm run build` 使用现成资源完成前端构建并执行书库版本清理；`npm run edgeone:build` 依赖本机书库和分析环境，更新指南与已发布书籍后生成 `dist/`、`dist.zip`。最终步骤按脚本核实，部署平台限制保留核查日期与来源。

E3 的请求目录以 Rust `Request` 枚举为准；协议字段在后端、会话和分析模块文档中定义，CLI 文档提供可用示例及定位链接。

## 5. 功能与模块设计文档

| 编号 | 文件与标题 | 内容概要 | 主要对应文件／目录 | 状态 |
| --- | --- | --- | --- | --- |
| M1 | `docs/library.md` · 书架、导入与书库 | EPUB 导入、规范 Markdown、章节与图片、元数据和书籍身份；书架搜索、排序、分类、详情、重置与删除；桌面与静态书架的操作差异；输入到阅读文档的坐标衔接 | `src/components/reader/LibraryHome.vue`、`src/reader/library.ts`、`src/App.vue`、`crates/readux-core/src/import/`、`library.rs`、`reader_markdown.rs`、`src/services/staticLibrary.ts` | 待编写，承接 reader |
| M2 | `docs/reader.md` · 阅读器与正文交互 | 正文块及虚拟行、胶囊和注音、整体与组分命中、选择及触摸交互；查找、章节跳转、阅读进度、排版和图片；分析更新后的渲染；加载、重试及页面切换 | `src/components/reader/ReaderDocumentView.vue`、`ReaderSurface.vue`、`ReaderNavigationPanel.vue`、`ReaderAppearancePanel.vue`、`src/reader/document.ts`、`document.worker.ts`、`rows.ts`、`virtualization.ts`、`capsuleProjection.ts`、`lookupPresentation.ts`、`src/composables/` | 待重写 |
| M3 | `docs/grammar_guide.md` · 语法指南与内容制作 | 课程入口、目录、原稿混排、译文与注音开关、例句和查询；内容、分析、词典快照的数据关系；内容导入、制作、审阅、保存与加载的实际流程；重生成范围及已有编辑的处理；气泡测试课与正式课程的关系 | `src/components/grammar/GrammarGuidePanel.vue`、`GrammarGuideExample.vue`、`src/grammar-guide/`、`scripts/import_grammar_guide_source.mjs`、`generate_grammar_guide_content.py`、`generate_grammar_guide_analysis.py`、`build_grammar_guide_package.mjs`、`build_morphology_stage11_guide.py`、`grammar_guide_assets.mjs` | 待编写，合并指南文档 |
| M4 | `docs/explanation_layout.md` · 查询说明与活用图展示 | 词典、活用和接续说明的内容分工；整体及内部查询状态；点击、悬浮、关闭与拖选；浮层、侧栏、抽屉和窄屏；句中活用图、前后接续树、节点与连接器、折叠和当前路径；正文高亮与图示范围的对应 | `src/explanation/`、`src/composables/useExplanation*.ts`、`src/components/explanation/`、`ReaderExplanationBubble.vue`、`ReaderConnectionTree.vue`、`ReaderMorphology*.vue`、`src/reader/morphologyBubble.ts`、`morphologyGraph.ts`、`lookupPresentation.ts` | 待重写，合并阶段展示规范 |
| M5 | `docs/storage.md` · 存储、资源与缓存 | 书库文件和 SQLite、阅读状态、偏好、provider 配置、用户规则、词典缓存、分析缓存、静态发布包和浏览器状态；各自的所有者、位置、格式、身份、更新和重建条件；开发、便携及浏览器的数据目录 | `crates/readux-core/src/library.rs`、`reader_state.rs`、`analysis_cache.rs`、`analysis_engine.rs`、`document_session.rs`、`rule_store.rs`、`providers.rs`、`dictionary/bundle.rs`、`src-tauri/src/reader_engine.rs`、`src/services/staticLibrary.ts`、`src/grammar-guide/` | 待编写 |
| M6 | `docs/backend.md` · 本机后端与宿主接口 | Tauri 命令、Rust `AnalysisService`、CLI `stdio`、开发 HTTP bridge 与浏览器适配；请求分派、资源定位、阻塞任务与子进程管理；服务错误和配置检查；模块注册与当前可访问能力 | `src-tauri/src/lib.rs`、`crates/readux-core/src/analysis.rs`、`providers.rs`、`src/services/reader.ts`、`nlp.ts`、`staticLibrary.ts`、`staticAnalysis.ts`、`scripts/nlp_bridge.mjs`、`scripts/nlp_provider.py` | 待编写 |
| M7 | `docs/reader_engine.md` · 文档会话与分析调度 | 应用、书籍和会话的生命周期；正文准备与分析单元计划；范围请求、优先级、分阶段分析、增量版本和合并；继续、取消、重试、关闭、缓存恢复；进度展示与查询产物有效期 | `src-tauri/src/reader_engine.rs`、`crates/readux-core/src/document_plan.rs`、`document_session.rs`、`analysis_engine.rs`、`analysis_progress.rs`、`src/composables/useDocumentSession.ts`、`src/reader/session.ts`、`documentOperation.ts`、`analysisProgress.ts` | 待重写 |
| M8 | `docs/language_analysis.md` · 核心分析流程与对象 | 从输入正文到查询和阅读投影的全流程；token、语素、活用链、构词、词汇整体、文节、句法、语法及表达的概念关系；每阶段的输入输出与证据；当前生产消费范围和后续分析任务 | `crates/readux-nlp/src/model.rs`、`unify.rs`、`application.rs`、`lexical.rs`、`formation.rs`、`projection.rs`、`bunsetsu.rs`、`clause.rs`、`crates/readux-core/src/language_analysis.rs`、`analysis_engine.rs`、`src/types/nlp.ts` | 待重写 |
| M9 | `docs/nlp_sources.md` · 分词、来源与坐标对齐 | UniDic/Vibrato 的 CWJ、CSJ 与语域路由；词元、词形、假名和发音字段；GiNZA 的词界与结构；旧字、作者注音、正文准备、字符与字节坐标；来源规范化、多对多对齐、gap、关系与诊断 | `crates/readux-nlp/src/native.rs`、`sources.rs`、`prepare.rs`、`routing.rs`、`kyujitai.rs`、`ruby.rs`、`external.rs`、`syntax.rs`、`alignment.rs`、`alignment_group.rs`、`structure_graph.rs`、`scripts/nlp_adapters.py` | 待重写 |
| M10 | `docs/morphology.md` · 语素、活用与接续分析 | 词汇核心和功能语素身份；词干、原子特征、链、operator 与 occurrence；输入输出状态、各类执行器、局部接续、词性转换、补助递归、插入和文言；词形还原、连接器及来源证据；查询与展示的输出契约 | `crates/readux-nlp/src/morphology.rs`、`morphology_machine.rs`、`linguistic_context.rs`、`adverb.rs`、`grammar.rs`、`configs/unidic-nlp/morphology/source/` | 待编写，承接状态机设计 |
| M11 | `docs/morphology_configuration.md` · 活用规则配置与维护 | 源文件分工及生成产物；v1 字段、定义和执行元数据；条件、规则顺序、paths、连接器、概念路由、原型与展示配置；新增和修改规则的实际步骤；编译、引用检查、快照更新及缓存影响 | `configs/unidic-nlp/morphology/source/`、`configs/unidic-nlp/morphology.json`、`scripts/build_morphology_config.py`、`check_morphology_connections.py`、`crates/readux-nlp/src/morphology_config.rs`、`src/reader/morphologyConfig.ts` | 待重写，合并配置协议 |
| M12 | `docs/dictionary.md` · 词典、查询目标与聚合 | 词汇范围发现、整体和内部目标、功能成员、候选与决定；表记、读音、别名、默认项及多词典矩阵；`.kdict`、SQLite、词条适配、释义结构、关联导航和缓存；目标版本与按需加载；静态查询快照 | `crates/readux-core/src/dictionary/`、`src/explanation/dictionaryMatrix.ts`、`src/components/dictionary/`、`src/types/index.ts`、`scripts/build_dictionary_bundle.py`、`dictionary_bundle.py`、`dictionary_schema.py`、`build_jcdict_bundles.py` | 待重写 |
| M13 | `docs/grammar.md` · 语法知识、规则与表达 | concept、sense、realization、rule、occurrence 的实际关系；助词、助动词及补助讲解；知识源、编译目录、索引和查询；语言规则预览及保存；表达候选和识别范围；知识内容、正文识别与指南教学内容的协作 | `configs/grammar/`、`crates/readux-core/resources/grammar/`、`grammar_catalog.rs`、`expression_catalog.rs`、`language_rules.rs`、`rule_store.rs`、`crates/readux-nlp/src/grammar.rs`、`expression.rs`、`rules.rs`、`src/grammar/`、`scripts/build_grammar_catalog.py` | 待编写 |
| M14 | `docs/user_state.md` · 词汇状态、摘录与学习功能 | 已知状态、曝光、选择范围、笔记、阅读偏好及当前 JSON 导出；身份和跨文档恢复；桌面与浏览器的保存方式；画像、汉字知识、完整导出、Anki 和词典助手的后续接入范围 | `crates/readux-core/src/reader_state.rs`、`src/types/reader.ts`、`src/App.vue`、`ReaderDocumentView.vue`、`src/services/staticLibrary.ts`；后续基础为 `crates/readux-core/src/profile/`、`export/`、`llm/`、`src/services/dictionaryAssistantPort.ts` | 待重写 |
| M15 | `docs/quality.md` · 验证、质量评估与问题定位 | 按功能和修改范围组织已有检查；分析、查询、坐标、快照、交互与持久化的验证方法；人工期望、固定语料和性能报告；质量工具的有效入口与缺失依赖；已知问题的登记方式 | `crates/readux-nlp/tests/`、`crates/readux-core/tests/`、`src/**/*.test.mjs`、`scripts/validate_*.py`、`verify_*.mjs`、`test_reader_*.py`、`data/validation/`；审计基础为 `crates/readux-quality-audit/`、`src-tauri/src/quality_audit.rs`、`src/components/quality/` | 待重写 |
| M16 | `docs/standalone_distribution.md` · 桌面资源与发行 | 开发资源到独立运行资源的制作；UniDic、词典、Python 和模型的发行结构；Windows 便携 ZIP、macOS DMG、CI 与资源 Release；版本、图标、许可、用户数据和升级；交付检查及当前平台范围 | `scripts/run_channel.ps1`、`prepare_installer_resources.py`、`prepare_release_resources.py`、`fetch_release_resources.py`、`package_portable.py`、`check_release_resources.mjs`、`src-tauri/build.rs`、`src-tauri/src/windows_icons.rs`、`src-tauri/tauri*.conf.json`、`.github/workflows/release.yml`、`THIRD_PARTY_NOTICES.md` | 待重写 |

### 职责划分与待核实重点

**书库、阅读器与会话**：M1 负责导入和书籍管理，M2 负责正文显示与操作，M7 负责分析任务与会话。M5 集中定义存储位置和数据生命周期，相关模块引用对应条目。

**分析、知识与说明**：M8 负责流程和对象关系，M9 负责来源与坐标，M10 负责活用执行，M11 负责规则制作，M12 负责词典与查询决定，M13 负责语法知识和表达。M4 定义查询结果及活用图怎样展示，M3 定义教学内容怎样组织和发布。

**用户状态与学习目标**：当前 `App.vue` 的 `exportSelections` 直接导出 JSON，词汇与摘录由 `reader_state.rs` 和静态宿主承接。M14 以这些实际路径说明现状，再记录 `profile/`、`export/`、`llm/` 的可复用内容和接入条件。

**质量工具**：`scripts/language_quality.py` 的部分委托脚本在当前目录中缺失，审计 crate 和 UI 保留了基础文件。M15 逐项列明可执行的检查、读取历史数据的工具、需要恢复的流程；研究记录引用已有结果时保留对应环境。

**指南名称**：当前 `lesson-1-1.json` 为《句法入门》，`lesson-2-1.json` 为《认识实词、虚词、助词》。原 `grammar_guide_1_1.md` 对应后者的内容规格。M3 按当前课节身份组织内容，并说明原稿、输入例句与生成资源之间的关系。

## 6. research 文档与实验入口

实验、测量、历史审阅和未来方案统一位于 `docs/research/`。每项说明研究问题、脚本入口、输入、环境、输出位置、主要结论及后续任务。共享环境准备引用开发说明；需要特定模型、GPU 或本机资料时在实验文档列明条件。

| 编号 | 文件与标题 | 内容概要及对应入口 | 状态 |
| --- | --- | --- | --- |
| R0 | `docs/research/README.md` · 研究与实验索引 | 按来源、活用、查询、GPU、运行资源、Web 与云端列出实验；标明已完成测量、可复现工具、局部保留及暂缓方案；区分脚本、环境、数据和报告目录 | 待编写 |
| R1 | `docs/research/nlp_behavior.md` · NLP 来源行为基线 | 保留历史环境和观察，说明与当前生产来源的关系；入口为 `scripts/probe_nlp_behavior.py`、`probe_ginza_provider.py`、`probe_kwja_provider.py`、`probe_syntax_providers.py`、`kwja_adapter.py`；数据为 `data/validation/behavior/` | 待校订 |
| R2 | `docs/research/nlp_validation_training.md` · 来源评估与 UniDic 训练实验 | 语料收集、标注、gold、字段词表、结构监督、预测与评估的组织；UniDic、GiNZA、KWJA 对比及原生 provider 对等性；训练数据和模型入口；对应 `scripts/collect_*validation.py`、`build_*gold.py`、`build_*validation_set.py`、`evaluate_*.py`、`compare_provider_outputs.py`、`prepare_unidic_training.py`、`train_unidic_model.py`、`nlp_retraining/` | 待编写 |
| R3 | `docs/research/adverb_lexicon.md` · 副词接续资格与词表审定 | 词表导出、资格候选、外部来源筛查、LLM 分类与人工审阅、运行映射生成；对应 `scripts/build_adverb_lexicon.py`、`classify_adverbs_llm.py`、`data/adverbs-llm/`、`configs/unidic-nlp/morphology/source/adverbs/`、`operators/adverb.json`、`crates/readux-nlp/src/adverb.rs` | 待编写 |
| R4 | `docs/research/morphology_review.md` · 活用分析审阅与验证 | 整理谓语关系、规则覆盖、阶段任务及样例结果，保留日期和环境；说明指南气泡测试、静态书库抽样和小说性能的用途；对应 `data/validation/morphology_*cases.json`、`crates/readux-nlp/tests/morphology*.rs`、`scripts/compare_morphology_dist.py`、`benchmark_p4_novel.py`、`review_p4_samples.py`、`export_p4_sample_report.py`、`verify_adverb_capabilities.mjs` | 待编写，合并审阅资料 |
| R5 | `docs/research/lookup_issue_tracking.md` · 查询与阅读投影问题记录 | 保留问题样例、复现、归因和状态，按当前实现复核；词典构建与内容审查工具一并建立索引；对应 `scripts/inspect_lookup_cases.py`、`audit_dictionary_bubble.py`、`audit_dictionary_detail_formats.py`、`verify_dictionary_v4.py`、`test_reader_morphology.py` 及文档已引用的数据 | 待迁移并校订 |
| R6 | `docs/research/ginza_cuda_measurement.md` · GiNZA CUDA 测量 | 保留精度、批次、显存、完整输出和人工复核的历史结论；对应 `scripts/benchmark_ginza_cuda.py`、`compare_ginza_precision.py`、`data/validation/ginza_cuda_*.json` | 待校订 |
| R7 | `docs/research/ginza_optimization_handoff.md` · GiNZA 管线优化实验 | 整理共享 BERT、CPU 解码、阶段重叠、序列化和多流的实测与复现；对应 `scripts/ginza_bert_engine.py`、`ginza_decode.pyx`、`ginza_batching.py`、`benchmark_ginza_optimized.py`、`benchmark_ginza_pipeline_suite.py`、`benchmark_ginza_gpu_concurrency.py`、`analyze_ginza_pipeline.py`、`profile_ginza_pipeline.py` | 待校订 |
| R8 | `docs/research/ginza_gpu_parallel_design.md` · GiNZA GPU 并行方案 | 长度分桶、跨请求合批、缓冲槽、异步传输和 worker 的设计；明确本机测量、目标设备外推与待验收条件；对应 `scripts/ginza_bert_engine.py`、`ginza_batching.py`、`data/validation/ginza_gpu_pipeline_report.json` | 待校订 |
| R9 | `docs/research/runtime_optimization.md` · 本机运行资源与优化研究 | GiNZA 最小环境、依赖及模型空间审计、原生替代资源预算和运行性能；对应 `scripts/audit_ginza_runtime.py`、`build_ginza_minimal_env.py`、`test_ginza_runtime_pruning.py`、`measure_external_nlp_architecture.py`、`measure_native_provider_resources.py`；承接独立发行文档中的研究部分 | 待编写 |
| R10 | `docs/research/static_storage_measurement.md` · 静态资源空间测量 | 保留测量方法、正文及指南规模、压缩与共用内容的统计；更新到当前资源结构的复现入口；对应 `scripts/measure_static_storage.py`、`measure_static_storage.mjs`、`data/validation/static-storage-summary.json` | 待校订 |
| R11 | `docs/research/web_product_strategy.md` · Web 产品与资源方向 | 区分已经实现的静态阅读、可复用设计与待决定的私有包、离线、同步和生态扩展；对应静态宿主、资源生成脚本及当前产品入口 | 待校订 |
| R12 | `docs/research/cloud_saas_roadmap.md` · 云端分析服务方案 | 保留云端推理、任务、存储、账户、额度及成本方案，注明价格和容量依据的日期；结合本机模型核心更新接入位置；当前仓库的相关基础为 `scripts/ginza_bert_engine.py`、`ginza_batching.py` 和文档会话单元 | 待校订，方案待决策 |
| R13 | `docs/research/cloud_gpu_first_experiment.md` · 云端 GPU 部署实验计划 | 保留暂缓状态、输入输出、镜像、平台调用和测量任务；列明计划中的文件及尚待实现部分。原文的 `cloud/inference/` 和 `scripts/cloud_gpu_experiment.py` 属于计划入口 | 待校订，实验暂缓 |

GPU 优化原型与桌面 provider 的关系须沿调用点说明。研究文档中的“生产执行核心”需要明确所属方案；桌面当前调用链以 `providers.rs`、`nlp_provider.py`、`nlp_adapters.py` 为依据。目标 GPU 性能预测和历史测量分别保留结论。

## 7. 索引、开发待办与旧文档去向

| 编号 | 文件与标题 | 职责与对应内容 | 状态 |
| --- | --- | --- | --- |
| I1 | 根目录 `README.md` · 项目介绍与快速入口 | 简短定位、当前使用入口、安装或体验方式、开发启动和文档导航；功能表述与 G1 一致，命令与 G3 一致；对应实际桌面和 Web 入口 | 待重写 |
| I2 | `docs/README.md` · 文档索引 | 按总体、入口、模块和研究组织全部正式文档，给出阅读顺序；以各篇实际存在的文件建立链接 | 待重写 |
| I3 | `docs/TODO.md` · 开发任务与完成条件 | 按当前模块重建产品和实现待办；每项引用设计、入口、已有基础及完成条件；承接尚未完成的研究决策和实现任务 | 待重写 |

本文跟踪文档交付，I3 跟踪软件开发。模块正文中的后续工作引用 I3，具体实验步骤留在相应研究文档。

以下处理覆盖本轮起点保留的全部 31 篇 Markdown。合并时先核对有效设计、输入示例和未完成事项，分别进入目标正文、研究记录和 TODO，再移除被替代文件并更新引用。

| 现有文件（位于 docs 下） | 计划处理 |
| --- | --- |
| `README.md` | 重写为 I2，修复索引与阅读顺序 |
| `TODO.md` | 重写为 I3，按当前实现核实阶段及状态 |
| `development.md` | 重写为 G3；桌面使用和交付细节分别进入 E1、M16，Web 制作进入 E2 |
| `static_library.md` | 扩写为 E2 |
| `reader.md` | 书架与导入进入 M1；正文交互重写为 M2；存储细节进入 M5 |
| `reader_engine.md` | 重写为 M7；界面和查询设计分别引用 M2、M4、M12 |
| `dictionary.md` | 对照当前查询调用重写为 M12 |
| `explanation_layout.md` | 重写为 M4，覆盖共用说明布局及图示 |
| `grammar_guide_display.md` | 合并至 M3，以现有课程、生成器与审阅操作说明设计 |
| `grammar_guide_1_1.md` | 有效内容规格合并至 M3 的 2.1 内容制作说明；原稿及例句正文按当前内容资源定位 |
| `language_analysis.md` | 重写为 M8；语法知识细节进入 M13 |
| `nlp_sources.md` | 重写为 M9，核实字段及协议版本 |
| `morphology_state_machine_design.md` | 重写为 M10，采用当前模块名称与执行模型 |
| `morphology_configuration.md` | 重写为 M11 |
| `morphology_storage_refactor_analysis.md` | 当前配置协议与编译规则进入 M11；迁移、测量和验证记录进入 R4 |
| `morphology_stage11_bubble_display.md` | 当前图示、词条及接续展示契约进入 M4；阶段实验与样例记录进入 R4 |
| `morphology_predicate_relations_review.md` | 稳定结论进入 M10、M4；审阅过程进入 R4 |
| `morphology_operator_inventory.md` | 当前规则组织进入 M11；逐项覆盖证据进入 R4；未完成工作进入 I3 |
| `morphology_refactor_execution_plan.md` | 完成记录与核实结果进入 R4；有效待办进入 I3，统一处理文首“全部完成”和表内未完成状态 |
| `lookup_issue_tracking.md` | 迁入 R5，逐项复核现状 |
| `user_state.md` | 重写为 M14；持久化协议进入 M5 |
| `quality.md` | 重写为 M15，按实际可调用工具组织验证 |
| `standalone_distribution.md` | 重写为 M16；资源与性能研究进入 R9 |
| `research/nlp_behavior.md` | 保留并校订为 R1 |
| `research/ginza_cuda_measurement.md` | 保留并校订为 R6 |
| `research/ginza_optimization_handoff.md` | 保留并校订为 R7 |
| `research/ginza_gpu_parallel_design.md` | 保留并校订为 R8 |
| `research/static_storage_measurement.md` | 保留并校订为 R10 |
| `research/web_product_strategy.md` | 保留并校订为 R11 |
| `research/cloud_saas_roadmap.md` | 保留并校订为 R12 |
| `research/cloud_gpu_first_experiment.md` | 保留暂缓状态并校订为 R13 |

当前索引中指向已删除文件的链接，在相关文档重写时改为有效目标。原文件名可用于旧引用的归属判断。根目录 `sources.md` 继续负责外部来源及标题索引；文档迁移时同步修正其涉及的仓库内引用。

`configs/unidic-nlp/morphology/source/README.md` 保留为就近维护索引，与 M11 相互引用。`docs/summarize.py` 作为现存辅助脚本在目录整理时登记用途，应用与模块的设计依据由本文规定的文档承担。

## 8. 编写顺序与完成条件

| 阶段 | 范围 | 完成条件 | 状态 |
| --- | --- | --- | --- |
| 0. 确认计划 | 本文的目录、职责、资料去向与顺序 | 用户审阅并确认；必要的产品目标分歧已经登记 | 待审阅 |
| 1. 总体设计 | G1、G2、G3 | 目标范围、当前结构和开发入口相互一致；待决策内容明确；能够据此定位各模块 | 待开始 |
| 2. 用户入口与阅读功能 | E1、E2、E3、M1、M2、M3、M4 | 每个实际入口有使用与运行说明；书架、阅读器、指南的行为和宿主差异可查；开发预览和实验入口有归属 | 待开始 |
| 3. 后端与分析模块 | M5 至 M15 | 数据、服务、会话、来源、活用、查询和知识模块的职责、主要对象及调用关系完整；实际功能与后续目标有明确状态 | 待开始 |
| 4. 交付与研究资料 | M16、R0 至 R13 | 发行流程与脚本一致；全部实验有入口及状态；历史数字保留日期、环境和来源；资料迁移去向落实 | 待开始 |
| 5. 总索引与待办核对 | I1、I2、I3及全文引用 | 旧文档去向逐项完成；索引链接有效；目标、架构、实现描述与待办一致；本文记录交付结果 | 待开始 |

顺序用于控制依赖。同一阶段逐篇完成，出现跨模块事实差异时先在本文登记，再调整受影响的正文。I2、I3 可随模块交付逐步更新，阶段 5 统一检查。

单篇设计文档完成时，应当能回答：模块解决什么问题，当前提供什么能力，主要输入输出和数据由谁管理，与其他模块怎样协作，代码及资源从哪里进入，怎样进行针对性验证，剩余工作是什么。入口文档以可复现的操作步骤为主，研究文档按实验问题组织。

验证复用项目已有命令和数据，范围与文档内容对应。目录规划和纯文字修改采用内容、路径及链接核对；运行说明需要核实相关启动或制作步骤；涉及实现改动时单独安排对应检查。提交前统一检查 diff，独立主题分开提交。

## 9. 进度与待确认事项

- [x] 阅读仓库指引和保留文档，定位入口、模块及主要实验工具。
- [x] 形成文档清单、职责、代码范围、旧资料去向和编写顺序。
- [ ] 用户确认本计划的目录与范围。
- [ ] 完成三个总体文档并确认目标和架构表述。
- [ ] 逐篇完成入口和功能模块文档。
- [ ] 完成研究资料整理、索引和开发待办更新。
- [ ] 核对全部交付文档及引用，结束本次文档更新。

后续每篇文档完成时，更新清单中的状态，并在下表记录结果。状态采用“待编写／待重写／待校订、编写中、待审阅、已完成”；需要用户选择的事项直接记录具体问题。

| 日期 | 编号或范围 | 交付及核实结果 | 待处理事项 |
| --- | --- | --- | --- |
| 2026-11-08 | 本计划 | 已完成保留文档与当前入口概况排查，形成重写和追踪清单 | 审阅文档划分、内容范围及编写顺序 |
