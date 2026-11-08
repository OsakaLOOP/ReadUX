# 文档更新总计划与追踪

更新日期：2026-11-09。文档基线：`34dc0f7` 后保留的 docs。产品与架构方向以本次用户说明为准。状态：目录与方向已审阅，总体文档及语言分析 L0–L6 已完成，其余现有文档交接整理。

本计划确定 ReadUX 文档的名称、内容范围、代码入口、现有资料去向和编写顺序。进度集中记录于本文。

## 交接要求与本轮范围

用户已审阅目录及方向，并进一步确定本次文档工作的范围：

- 完成现有实现对应的“待编写”“待重写”文档，重写和整合保留文档中的有效内容；当前重点为三个总体文档。
- 后续语言分析及应用模块的具体开发方法尚待讨论。标记“待设计”的条目保留方向、现有基础和问题，详细算法、规则格式、接口与选型暂缓。
- `docs/research/` 内部文档保持原状。本次不在其中创建、改写、合并或迁移文档；总体文档简要说明研究范围并引用原文。
- 当前代码、资源和目录名称按实际引用说明。grammar 原整体流程及旧语法库界面设计在文档中标明过时，现用词条和解说资源继续记录实际用途。

接替 agent 先阅读[开发目标](goals.md)、[总体架构](architecture.md)和[开发说明](development.md)，再依据本文定位现有模块。各项清单的内容范围受以上交接要求约束。L7 至 L10、M10 至 M12 的详细设计暂缓；M2 和 L11 先完成当前实现及既有设计的整理，后续部分仅保留方向。研究内容的交付要求集中见第 7 节。

