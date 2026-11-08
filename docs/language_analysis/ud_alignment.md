# GiNZA 与 UD 对齐

更新日期：2026-11-09。本文依据应用适配器、来源协议、对齐模块及状态机调用，说明 GiNZA 信息怎样进入现有分析。基础坐标见 [L2](lexical_input.md)。

## provider 的输入与输出

[providers.rs](../../crates/readux-core/src/providers.rs) 管理 GiNZA 配置、Python 进程与执行，[nlp_provider.py](../../scripts/nlp_provider.py) 负责常驻请求，[nlp_adapters.py](../../scripts/nlp_adapters.py) 的 `GinzaAdapter` 组织来源产物。当前默认模型为 `ja_ginza`，实际模型和词典由配置决定。

适配器将准备后的共同正文直接传给 spaCy/GiNZA，保留模型正式输出的 token。适配器当前保存恒等正文映射；`norm_` 作为 token 特征保留。来源协议同时具备规范化文本、字符映射和删除范围字段，可表达其他来源的文本转换。

完整来源采用 `readux.source-analysis.v1`，包含 provider 清单、正文身份、`normalized_text`、`normalization_map`、节点、关系、诊断和耗时。清单记录模型、程序及词典版本、执行参数与能力。

| 节点或关系 | 适配器的实际输出 |
| --- | --- |
| 正式 token | `t<index>`，实际范围及 `lemma`、`norm`、通用 POS、日文 tag、morph、空白、子词和文节位置 |
| compound | 正式 token 内存在多项子词时生成，成员及主辞引用该正式 token |
| 文节 | `ginza.bunsetu_span` 的范围、token 成员和主辞 |
| 句子 | spaCy 句子范围、成员及 root |
| 小句 | `doc.user_data["clauses"]` 给出的成员、主辞和范围，完整协议允许多段范围 |
| 实体 | `doc.ents` 的范围、成员及模型标签 |
| UD 依存 | 每个正式 token 到其 `token.head` 的关系，保存 `token.dep_`；root 使用独立端点 |

