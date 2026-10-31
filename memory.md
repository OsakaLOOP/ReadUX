# 开发入口记忆

- 活用配置已采用 `readux.morphology-config.v1`。`rule_metadata` 统一保存输出状态，`input_label` 与 `input_category_mode`／`input_categories` 分离说明与条件；`intrinsic_features` 保存三个原子特征。前端通过 `morphologyPrototype` 读取 Rust 的 `normalized_form`，接续规则和分类在加载时建立索引。文言路径使用 `ctype_mode: classical_prefix`；展示前缀表为有序数组。最终协议与后续分文件编译方案见 `docs/morphology_storage_refactor_analysis.md`，当前运行入口仍为 `configs/unidic-nlp/morphology.json`，编译器尚待实施。

- 静态书库生成入口为 scripts/generate_static_library.py，输出位于 public/library。浏览器宿主入口为 src/services/staticLibrary.ts 和 src/services/staticAnalysis.ts，分析结果按六个桌面单元分片；开发预览使用 npm run static:dev。

- EdgeOne Pages 静态部署使用 `npm run build` 和 `dist`，浏览器默认打开静态书架，资源结构与部署说明见 `docs/static_library.md`。指南分析与词典的生产压缩入口为 `scripts/grammar_guide_assets.mjs`，解压入口为 `src/grammar-guide/loadJson.ts`；开发模式读取 JSON，正式构建读取 `.json.gz`。指南三课合计 812 条例句，平台单文件限制为 25 MB。

- 解释布局状态入口为 `src/explanation/morphologyLayout.ts`，抽屉高度与交互入口为 `src/composables/useExplanationSheet.ts`。工具栏按钮默认选择固定布局，960 px 以内使用 sheet，其余使用独立右侧栏，列宽动画期间正文行宽固定。sheet 左右留 12 px、底边贴屏，把手高 22 px，内容统一滚动；换词保持展开，收纳后换词自动恢复词典位置。指南使用激活时创建、延迟挂载的 Teleport；退出不使用词条高度或三角死区，仅 sheet 把手上方保留 32 px。阅读器拖选超过 5 px 后才关闭查询并捕获指针。说明见 `docs/explanation_layout.md`。

- 语法指南“活用与接续 · 气泡测试”收录 680 条去重输入，含用户提供的 15 条复杂句；静态测试清单位于 `data/validation/morphology_static_cases.json`，历史清单为 `docs/morphology_example_status.md`。内容生成入口为 `scripts/build_morphology_stage11_guide.py`，快照生成使用 `scripts/generate_grammar_guide_analysis.py --lesson stage11`，可用 `--group complex` 单独更新复杂句。Vite 只忽略 `unidic-cwj-*` 与 `unidic-csj-*` 词典目录；`configs/unidic-nlp` 必须参与监听，否则接续配置更新后会造成气泡渲染异常和课节切换中断。

- 全量接续词条的结构分类入口为 morphology.json 的 connection_types，覆盖全部 150 个 operator；完整性检查为 scripts/check_morphology_connections.py，只检查分类、路径、连接器、概念和例句。connector_alternatives 关联「になる」的两种に；dependency_type 与 context_range/context_surface 分别保存结构和连接器上下文。「まい」「だろう／でしょう」「が／から」及准体名词化の的独立讲解位于 grammar source bundle morphology_connections_20261028.json，variants.concept 区分の与さ。

- 「なの／なん」采用与て形相同的连接器关系。名词化路径 no_noun_na 与 no_na_attributive 用 connector 指向 attributive_na，用 connector_paths 引用「な」的 noun_nominalization 或 na 路径；图示必须有独立「な」节点，显示「名词／ナ形容词→な→の」。前项范围读取连接器输入，例如「はず」，connector_operator_id 定位「な」。「な」的 noun_nominalization 用 following 确认「の／ん」，一般分支用 exclude_following 与之区分。配置说明见 docs/morphology_configuration.md。

- 指南的路径高亮与前项范围读取静态分析快照；配置热更新会更新图示文案。规则修改后须重新编译分析服务并生成相关快照。开发 bridge 占用 target/debug/readux-nlp.exe 时，可用 cargo rustc -p readux-core --bin readux-nlp -- -o target/debug/readux-nlp-guide.exe 生成独立指南服务，再以生成器的 --service 参数指定。命令说明见 docs/development.md。

