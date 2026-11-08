# 词条、用法与解说资源

更新日期：2026-11-09。本文说明当前运行资源、规则到词条的关联和实际取用顺序。状态机与配置分别见 [L4](morphology.md)、[L5](morphology_configuration.md)。

## 资源职责

阅读说明组合词汇词典、内置语法词条和活用概念解说。相近目录中的资源按完整路径区分。

| 资源 | 维护内容及实际消费 |
| --- | --- |
| `configs/unidic-nlp/morphology/source/operators/` | operator 标题、原型、概念及变体；Rust 识别及前端活用内容 |
| `configs/grammar/particles.json` | 内置助词词条、用法、接续说明及例句；`particleEntries.ts` |
| `configs/grammar/auxiliaries.json` | 内置助动词及补助用法；`particleEntries.ts`、`morphologyBubble.ts` |
| `configs/grammar/migrations/te_construction_ids.json` | 历史 ID、本体及构式映射；前端部分构式查询分支 |
| `crates/readux-core/resources/grammar/source/` | 概念、义项、实现形式、规则、重定向、解说及来源元数据 |
| `crates/readux-core/resources/grammar/compiled/` | 编译后的目录、解说、搜索索引及 manifest |
| 词典 `.kdict` 及查询索引 | 词汇词条与义项内容；Rust 词典查询和前端词典面板 |

`configs/unidic-nlp/morphology/source/classical/auxiliaries.json` 保存文言执行条件，`configs/grammar/auxiliaries.json` 保存内置讲解词条。两个文件各自维护。

## 规则、概念与实际出现

规则定义通过 `concept` 指向知识身份；`variants.concept` 可依据完整 token 条件选择不同概念。Rust 的 `concept(kind, token)` 将本句选择写入 `MorphologyOperator.concept_id`，前端按该值读取解说。

| 身份 | 当前用途 |
| --- | --- |
| operator kind | 已登记的操作，如 `past`、`copula_past`、`te_kureru` |
| operator 实例 ID | 本句某一步及其状态、范围和证据 |
| concept_id | 相关知识或词条内容，如 `grammar.auxiliary.da` |
| sense_id | 概念目录中的具体用法身份 |
| realization_id／rule_id | grammar 目录中的实现形式与通用识别规则身份 |
| explanation_id | 一份版本化解说内容 |
| occurrence ID | 正文中一次命中及其 operators、成员和实际范围 |

活用 operator 当前直接关联 concept。样态、传闻等执行证据和 candidates 可记录具体解释依据；活用说明的主读取路径没有按 sense_id 选择正文。登记目录中的 sense 或 realization 字段表明知识组织能力，实际用法确定以调用路径为准。

2026-11-09 的 153 个 operator 及变体共引用 125 个 concept，125 个均有编译解说，编译解说中每个 concept 对应一项。前端 `explanationByConcept` 直接以 concept 为键建立 Map；将来同 concept 增加多份解说时，需要明确新的内容选择方式。

## 活用内容的读取顺序

[morphologyBubble.ts](../../src/reader/morphologyBubble.ts) 的 `morphologyEntry` 返回 headword、identity、conjugation、usage、transition、summary 和 examples。

| 显示内容 | 当前来源与顺序 |
| --- | --- |
| 本句原型 | `normalized_form`，缺失时读静态 normalized／prototype |
| headword | 优先 support.expanded；て形和缩约用法移除原型开头て／で；列举、意向、名词变化用法取 support.bases 首项；其他取原型 |
| 身份 | support、本句 POS、执行器及接续结构联合生成补助动词、复合语法形式、插入成分等名称 |
| 当前活用 | 当前末成员的来源 cForm、状态及现象标签 |
| 用法名称 | operator 定义的中文 title，随后使用 operator.label |
| 局部变化 | `lookupPresentation.ts` 的推导步骤，缩约分支保留实际缩略词形 |
| 一般摘要 | concept 解说的 function_summary → 内置同 headword 首个 sense 的 meaning → 定义 title → operator.description |
| `polite_negative` 摘要 | 内置「ません」首个 sense 的 meaning；缺失时使用代码内的敬体否定摘要 |
| 例句 | concept 的 example_pair 内容块；缺失时取内置首个 sense 的 examples；随后使用 concept 的单语 examples；最多显示两条 |

例句选择按当前数组存在性实现，内置空 examples 数组也会作为可用值参与选择。`example_pair.label` 是日文，`text` 是中文；普通 examples 只有日文时中文留空。

「でした」由 `copula_past` 选择实际原型，concept 仍为 `grammar.auxiliary.da`，摘要说明判断及过去形式，例句来自该 concept。「済んだ」中的过去使用 `morphology.tense.past`，词典区的「済む」义项另外来自词汇词典。

## 内置助词与助动词

[particleEntries.ts](../../src/grammar/particleEntries.ts) 直接导入 particles、auxiliaries 和迁移映射，转换为共用 `DictionaryLookup`／`DictEntry`，词典名为「ReadUX 内置语法词条」，mode 为 `builtin-grammar`。

内置 entry 保存 id、headword、reading、pronunciation、variants 和 senses。sense 保存 id、subtype、title、meaning、connection 及日中例句。词性小类例如「格助詞」「接続助詞」用于同形筛选。

