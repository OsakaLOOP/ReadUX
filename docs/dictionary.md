# 词典、查询目标与聚合

更新日期：2026-11-09。本文为 M5，依据当前词典引擎、目标构建和前端调用说明词汇范围、表记矩阵及内容。分析到界面的映射见 [L11](language_analysis/presentation.md)，说明容器见 [M3](explanation_layout.md)，静态资源见 [E2](static_library.md)。

## 查询目标

`dictionary/targets.rs` 根据统一文档及词典证据生成 `LookupTargetGroup`，协议为 `readux.lookup-targets.v1`。目标包含正文范围、成员、核心链与构词引用、查询形、读音及词性证据、元数据提示和 `MatrixRequest`。完整单元分析保存目标组，阅读器通过目标 ID 查询；主动搜索及正文参照使用相应请求。

| 目标     | 查询依据                              |
| -------- | ------------------------------------- |
| 外层词汇 | `outer_targets`；词典验证或最小回退目标 |
| 内部词汇 | `inner_targets`；用 `parent_outer_id` 关联外层 |
| 候选词汇 | `candidate_targets`；未采用的核心／构词候选，记录 pending 或 rejected |
| 语法目标 | `grammar_targets`；范围、上下文、规范形、链及可用 concept |
| 主动搜索 | `search {word}`；后端建立 search 查询形 |
| 词典记录 | `query_entry {entry_key}`；读取源记录完整内容 |

目标组同时保存排除范围、覆盖状态、默认目标及来源版本。词典查询结果和活用解说分别生成，正文使用共同范围关联；内置助词、助动词查询见 [L6](language_analysis/entries.md)。

## 多粒度词汇查询

当前构建器从词汇核心、token、构词候选、作者注音以及词典元数据发现范围。候选受 anchor 范围、功能成分、标点和空白等边界约束，再查询词典轻量索引进行确认。

| 粒度               | 查询要求                                                 | 代表验收对象                                                       |
| ------------------ | -------------------------------------------------------- | ------------------------------------------------------------------ |
| 整体词 | 当前起点具有独立词汇依据的最长候选优先 | 方向転換、神戸市等用于范围核对 |
| 内部词 | 保存外层所含的较小候选及词典 component_roles | 教科用図書等用于包含关系核对 |
| 活用核心 | 通过链的 lookup form 和配对读音查询 | 読みませんでした → 読む |
| 功能成分 | 生成语法目标及独立活用详情，查询形采用本体 | てる对应いる，说明对应补助构式 |
| token 内部成分 | 使用有明确相对范围和查询形的词典元数据 | 实际覆盖按条目 component_roles 核实 |

词典提示保存 entry ID、词头、读音、词性、记录类别、词汇角色、语法功能、内部成分和前后范围约束。独立词汇要求 `lexical_role=independent`、兼容性可接受且没有 grammar_functions。来源 token、词典采用与界面显示继续通过引用关联。

内部候选按包含关系组织；多个成员组成的内部候选需要独立词汇依据。部分以名词为核心的サ变链按现有专门分支处理。宿主从 `decision=queryable` 且包含当前位置的内部目标中选择最小范围；补助父子链由活用分析独立管理。

## 候选发现与词典决定

`targets::build_profiled` 首先识别功能成员及连接器范围，生成语法目标与排除区域，然后建立词汇候选。`contains_exact_batch` 批量确认精确键，候选按表记、读音和词性读取条目预览，并按 scope 约束保留提示。

外层目标从 anchor 起点向后选择同一起点的最长独立词汇候选，长度相同时考虑核心与构词依据；随后考虑作者注音的最小候选及普通最小回退。已确认目标标为 accepted，最小回退标为 fallback。内部对象保存 queryable 或 component，未采用的核心及构词候选记录 pending／rejected。

`reason` 描述 author_ruby、dictionary_lexical_entry、nonlexical_entry、no_entry 等当前原因。默认目标优先 accepted，再按长度及位置选择。词典无内容、范围候选未采用与请求失败分别通过目标决定、矩阵可用性及响应错误表达。

`LookupReport` 提供目标及候选诊断，调用为 `lookup_report {analysis_id}`。查询与投影问题记录保留于 [lookup_issue_tracking.md](lookup_issue_tracking.md)；具体样例的状态按记录日期和当前分析核实。

## 表记与读音

查询形按来源字段配对读音：出现表记对应 kana，基本表记对应 kanaBase，词元对应 lForm，活用链提供自身还原形式。`QueryForm` 保存 kind、form、reading 和 reading_field。词元及可能形式恢复的基础契约见 [L2](language_analysis/lexical_input.md)。