- 谓语关系审阅记录位于 `docs/morphology_predicate_relations_review.md`。判断词尾保留前项类别，名词谓语通过判断实体身份共享接续表；前端箭头按接续功能命名，既有静态指南由链内前置成分恢复判断对象身份。D3 父节点按直属子节点顶部至末个分类标题底部居中，末类列表不参与居中计算。

- 接续图中的复合词条合并由 `presentation.sentence_compounds` 配置，现用于「かもしれない」；整体节点采用完整成员范围，活用读取末尾「ない」，后接补助层独立排列。

- 活用气泡的内容投影位于 `src/reader/morphologyBubble.ts`，D3 布局位于 `src/reader/morphologyGraph.ts`，绘制组件为 `ReaderMorphologyGraph.vue`。句中节点内部固定两行，中文标签与日文词形分字体；每行由左向右连接。补助层从连接器之前折行，纯接续形连续排列；认识单元直接承接使役被动、极性、礼貌、时态或补助层时折行一次，内部连续排列，展示分组为 `presentation.sentence_layers`。前接图保留全部词性节点，细分列表采用单项展开，当前路径固定显示；分类节点统一定宽，展开图标和计数在左，标题居中。列表使用从词性节点向下伸出的悬挂引导线，当前连接显示高亮箭头。39 条接续分支验收样例位于 `data/validation/morphology_stage11_cases.json`，同时用于指南内容与静态快照的生成校验。

- 接续图混合直接路径与连接器路径时，直接分支使用隐藏布局节点预留连接器列，全部词类位于同一列并按完整列表高度分配间距。可见连线跳过隐藏节点，父节点按实际可见子节点范围居中，避免「の」与「な→の」混排时重叠。

- 句中节点宽度按主行与副行的较宽内容加 24 px 计算，文字居中；70 px 下限已取消，短内容不再产生多余留白。

- 阅读器与指南的气泡定位共用 src/explanation/geometry.ts 的 placeMorphologyPanels。以当前悬浮词条为基准，反向可用高度大于正向可用高度减去活用气泡独立显示高度时，接续气泡放在反向；否则保留同向外侧堆叠。上下词条间距均为 10 px，页面边距均为 12 px。

- 接续面板的两栏高度由 `ReaderConnectionTree.vue` 分配，优先完整显示较短栏，滚动栏至少占可用图示高度的三分之一；外层面板固定。`data-natural-height` 声明完整内容高度供 `measureIntrinsicPanel` 定位，`ReaderMorphologyGraph.vue` 负责两轴居中及当前规则条目的纵向定位。同一活用链以首个句中节点 ID 保留组件实例，支持连续的聚焦颜色过渡。

- 词干气泡入口由 `lookupPresentation.ts` 的 `readerMorphologyHits` 生成。同形词干、名词及形容动词等沿用 `morphologyStemDiagram` 定位范围；名词、代名词和副词以 `entryVisible: false` 仅显示接续气泡，节点标注“无活用”。句中关系不足两个元素时，各词性均隐藏接续气泡，独立的无活用词保留词典查询。`placeMorphologyPanels` 接收空词条尺寸时按单个接续面板定位。词干与后续成分共享完整图示，旧连接线在新布局首帧移除。

- 活用链第 11 阶段的气泡接入规范见 docs/morphology_stage11_bubble_display.md：现有活用词条气泡保持原职责，新增接续方式气泡并与其上下堆叠；补助父子关系递归显示，当前词条前接规则按唯一线性路径或多分支投影。て／で共用连接规则，普通规则只回看声明所需的前项；条目类别、用法说明、例句与词形标签分层呈现。后续按文档清单处理「ない」「ずに」「た／だ」及判断收尾等多路径规则。

- 用户提供的七句谓语接续验收见 `docs/morphology_predicate_examples_2026-10-27.md`，测试入口为 `p4_language.rs` 的 `supplied_predicates_*` 与 `attitude_and_humble_support_*`。规则覆盖「しかない」继续否定、意向形＋とする、こと名词化和ていい；「てかかる」按态度补助处理，原始动词词性保留。「ていただく」采用通用授受样例验证。补助概念讲解位于 grammar source 的 `morphology_support_20261027.json`，许可讲解位于 `morphology_permission_20261027.json`。

- 活用链的局部接续设计及 7 条长句验收见 `docs/morphology_local_connections_2026-10-27.md`。执行入口为 `morphology_machine.rs`；阶段数字仅作展示，合法性读取当前词类、活用形、精确活用型及最近连接器。全部 `support` 统一交补助执行器；「たりとか」保持原连接许可，名词性＋になる保留前项身份。真实分词回归为 `p4_language.rs` 的 `recursive_predicates_*` 与 `conditional_toka_*`，单体／分词とか及非法重复活用见 `morphology_reconstruction.rs`。

