# 开发入口记忆

- 静态资源空间测量入口为 `scripts/measure_static_storage.py` 与 `scripts/measure_static_storage.mjs`，报告为 `docs/static_storage_measurement.md`，数据为 `data/validation/static-storage-summary.json`。结合用户策略文档的路线建议见 `docs/web_product_strategy.md`，属于讨论稿。正式书库位于 Windows 文档目录的 `ReadUX Library/library.sqlite`，正文为 `books/<id>/content.md`；桌面分析缓存为 `%APPDATA%/com.readux.app/analysis-cache/*.msgpack`，根对象包含 `document` 和 `lookup`。仓库缓存同时包含旧协议，统计须按正文去重。当前指南 `lesson-1-1.json` 为《句法入门》，`lesson-2-1.json` 为《认识实词、虚词、助词》；历史规格文件名仍为 `grammar_guide_1_1.md`。

- 活用变化的局部提示由 `configs/unidic-nlp/morphology.json` 的 `presentation.transition_tails` 按语法身份、前接形及末尾假名配置。敬体否定整体识别为 `polite_negative`，内置词条位于 `configs/grammar/auxiliaries.json`，成分查询路由位于 `src/grammar/particleEntries.ts`。

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