已完成 G1、G2、G3、总索引及 L0–L6。L0–L6 按现有实现说明输入、UD、状态机、规则和词条，并结合运行截图对应阅读体验。用户进一步明确，具体语法规则的深入设计、完整 JSON 和复杂分支可以留给下一位 agent；交接清单见 [L5](language_analysis/morphology_configuration.md#规则细节交接)和 [L6](language_analysis/entries.md#内容维护与后续细化)。后续 agent 继续整理 L11、M3、M4 及其他模块，每篇完成后更新状态和链接，集中进行相应范围的最终检查。

## 1. 文档依据与当前方向

依据包括保留的 23 篇顶层文档、8 篇 research 文档，以及当前代码、配置、资源和可访问入口。根目录 README 和 memory 用于了解指引及定位文件。旧架构和功能清单需要依据最新目标重新编写；现有代码用于说明实现基础及接入程度。

当前核心成果由三个部分组成：活用分析状态机；以 `configs/unidic-nlp/morphology/` 为主要维护入口的规则、身份与词条关联；为分析结果设计的气泡、侧栏、抽屉及 D3 图组件。文档优先完整说明这些成果的设计、数据和协作关系。

后续语言分析沿用活用模块已经形成的方法：依据底层词性、实体身份、活用状态和录入规则，整合 GiNZA 的 UD 依存信息，确定命中对象与关系，选择具体义项或用法，生成解说并在 UI 中呈现。主要开发方向为词组整体范围及修饰、并列关系，基于格助词的句中成分和基本依存关系，以及已登记语法模式的识别与提取。

应用功能规划保留 Anki 导出、丰富的收藏管理及统计、通过 epub.js 或同类方案替换自研阅读器、LLM 调用。书架、词典、语法指南、桌面与静态 Web 入口作为现有应用基础分别说明。研究资料及实验继续保留在 `docs/research/`，覆盖已有的云端推理、GPU、来源比较、训练和测量内容。

正文区分当前实现、后续方向和研究结果。当前实现说明实际调用和数据；后续方向记录用户已确定的目标和现有基础；研究结果通过现存原文引用。已有文档中的有效内容按当前目标整理。

## 2. 核心资产与调用核对

### 2.1 活用分析、规则与前端展示

| 组成 | 当前代码与资源 | 文档需要说明的设计 |
| --- | --- | --- |
| 词法与状态 | `crates/readux-nlp/src/model.rs`、`morphology.rs`、`morphology_machine.rs` | UniDic 字段、词性和身份怎样形成输入状态；各类执行器怎样判断接续、转换状态及组织父子链；实际范围、词形、operator 和 occurrence 的关系 |
| 规则维护 | `configs/unidic-nlp/morphology/source/`、`configs/unidic-nlp/morphology.json`、`scripts/build_morphology_config.py` | 定义与执行元数据、局部接续、路径和连接器、文言、插入、副词资格、原型及展示配置；源文件到运行产物的编译和引用 |
| GiNZA 证据 | `crates/readux-nlp/src/linguistic_context.rs`、`external.rs`、`scripts/nlp_adapters.py` | 正式 token、POS、活用信息、UD 依存及来源身份的读取；定向依存检查与两端关系检查的用途；插入补助等构式对 `case`、`mark`、`advcl`、`aux`、`xcomp` 的实际使用 |
| 词条与解说 | operator 的 `concept`／`variants.concept`、`configs/grammar/`、`crates/readux-core/resources/grammar/source/` 与 `compiled/` | 规则身份到词条、具体用法、解说及例句的关联；各类资源的真实维护位置、生成方式和当前使用方式 |
| 内容投影 | `src/reader/lookupPresentation.ts`、`morphologyBubble.ts`、`morphologyConfig.ts` | 从活用产物生成词条、当前步骤、句中组成和接续规则树；查询目标、图元素和正文高亮范围怎样对应 |
| 三种视图 | `src/explanation/`、`src/composables/useExplanation*.ts`、`src/components/explanation/`、`ReaderExplanationBubble.vue` | 气泡、侧栏和抽屉共用的激活状态与内容；布局切换、窄屏、测量、滚动、关闭及触摸交互 |
| D3 图组件 | `src/reader/morphologyGraph.ts`、`ReaderConnectionTree.vue`、`ReaderMorphologyGraph.vue` | 句中活用图与规则树的数据、布局及绘制；节点、连接器、关系文字、折叠、当前路径、尺寸和过渡；后续分析模块可复用的范围 |

### 2.2 grammar 资源与过时设计的核对

核对日期：2026-11-09。`configs/grammar/` 与 `resources/grammar/` 承担不同职责，当前活用词条解说同时使用了这些资源。

| 对象 | 实际引用及结论 | 文档处理 |
| --- | --- | --- |
| `configs/grammar/particles.json` | `src/grammar/particleEntries.ts` 直接读取；`grammarMorphemeLookup` 由阅读器与指南例句调用，生成内置助词解说 | 按现用词条资源说明 |
| `configs/grammar/auxiliaries.json` | `particleEntries.ts` 和 `morphologyBubble.ts` 直接读取；后者在活用词条中提供部分摘要和例句，`ません` 的摘要有专门选择逻辑 | 按现用助动词资源说明，并记录与 concept 解说的取用次序 |
| `configs/grammar/migrations/te_construction_ids.json` | 前端仍有直接导入；`grammarMorphemeLookup` 的部分补助词处理使用映射，文件同时保留原构式 ID 信息 | 标明历史映射及现存调用，具体用途按调用分支说明 |
| `crates/readux-core/resources/grammar/compiled/grammar_explanations.json` | `morphologyBubble.ts` 按 operator 的 `concept_id` 读取摘要及例句；当前 153 个 operator 及变体共引用 125 个概念，125 个均有解说；`check_morphology_connections.py` 检查解说及例句 | 作为活用说明的有效依赖，记录 source → compiled → 前端的关系 |
| grammar catalog 查询 | `grammar_catalog.rs` 嵌入编译目录及解说，Tauri 注册目录搜索与概念查询；`GrammarLibraryPanel.vue` 保留界面，主应用未接入该组件 | 说明已有请求和内容基础，标明旧语法库界面设计过时 |
| 通用 grammar 应用流程 | `language_analysis.rs` 中保留 `apply` 及规则转解说实现；普通分析链路未调用 `language_analysis::apply`，`refresh_language` 当前复制并保存已有文档；规则预览仍通过 `language_analysis::preview` 执行 | 标明原整体流程设计过时；已有规则预览、对象和资源按实际用途说明 |

资源名称相近时，文档须写明完整路径。`configs/unidic-nlp/morphology/source/classical/auxiliaries.json` 属于活用执行规则；`configs/grammar/auxiliaries.json` 属于内置讲解词条。当前词条资源与后续语法模式模块之间的关系，需要依据新模块职责设计。

本轮只更新文档计划和相关说明，代码与资源保持原状。

## 3. 当前入口

| 入口 | 定位依据 | 对应文档 |
| --- | --- | --- |
| 桌面端 | `src/main.ts`、`src/App.vue`、`src-tauri/src/lib.rs`、`src-tauri/src/reader_engine.rs`；书库、EPUB 导入、文本输入、阅读与查询 | E1、M1、M2、M8、M9 |
| 静态 Web | `src/services/reader.ts`、`staticLibrary.ts`、`staticAnalysis.ts`、`public/library/`；预生成书籍、查询资源和浏览器状态 | E2、M7 |
| 语法指南 | `src/components/grammar/GrammarGuidePanel.vue`、`GrammarGuideExample.vue`、`src/grammar-guide/`；三课入口及内容快照 | M6；已有气泡测试样例同时用于说明和验证现有模块 |
| 开发 CLI | `crates/readux-core/src/bin/readux-nlp.rs`；`inspect`、`repl`、`stdio` 共用 `AnalysisService` | E3 |
| 前端预览与 bridge | `package.json`、`vite.config.ts`、`scripts/nlp_bridge.mjs`；5173 静态前端、1420 NLP bridge、header 预览及指南直达参数 | G3 |
| 实验与制作工具 | `scripts/` 提供入口，`experiments/` 保存环境及部分结果，`data/validation/` 保存语料和验证数据 | 三个总体文档简述并引用现有研究；资源制作同时引用相应模块 |

当前静态书籍按范围读取已生成分析；桌面端由本机 Rust 服务与 GiNZA 进程生成分析。指南例句使用独立制作的内容、分析及词典快照。各篇文档分别说明启动条件、能力范围、资源准备和结果保存位置。

## 4. 三篇总体文档

| 编号 | 文件与名称 | 内容概要 | 对应文件／目录 | 状态 |
| --- | --- | --- | --- | --- |
| G1 | `docs/goals.md` · 开发目标与功能范围 | 核心成果、当前入口、后续方向、研究入口与文档交接 | 活用状态机与配置、`src/reader/`、`src/App.vue`、阅读及指南组件 | 已完成 |
| G2 | `docs/architecture.md` · 总体架构 | 当前组成及实际分析链路、活用规则与词条、说明视图和图组件、静态资源及数据；后续方向单独简述 | `Cargo.toml`、`src/services/`、`src-tauri/src/`、`crates/readux-nlp/src/`、`crates/readux-core/src/`、配置和说明组件 | 已完成 |
| G3 | `docs/development.md` · 开发说明 | 当前运行入口、资源及配置、规则与解说维护、快照制作、构建发行、针对性检查和交接 | `package.json`、`vite.config.ts`、Tauri 配置、`scripts/`、release workflow | 已完成 |

G2 以实际调用为主说明现有实现，后续方向独立简述。组件图、分析数据流、静态资源数据流及模块职责表相互对应。研究方向通过链接引用 research 原文。

G3 已记录 Tauri 开发 URL 为 1420、前端启动为 5173 的配置差异，并根据当前 CLI 的配置合并能力提供 URL 覆盖命令。

## 5. 语言分析文档目录

语言分析统一组织在 `docs/language_analysis/`。目录覆盖现有活用核心、底层来源、规则和词条，以及三个后续模块的分析、匹配、用法确定、解说和展示。目录划分对应文档职责，具体代码组织在详细设计时确定。

| 编号 | 文件与名称 | 内容概要 | 当前对应文件／目录或拟接入位置 | 状态 |
| --- | --- | --- | --- | --- |
| L0 | [语言分析总览](language_analysis/README.md) | 概念与模块索引；现有能力及后续三个模块；词性、身份、状态、规则、UD、词条和 UI 的关系；阅读顺序及开发依赖 | `crates/readux-nlp/src/lib.rs`、`model.rs`、`unify.rs`、`crates/readux-core/src/analysis_engine.rs`、现有语言分析文档 | 已完成 |
| L1 | [分析流程与模块协作](language_analysis/pipeline.md) | 正文准备、分词和来源对齐、活用分析、查询及投影的实际调用；词组、句中成分和语法模式的后续接入；各阶段输入输出和对象引用；即时分析与静态快照 | `crates/readux-nlp/src/unify.rs`、`application.rs`、`formation.rs`、`lexical.rs`、`projection.rs`、`crates/readux-core/src/analysis_engine.rs`、`document_session.rs`、`src/types/nlp.ts` | 已完成 |
| L2 | [正文、分词与基础身份](language_analysis/lexical_input.md) | UniDic/Vibrato、CWJ/CSJ 及语域；POS 层级、词元、表记、假名、发音、活用型和活用形；旧字、ruby、正文映射、字符及字节范围；后续规则所用底层字段 | `crates/readux-nlp/src/model.rs`、`sources.rs`、`prepare.rs`、`routing.rs`、`kyujitai.rs`、`ruby.rs`、`src/types/nlp.ts` | 已完成 |
| L3 | [GiNZA 与 UD 对齐](language_analysis/ud_alignment.md) | provider 的正式词元、主辞、有向依存和标签；与 UniDic 的多对多映射；范围和身份转换；局部结构查询；规则采用 UD 证据的条件、方向及冲突处理 | `scripts/nlp_adapters.py`、`crates/readux-nlp/src/external.rs`、`syntax.rs`、`alignment.rs`、`alignment_group.rs`、`structure_graph.rs`、`linguistic_context.rs` | 已完成；逐规则细节后续深化 |
| L4 | [活用状态机](language_analysis/morphology.md) | 状态、实体类别、词汇和功能身份、核心、链及父子链；执行器、局部接续、词性转换、补助递归、插入、文言和副词；实际词形与原型；operator、occurrence、连接路径及来源证据 | `crates/readux-nlp/src/morphology.rs`、`morphology_machine.rs`、`morphology_config.rs`、`linguistic_context.rs`、`adverb.rs` | 已完成；逐规则细节后续深化 |
| L5 | [活用规则与配置维护](language_analysis/morphology_configuration.md) | source 文件分工与 v1 产物；definition/execution、词性及状态条件、规则顺序、连接器、paths、变体和 presentation；新增规则和更新词条的操作；编译、引用与快照检查 | `configs/unidic-nlp/morphology/source/`、`configs/unidic-nlp/morphology.json`、`scripts/build_morphology_config.py`、`check_morphology_connections.py`、`src/reader/morphologyConfig.ts` | 已完成；完整 JSON 及复杂分支按交接清单深化 |
| L6 | [词条、用法与解说资源](language_analysis/entries.md) | 当前规则到 concept、解说及例句的读取关系；活用定义、内置助词与助动词、source/compiled 的各自作用；词条与用法身份、来源及维护入口；按第 2.2 节标明过时设计 | `configs/unidic-nlp/morphology/source/operators/`、`configs/grammar/`、`crates/readux-core/resources/grammar/`、`grammar_catalog.rs`、`scripts/build_grammar_catalog.py`、`src/grammar/particleEntries.ts`、`src/reader/morphologyBubble.ts` | 已完成；逐用法内容后续深化 |
| L7 | `docs/language_analysis/phrases.md` · 词组范围、修饰与并列 | 词组核心、整体范围、内部成员与嵌套；修饰项、被修饰项、修饰方法及并列成员；依据词性、身份、活用状态、连接成分和 UD 的匹配条件；具体用法、解说及 UI 对象；边界、歧义与代表例句 | 现有基础为 `linguistic_context.rs`、`structure_graph.rs`、`bunsetsu.rs`、`formation.rs`、`morphology.rs`；新增词组规则及分析模块在详细设计时定位 | 待设计，主要后续模块 |
| L8 | `docs/language_analysis/case_dependencies.md` · 格关系、句中成分与基本依存 | 词组及格助词的范围和身份；所依存谓语、句中成分及基本关系；助词同形、具体用法、接续和 UD 方向的联合判断；确定结果、解说和 UI；与修饰、并列及活用的衔接 | 现有基础为 `configs/grammar/particles.json`、`linguistic_context.rs`、`external.rs`、`structure_graph.rs`、`morphology.rs`、`src/grammar/particleEntries.ts`；格关系分析及规则待设计 | 待设计，主要后续模块 |
| L9 | `docs/language_analysis/grammar_patterns.md` · 登记语法模式的识别与提取 | 已登记模式的形式、成员身份和条件；接续、活用、词组与依存约束；连续及登记的非连续模式、命中范围与命名捕获；模式和词面相同形式的区分；用法选择、解说、提取及 UI | 可核查基础为 `crates/readux-nlp/src/rules.rs`、`grammar.rs`、`expression.rs`、`crates/readux-core/src/language_rules.rs`、`language_analysis.rs`、`resources/grammar/source/`；原流程标明过时，具体复用范围按新职责决定 | 待设计，主要后续模块 |
| L10 | `docs/language_analysis/matching_usage.md` · 匹配与义项用法确定 | 三个模块共用的判断原则；底层词性、领域身份、活用状态、局部条件和 UD 证据怎样形成确定结果；同形、多用法、重叠命中及组合规则；规则、用法及解说的关联；未确定结果与代表正反例 | 活用参照为 `morphology_machine.rs`、`morphology_config.rs`、`linguistic_context.rs`；现有 `Rule`、`RuleMatch`、`Explanation`、词条用法字段提供对象基础 | 待设计 |
| L11 | `docs/language_analysis/presentation.md` · 分析结果、解说与界面投影 | 各模块输出怎样映射到正文范围、整体与组分、查询目标、解说和关系图；当前活用投影；词组、格关系和模式的新增显示要求；同一结果在三种视图、指南及阅读器中的一致性 | `src/reader/lookupPresentation.ts`、`morphologyBubble.ts`、`src/types/nlp.ts`、`src/explanation/`、`ReaderConnectionTree.vue`、`ReaderMorphologyGraph.vue`、指南快照 | 待编写与设计 |

### 5.1 后续设计提纲（暂缓）

L7、L8、L9 保留对象、范围、词性与身份、规则、UD、用法、解说和 UI 等待讨论事项。L10 的共用判断方法留待后续设计，L11 先完整说明现有活用结果怎样交付界面。

现有 `bunsetsu.rs` 根据来源结构映射文节，`clause.rs` 根据来源范围建立句子和小句对象。新词组模块需要明确自己的整体范围与关系判定；格关系模块需要根据助词身份和上下文确定句中成分及依存对象。相应文档须说明现有结构对象可提供哪些输入，以及新增判断由哪个模块负责。

语法模式的识别范围以登记内容为准。模式命中、具体用法和解说分别对应可定位的依据。当前通用规则中的原子条件、捕获和范围对象可以逐项评估；旧文档中整体语法流程标明过时。

### 5.2 模块关系与待讨论事项

活用、基础身份和 UD 对齐提供现有输入。后续词组、格关系和语法模式的协作关系，以及匹配、用法和 UI 的具体方法，在进一步讨论后确定。

当前计划保留这些主题的文档位置。执行轮次、具体规则结构与代码目录均属后续设计内容。

## 6. 入口与应用模块文档

### 6.1 用户入口

| 编号 | 文件与名称 | 内容概要 | 对应文件／目录 | 状态 |
| --- | --- | --- | --- | --- |
| E1 | `docs/desktop.md` · 桌面端使用与运行 | 启动及资源要求、书架和输入、阅读及分析说明、查询与收藏、设置、状态恢复；便携包与平台范围；资源问题定位 | `src/App.vue`、`src/components/reader/`、`src-tauri/src/lib.rs`、`reader_engine.rs`、`src-tauri/tauri*.conf.json` | 待编写 |
| E2 | `docs/static_library.md` · 静态 Web 与内容部署 | 网页入口、预生成资源、发布书目、范围加载、浏览器状态；生成、版本清理、预览、gzip 和部署；指南链接及能力范围 | `src/services/staticLibrary.ts`、`staticAnalysis.ts`、`public/library/`、`scripts/generate_static_library.py`、`build_edgeone.py`、`static_library_document.mjs`、`verify_static_library.mjs` | 待重写 |
| E3 | `docs/cli.md` · CLI 与本机请求协议 | inspect、repl、stdio；逐行 JSON、资源和错误；分析、会话、查询、provider、现有规则预览和管理；UniDic/Vibrato 词典构建工具 | `crates/readux-core/src/bin/readux-nlp.rs`、`unidic-vibrato-build.rs`、`analysis.rs`、`scripts/nlp_bridge.mjs` | 待编写 |

E2 写明 `npm run build` 使用现成资源构建前端和清理版本，`npm run edgeone:build` 从本机书库与分析服务更新资源后生成 `dist/` 和 ZIP；具体步骤按当前脚本核实。E3 的请求目录以 `Request` 枚举为准。

### 6.2 应用设计

| 编号 | 文件与名称 | 内容概要 | 对应文件／目录或拟接入位置 | 状态 |
| --- | --- | --- | --- | --- |
| M1 | `docs/library.md` · 书架、导入与书库 | EPUB 导入、正文和资源、书籍身份及目录；书架检索、分类、详情、打开与删除；现有导入与后续阅读引擎的边界 | `src/components/reader/LibraryHome.vue`、`src/reader/library.ts`、`src/App.vue`、`crates/readux-core/src/import/`、`library.rs`、`reader_markdown.rs` | 待编写 |
| M2 | `docs/reader.md` · 阅读器与替换设计 | 现有正文、胶囊、ruby、图片、导航、选择及进度；epub.js 或同类方案的接入目标；原书结构与分析文本坐标、定位锚点、选区、注音和高亮的对应；阅读引擎与分析说明组件的接口；替换步骤及关键验证 | `ReaderDocumentView.vue`、`ReaderImageBlock.vue`、`ReaderNavigationPanel.vue`、`ReaderAppearancePanel.vue`、`src/reader/document.ts`、`document.worker.ts`、`rows.ts`、`virtualization.ts`、`src/utils/markdownDocument.ts`、EPUB 导入；具体阅读器待选型 | 待重写与设计 |
| M3 | `docs/explanation_layout.md` · 气泡、侧栏与抽屉 | 三种视图共用的内容和激活对象；词典、活用及接续说明的组织；布局切换、窄屏、尺寸、滚动、关闭和触摸；正文命中及高亮协作；后续分析结果的接入要求 | `src/explanation/`、`src/composables/useExplanation*.ts`、`src/components/explanation/`、`ReaderExplanationBubble.vue`、阅读器与指南集成 | 待重写，核心资产 |
| M4 | `docs/analysis_graphs.md` · D3 分析图与交互控件 | 句中活用图和接续树；数据、布局、绘制、节点与连接器、折叠、当前路径、尺寸测量和动画；图节点与正文定位；词组和依存图的展示需求及可复用部分 | `src/reader/morphologyGraph.ts`、`morphologyBubble.ts`、`ReaderConnectionTree.vue`、`ReaderMorphologyGraph.vue`、`ReaderMorphologySummary.vue` | 待编写，核心资产 |
| M5 | `docs/dictionary.md` · 词典、查询目标与聚合 | 词汇范围、整体与内部查询、词形还原、表记和读音、词典矩阵、内容解析及关联导航；词典资源和缓存；静态查询；与领域解说的协作 | `crates/readux-core/src/dictionary/`、`src/components/dictionary/`、`src/explanation/dictionaryMatrix.ts`、`scripts/build_dictionary_bundle.py`、`dictionary_bundle.py`、`dictionary_schema.py` | 待重写 |
| M6 | `docs/grammar_guide.md` · 语法指南与内容制作 | 课程目录、原稿混排、例句、译文和注音；内容、分析及词典快照；制作、审阅、保存、加载和局部重生成；复用分析说明与图组件；测试课的组织 | `src/components/grammar/`、`src/grammar-guide/`、`scripts/import_grammar_guide_source.mjs`、`generate_grammar_guide_content.py`、`generate_grammar_guide_analysis.py`、`build_morphology_stage11_guide.py`、`build_grammar_guide_package.mjs`、`grammar_guide_assets.mjs` | 待编写 |
| M7 | `docs/storage.md` · 存储、资源与缓存 | 书库、阅读位置、收藏笔记、设置、provider 配置、规则词条、词典缓存、分析缓存、静态包和浏览器数据；所有权、位置、格式、版本与重建；阅读器定位及收藏数据的后续变化 | `library.rs`、`reader_state.rs`、`analysis_cache.rs`、`analysis_engine.rs`、`rule_store.rs`、`providers.rs`、`dictionary/bundle.rs`、`src/services/staticLibrary.ts`、指南数据 | 待编写 |
| M8 | `docs/backend.md` · 本机后端与宿主接口 | Tauri、Rust AnalysisService、CLI、开发 bridge 和静态适配；请求、资源定位、配置、进程与错误；领域分析模块调用位置 | `src-tauri/src/lib.rs`、`crates/readux-core/src/analysis.rs`、`providers.rs`、`src/services/`、`scripts/nlp_bridge.mjs`、`nlp_provider.py` | 待编写 |
| M9 | `docs/reader_engine.md` · 文档会话与分析调度 | 应用、书籍和文档生命周期；分析单元、范围请求、阶段进度、增量更新、取消、重试和缓存；查询有效期；阅读器替换后的范围请求与定位协作 | `src-tauri/src/reader_engine.rs`、`crates/readux-core/src/document_plan.rs`、`document_session.rs`、`analysis_engine.rs`、`analysis_progress.rs`、`src/composables/useDocumentSession.ts`、`src/reader/session.ts`、`documentOperation.ts` | 待重写 |
| M10 | `docs/collections.md` · 收藏管理与统计 | 当前摘录、出处、词形、读音和笔记；后续词汇、语法、句子等收藏对象的范围；跨书管理、检索、分组与标签、编辑、批量操作及去重；收藏数量、类型、来源及时间等统计；数据与分析目标、词典及导出的关联 | `crates/readux-core/src/reader_state.rs` 的 SavedSelection、`src/types/reader.ts`、`src/App.vue`、`ReaderDocumentView.vue`、`src/services/staticLibrary.ts`；管理页与统计待设计 | 待设计 |
| M11 | `docs/anki_export.md` · Anki 导出 | 从收藏及选择生成卡片；词汇、读音、上下文、释义、用法解说、来源和笔记的字段；卡片组织、批量导出、去重及更新；导出方式和桌面／浏览器差异 | `src/App.vue` 的 JSON 摘录导出、`SavedSelection`、词典查询与领域解说；`crates/readux-core/src/export/` 为待评估的旧代码，引用类型与接口需核实 | 待设计 |
| M12 | `docs/llm.md` · LLM 调用与阅读辅助 | 调用入口、配置和提供商接口；正文、词典候选和领域分析结果的上下文；请求、结果及 UI；义项辅助与其他阅读任务的范围；超时、取消及结果保存 | `crates/readux-core/src/llm/`、`src/types/llm.ts`、`src/services/dictionaryAssistantPort.ts`；现有代码未注册进 core，前端端口未接入气泡，实际调用待设计 | 待设计 |
| M13 | `docs/quality.md` · 验证与问题定位 | 按实际模块选择现有检查；规则、范围、身份、用法、解说和 UI 的代表样例；指南及书籍快照、持久数据和性能检查；问题定位与实验索引 | `crates/readux-nlp/tests/`、`crates/readux-core/tests/`、`src/**/*.test.mjs`、`scripts/validate_*.py`、`verify_*.mjs`、`test_reader_*.py`、`data/validation/` | 待重写 |
| M14 | `docs/standalone_distribution.md` · 桌面资源与发行 | UniDic、词典、Python 及模型的资源准备；Windows 便携 ZIP、macOS DMG、CI 和资源 Release；图标、版本、许可、目录和升级；交付检查 | `scripts/run_channel.ps1`、`prepare_installer_resources.py`、`fetch_release_resources.py`、`package_portable.py`、`check_release_resources.mjs`、`src-tauri/build.rs`、`src-tauri/tauri*.conf.json`、`.github/workflows/release.yml` | 待重写 |

M3 定义共用的说明视图，M4 定义图组件，L11 定义分析结果到内容和图示的语义映射。L7 至 L9 各自说明新增分析结果的显示目标。

M2 先记录现有正文和分析坐标，简述阅读器替换方向及集成关注点。epub.js 为候选，具体选择及接口留待后续设计。

M10、M11、M12 保留用户确认的方向，当前收藏、摘录和相关调用基础在存储、阅读器及总体文档中说明；新对象、统计、卡片和 LLM 任务的详细设计暂缓。

## 7. research 引用范围

本轮 research 工作限于三个总体文档中的简要说明和链接。`docs/research/` 原文保持不动，既有测量、GPU 与云端推理方案按原有环境、日期和状态保留。

| 原文 | 总体文档中的用途 |
| --- | --- |
| `docs/research/nlp_behavior.md` | 来源行为与实验基础 |
| `docs/research/ginza_cuda_measurement.md` | GiNZA CUDA 测量入口 |
| `docs/research/ginza_optimization_handoff.md` | GiNZA 管线优化研究入口 |
| `docs/research/ginza_gpu_parallel_design.md` | GPU 并行方案入口 |
| `docs/research/static_storage_measurement.md` | 静态资源空间测量入口 |
| `docs/research/web_product_strategy.md` | Web 及资源形态研究入口 |
| `docs/research/cloud_saas_roadmap.md` | 云端推理服务研究入口 |
| `docs/research/cloud_gpu_first_experiment.md` | 云端部署实验入口 |

其他已有实验工具按开发和模块说明中的实际用途定位。既有研究记录保留当前位置，新建 research 索引、实验文档、报告合并和迁移均不列入本次交付。

## 8. 索引与现有文档去向

| 编号 | 文件与名称 | 内容概要 | 对应内容 | 状态 |
| --- | --- | --- | --- | --- |
| I1 | 根目录 `README.md` · 项目介绍与快速入口 | 依据最新目标说明核心成果、当前入口和开发方式 | G1、G3、E1 至 E3 | 待重写 |
| I2 | `docs/README.md` · 文档索引 | 三个总体文档和当前模块的有效链接；后续逐项更新模块状态 | 所有实际交付文档 | 已完成当前索引 |
| I3 | `docs/TODO.md` · 当前方向与文档后续事项 | 按本轮已确认方向整理，详细开发任务留待后续设计 | 现有模块维护与用户已确定方向 | 待重写 |

本文跟踪文档交付。I3 按当前范围整理方向及后续事项；具体开发计划在方案确定后补充。稳定设计进入所属模块，原有阶段观察和测量按需要保留当前位置或由相关模块引用，research 内部保持原状。

| 原有文件（相对 docs） | 处理与目标 |
| --- | --- |
| `README.md` | 重写为 I2 |
| `TODO.md` | 依据当前目标重写为 I3 |
| `development.md` | 重写为 G3；入口和发行细节进入 E1、E2、M14 |
| `static_library.md` | 重写为 E2 |
| `reader.md` | 书架和导入进入 M1；阅读功能与替换设计进入 M2；存储进入 M7 |
| `reader_engine.md` | 会话调度重写为 M9；活用和查询投影分别进入 L11、M5 |
| `dictionary.md` | 按现有查询实现和当前职责重写为 M5 |
| `explanation_layout.md` | 重写为 M3；图组件进入 M4，分析投影进入 L11 |
| `grammar_guide_display.md` | 合并至 M6，核实制作、审阅与页面操作 |
| `grammar_guide_1_1.md` | 有效规格合并至 M6；按当前课程身份说明内容，原文件对应 2.1《认识实词、虚词、助词》 |
| `language_analysis.md` | L0、L1 已完成；原文加历史说明，L7 至 L11 依后续范围继续整理 |
| `nlp_sources.md` | 当前输入、来源与坐标已整合至 L2、L3，原文保留历史说明 |
| `morphology_state_machine_design.md` | 当前状态机设计已整合至 L4，原草案保留历史说明 |
| `morphology_configuration.md` | 当前维护契约已整合至 L5，专项规则记录供后续细化 |
| `morphology_storage_refactor_analysis.md` | 当前协议与编译已整合至 L5，原文标明历史数量及验证记录 |
| `morphology_stage11_bubble_display.md` | 当前内容、视图和图示分别进入 L11、M3、M4；已有样例按模块引用 |
| `morphology_predicate_relations_review.md` | 稳定结论进入 L4、L11；已有审阅记录保留引用 |
| `morphology_operator_inventory.md` | 规则组织进入 L5；覆盖记录由 L5 和 M13 引用 |
| `morphology_refactor_execution_plan.md` | 核实已完成记录和有效维护事项，分别归入模块说明及 I3 |
| `lookup_issue_tracking.md` | 保留当前位置，按现用查询路径核实状态，与 M5、M13 相互引用 |
| `user_state.md` | 当前收藏、笔记及导出整理入 M2、M7；后续方向在总体文档简述，M10 至 M12 详细设计暂缓 |
| `quality.md` | 现有开发验证整理为 M13，保留有效工具及实验记录引用 |
| `standalone_distribution.md` | 发行说明重写为 M14；已有运行资源及性能结果按实际用途引用 |
| `research/nlp_behavior.md` | 原文保持不动，仅总体文档简述引用 |
| `research/ginza_cuda_measurement.md` | 原文保持不动，仅总体文档简述引用 |
| `research/ginza_optimization_handoff.md` | 原文保持不动，仅总体文档简述引用 |
| `research/ginza_gpu_parallel_design.md` | 原文保持不动，仅总体文档简述引用 |
| `research/static_storage_measurement.md` | 原文保持不动，仅总体文档简述引用 |
| `research/web_product_strategy.md` | 原文保持不动，仅总体文档简述引用 |
| `research/cloud_saas_roadmap.md` | 原文保持不动，仅总体文档简述引用 |
| `research/cloud_gpu_first_experiment.md` | 原文保持不动，仅总体文档简述引用 |

覆盖以上 31 篇原有文档。实际合并时先完成目标内容，再处理原文件及引用；旧文档状态说明随相应正文一起更新。

`configs/unidic-nlp/morphology/source/README.md` 保留就近维护索引，引用 L5、L6。根目录 `sources.md` 继续维护外部资料来源及标题索引。`docs/summarize.py` 作为现存辅助工具登记用途。文档中的新文件路径在实际创建前采用代码格式。

## 9. 编写顺序、完成条件与追踪

| 阶段 | 范围 | 完成条件 | 状态 |
| --- | --- | --- | --- |
| 0. 审阅本计划 | 核心资产、目标、目录、旧资料去向 | 用户认可目录，并确定现有文档优先、后续设计暂缓、research 原文保持现状的范围 | 已完成 |
| 1. 总体文档 | G1、G2、G3、I2 | 当前成果、方向、结构和入口明确，交接要求及有效索引可查 | 已完成 |
| 2. 核心成果 | L0 至 L6、L11 的当前实现，M3、M4 | 活用状态机、规则词条、GiNZA 证据、内容投影、三种视图和 D3 图的调用及职责完整；grammar 的有效资源与过时设计区分准确 | L0–L6 已完成；L11、M3、M4 待整理；逐规则细节另行深化 |
| 3. 后续语言分析设计 | L7 至 L10、L11 的后续部分 | 现阶段保留方向与问题，具体方案等待讨论 | 暂缓 |
| 4. 现有应用入口与模块 | E1 至 E3、M1、M2 当前实现、M5 至 M9、M13、M14 | 入口可查；存储、后端、阅读器及调度与代码一致；M10 至 M12 的详细设计暂缓 | 交接待整理 |
| 5. 最终整理 | I1、I3、模块交付后的 I2 维护及旧文档引用 | README、方向和模块说明一致；旧文档内容与链接整理完成；research 文件无变更 | 交接待整理 |

文档按依赖逐篇完成，同一规则或协议在所属文档中定义，其他文档引用。单篇交付需明确用途、当前状态、主要对象、输入输出、协作关系、代码入口和后续工作。当前功能说明依据实际调用，新模块设计依据用户确定的方向和可用基础。

验证以现有工具及代表样例为主，按修改范围选择。纯文档计划检查名称、路径、内容范围和资料去向；运行说明核实对应入口；详细设计采用能解释匹配、范围、用法与显示结果的样例。提交前统一检查 diff，独立主题分开提交。

- [x] 完成保留文档与当前入口概况排查。
- [x] 按 2026-11-09 用户说明重设目标、核心资产和语言分析目录。
- [x] 核查 configs/grammar、活用解说资源及原 grammar 流程的实际调用。
- [x] 用户审阅修订后的目录及范围，并确定文档整理及交接要求。
- [x] 完成三个总体文档与当前总索引。
- [x] 完成 L0–L6，更新正式入口、旧资料状态和配置源目录索引。
- [ ] 下一位 agent 按 L5、L6 交接清单深化逐条语法规则、完整 JSON 和复杂样例。
- [ ] 接替 agent 完成现有核心成果文档的重写和整合。
- [ ] 接替 agent 完成现有入口和应用模块文档。
- [ ] 完成索引、开发任务及旧文档整理。

| 日期 | 范围 | 结果 | 下一步 |
| --- | --- | --- | --- |
| 2026-11-08 | 入口与保留文档 | 完成概况排查和首份追踪计划 | 根据用户说明修订目标 |
| 2026-11-09 | 当前修订 | 以活用核心和三种视图重设目录；保留三个后续模块方向；核实 grammar 资源及旧流程 | 用户已审阅并进一步限定文档工作范围 |
| 2026-11-09 | G1、G2、G3、I2 | 完成目标、当前架构、开发说明及总索引；写明现有文档优先、后续设计暂缓和 research 原文保持现状的交接要求 | 接替 agent 按清单整理现有模块，每篇完成后更新状态 |
| 2026-11-09 | L0–L6 及索引 | 核实正文与词法、完整来源和 UD、状态机、配置及解说读取；创建七篇文档，结合运行截图说明用户体验，更新总体文档及旧资料状态 | 用户指定的逐规则完整 JSON 与复杂分支留给下一位 agent；L11、M3、M4 继续整理 |