- 活用链重构的例句进度入口为 `docs/morphology_example_status.md`，按当前版本真实 CWJ UniDic 复测整理 199 条去重输入，分为已通过、边界通过、局部通过、仍有问题和待复核；来源包括三轮合成句、阶段 9／10 记录、专项测试和修复记录。合成分词通过与真实分词命中范围分别记录。

- 静态资源空间测量入口为 `scripts/measure_static_storage.py` 与 `scripts/measure_static_storage.mjs`，报告为 `docs/static_storage_measurement.md`，数据为 `data/validation/static-storage-summary.json`。结合用户策略文档的路线建议见 `docs/web_product_strategy.md`，属于讨论稿。正式书库位于 Windows 文档目录的 `ReadUX Library/library.sqlite`，正文为 `books/<id>/content.md`；桌面分析缓存为 `%APPDATA%/com.readux.app/analysis-cache/*.msgpack`，根对象包含 `document` 和 `lookup`。仓库缓存同时包含旧协议，统计须按正文去重。当前指南 `lesson-1-1.json` 为《句法入门》，`lesson-2-1.json` 为《认识实词、虚词、助词》；历史规格文件名仍为 `grammar_guide_1_1.md`。

- 活用变化范围由 `src/reader/lookupPresentation.ts` 根据前后词形与前序成分起点统一投影，词干及其后首个活用显示完整变化，后续变化保留对应成分，纯追加保留上一成分。独立连接器使用 `connector_operator_id` 定位，后项原型仅取自身成员，覆盖「こと」「とする」「たりとかする」；实际「い」对应输入原型「いる」时继续显示完整成分。语法指南真实快照回归位于 `morphologyGraph.test.mjs`。敬体否定整体识别为 `polite_negative`，内置词条位于 `configs/grammar/auxiliaries.json`，成分查询路由位于 `src/grammar/particleEntries.ts`。

- 活用链重构设计与六类实体、专用执行器、规则树和展示契约记录在 `docs/morphology_state_machine_design.md`。旧 `configs/grammar/teConstructions.json` 已删除；19 个补助动词本体、18 个 `te_*` 规则和 3 个缩约 operator 的历史 ID 位于 `configs/grammar/migrations/te_construction_ids.json`，前端规则展示读取迁移元数据，正式查询归属使用补助本体 concept。

- 插入语执行器通过 `morphology.json` 的 `insertion` 与 `insertion_positions` 统一限定成员和位置，直接在所属链内增加 operator 并保持状态与查询身份。阶段 7 验收入口为 `crates/readux-nlp/tests/morphology_reconstruction.rs` 的 `stage_seven_*` 用例；补助词仍连接原链，递归深度只统计补助构式。

- 语法指南设计位于 `docs/grammar_guide_display.md`，1.1 原稿、19 条原稿例句与 1.1-01 至 1.1-70 共 70 条补充例句规格位于 `docs/grammar_guide_1_1.md`。原稿为 `D:\DCMT\dev notes\ReadUX 日文语法指南.md`，用户明确要求逐字保留原稿。第一阶段只确定原稿混排和例句清单；固定讲解文字不进入例句卡片，语法说明来自生成并审阅的标签。后续静态 JSON 保存词典正文、活用说明、注音、分类和目标，前台使用指定版本。正式阅读器胶囊入口为 `ReaderDocumentView.vue` 的 `reader-capsule` 与 ruby runs，生成阶段可复用 `src/reader/lookupPresentation.ts`；`BunsetsuCapsule.vue` 使用 `src/types/index.ts` 的 `AnnotatedToken`，相关数据结构见 `src/types/nlp.ts`。

- header 动画入口为 `src/components/common/ReadingMark.vue`，统一 SVG 位于 `src/branding/readux-animated.svg`，生成脚本为 `scripts/generate_header_mark.py`，时序控制为 `src/reader/markPlayback.ts`。`npm run frontend:dev` 使用独立占位书架预览，入口为 `src/dev/HeaderAnimationPreview.vue`，支持打开、关闭及播放中反向操作；该模式独立于后端书库与 NLP 服务。