`token_forms` 在存在不同于词面的 base／potential_lemma 时采用还原查询，其他情况优先出现表记并核对同音字段。多成员与作者 ruby 读音保留各自证据，查询依实际词性参与兼容性和记录排序。

`lookup_matrix_profiled` 先发现精确表记、别名及读音候选，再构建 forms，并读取当前表记下符合读音的实质记录。原表记保留，其他读音与同音表记通过各自证据和矩阵条目呈现；表头显示词典自身读音及匹配信息。

作者 ruby 采用校验后的完整范围和读音，候选构建保留整体及内部成分；实际外层选择继续经过词汇及回退分支。读音规范化和表记身份由 `lookup_state.rs` 管理，原始形式保存在 variants 与词典记录中。

`targets::query` 聚合同一目标的所有查询形。假名基本形只发现同音候选时，后续来源词元的精确表记与读音命中可更新默认项，例如「ある／アル」与「有る／アル」。显式 `selected_form` 以 form ID 或 display form 选择目标表记。

精确键关联多条源记录，排序考虑表记、读音、词性及记录类别。接辞、汉字、助词等记录可作为查询内容，是否用来确认独立词汇范围另由 metadata 决定。规范化的接辞符号及假名形式用于身份比较，展示保留源词头。

| 查询情况                   | 默认与备选                                       |
| -------------------------- | ------------------------------------------------ |
| 表记与读音匹配             | 原表记优先，匹配读音的记录靠前；异读记录可切换   |
| 原表记收录，指定读音未收录 | 原表记及其实际读音保留，并列出指定读音的候选表记 |
| 原表记未收录               | 原表记显示未收录，同音候选供用户选择             |
| 纯假名查询                 | 优先显示有内容的表记，同时保留输入形式           |

「行く」等异读、「橋場／羽柴」等同音异表记用于核对矩阵身份；「読みませんでした」按活用核心查询「読む」，补助缩约按本体查询。各查询形自身的配对读音、词典实际读音和原文表记分别保留。

## 表记矩阵

查询结果采用“表记组 × 词典列 × 词典记录”的结构。表记组保留原始 variants、读音、证据和全部词典可用性；同一单元格可包含多个读音、词性或记录类型相容的 occurrence。

`DictionaryLookup` 保存 query、observed_form、reading、pos、selected_form_id、mode、forms、dictionary_names、entries 和可用 timing。表记组含规范身份、原始 variants、读音、证据及各词典 availability；当前表记的 entries 保存记录预览或已加载内容。

`TooltipPanel.vue` 维护当前词典及记录选择。`dictionaryMatrix.ts` 的 `dictionaryForForm` 优先保留仍可用的词典，`formForDictionary` 为切换词典选择可用表记。表记变化通知宿主重新取得对应结果，词典切换优先保留当前表记；整体及内部面板分别管理历史和响应代次。

阅读器从工具栏配置词典顺序，保存至 `readux.reader.dictionaryOrder`，宿主按顺序组织结果。正文参照点击后由宿主调用 `search` 并保存当前面板历史；返回恢复前一查询。当前导出摘录读取选择数据，词典正文的导出字段留待 Anki 设计确定。

## 目标版本与执行

会话查询 `query_lookup_document` 携带 session_id、text_version、generation、unit_id、artifact_revision 和 target_id。服务取得对应目标组后执行查询，正文异步响应再核对当前命中和请求代次。单独分析使用 analysis_id 的 `lookup_targets`、`query_target`；token 和候选查询另有 `query`、`query_candidate`。