`sub_tokens` 中的表面串、词元和读法作为正式 token 的特征保存。跨正式 token 的构词候选通过 `compound` 依存读取，详见 [L1](pipeline.md#各层对象的职责)。

## 主辞与依存方向

结构节点的 `head` 表示该结构内部的主辞。依存关系的 `source` 表示从属 token，`target` 表示其主辞 token，方向为从属项 → 主辞。句根的 target 为 `Root`。

例如一个助词通过 `case` 依存到名词时，助词是 source，名词是 target。具体谓语归属需要继续查看名词到谓语的关系。状态机当前使用的 `aux`、`cop`、`mark`、`advcl` 等标签分别承担局部接续证据，消费条件由实际调用决定。

完整来源协议还定义 `PredicateArgument`、`Coreference`、`Bridging`、`Discourse` 等关系类型。当前 GiNZA 适配器生成 dependency；其他类型的协议定义为扩展能力基础。

## 连续跨度与完整来源

[syntax.rs](../../crates/readux-nlp/src/syntax.rs) 的 `readux.syntax-artifact.v2` 保存连续跨度、主辞范围、表面串、标签及正文身份。`SourceArtifact.syntax()` 为统一结构合并提供该投影；完整节点和关系继续保存在 `external_sources`。

`SourceArtifact::from_syntax` 接收独立导入的连续跨度，保存 `model: syntax_import`，并记录 `syntax_import_without_dependencies`。原来的跨度标签可提供特征，完整依存关系需要通过完整来源协议输入。

[unify_with_sources](../../crates/readux-nlp/src/unify.rs) 在装配前校验每份完整来源。校验涵盖协议、坐标类型、正文身份、规范化映射、节点范围和表面串、成员及关系引用；无效输入返回错误。合法来源中的局部覆盖差异继续作为对齐及结构诊断保存。

## 与 UniDic 的多对多映射

对齐使用双方在共同正文中的字符交集，来源 token 与统一 token 分别保留身份。

[alignment.rs](../../crates/readux-nlp/src/alignment.rs) 生成逐来源 token 的结果，协议为 `readux.provider-token-alignment.v2`。

| 状态 | 判断 |
| --- | --- |
| `exact` | 来源范围对应一个完整 UniDic token |
| `compound` | 多个相邻 UniDic token 连续、完整覆盖来源范围 |
| `partial` | 存在重叠，包含 UniDic 内部切点或 gap 等边界差异 |
| `unmatched` | 范围内没有重叠的 UniDic token |

[alignment_group.rs](../../crates/readux-nlp/src/alignment_group.rs) 以来源 token 与 morpheme 为二部图节点，以字符交集为边，生成连通分量。每组保存两侧成员、逐项交集、总范围、两侧 gap 及 `1:1`、`1:n`、`n:1`、`n:m` 数量关系。

组状态 `complete` 表示两侧覆盖相同，允许双方具有相同 gap；`partial` 表示两侧覆盖不同；`unmatched` 表示单侧内容。完整分组与单个来源 token 的逐项映射可表达不同层次的事实，例如一组覆盖完整，组内个别边界仍切入 UniDic token。

`cover_range` 返回相交成员、交集、gap、完整性及原因。范围边界包含完整 token、存在成员且未覆盖部分均为空白时，可视为结构覆盖完整；构词候选及状态机另行执行自己的连续性条件。

## 统一来源图与结构选择

[structure_graph.rs](../../crates/readux-nlp/src/structure_graph.rs) 生成 `readux.structure-graph.v1`，在 `document_id` 内组织映射实体、类型化关系和结构候选。映射实体保存来源 ID、原始及共同正文范围、morpheme 引用、对齐组、成员、主辞和 `head_morpheme_ids`。主辞覆盖多个 UniDic token 时保留全部成员。

图构建核对部分覆盖、主辞与成员越界、compound 内 gap、来源句界及依存端点。存在局部问题的实体和关系保留 `complete` 及诊断。原始来源继续可供追溯。

当前选择策略为 `source-preference.v1`，优先 GiNZA。相同类型及相同范围的候选合并证据；同类重叠候选按策略选择，保留竞争 ID。小句和实体允许嵌套。完整模型来源存在时，选择结果应用到统一 `structure`。

`linguistic_context::Context` 直接读取完整来源列表中首份 `provider.id == "ginza"` 的来源。局部活用查询与结构图的选择属于不同调用路径，条件细节如下。

## 局部结构查询接口

接口位于 [linguistic_context.rs](../../crates/readux-nlp/src/linguistic_context.rs)。

| 接口 | 当前语义 |
| --- | --- |
| `tokens(range)` | 返回与范围相交、具有单段正文范围的正式 token |
| `evidence(range)` | 为相交正式 token 生成 `SourceEvidence` |
| `relation(left, right, labels)` | 检查两端是否存在指定标签关系，接受两个方向 |
| `depends_on(dependent, head, labels)` | 严格检查从属项 → 主辞方向 |
| `same_word` | 一个正式 token 完整包含所给左右范围 |
| `same_bunsetsu` | 一个来源文节包含左右范围 |
| `auxiliary` | 相交正式 token 的通用 POS 包含 AUX |
| `tag`、`inflection` | 读取与范围完全相同的正式 token 的日文 tag 或 Inflection |
| `functional_position` | 正式 token 的文节位置为 SYN_HEAD 或 FUNC |
| `verb_core` | 严格匹配范围、VERB、日文动词 tag、词元前缀和可解析活用 |
| `core_members` | 正式 token 起点与当前 morpheme 一致，完整、连续覆盖多个允许词类时返回核心成员 |

`SourceEvidence` 保存 provider、可选节点 ID、可选关系 ID 和原因。相交查询取得的证据可覆盖多个正式 token；`tag`、`inflection` 等要求完整范围的接口提供更严格的读取条件。

## UD 在活用中的实际使用

普通连接规则可通过 `auxiliary_relation` 要求 `aux`／`cop` 两端关系，通过 `functional` 接受非自立 POS、GiNZA AUX、功能位置或已确认的辅助关系。サ变条件可使用 GiNZA 日文 tag。UniDic 缺少活用形时，完全同范围的 GiNZA `Inflection` 可以补充状态，并记录相应证据。

正式词界可确认复合核心。名词＋助动词等特殊分词由 `verb_core` 的精确条件恢复；已识别的名词化、副词述语化、补助和插入接续具有专门的组链处理。来源词界继续保存为构词证据。

### て形后的插入补助

「読んではいる」等结构先核对连接器、插入成员、后项本体及字符连续性。存在 GiNZA 时，`confirms_inserted_support` 同时要求：

| 从属范围 | 主辞范围 | 允许标签 |
| --- | --- | --- |
| て／で连接器 | 前项词汇核心 | `mark` |
| 补助词 | 同一前项核心 | `advcl`、`aux`、`xcomp` |
| 每个插入成员 | 同一前项核心 | `case`、`mark` |

三组条件通过 `depends_on` 检查方向。`bridge_support_for_position` 在没有 GiNZA 来源时允许按词法及位置声明继续；基础 Analyze 因而可以识别该结构。完整单元已有 GiNZA 时会执行上述依存条件。

连续词尾与插入补助竞争时，执行器先确认完整插入区间和补助入口，符合条件后选择插入连接。「たりとか」等具有自身前项规则的连续形式继续按登记条件处理。

## 冲突、待定与后续用途

来源正文身份错误属于输入错误。合法来源的边界差异、结构竞争和不完整覆盖保存在映射与诊断中。状态机发现辅助关系与本地连接条件冲突时可记录 `source_connection_conflict` 并将当前对象标为 `candidate`；路径零命中或多命中同样保留候选和诊断。

当前状态机按明确的局部条件使用 UD。后续词组、格关系及语法模式模块将另行确定关系组合、冲突处理及具体用法规则。UD 标签逐条对应的完整 JSON、复杂句中的端点选择与正反例，列入 [L5 的规则细节交接](morphology_configuration.md#规则细节交接)。