- 封面代表色由 `src/reader/coverColor.ts` 按色度、明度和透明度连续加权计算，灰度封面保留自身颜色；缓存位于 `readux.cover-representative-colors-v2`。`scripts/verify_reader_return.mjs` 在正式应用入口验证书架即时返回、加载页移除和 header 封面倒放。

- 书架打开动画的徽标入口为 `src/components/reader/ReaderOpeningMark.vue`，书架与阅读加载页共用。Windows EXE 图标由 `src-tauri/build.rs` 嵌入，`icons` 目录须显式声明为 Cargo 构建依赖；仅生成 ICO 或构建前端不会更新 EXE。

- 品牌矢量源为 `public/branding/readux-wordmark.svg`，其中 `readux-mark` 组定义独立 UX 图标。`python -X utf8 scripts/generate_branding.py` 提取 logo，并通过 Tauri CLI 同步网页 PNG 与 `src-tauri/icons/`。header 入口为 `src/components/common/AppHeader.vue`；假名使用 Noto Sans JP Bold 的矢量轮廓。

- 活用链的规则、概念路由、原型与展示统一在 `configs/unidic-nlp/morphology.json`；Rust 读取入口为 `morphology_config.rs`，前端适配为 `src/reader/morphologyConfig.ts`，维护指导见 `docs/morphology_configuration.md`。配置内容参与完整分析缓存标识。

- GiNZA CUDA 报告位于 `docs/ginza_cuda_measurement.md`；测量脚本为 `scripts/benchmark_ginza_cuda.py` 与 `scripts/compare_ginza_precision.py`。机器摘要及二十处人工复核位于 `data/validation/ginza_cuda_*.json`；本机完整核心记录归档为 `.agents/analysis/ginza-cuda-core-records.zip`，原始 `cuda-clean` 等重复输出可清理。正式微调权重为 `megagonlabs/transformers-ud-japanese-bert-large-ginza`。`experiments/ginza-cuda` 为指向 C 盘临时环境的 junction。
- BERT 优化交接位于 `docs/ginza_optimization_handoff.md`；共享 GPU BERT、CPU 进程池、Cython 状态解码、双批次重叠和 GPU 多流实验入口分别为 `scripts/ginza_bert_engine.py`、`scripts/ginza_decode.pyx`、`scripts/benchmark_ginza_pipeline_suite.py`、`scripts/benchmark_ginza_gpu_concurrency.py`。固定 9068 token 语料的重叠 suite 为 2188 token/s，JSON 编码后 2019 token/s，输出与 FP16 基准一致；4060 Ti 设计和 GPU 50% CPU 比例仍交下一 agent。
- GPU 并行设计位于 `docs/ginza_gpu_parallel_design.md`；生产路径固定 spaCy 原版状态解码，使用独立 non-blocking D2H 流、两个 pinned/shared buffer 槽、长度分桶、跨请求轮转和八个 CPU worker。3050 trace 的 GPU kernel active 1.19 s、完整 wall 5.44 s，4060 Ti 目标按 GPU 2.2×、CPU 1.5× 建模。

- 阅读分析的异常恢复验证入口为 `scripts/test_reader_recovery.py`，使用独立数据目录验证解释器缺失、单元重试、进程重建和损坏缓存恢复。会话控制前端位于 `src/composables/useDocumentSession.ts`；`src/reader/session.ts` 的 `documentControlRequest` 按后端协议区分增量与完整快照命令。

- 胶囊的词汇范围与作者注音查询由 `crates/readux-core/src/dictionary/targets.rs` 生成；`src/reader/lookupPresentation.ts` 提供活用投影，`ReaderDocumentView.vue` 组合两者。问题样本的实测入口为 `scripts/inspect_lookup_cases.py`，通过本机词典索引和 CLI 查询保存证据。
- 连续词汇的词典别名由 `dictionary/lookup.rs` 的 `lookup_exact_form_profiled_with_pos` 解析；`lookup_state.rs` 的 `entry_matches_form` 接受带原查询形及词典范围的显式别名证据。「めいっぱい」的整体词条为「目一杯」，UniDic 可将其切为接尾辞「め」与副词「いっぱい」。
- 可能动词的词元选择由 `morphology.rs` 的 `potential_lemma` 判定，条件为 UniDic 下一段活用与词元到基本表记的五段可能形式对应关系。`scripts/test_reader_morphology.py` 通过正式服务验证查询首选词条及普通动词对照。

