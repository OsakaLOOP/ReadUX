# 活用状态机

更新日期：2026-11-09。本文说明当前实体、执行过程和结果契约。规则维护见 [L5](morphology_configuration.md)，词条与解说见 [L6](entries.md)。

## 输入与基本对象

[morphology.rs](../../crates/readux-nlp/src/morphology.rs) 的 `collect_with_external` 接收 UniDic 来源 token、统一 morpheme 及完整外部来源，先检查双方数量、范围和表面串，再调用 `collect_atoms` 与 [morphology_machine.rs](../../crates/readux-nlp/src/morphology_machine.rs) 的 `compose`。结果协议为 `readux.morphology-artifact.v3`。

| 对象 | 含义及主要字段 |
| --- | --- |
| `MorphologyState` | 当前 `category`、连接形 `form`、精确 `conjugation_type`、原始 `conjugation_form`、`entity_class` |
| `MorphologyChain` | 核心及成员、词汇／功能角色、实际词形、查询与显示形、operators、最终状态、父链和来源证据 |
| `MorphologyOperator` | 一次操作的 kind 与实例 ID、成员及字符范围、前后状态、concept、原型、执行元数据及 connection |
| `MorphologyConnection` | 具体规则及命中路径、局部前项范围、连接器实例、上下文范围、结构分类及诊断 |
| `MorphologyOccurrence` | 可解释的一次出现；引用 chain 和 operators，保存命中区间、上下文、成员、候选及状态 |

`kind` 关联配置中的 operator 定义，`operator_id` 标识本句中的一步。原子 `conjugation`、`initial_alternation`、`final_alternation` 根据 UniDic 字段生成；配置 operator 继续描述具体语法作用。

## 六类实体与执行器

六类实体定义在配置 `entity_classes`，用于分类和状态说明。运行分派进一步包括名词化、副词述语化和组合执行器。

| 实体类别 | 典型操作 | 执行后的关键状态 |
| --- | --- | --- |
| 动词循环类 | 使役、被动／可能、礼貌、时态、意向等 | 继续依据当前类别、活用型和活用形接续 |
| イ形容词转换类 | たい、ない、やすい、にくい、典型性らしい | 形成形容词性状态，后续采用对应活用 |
| ナ形容词判断类 | そう、よう、みたい、はず及判断词尾 | 区分判断词干和收尾，保存当前谓语类别 |
| 文言活用类 | 文言用言范式及文语助动词 | 使用文言活用形与前接许可 |
| 补助动词类 | て形、连用形、たり、意向和变化等补助接续 | 建立功能子链，读取后项本体活用 |
| 插入语类 | 指定位置的は、も、だけ、とか等 | 扩展所属范围，继承前项状态和停机许可 |

当前九种配置分派值为 `verb_cycle`、`i_adjective_conversion`、`verb_cycle+i_adjective_conversion`、`na_adjective_judgement`、`nominalization`、`auxiliary_verb`、`classical_conjugation`、`insertion`、`adverbial_predicate`。补助构式使用独立入口调用补助执行器；通用分派中的 `te_connection` 负责连接形准备。

原始词性、实体类别和链角色各有用途。词元名列补助本体表时可以具有补助实体类别，真实补助接续成立后才由执行器设置功能角色及父链。

## 状态、顺序与停机

连接形为 `stem`、`irrealis`、`continuative`、`terminal`、`attributive`、`conditional`、`imperative`、`volitional`、`te`、`other`。初始化通过有序 `forms` 表映射 UniDic 活用形，文言输入先检查专用表。

局部输入由当前类别、活用形、精确活用型及最近有效语法步骤判断。`stage`、`order_rank` 和规则组保存说明及投影信息；规则数组顺序参与实际匹配。「たりする」后接否定、过去后接名词化、认识形式后接变化补助等结构可沿新状态继续组合。

`input_category_mode` 可精确匹配、接受任意类别或使用判断谓语兼容关系。`exclude_input_ctypes` 采用完整名称匹配，例如「助動詞-タ」与「助動詞-タイ」分别判断。插入步骤查找前项时可跳过保持状态的 insertion。

