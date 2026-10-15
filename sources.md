# 来源与文档索引

当前设计依据本次确认的交付目标，以及工作区的模块注册、调用关系、类型和资源配置。外部链接沿用已登记来源，供依赖与格式核对。历史资料保存在 [docs_backup](docs_backup/README.md)。

## 决策与实现依据

| 依据 | 对应内容 |
| --- | --- |
| [活用链配置](configs/unidic-nlp/morphology.json)与[活用验收](crates/readux-core/tests/p4_language.rs) | 2026-10-15 使用本机 UniDic 核对「ゆく」「たら／だら」「の／ん」「に／な／だ」的分词、活用字段与链归属；维护指导见 [活用链配置](docs/morphology_configuration.md)。 |
| [词典问题样本采集](scripts/inspect_lookup_cases.py)与[本机词典缓存](data/dicts) | 2026-09-28 核对「橋場／羽柴」「軒／ケン」「戸外／ソト」「内外／ウチ」的索引、词头和服务输出；记录保存在 `.agents/analysis/lookup-cases-before.json` 与 `lookup-cases-after.json`。 |
| [七日の喰い神研究文本](file:///D:/Downloads/epub-exp/source/七日の喰い神%20%28ガガガ文庫%29%20%28カミツキレイニー%29/output.md) | 本机小说语料；P4 性能测量选取第一话开头约一万字符，语料摘要和划分范围保存在测量结果中。 |
| 2026-09-21 用户确认 | 完成模块接入、恢复完整桌面功能；允许使用本机 Python 环境、外部模型和词典 |
| 2026-09-22 用户确认 | P4 必须保证 token、语法活用链及整体构词；依存等主要作为实验。词典验证、查询决定与聚合归下一模块；当前修复和长期方案严格分离。 |
| [P4 完成范围核对](docs/p4_handoff.md) | 以 `522a801` 及历史 `df5a05f` 的 P4 清单核对原任务归属；读取十段产物并实际查询成立した、てる、読みませんでした。本机响应为 `experiments/p4-handoff-query.json`。 |
| [应用服务](crates/readux-core/src/analysis.rs)与[模块入口](crates/readux-core/src/lib.rs) | 实际请求类型、分析流程、注册模块和状态管理 |
| [统一对象](crates/readux-nlp/src/model.rs)、[来源对齐](crates/readux-nlp/src/alignment.rs)与[结构协议](crates/readux-nlp/src/syntax.rs) | 字段、坐标、实体、对齐与来源结果 |
| [词典查询](crates/readux-core/src/dictionary/lookup.rs)与[应用输出](crates/readux-core/src/output.rs) | 查询矩阵及当前接入差距 |
| [桌面宿主](src-tauri/src/lib.rs)、[前端入口](src/App.vue)与[构建配置](src-tauri/tauri.conf.json) | 实际桌面命令、界面与资源 |
| [重构对象设计备份](docs_backup/unidic_rewrite_acceptance.md) | 统一对象、应用分层、会话与查询的既定契约 |
| [阅读模块备份](docs_backup/reader_library_and_scroll_reader.md) | 书架、导入、章节、进度和排版的功能参照 |
| [词典协议备份](docs_backup/dictionary_lookup_and_bubble_refactor.md)与[解释交互备份](docs_backup/explanation_targets_and_dictionary_ui.md) | 表记矩阵、正文模型及解释会话的功能参照 |
| [语法模块备份](docs_backup/grammar_morphology_and_functional_pipeline.md) | 活用所有权、知识目录和精确讲解的功能参照 |

## 外部来源

2026-09-21 本机行为核对使用已安装源码及实际推理，证据见 [NLP 行为基线](docs/nlp_behavior.md)。源码对应入口为 [KWJA CLI](https://github.com/ku-nlp/kwja/blob/v2.1.3/src/kwja/cli/cli.py)（规范化、常驻加载、tiny 断句）、[KWJA 任务常量](https://github.com/ku-nlp/kwja/blob/v2.1.3/src/kwja/utils/constants.py)、[GiNZA 文节实现](https://github.com/megagonlabs/ginza/blob/v5.2.1/ginza/bunsetu_recognizer.py)、[rhoknp 基本句](https://github.com/ku-nlp/rhoknp/blob/v1.3.2/src/rhoknp/units/base_phrase.py)、[rhoknp 关系标签](https://github.com/ku-nlp/rhoknp/blob/v1.3.2/src/rhoknp/cohesion/rel.py)。实际环境版本以行为证据 JSON 为准。

| 来源 | 链接 | 用途 |
| --- | --- | --- |
| UniDic | <https://clrd.ninjal.ac.jp/unidic/> | CWJ／CSJ 资源、字段和版本入口 |
| UniDic 2025.12 archive | <https://clrd.ninjal.ac.jp/unidic_archive/2512/> | Actions 下载 `unidic-cwj-202512_full.zip`、`unidic-csj-202512_full.zip` 并按清单校验 SHA-256 |
| Vibrato | <https://github.com/daac-tools/vibrato> | Rust 形态分析引擎；仓库实现位于 `vendor/vibrato` |
| GiNZA | <https://github.com/megagonlabs/ginza> | spaCy／Sudachi 分析流程、任务和部署依赖 |
| GiNZA v5.3 官网 | <https://megagonlabs.github.io/ginza/> | v5.3.0 兼容性、`ja_ginza_bert_large`、GPU 与精度说明 |
| GiNZA v5.3 release | <https://github.com/megagonlabs/ginza/releases/tag/v5.3.0> | 发布日期、wheel 体积、依赖升级和 SHA-256 |
| ja-ginza-bert-large PyPI | <https://pypi.org/project/ja-ginza-bert-large/> | 模型包版本与运行时依赖 |
| BERT large Japanese v2 | <https://huggingface.co/tohoku-nlp/bert-large-japanese-v2> | 24 层 BERT large 配置、权重体积与许可证 |
| spaCy pipeline | <https://spacy.io/usage/processing-pipelines> | GPU、批处理与管线执行参考 |
| GiNZA PyPI | <https://pypi.org/project/ginza/5.2.1/>；<https://pypi.org/project/ja-ginza/5.2.0/> | Actions 安装固定的 GiNZA 与 `ja-ginza` 版本 |
| Python standalone | <https://github.com/astral-sh/python-build-standalone/releases> | Actions 下载 Windows x64 与 macOS arm64 的 Python 3.11 运行时 |
| GiNZA 文节 API | <https://megagonlabs.github.io/ginza/bunsetu_api.html> | 文节、主辞和小句结果的适配依据 |
| GiNZA 模型 | <https://github.com/megagonlabs/ginza/releases> | 模型包和版本入口 |
| KWJA | <https://github.com/ku-nlp/kwja> | 现有模型任务、执行流程和资源入口 |
| KWJA 论文 | <https://aclanthology.org/2023.acl-demo.52/> | 字符、词与关系任务的背景资料 |
| spaCy 日文 tokenizer | <https://github.com/explosion/spaCy/blob/v3.8.16/spacy/lang/ja/__init__.py> | 依据本机同版源码核对 Sudachi 实例、切分模式与模型加载入口 |
| SudachiPy Dictionary | <https://github.com/WorksApplications/sudachi.rs/blob/v0.6.11/python/py_src/sudachipy/sudachipy.pyi> | 本机接口声明确认独立 system.dic 参数和默认资源配置 |
| KWJA 词输出组件 | <https://github.com/ku-nlp/kwja/blob/v2.1.3/src/kwja/callbacks/word_module_writer.py> | 本机源码确认 JumanDic、读音词表与 Jinf 的实际使用位置 |
| KWJA 模型资源 | <https://github.com/ku-nlp/kwja/blob/v2.1.3/src/kwja/cli/utils.py> | 本机源码确认权重版本、文件名与缓存目录 |
| KWJA 推理数据集 | <https://github.com/ku-nlp/kwja/blob/v2.1.3/src/kwja/datamodule/datasets/word_inference.py> | 本机源码与初始化实测确认 checkpoint 配置中的 tokenizer |
| rhoknp | <https://github.com/ku-nlp/rhoknp> | Juman／KNP 文档、基本句和关系对象 |
| EPUB 3.3 | <https://www.w3.org/TR/epub-33/> | EPUB 导入的容器、导航及正文结构 |
| Unicode 规范化 | <https://www.unicode.org/reports/tr15/> | 查询键与来源文本规范化的边界 |

本地依赖版本和资源条件以运行配置及实际资源清单记录。既有外部文档索引与研究引用完整保存在 [来源备份](docs_backup/sources_before_consolidation.md)。

## 当前文档完整标题索引

- [桌面功能验收清单](docs/acceptance.md)
  - 执行条件
  - 协议与来源
  - 语言与查询
  - 多粒度查询专项（下一模块）
  - 阅读与个人状态
  - 助手、审计与交付
  - 异常恢复

- [重构实施记录](docs/implementation_progress.md)
  - 阶段状态
  - 来源行为与设计依据
  - 本轮实现入口
  - 已完成验证
  - 接续顺序

- [本机 NLP 行为基线](docs/nlp_behavior.md)
  - 语料与复现
  - 已验证环境
  - 输出性质与设计决定
    - 正文及坐标
    - 词法与活用
    - 主辞、依存与语义
    - 分析单元与语域
  - 实施验收重点

- [ReadUX](README.md)
  - 文档
  - 代码入口
  - 开发入口

- [总体架构](docs/architecture.md)
  - 处理流程
  - 四级职责
  - 文本与身份
  - 证据与决定
  - 文档会话
  - 缓存与失效
  - 代码入口

- [开发与桌面交付](docs/development.md)
  - 运行组成
  - 当前开发入口
  - 当前资源定位
  - 资源与用户数据
  - 检查与构建
  - 桌面交付验收

- [独立桌面发行版](docs/standalone_distribution.md)
  - 目标
  - 当前状态
  - P1：最小必要资源
    - 资源闭包
    - 当前基线
    - 现有入口审计
    - P1 验收
  - P2：完整环境打包
    - 交付结构
    - 实施顺序
  - P3：分析架构优化
    - 测量边界
    - 优化候选
    - 结果门禁
  - 工作记录
  - 相关入口

- [词典与解释](docs/dictionary.md)
  - 查询目标
  - 多粒度词汇查询
  - 候选发现与词典决定
  - 表记与读音
  - 表记矩阵
  - 目标版本与执行
  - 词典内容与存储
  - 悬浮与显示
  - 验收与入口

- [语言分析](docs/language_analysis.md)
  - 对象与依赖
  - 活用与词形
  - 构词与词典整体词
  - 文节与句法
  - 语法识别与知识目录
  - 表达识别
  - 验收与入口

- [分析来源与对齐](docs/nlp_sources.md)
  - 来源职责
  - 正文准备与语域
  - UniDic 字段契约
  - 来源结果协议
  - 对齐机制
  - 本机执行
  - 代码入口

- [质量验收](docs/quality.md)
  - 完成标准
  - 协议与语言质量
  - 多粒度查询验收
  - 分层差分
  - 审计产物与界面
  - 性能与异常
  - 代码入口

- [书库与阅读器](docs/reader.md)
  - 输入流程
  - 书库与持久化
  - 书架
  - 阅读投影
  - 章节与进度
  - 验收与入口

- [当前设计](docs/README.md)
  - 交付目标
  - 模块索引
  - 完整功能范围
  - 文档维护

- [完整桌面应用实施 TODO](docs/TODO.md)
  - 工作区状态
  - 实施顺序
  - P0：固定接口与验收集
  - P1：接入本机 GiNZA 与 KWJA
  - P2：完成对齐与结构协议
  - P3：建立文档会话与增量分析
  - P4：核验来源输出与统一证据
  - P4 完善：活用链与整体构词
    - P4 独立核验与修复待办
  - P5：恢复完整词典与解释交互
    - P5A：多粒度查询目标
    - P5B：词典验证、聚合与决定
    - P5C：短文本查询与投影
    - P5D：语法与表达
  - P6：恢复书库与阅读器
  - P7：接入用户状态、收藏与导出
  - P8：接入词典助手
  - P9：恢复质量审计与性能诊断
  - P10：生成并验收桌面应用

- [用户状态与输出](docs/user_state.md)
  - 状态所有权
  - 个人操作
  - 数据迁移
  - 选择、收藏与导出
  - 词典助手
  - 验收与入口
- [P4 完成范围与后续开发对齐](docs/p4_handoff.md)
  - 完成判断
  - 原任务与新增任务的归属
  - 下一模块的现有接入差距
    - 定向查询实测
  - P4 自身尚待核验的契约
  - 本轮文档修正与长期方案的分离

- [P4 活用链与整体构词](docs/p4_morphology_formation.md)
  - 目标与范围
  - 已核对的入口与样本
  - 来源协作与判定顺序
  - 活用状态机
    - 状态与输入
    - 主要转移
    - 链与 occurrence 契约
  - 整体构词
    - 候选生成
    - 核心与活用协作
    - 整体对象与查询形式
  - 接入与生命周期
  - 验证与现有输出
  - 文档关联

- [P4 样本输出复核](docs/p4_sample_review.md)
  - 实现结果
  - 十段回测
  - 查询与语言边界
  - 验证记录

## 云端 SaaS 规划来源与索引（2026-10-12）

官网、release、PyPI 和 Hugging Face 数据由用户提供，模型版本采用 GiNZA 5.3.0。GPU 使用用户提供的实际价格，按 720 小时计算月租。AutoDL 来源为 <https://www.autodl.com/>；美元报价的平台归属列入后续信息清单，采购参考入口为 <https://www.runpod.io/pricing> 和 <https://vast.ai/>。

- [云端分析 SaaS 路线](docs/cloud_saas_roadmap.md)
  - 目标与阶段选择
  - 模型与 GPU
  - 本机实测与性能判断
  - Serverless：独立请求与弹性并发
  - 常驻单卡：共享 GPU 与 CPU 并行
  - GiNZA 需要的优化
  - 两种模式的单次时间预算
  - 实验架构
  - 两种部署模式的共同设计
  - 产品架构与缓存
  - 配额与成本控制
  - 定价与收支平衡
  - 最优部署比较：RunPod 与恒源云 4060 Ti
  - 容量与常驻切换
  - 阶段路线
  - 前置要求
  - 后续信息

- [首个部署实验](docs/cloud_gpu_first_experiment.md)
  - 任务目标
  - 实验组成
  - API 协议
  - 代码和镜像组织
  - 模型缓存与启动
  - Endpoint 初始配置
  - 性能预算与 GiNZA 优化边界
  - 实验内容与报告
  - 执行顺序与完成条件
  - 前置要求
  - 来源

2026-10-13 用户确定前期 RunPod serverless、后期常驻单卡，实验使用本地脚本直接调用。RunPod 16 GB GPU 档实际价格为 0.58 美元/小时。以下为后续实施的官方文档入口：

| 来源 | 链接 | 对应内容 |
| --- | --- | --- |
| RunPod serverless | <https://docs.runpod.io/serverless/overview> | Endpoint 与 worker 架构 |
| RunPod handler | <https://docs.runpod.io/serverless/workers/handlers/overview> | Python handler 入口 |
| RunPod requests | <https://docs.runpod.io/serverless/endpoints/send-requests> | 提交与查询异步任务 |
| RunPod configuration | <https://docs.runpod.io/serverless/endpoints/endpoint-configurations> | Worker 数量、保留与执行配置 |
| RunPod cached models | <https://docs.runpod.io/serverless/workers/model-caching> | 模型缓存加载 |
| RunPod pricing | <https://docs.runpod.io/serverless/pricing> | GPU 与持续资源计价 |

## 雨云 P40 显卡云报价（2026-10-13）

来源为用户提供的雨云商品信息。高配档实际 168 元/月，8 核、16 GB 内存、4 GB 显存、25 Mbps 上行、100 Mbps 下行、30–250 GB 磁盘及公网 IPv4；旗舰档实际 258 元/月，16 核、16 GB 内存、8 GB 显存、35 Mbps 上行、150 Mbps 下行，当前售空。商品入口：<https://www.rainyun.com/>。用户指出 P40 的实际推理性能低于本机 RTX 3050 Laptop。

## 恒源云 RTX 4060 Ti 16G 报价（2026-10-13）

来源为用户提供的恒源云实例列表。西北区 RTX 4060 Ti 16G，8 核 AMD EPYC 7601、32 GB 内存、20 GB 系统盘、50 GB NVMe 数据盘，列表标示 800 Mbps 上下行；原价 324 元/月，8.5 折后 275.40 元/月。方案中的每月 30 元备份及杂项费用是财务预算。

## 校内 eScience 资源报价 [废弃]

来源为用户提供的学校 eScience 中心收费表。该方案须经课题组办理，已退出部署路线。历史报价：2080Ti 为 0.6 元/卡/小时，3080 为 0.7 元/卡/小时。

## 旧字规范化来源

- [kyujitai.js 1.3.0](https://github.com/hakatashi/kyujitai.js)，MIT License；仓库副本：`vendor/kyujitai.js`。Rust 前置归一化使用 `dist/kyujitai.json` 的 `kyuji` 字符映射；`douon` 词语规则保留给后置词汇层。
  - install
  - Use
  - Usage
    - new Kyujitai([options], [callback])
    - kyujitai.encode(string, [options])
    - kyujitai.decode(string, [options])

## 竞品与生态参考 (2026-09-23)

| 来源 | 链接 | 用途 |
| --- | --- | --- |
| asbplayer | <https://github.com/asbplayer/asbplayer> | 浏览器端字幕/视频语言学习播放器，支持 Yomitan 与 Anki 联动，提供“句子挖掘”工作流参考。 |
| Hoshi Reader | <https://github.com/Manhhao/Hoshi-Reader> (iOS), <https://github.com/HuangAntimony/Hoshi-Reader-Android> (Android) | 跨平台开源日文小说/有声书阅读器，集成查词与闪卡创建，验证核心阅读器需求边界。 |
| moemeter (萌メーター) | <https://moemeter.netlify.app/books> | 针对 TMW Novel Club 的 Bookmeter 增强工具，提供社区阅读统计、排行榜功能参考。 |
| Jiten.moe | <https://jiten.moe/decks/media> | 提供媒体词汇统计、阅读跟踪与已知/未知词汇状态管理，为本地阅读数据持久化提供参考。 |

## GiNZA CUDA 实验来源与索引（2026-10-13）

| 来源 | 链接 | 对应内容 |
| --- | --- | --- |
| GiNZA 正式微调 BERT large | <https://huggingface.co/megagonlabs/transformers-ud-japanese-bert-large-ginza> | 实际权重与参数 |
| GiNZA BERT large 管线 | <https://github.com/megagonlabs/ginza/blob/v5.3.0/config/ja_ginza_bert_large.cfg> | 模型配置 |
| spaCy EntityRecognizer | <https://spacy.io/api/entityrecognizer> | 实体识别 |
| spaCy transition_parser.pyx | <https://github.com/explosion/spaCy/blob/v3.8.16/spacy/pipeline/transition_parser.pyx> | CPU 解码 |
| spaCy parser_model.pyx | <https://github.com/explosion/spaCy/blob/v3.8.16/spacy/ml/parser_model.pyx> | 特征预计算和传输 |
| spaCy parser.py | <https://github.com/explosion/spaCy/blob/v3.8.16/spacy/ml/models/parser.py> | 状态特征配置 |
| bitsandbytes 0.48.2 | <https://huggingface.co/docs/bitsandbytes/v0.48.2/en/index> | 量化实现 |

- [GiNZA CUDA 测量与输出复核](docs/ginza_cuda_measurement.md)
  - 结论
  - 环境与文本
  - 理论显存
  - 相同正文的配置摘要
  - 批大小与显存上限
  - NER 算法与瓶颈
  - 全量输出差异
  - 小模型与 BERT FP32：随机十处复核
  - 小模型与 BERT FP16：随机十处复核
  - 实验入口与保存结果

## GiNZA BERT 优化交接来源与索引（2026-10-13）

本轮使用本机 spaCy 3.8.16、spaCy Transformers 1.4.0 源码和真实测量。GPU/CPU 倍率由本轮用户提供：4060 Ti 16 GiB 或更高平台，频率 1.5×、计算核心和 Tensor Core 2.1×、FLOPS 2.2×，期望 GPU 部分提升超过 2×。这些数值作为下一阶段分析输入，目标硬件实际推理尚待测量。

| 来源 | 链接 | 对应内容 |
| --- | --- | --- |
| spaCy BILUO 状态与动作 | <https://github.com/explosion/spaCy/blob/v3.8.16/spacy/pipeline/_parser_internals/ner.pyx> | 合法动作、状态更新与实体写回 |
| spaCy StateC | <https://github.com/explosion/spaCy/blob/v3.8.16/spacy/pipeline/_parser_internals/_state.pxd> | token 上下文与共享批次 offset |
| spaCy TransitionSystem | <https://github.com/explosion/spaCy/blob/v3.8.16/spacy/pipeline/_parser_internals/transition_system.pxd> | Cython 动作接口 |
| spaCy Transformers pipeline | <https://github.com/explosion/spacy-transformers/blob/v1.4.0/spacy_transformers/pipeline_component.py> | 长度组织、max_batch_items 与分批 |
| spaCy TransformerListener | <https://github.com/explosion/spacy-transformers/blob/v1.4.0/spacy_transformers/layers/listener.py> | Doc Transformer 特征传递 |
| spaCy Transformer pooling | <https://github.com/explosion/spacy-transformers/blob/v1.4.0/spacy_transformers/layers/trfs2arrays.py> | 特征对齐与池化 |
| NVIDIA CUDA Asynchronous Execution | <https://docs.nvidia.com/cuda/cuda-programming-guide/02-basics/asynchronous-execution.html> | stream、event、pinned host memory 与异步 D2H |
| CuPy ndarray.get | <https://docs.cupy.dev/en/stable/reference/generated/cupy.ndarray.html> | pinned `out` 与 `blocking=False` 传输语义 |
| NVIDIA GeForce RTX 4060 Ti specifications | <https://www.nvidia.com/en-us/geforce/graphics-cards/40-series/rtx-4060-4060ti/> | 4352 CUDA cores、2.54 GHz、16 GB、128-bit |

- [GiNZA BERT 优化交接与流程模型](docs/ginza_optimization_handoff.md)
  - 本轮结论与范围
  - 环境、输入与测量口径
  - 完整管线对照
  - 已完成的执行组织
  - 已有流程成本模型
    - 时间变量与实测阶段
    - 吞吐与调度关系
    - GPU 并行实测
  - 面向 4060 Ti 的分析输入与剩余任务
  - 文件入口与复现