- 正文窗口划分入口位于 `crates/readux-core/src/document_plan.rs`；P4 小说性能采集入口位于 `scripts/benchmark_p4_novel.py`，十段测量结果默认保存在 `.agents/analysis/p4-novel-10.json`。
- 当前本机结构来源由 `crates/readux-core/src/providers.rs` 与 `scripts/nlp_provider.py` 调度 GiNZA；KWJA 的离线适配器位于 `scripts/kwja_adapter.py`，不进入桌面来源配置。
- P4 活用链与整体构词的实现契约位于 `docs/p4_morphology_formation.md`。
- `morphology.rs` 定义链、状态和 occurrence；`morphology_machine.rs` 执行连接判定；`linguistic_context.rs` 读取 GiNZA 正式词界、活用字段及关系。完整来源通过 `unify_with_sources` 接入。
- 整体词保存在 `FormationNode.word`，由 `formation.rs` 生成，查询候选由 `lexical.rs` 提供。GiNZA 适配器的显式 compound 通常对应单个正式 token；跨正式 token 的构词需要读取 `compound` dependency。
- P01–P10 的人工阅读入口是 `data/validation/p01-p4-integration.md` 至 `p10-p4-integration.md`。大型 `p4-sample-review.json` 由采集和验证脚本处理，开发时直接阅读 Markdown。
- 全量采集使用 `python -X utf8 scripts/review_p4_samples.py`；现有报告导出使用 `python -X utf8 scripts/export_p4_sample_report.py --saved`。原始响应保存在 `experiments/p4-sample-review/`。
- 旧字模块的正式名称为 `kyujitai.js`，Rust 入口是 `kyujitai.rs`；审计中曾用 `kyukanji.js` 指代该能力。
- 缩约规范形式用于查询和展示：清浊变体统一为 `ている／てしまう／ておく`，补助动词查询形为 `いる／しまう／おく`。实际缩略、浊化和活用保存在原始成员与连接信息中。
- P4 的必须验收范围从 token 到语法活用链和整体构词。正式 token 支持整体构词；依存、文节及其他句法结果主要作为实验性辅助证据。
- P4 人工报告采用“原始组成、构成形态、最终对象”三列，每条链竖向独占一行；补助用言用前接词形说明归属。内部 ID、完整状态和来源引用保存在机器报告。
- P4 完成范围的最新核对为 `docs/p4_handoff.md`。P4 提供 token、活用链和构词候选，词典验证、查询聚合与决定归下一模块；当前修复与长期方案严格分开。本轮职责整理只修改文档。
- 多粒度查询协议在 `docs/dictionary.md`，验收在 `docs/quality.md` 和 A28–A33。P5A–P5C 负责词典查询、绑定、决定和交互；P5D 独立实施语法与表达，结合词典、形态和结构证据，规则仅覆盖适合声明条件的部分。P7 持久化词汇状态、收藏及阅读数据。
- P5 查词处理草案位于 `docs/lookup_processing_draft.md`：活用功能范围先排除，剩余词汇范围按最大范围无重叠贪心覆盖，再归属内部子对象；词典元数据由后续解析器提供，P5 仅接收并用于筛选和排序。
- 阅读器 Engine 生命周期、UniDic／GiNZA 分析进度和词典气泡投影协议位于 `docs/reader_engine.md`；外层词汇范围由词典确认的整体词或未覆盖的独立词构成，悬浮时并排查询外层词条与当前位置的最小可查询组分。
- 阅读器应用入口为 `src/App.vue`，正文投影和虚拟行位于 `src/components/reader/ReaderDocumentView.vue`，宿主调用适配位于 `src/services/reader.ts`；新用户状态文件为 `reader-state.sqlite`，选择与笔记类型位于 `src/types/reader.ts`。
- 独立桌面发行版的资源、打包和优化追踪位于 `docs/standalone_distribution.md`；现代语 UniDic 与 GiNZA 运行时的第一轮审计入口为 `scripts/audit_ginza_runtime.py`。
- 便携发行入口为 `npm run insider` 与 `.github/workflows/release.yml`，输出 Windows 便携 ZIP 和 macOS DMG。本地 `scripts/prepare_installer_resources.py`、CI 的 `scripts/fetch_release_resources.py` 准备 `release-resources`，Windows venv 须转换为独立解释器。`target/release/ReadUX.exe` 与便携目录共用同一 release 构建；窗口图标入口为 `src-tauri/src/windows_icons.rs`，阅读状态同步更新 `ICON_BIG` 与 `ICON_SMALL`，Windows 任务栏可能选用任一尺寸。