`particleLookup` 先按请求的助词／助动词类别选择资源。助动词可按 lemma 匹配；其他分支使用请求表记及词性小类，或表记变体。`makeEntry` 选取相同 subtype 的 senses，未取得子集时使用条目全部 senses。格助词「が」等条目可以同时显示多个同类用法。

`grammarMorphemeLookup` 根据 morpheme 的表面串、词元、读法和 POS 查询；传入文档时，先检查是否属于「ません」整体，再处理持续补助「いる」等特定身份。阅读器和指南例句均调用该函数。

`chainGrammarLookups` 保留复合说明接口：按敬体否定、迁移表中的て形构式、意向、ので整体及组分、普通语素等顺序组织内容。主阅读与指南中的直接调用以 `grammarMorphemeLookup` 为主，引用该辅助接口时需核对使用位置。

## 补助本体与历史映射

活用识别中的「てくれる」「てしまう」由连接规则与补助本体协作，concept 分别关联くれる、しまう等本体解说。实际连接器和后项范围继续保存在 operator、connection 和 occurrence 中。

`te_construction_ids.json` 保存 component 与 operator 两类映射，包括 legacy_id、canonical_operator、representation、component、connector 和 canonical_concept_id。`particleEntries.ts` 仍据此建立部分构式查询和缺少本体词条时的回退内容。活用气泡的主要解说读取使用 operator 的 concept；迁移映射的存在和该主路径分别说明。

## grammar 源内容与编译

源目录为 [resources/grammar/source](../../crates/readux-core/resources/grammar/source/)。

| 目录或文件 | 职责 |
| --- | --- |
| `concepts/` | 概念、标签、等级、关系、默认解说及状态 |
| `senses/` | 具体义项、语义条件、排除及关联 |
| `realizations/` | 形式、连接签名、可能义项及 rule 引用 |
| `rules/` | 通用规则、条件、捕获、优先级及例子 |
| `redirects/` | 概念重定向 |
| `explanations/` | 独立解说正文 |
| `bundles/` | 按主题组织概念、解说及相关对象，由编译器展开 |
| `catalog_metadata.json` | 缺省 provenance 和 review_status |

活用主题 bundle 包括 `morphology_support_20261027.json`、`morphology_permission_20261027.json`、`morphology_connections_20261028.json`、`morphology_classical_auxiliaries.json` 和 `morphology_adverbs.json` 等。日期或阶段名称属于现有文件名，按内容归属维护。

[build_grammar_catalog.py](../../scripts/build_grammar_catalog.py) 载入独立对象并展开 bundles，核对字段、ID、引用、解说、来源与状态，生成：

| 产物 | 内容及当前消费 |
| --- | --- |
| `grammar_catalog.json` | 启用概念、senses、realizations、按优先级排列的 rules 及 redirects；Rust 目录和通用规则资源 |
| `grammar_explanations.json` | 版本化解释文本；Rust 目录详情与前端活用说明 |
| `grammar_search_index.json` | 标签、别名、领域、形式及义项提示等搜索数据 |
| `manifest.json` | 目录及内容版本、统计和内容身份 |

`GrammarExplanationDocument` 保存 title、compact_summary、function_summary、connection、formation、usage_notes、语义及语用约束、例句、反例、来源、内容版本、制作及审阅状态、provenance 和 body_blocks。当前活用面板主要取功能摘要及例句，其他字段可通过目录详情取得。

provenance 描述内容来源、作者、日期和版本；source_refs 指向内容所列依据，authoring_status 与 review_status 分别保存制作和审阅状态。源 metadata 的默认值为 AI 来源及 `ai_checked`，具体条目可覆盖，阅读维护时应同时查看实际状态字段。

## 现有查询服务与过时设计

[grammar_catalog.rs](../../crates/readux-core/src/grammar_catalog.rs) 编译时嵌入 catalog 和 explanations。`search` 按启用状态、标签、别名、类别、JLPT、审阅状态和来源筛选；`get` 返回 concept、全部 senses、默认 explanation 及同概念的全部 explanations。

Tauri 注册 `search_grammar_catalog`、`get_grammar_concept`。`GrammarLibraryPanel.vue` 保留界面实现，主应用未接入该组件。原语法库界面设计已过时，现用解说资源继续服务活用及内置查询。

原 grammar 整体分析流程已过时：`language_analysis::apply` 保留规则应用和解释组织，普通分析链路未调用；`refresh_language` 当前复制并记住文档。规则预览通过 `language_analysis::preview` 执行。L9 的登记模式和 L10 的用法确定将依据新的模块职责讨论。

## 内容维护与后续细化

维护活用词条时，先确定 operator、概念和所需具体用法，再更新相应 bundle 或独立内容；内置助词、助动词按实际消费另行更新。中文功能标题、日语原型、实际词形、功能摘要及例句各自对照本句。

```powershell
python -X utf8 scripts/build_grammar_catalog.py
npm run morphology:build
```

grammar 编译后，活用配置编译会检查 concept 的有效摘要和可用例句。最终核对可使用 `build_grammar_catalog.py --check` 与 `npm run morphology:check`；仅修改文档时集中核对引用和内容依据。

本轮完成资源职责、身份和实际读取关系。下一位 agent 可进一步整理各语法家族的完整 source JSON、bundle 扩展样式、多用法条目的逐项证据和例句，以及概念正文到各显示字段的对应。L5 已列出逐规则细化清单。后续义项自动选择、多个解说的选择策略及新增模式模块继续按总计划讨论。