配置停机模式为 `continue`、`sentence_or_pause`、`pause_or_continue`、`preserve`。`terminal` 记录可停机性；判断与文言执行器还结合实际终止、连体、条件、命令及连用状态判断。后续成员仍需满足自己的局部条件，字符不连续和明确边界规则结束当前组合。

## 组链过程

1. 从每个有效 token 建立原子链，保存原始活用、词首及词尾变化、查询形和基础状态。
2. 按正文顺序读取原子，处理登记的分词恢复、连续词尾和 GiNZA 活用补充。
3. 对连续前项计算局部转移，并结合正式词界确定核心、附属、补助、插入或边界。
4. 执行所属执行器，合并成员或建立父子链，保存状态、概念和来源证据。
5. 后项谓语完成自身活用后，关联义务或禁止等 `predicate_links`。
6. 识别具体接续路径及连接器，生成 occurrence、状态和诊断。

连续词尾通常先于逐 token 连接匹配。「でし＋た」「ませ＋ん」「か＋も＋しれ＋ない」由完整序列取得形式身份，`head` 指定提供词元或活用信息的成员。插入补助等特殊入口先确认完整结构，再决定与连续词尾的选择。

## 各执行范围

### 动词循环与形容词转换

动词循环执行器处理动词范畴内的许可。组合分派依据输出类别或前项形容词状态选择动词、形容词执行器。`たい`、否定 `ない`、难易接尾辞等输出形容词状态后，后续否定、过去和其他连接读取新状态。

