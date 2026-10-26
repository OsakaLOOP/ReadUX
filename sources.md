# 来源与文档索引

当前设计依据本次确认的交付目标，以及工作区的模块注册、调用关系、类型和资源配置。外部链接沿用已登记来源，供依赖与格式核对。历史资料保存在 [docs_backup](docs_backup/README.md)。

## 决策与实现依据

活用第 7 阶段依据[原始重构要求](</D:/DCMT/dev notes/readux活用重构.md>)开头的插入语、阶段顺序与「気付きたくなかっただけなのだ」示例，以及[执行计划第 7 项](docs/morphology_refactor_execution_plan.md)、[插入语执行器设计](docs/morphology_state_machine_design.md)。实现依据统一白名单、原链状态保持和补助父链关系；运行证据见 [Rust 活用验收](crates/readux-nlp/tests/morphology_reconstruction.rs)的六组 stage_seven 用例，以及 [真实 UniDic 验收](crates/readux-core/tests/p4_language.rs)中的插入、判断收尾和条件形样例。

活用第 5、6 阶段依据[原始重构要求](</D:/DCMT/dev notes/readux活用重构.md>)中的判断结构和文语助动词表，以及[状态机设计](docs/morphology_state_machine_design.md)。原稿的文语部分包含助动词完整活用表、接续、意义与时代备注，教育文法部分包含学校文法与教育文法对照；本次逐项核对词形、前接条件、现代判断边界，并使用统一语法词条链路实现。文语表的参考入口为[日语助动词](https://ja.wikipedia.org/wiki/助動詞_(国文法))，术语对照入口为[活用（日语）](https://zh.wikipedia.org/wiki/活用_(日語))。

| 依据 | 对应内容 |
| --- | --- |
| [静态资源测量入口](scripts/measure_static_storage.py)、[正式投影与压缩测量](scripts/measure_static_storage.mjs)、[字节数汇总](data/validation/static-storage-summary.json) | 2026-10-24 只读盘点 Windows 文档目录的 ReadUX Library、桌面及仓库 MessagePack 缓存、两课指南、四个词典查询库和源包。覆盖 8 本书的体积、5 本书的 26 个分析单元及 132 条指南例句，158 份快照通过正式投影和序列化还原核对。方法与预算见[空间报告](docs/static_storage_measurement.md)。 |
| [msgpack](https://pypi.org/project/msgpack/)、[Brotli](https://pypi.org/project/Brotli/) | 测量工具依赖的来源登记；文件体积来自本机实际压缩，未引用外部容量和托管报价。 |
| 本机《三省堂Super大辞林3.1》「ぬ」词条（记录 159341）及[指南保存的词法来源](src/grammar-guide/lesson-1-1-analysis.json) | 2026-10-23 通过正式词典服务读取「ぬ」完整词条：现代否定「ぬ」源自文语「ず」的连体形，终止形、连体形在口语中常用「ん」，接「ます」时通常为「ません」。指南「ありません」的 UniDic 来源为「ませ」（助動詞-マス／未然形-一般）和「ん」（词元ず／助動詞-ヌ／終止形-撥音便），用于敬体否定的整体词条及音变解释。 |
| [本机词典索引](data/dicts)、[GiNZA 适配器](scripts/nlp_adapters.py)与[GPU 模型入口](scripts/ginza_bert_engine.py) | 2026-10-16 核对「気合いが入っていたりとかしていない」「めいっぱい楽しもうね」的 UniDic 与两种 GiNZA 模型输出；三套词典均有「めいっぱい」到「目一杯」记录的显式别名。查询归属问题见[问题追踪](docs/lookup_issue_tracking.md)。 |
| [活用链配置](configs/unidic-nlp/morphology.json)与[状态机](crates/readux-nlp/src/morphology_machine.rs) | 2026-10-16 使用本机 UniDic 人工句子核对逆接、原因、条件、义务及禁止结构的分词与活用；四类音便采用原始活用字段，融合形归为口语缩略。范围及维护约定见 [活用链配置指导](docs/morphology_configuration.md)。 |
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

## 产品路线讨论来源（2026-10-24）

| 来源 | 采用内容 |
| --- | --- |
| [市场与产品策略原稿](</D:/Downloads/market_and_product_strategy.md>)，更新于 2026-10-23 | 通读全文，包括用户流程、产品形态、输入输出桥接及语法指南专项评价。原稿关于社区规模、竞品体积和质量比例沿用其调研口径，本次讨论重点为产品路线与架构。建议见 [Web 产品路线](docs/web_product_strategy.md)。 |
| [空间实测](docs/static_storage_measurement.md)、[现有总体架构](docs/architecture.md)、[云端 SaaS 方案](docs/cloud_saas_roadmap.md)、[宿主请求入口](src/services/reader.ts) | 核对静态阅读产物、生成与读取职责、GPU 部署阶段，以及正式书籍阅读器的 Tauri 依赖。新路线属于讨论建议。 |
| 原稿引用的[沉浸式外语习得／间隔重复知识库](https://my.feishu.cn/wiki/YeOSwsG7giLuQxkcDFscUXVZn2f)及[JarrettYe 主页](https://www.zhihu.com/people/JarrettYe/) | 原稿中的社群背景链接，登记供后续市场核查；本次产品容量与架构判断依据本机资源测量和代码。 |

### 策略原稿完整标题索引

- 市面同类产品调研、用户群体画像与 ReadUX 形态策略分析
  - 1. 市面主流产品形态与核心用户群体调研
    - 1.1 海外与全球主流标准：Yomitan + Anki 工具链
    - 1.2 国内沉浸式与间隔重复领军社群：Jarrett Ye（叶峻峣 / Thoughts Memo）
    - 1.3 现代开源阅读应用代表：Hoshi Reader 与 Fushi
      - （1）Hoshi Reader
      - （2）Fushi（hajisensai/Fushi / fushi.moe）
  - 2. ReadUX 与市面产品的核心形态与技术指标对比
    - 2.1 全维度横向对比表
    - 2.2 ReadUX 解决的三大核心痛点（为什么 Yomitan 模式不够用）
  - 3. 核心战略抉择：作为插件/外挂整合分发 vs. 独立应用工作
    - 3.1 方案对比评估
      - 路线 A：纯插件 / 外挂模式（如做成浏览器插件，或 Yomitan / ッツ Reader / Hoshi 的外挂后端）
      - 路线 B：纯封闭独立桌面应用（与现有 Anki / ッツ 生态完全隔离）
    - 3.2 推荐策略：“独立核心桌面端 + 生态桥接协议（Dual-Track 架构）”
      - 1. 主体形态：坚持独立桌面应用（ReadUX Desktop），打造不可替代的“胶囊阅读与语法实验室”
      - 2. 向下兼容外挂（输出桥接）：原生打通 AnkiConnect 一键导出（融入 Yomitan + Anki 生态）
      - 3. 向上轻量外挂（输入桥接）：提供“剪贴板监听 / 浏览器正文一键投送（Send to ReadUX）”
  - 4. 面向核心种子用户的验证与推广建议
  - 5. 《日文语法指南》模块核查与专项评价
    - 5.1 对您描述的核实结论：完全属实，且底层工程闭环度极高
    - 5.2 对《日文语法指南》的内容与产品价值评价
      - 1. 教学法与语言学视角：罕见的“专为中文母语者重构的现代日语认知框架”
      - 2. 交互体验视角：“双向锚定 + 零配置沙盒”是绝佳的产品转化漏斗
      - 3. 少量可进一步打磨的细节建议（吹毛求疵）

## 语法指南展示设计来源（2026-10-21）

本轮范围为原稿展示、胶囊 JSON 预生成及后台审阅、修改和加载。现有词典、活用链与来源标签用于生成阶段，前台使用固定内容。设计见 [展示模块](docs/grammar_guide_display.md)及 [1.1 内容规格](docs/grammar_guide_1_1.md)。

| 来源 | 链接 | 阅读内容与采用范围 |
| --- | --- | --- |
| 用户提供的《ReadUX 日文语法指南》 | [本机原稿](</D:/DCMT/dev notes/ReadUX 日文语法指南.md>) | 1.1 全文、前半部 0–10 章结构、学校文法附录、五张对照表，以及后半部词干／助词／屈折／语法／句法／篇章补充提纲。原稿逐字保留，例句插入配置独立保存。 |
| 2026-10-21 用户要求 | 当前会话 | 原稿禁止删改；胶囊所有信息提前生成并固定为 JSON，后台可审阅、修改和加载；仅保留各任务核心验收指标。 |
| UniDic FAQ | [官方 FAQ](https://clrd.ninjal.ac.jp/unidic/faq.html) | 四级词性、cType／cForm、词元读法、出现假名与出现发音的区别；实际字段顺序以本机 2025.12 schema 为准。 |
| 日语 UD AUX | [AUX：auxiliary verb](https://universaldependencies.org/ja/pos/AUX_.html) | Definition、Examples；包含助动词及特定功能动词／形容词，「食べている」「勉強する」等例子。 |
| 日语 UD ADP | [ADP：adposition](https://universaldependencies.org/ja/pos/ADP.html) | Definition、Examples；覆盖格／系／副助词；其他日语助词可归 CCONJ、SCONJ、PART，故 UPOS 与教材分类分别呈现。 |
| GiNZA 官方说明 | [GiNZA](https://megagonlabs.github.io/ginza/) | 命令行及 Python 输出中的 UPOS、细粒度 tag、Reading、Inflection；来源原值按当前适配器输出展示。 |
| 本机来源协议 | [model.rs](crates/readux-nlp/src/model.rs)、[nlp_adapters.py](scripts/nlp_adapters.py)、[nlp.ts](src/types/nlp.ts) | UniDic 29 字段、GiNZA features.pos／tag／morph、scalar 范围及现有活用链。 |
| 本机展示入口 | [正式阅读器](src/components/reader/ReaderDocumentView.vue)、[阅读投影](src/reader/lookupPresentation.ts)、[Markdown 导入](src/reader/document.ts)、[文法库](src/components/GrammarLibraryPanel.vue) | 胶囊范围、ruby runs、词典与活用命中、Markdown 标记清理及字符串例句现状。 |

### 原稿完整标题索引

以下按原稿顺序保留全部标题，展示时采用原稿顺序。

- 0. 引言
  - 元认知, 概念先行
  - 二八定律
  - 内容的组织形式
- 1. 词语和活用基础
  - 1.1 认识实词、虚词、助词
  - 1.2 实词的活用变形: 词干与词尾
  - 1.3 词干的分类与实词活用规则
  - 1.4 派生构成的复合词
  - 1.5 综合应用(1)
- 2. 助词、文节、句法基础
  - 2.1 常见助词及功能
  - 2.2 文节的划分
  - 2.3 简单句中的文节间联系
  - 2.4 综合应用(2)
- 3. 黏着和表达语法基础
  - 3.1 [[黏着: 实词与虚词的接续和组合变形]]
  - 3.2 固定化、词汇化的黏着构式表达(复合辞)
- 4. 综合应用(3)
- 5. 基于黏着的复杂语法表达
- 6. 小句和复合句句法
- 7. 篇章文法
- 8. 综合应用(4)
- 9.  UD通用依存语法
- 10. 话语交际
- 附录: 学校文法对照
  - 一、 一段动词词干（《新标日》称：二类动词）
  - 二、 五段动词词干（《新标日》称：一类动词）
  - 三、 い形容词词干（《新标日》称：一类形容词）
  - 四、 名词与な形容词词干（《新标日》称：名词与二类形容词）
- 2. 基础助词
  - 一、 第一层：格助词（给名词分配事件角色）
  - 二、 第二层：副助词（给角色加上范围与程度滤镜）
  - 三、 第三层：系助词（把词语提拔为全句的“话题”或“焦点”）
- 3. 基础屈折结构
  - 一、 接上后变成“新的一段动词”的后缀（语态层）
  - 二、 接上后把整体变成“い形容词”的后缀（愿望与否定层）
  - 三、 接上后把整体变成“な形容词”的后缀（样态与比况层）
  - 四、 句尾封口后缀（时态、礼貌与意志，接龙到此结束）
- 4. 基础语法
  - 一、 套路1：用“连用形”直接拼装（紧密度最高）
  - 二、 套路2：用“て形”外接补助动词与评价词（最常用）
    - 1. 接“补助动词”（原来的实义动词退居二线，用来表状态、准备或方向）
    - 2. 接「は / も」与评价词（构成许可、禁止与义务公式）
  - 三、 套路3：用“形式名词”把句子打包成名词
- 5. 基础句法
  - 一、 定语从句（连体修饰）：所有修饰语一律排在名词前面
  - 二、 引用小句与疑问小句（句中套句）
  - 三、 复句的四大连接逻辑（条件、原因、转折、并列）
  - 四、 长句阅读的两大导航信标
- 6. 基础文法
  - 一、 找回消失的“我、你、他”（省略主语的三大线索）
  - 二、 粘合上下文的句尾：「～のだ（んだ）」与「～わけだ」
  - 三、 串起段落的两大路标：指示词与接续词
      - 主語・述語
      - 連用修飾語
      - 連体修飾語
      - 接続語
      - 独立語
      - 並立語
    - 目的語と補語
      - 対象語（補語）
      - 状況語
    - 修飾語の特徴
  - 品詞体系
    - 自立語
    - 付属語
  - 名詞の格
  - 活用形と種類
    - 音象徴語彙（オノマトペ）

原稿另含五张具名对照表，使用段落标题标记：表 1 实词、虚词、助词的多维判断标准；表 2 助词内部六子类；表 3 构词与黏着四层机制；表 4 传统连用形的四种实际表现；表 5 句法分析三层级。

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

- [语法指南展示与交互设计](docs/grammar_guide_display.md)
  - 1. 原稿与内容组织
  - 2. 固定 JSON 内容包
    - 2.1 提前生成与前台加载
    - 2.2 数据内容
    - 2.3 结构示意
  - 3. 后台审阅与编辑
  - 4. Markdown 与胶囊交互
  - 5. 中文术语与教学分类
    - 5.1 生成阶段的映射
    - 5.2 来源标签
  - 6. 现有代码复用
  - 7. 任务与核心验收指标
  - 8. 来源

- [1.1 认识实词、虚词、助词：内容与交互规格](docs/grammar_guide_1_1.md)
  - 1. 原稿原文
  - 2. 例句插入关系
  - 3. 原稿例子接入
  - 4. 补充胶囊例句
    - P01 组成单元、含义和语法身份
    - P02 三类单元
    - P03 词汇义、语法义与语境
    - P04 独立成分与独立成句
    - P05 实词的范围包括抽象词和副词
    - P06 虚词为前接结构增加意义
    - P07 书写形式提供观察线索
    - P08 功能用法与词汇意义的联系
    - P09 助词标示关系、范围与连接
    - P10 学校文法中的自立语与附属语
    - P11 三类之外的辅助成分
    - P12 同词不同用法的对照
      - P12a 訳／わけ
      - P12b 事／こと
      - P12c 物／もの
      - P12d 所／ところ
      - P12e いる的两种用法
      - P12f 見る／みる
      - P12g 付く／について
  - 5. 固定数据制作
  - 6. 核心验收指标

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

## Windows 安装与图标管理（2026-10-16）

- [Tauri Windows Installer](https://v2.tauri.app/distribute/windows-installer/)
  - Building
    - Build Windows apps on Linux and macOS
    - Building for 32-bit or ARM
  - Supporting Windows 7
  - FIPS Compliance
  - WebView2 Installation Options
    - Downloaded Bootstrapper
    - Embedded Bootstrapper
    - Offline Installer
    - Fixed Version
    - Skipping Installation
  - Minimum Webview2 version
  - Customizing the WiX Installer
    - Installer Template
    - Internationalization
  - Customizing the NSIS Installer
    - Installer Template
    - Extending the Installer
    - Install Modes
    - Internationalization
- [Tauri CLI 2.11.4 NSIS 模板](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.11.4/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi)：版本比较、覆盖升级、注册表、快捷方式与卸载数据选项。
- [Tauri NSIS 工具宏](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.11.4/crates/tauri-bundler/src/bundle/windows/nsis/utils.nsh)：进程检查与快捷方式 AppUserModelID。
- [Tao 0.35.3 Windows 窗口实现](https://docs.rs/crate/tao/0.35.3/source/src/platform_impl/windows/window.rs)：窗口小图标与任务栏大图标的独立设置；本机 Cargo 源码已复核。
- [Windows WM_SETICON](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-seticon)、[Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects)：任务栏图标与分析进程生命周期接口。

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

## 语法与活用链分析重构来源与索引（2026-10-16）

| 来源 | 链接/路径 | 对应内容 |
| --- | --- | --- |
| 活用链配置 | [configs/unidic-nlp/morphology.json](configs/unidic-nlp/morphology.json) | L2 算子(95)、特征(42)、序列(14)、谓词连接(12)、连接规则(29)、活用形映射(9)及插入模式(2) |
| 活用状态机 | [crates/readux-nlp/src/morphology_machine.rs](crates/readux-nlp/src/morphology_machine.rs) | 连续词尾合并、verb_core 动词核心合并、插入助词 bridge_support 与谓词构式连接 |
| 编译语法目录 | [crates/readux-core/resources/grammar/compiled/grammar_catalog.json](crates/readux-core/resources/grammar/compiled/grammar_catalog.json) | L3 语法概念(113)、义项(162)、实现(125)与规则(125) |
| 表达模式库 | [crates/readux-core/resources/expression_patterns.json](crates/readux-core/resources/expression_patterns.json) | 连续惯用模式(15)与呼应非连续模式(12) |
| 语法应用层分析 | [crates/readux-core/src/language_analysis.rs](crates/readux-core/src/language_analysis.rs) | L2 补助用言投影、L2/L3 概念互斥过滤、声明式规则匹配与解释生成 |
| 词元假名适配表 | [crates/readux-core/src/language_rules.rs](crates/readux-core/src/language_rules.rs) | 36 组假名到汉字 lemma 映射及 UniDic 词元规范化 |

- [语法与活用链分析现状及重构架构分析](docs/grammar_morphology_architecture_analysis.md)
  - 1. 概述与设计定位
  - 2. 识别模式的范围与清单统计
    - 2.1 L2 活用链系统
    - 2.2 L3 语法表达系统
  - 3. 规则的存储形式与存储位置
    - 3.1 L2 规则存储：configs/unidic-nlp/morphology.json
    - 3.2 L3 语法与表达规则存储
  - 4. 全面架构分析梳理
    - 4.1 数据流与生命周期流程
    - 4.2 核心数据结构与抽象
  - 5. 现状评估、存在问题与重构动因
    - 5.1 痛点一：L2 有限状态机与 L3 规则匹配器的职责割裂与边界重叠
    - 5.2 痛点二：句法约束硬编码，缺乏统一声明式图匹配能力
    - 5.3 痛点三：词元（Lemma）还原与表记适配的隐式字典依赖
    - 5.4 痛点四：多源分词粒度冲突的启发式修补
    - 5.5 痛点五：规则的可维护性与自动化验证闭环
  - 6. 重构架构设计建议（Blueprint）