完整单元分析将文档与 lookup 一起保存到分析缓存。阅读器按可见范围加载目标，缓存复用及会话产物有效期由服务管理；规则、词典或来源更新后的缓存和静态制作见 [L1](language_analysis/pipeline.md#缓存及静态消费)。

本机词典引擎保存已打开数据库、矩阵查询、definition block 和 presentation 缓存。definition block 容量为 64，presentation 为 128，按当前引擎实例管理；查询耗时及命中信息可从响应和性能统计读取。

## 词典内容与存储

| 对象                | 职责                                           |
| ------------------- | ---------------------------------------------- |
| `.kdict`            | format v1、schema v4 的源包，包含词头、索引关系及压缩释义 |
| schema v4 SQLite    | `dicts/` 本机查询数据库，按源包及 preview 版本校验并重建 |
| 词典 occurrence     | 词典、源 entry 和子记录共同限定的稳定内容身份  |
| sense tree          | 真实义项编号、层级、局部标签、释义和例句       |
| section 与 relation | 惯用句、派生、表记说明和正文参照关系           |

`bundle::prepare_dictionary_sources` 扫描 `dict-sources/*.kdict`，根据 bundle ID、schema 和 preview 版本检查本机数据库并按需重建。源码 `.mdx`、TXT 或旧 SQLite 通过以下制作入口转换：

```powershell
python -X utf8 scripts/build_dictionary_bundle.py <源词典文件> <输出路径.kdict> --name <词典显示名>
```

`build_dictionary_bundle.py` 提供参数入口，`dictionary_bundle.py` 编写源包，`dictionary_schema.py` 维护规范键；`bundle.rs` 建立 Rust 查询数据库与预览。资源路径按 [E1](desktop.md) 和 [E3](cli.md) 的宿主规则确定。

大辞林、小学馆和 Crown 由专用 adapter 解析。`DictEntry` 的 entry_key 标识源条目，occurrence_id 与 source_record_index 区分解析记录。header 保存表记、读音、词性、发音及词源；senses 保存真实层级、标签、释义和例句，sections 保存其他说明，links 保存参照关系。

`DictionaryContent.vue`、`DictionarySenseTree.vue` 和 `DictionaryExamples.vue` 渲染结构化正文。记录 preview 包含词头、元数据和内容可用性；`TooltipPanel` 遇到 `content_loaded=false` 时调用 `query_entry`，按源 entry key 缓存完整内容。清洗及兼容 HTML 由 `html.rs`、`bubble_html.rs` 和对应前端渲染协作。

## 悬浮与显示

阅读器固定布局点击激活，悬浮布局按正文命中打开词典及当前活用说明。词汇状态由 `reader_word`／`reader_mark` 等独立接口管理，词典内容由当前 target 查询。三种视图的关闭、尺寸与触摸见 [M3](explanation_layout.md)。

整体与内部通道各自维护加载、错误、历史和请求代次；同一外层词内移动时依据内部目标及活用详情更新内容。锚点卸载、对象失效、滚动或关闭由对应宿主处理。

当前外层与内部目标可并排显示，内部对象按最小 queryable 范围选择；与活用详情的显示组织见 L11。词典内容记录自身词性和读音，表头的原文信息来自 `readerTargetPresentation`。

内置语法查询将助词和助动词转换为同一 `DictionaryLookup`，词典名为「ReadUX 内置语法词条」。活用气泡另外由 operator concept 读取解说，两类内容通过本句成员和范围协作，资源职责见 [L6](language_analysis/entries.md)。

指南和静态书籍在制作阶段保存默认及各表记的词典结果，并预先加载正文。指南使用 `guideLinkedLookup` 查已发布的课节词条；静态书籍的 `search` 使用当前书籍 words 索引。可查询范围由对应快照及内容包决定，生成方式见 [E2](static_library.md)。

## 维护与实现入口

| 范围 | 入口 |
| --- | --- |
| 目标及元数据决定 | [targets.rs](../crates/readux-core/src/dictionary/targets.rs) |
| 查询、排序及缓存 | [lookup.rs](../crates/readux-core/src/dictionary/lookup.rs)、[aggregate.rs](../crates/readux-core/src/dictionary/aggregate.rs) |
| 表记组与内容协议 | [lookup_state.rs](../crates/readux-core/src/dictionary/lookup_state.rs)、[model.rs](../crates/readux-core/src/dictionary/model.rs) |
| 源包及数据库 | [bundle.rs](../crates/readux-core/src/dictionary/bundle.rs)、[build_dictionary_bundle.py](../scripts/build_dictionary_bundle.py) |
| 表记与词典选择 | [dictionaryMatrix.ts](../src/explanation/dictionaryMatrix.ts)、[TooltipPanel.vue](../src/components/TooltipPanel.vue) |
| 内容解析与绘制 | [adapters](../crates/readux-core/src/dictionary/adapters/)、[dictionary 组件](../src/components/dictionary/) |

已有验证包括 Rust 词典用例、`scripts/test_dictionary_schema.py`、`src/explanation/dictionaryMatrix.test.mjs` 和投影样例。代表对象为异读、假名、别名、无内容表记、整体与内部范围、词典顺序、表记切换和链接返回；按修改范围选择最终检查。

后续词组及登记模式可能提供新的整体和成员查询目标，收藏及 Anki 会确定要保存的词典字段；具体协议依对应模块方案讨论。