`読みたい→読みたくない／読みたかった` 的词汇查询继续归「読む」，愿望及后续操作保存各自范围与概念。可能动词的词元恢复另由 `potential_lemma` 确认，见 [L2](lexical_input.md#查询形式与词元恢复)。

### 判断与名词化

判断执行器处理词干及「だ／です／だった／でした／で／な／に」等收尾。名词、代词形成判断谓语后继续保留相应类别和判断身份；副词的本句判断可转为形状词状态。はず输出名词，らしい按其规则保持形容词活用。

「女の子でした」的 `copula_past` 覆盖完整「でし＋た」，本句原型为「でした」，输入对应名词词干。「女の子」的词典查询、完整判断过去说明和图中成员活用分别由各自字段提供。

样态そう与传闻そう根据前项状态生成 `judgement_usage:appearance` 或 `hearsay` 等证据。词条摘要按 concept 读取，具体证据的显示取用见 L6；这些证据和概念义项 ID 分别保存。

名词化执行器将「の／こと／さ」等规则输出为名词状态，后续可连接判断或变化形式。「気付きたくなかっただけなのだ」中的だけ保持前项状态，な、の、だ依次执行自己的接续。名词化路径显式关联需要的な连接器。

### 补助父子链与递归

补助执行器统一处理配置中具有 `support` 的用法。连接组包括 `te`、`contracted`、`continuative`、`enumerative`、`volitional`、`nominal_change`、`adjectival_change`，分别校验相应连接器和当前形态。

后项建立 `parent_chain_id`，角色为 functional；后项词元、查询形及活用型保留自身身份。「諦めてくれ」中，词汇链提供「諦める→諦め」及て连接器，补助链提供「くれる→くれ」及命令形。图示按父子链组合完整谓词家族。

「読んでしまった」中的しまう继续接过去；「名词＋に＋なる」保留前项名词类别，なる采用自己的动词活用；「く形＋なる」使用同一补助入口。ほしい、许可いい等补助形容词采用形容词状态。

递归深度统计父子链中的补助构式，配置上限为 8。连接器、插入成员不单独增加补助层数。输入、连接形、连续性或深度不符合时保留 candidate 与诊断。

### 插入成分

插入成员由词性、助词小类及基本表记白名单共同限定；四组位置约束て形补助、谓语末尾、名词后和连用形焦点成分。多个成员逐项生成 operator，继承词汇核心、查询原型、完整状态和停机许可。

て形后的插入补助存在 GiNZA 时检查连接器、每个插入成员、补助词到同一谓词的有向依存；基础词法入口缺少 GiNZA 时使用登记的词法和位置条件。具体标签和方向见 [L3](ud_alignment.md#て形后的插入补助)。

插入 occurrence 的命中范围为自身成分，context 指向所属链上下文；补助 occurrence 收集连接器、插入成员和补助词的实际命中范围。「たりとか」采用连续序列身份，保持たり到する／できる的接续许可。

### 文言

文言执行器读取 `classical_forms` 与 `classical_auxiliaries`，保存词形、范式和前接证据。接续同时检查前项类别、活用形和活用型，例如り的サ变未然形与四段已然形分别声明。なり的断定／传闻、たり的完了／断定按规则身份区分。

现代べき后接だ／です时形成判断词干；其他文言べし、べく、べき按文言条件处理。「ごとく＋なり」组合保留完整成员并继承ごとし之前的输入状态。文言词条、concept、路径和前端投影沿用共用协议。

### 副词接续

副词资格来自已审定词元映射、明确 POS 及本句判断证据。`adverb_suru`、`adverb_to`、`adverb_to_suru` 使用专门分派，分别处理动词化与状态修饰。な／だ和タリ相关形式继续使用相应执行入口。未审定与已审定 NONE 分别保留。

## 词形与所有权

链分别保存词汇本体、实际词面、辞书形、词元形、查询形和显示形。operator 的 `normalized_form` 保存分析阶段选择的本句原型；缩约补助的查询形采用展开本体，实际缩略、清浊、音便及活用保存在成员与证据中。

补助连接器通常归父链，后项本体及后续活用归子链。句中图、词典查询和活用说明通过父子关系取得完整上下文。`source_morpheme_range` 是成员索引区间，`char_range` 是正文字符区间；两者分别用于读取来源字段和定位正文。

## 路径、范围与 occurrence

链形成后，`identify_connection_paths` 匹配 `connection_rules`，核对状态、前后成员、最近 operator、证据和必要资格。全部命中保存在 `matched_paths`，唯一命中写入 `path`；零命中或多命中记录 `connection_path_count`，链标为 candidate。

| 范围或引用 | 当前用途 |
| --- | --- |
| operator 的 `char_range` | 当前步骤自身成员 |
| connection 的 `predecessor_range` | 具体路径所需的局部前项 |
| `connector_operator_id` | 同链或父链中的连接器步骤 |
| connection 的 `context_range` | 含连接器及合法插入成员的局部上下文 |
| occurrence 的 `hit_ranges` | 实际解释命中的有序区间 |
| occurrence 的 `context_range` | 沿父链回溯后的谓词上下文 |

连接器继承时，前项范围读取连接器的输入；上下文延伸到后项前，连接器节点保持独立。「名词＋な＋の」与「动词＋て＋补助」均可据此生成完整接续图。

occurrence 从非原子 operator 生成，保留相关实例 ID、成员、候选和来源证据。`conjugation` 等三个原子特征另供词干及词形说明使用。

## 状态与维护依据

常规链保存 `observed` 等状态；输入约束、路径冲突和超限可形成 `candidate`；未连接的助动词保留 `pending` 并记录 `unattached_auxiliary`。候选字段可保存被动／可能等解释选项，语义选择按实际执行证据理解。

现有验证入口包括 [morphology_reconstruction.rs](../../crates/readux-nlp/tests/morphology_reconstruction.rs)、[morphology_stages_5_6.rs](../../crates/readux-nlp/tests/morphology_stages_5_6.rs)、[p4_language.rs](../../crates/readux-core/tests/p4_language.rs) 及前端投影样例。既有执行计划和覆盖表用于追溯阶段记录，当前契约以本文、L5 和实际实现为准。

逐条规则的完整条件、JSON 和复杂例句留待下一轮细化，范围集中见 [L5](morphology_configuration.md#规则细节交接)。
