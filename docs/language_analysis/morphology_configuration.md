# 活用规则与配置维护

更新日期：2026-11-09。本页说明字段用途、限定集合、源文件编排及编译约束。按学校语法和实际功能细分的规则编写方法见[规则分类与编写指南](morphology_rule_categories.md)，名称、讲解及 D3 图规范见[规则显示规范与修订设计](morphology_display_conventions.md)。执行过程见 [L4](morphology.md)，词条资源见 [L6](entries.md)。

## 维护入口与产物

正式维护入口为 [configs/unidic-nlp/morphology/source/](../../configs/unidic-nlp/morphology/source/README.md)。[build_morphology_config.py](../../scripts/build_morphology_config.py) 读取明确列出的源文件，生成 [morphology.json](../../configs/unidic-nlp/morphology.json)，协议为 `readux.morphology-config.v1`。

Rust 的 [morphology_config.rs](../../crates/readux-nlp/src/morphology_config.rs) 编译时嵌入产物，前端 [morphologyConfig.ts](../../src/reader/morphologyConfig.ts) 导入同一文件。两端分别建立 operator 到规则组及接续分类的索引。运行规则修改后，需要重新编译分析服务；现成书籍和指南分析另行重新生成。

## 源文件分工

下表路径均相对于 `configs/unidic-nlp/morphology/source/`。

| 文件 | 维护内容 |
| --- | --- |
| `base.json` | 补助深度、三个原子特征、活用形映射、输入恢复 |
| `operators/verb.json` | 动词循环与动词／形容词组合分派 |
| `operators/conversion.json` | イ形容词转换及名词化 |
| `operators/judgement.json` | 判断词干及词尾 |
| `operators/auxiliary.json` | 补助本体、连接器和缩约用法 |
| `operators/classical.json` | 文言 operator |
| `operators/insertion.json` | 插入成分及举例形式 |
| `operators/adverb.json` | 副词述语化与状态修饰 |
| `rules/features.json` | 单 token 的功能识别与排除条件 |
| `rules/connections.json` | 相邻成员的有序组链动作 |
| `rules/sequences.json` | 完整连续词尾及其核心成员 |
| `rules/predicate_links.json` | 已形成谓语之间的构式关联 |
| `rules/insertion.json` | 插入白名单及四组允许位置 |
| `classical/forms.json` | 文言用言范式 |
| `classical/auxiliaries.json` | 文言助动词、词形与具名 inputs |
| `paths/explicit.json` | 独立维护的接续路径 |
| `paths/classical.json` | 引用文言 inputs 的路径展示分支 |
| `paths/predicates.json` | 引用谓语构式前项的路径 |
| `adverbs/reviewed.json` | 发布到运行配置的已审定词元资格 |
| `adverbs/unidic_multi.json` | UniDic 同词元资格与同形同音词元关联资料 |
| `presentation.json` | 六类实体、规则总览树和展示配置 |

`operators` 按执行器分文件，编译器核对归属。学校语法类别和具体功能通过规则内容说明，例如 `auxiliary.json` 同时包含动作阶段、授受、趋向和感情评价等规则。规则数组集中维护完整顺序；路径源文件分别承担显式条件、文言输入引用和谓语前缀继承。当前源目录共有 21 个 JSON 文件，日常编译直接读取其中 20 个；`adverbs/unidic_multi.json` 是资格整理资料。

## 运行配置基准

2026-11-09 的已保存运行产物有 21 个顶层字段、153 个 operator、92 组接续规则和 455 条路径。operator 默认及变体共引用 125 个 concept，均有编译解说。数量用于描述本次文档依据，后续规则按协议维护。

| 区段 | 当前内容及职责 |
| --- | --- |
| `schema`、`max_auxiliary_depth` | 协议版本；补助递归上限 8 |
| `intrinsic_features` | conjugation、initial_alternation、final_alternation 三种原子特征 |
| `operators`、`rule_metadata` | 153 份定义和对应执行元数据 |
| `connection_types` | direct 86、connector 31、sequence 18、predicate 12、inflection 4、insertion 2 |
| `connection_rules` | 92 组、455 条路径 |
| `features`、`connections`、`sequences`、`predicate_links` | 分别为 48、48、30、12 条 |
| `forms`、`input_recoveries` | 9 条活用形映射、1 条输入恢复 |
| `classical_forms`、`classical_auxiliaries` | 5 个范式、32 条助动词 |
| `insertion`、`insertion_positions` | 1 组成员白名单、4 组位置 |
| `entity_classes`、`rule_tree` | 六类实体及六个规则树根节点 |
| `presentation` | 10 个显示配置区段 |
| `adverb_capabilities` | 2,595 个词元资格映射 |

原子特征与接续分类中的 inflection 分别维护。后者包含意向、条件、命令和文言词内活用 operator。

## definition 与 execution

每个 operator 源条目有四个组成字段：`definition`、`execution`、`connection_type`、`connection_rule`。编译器分别生成定义、执行元数据、分类成员和规则归属。

`definition` 保存 `label`、中文 `title`、`concept`、`prototype`，以及可选的 `display_forms`、`variants`、`token_lemma`、`normalized`、`candidates`、`support`、`compound_candidate`、`connector`、`connector_alternatives`、`display_suffix`。

`execution` 保存 `stage`、`executor`、`rule_group`、`input_label`、`input_category_mode`、`input_categories`、`input_forms`、`exclude_input_ctypes`、`output_category`、`output_form`、`preserve_state`、`terminal_mode`、`order_rank`。其中 `input_label` 供说明使用；机器读取类别和状态条件。

| 输入模式 | 当前判断 |
| --- | --- |
| `any` | 接受任意类别，继续检查其余条件 |
| `exact` | 当前类别属于 `input_categories` |
| `judgement_compatible` | 接受显式类别，以及已经具有判断身份的兼容谓语状态 |

`input_forms` 为空时不增加活用形限制。`exclude_input_ctypes` 按完整名称排除。`exact` 配合空的 `input_categories` 在通用判断中接受零个类别；现有文言规则采用此写法，实际许可由 `classical_auxiliaries.inputs` 的专用判断提供。

`preserve_state` 先继承前项状态，`output_form` 再更新连接形。词类变化、实体身份及特殊词形继续由专用执行器处理。`output_category` 采用 UniDic 类别或执行器支持的继承值。补助词采用自己的活用型和活用形。

## token 条件与顺序

`TokenPattern` 支持 `equals`、`starts_with`、`contains`。每种模式内不同字段同时成立，同一字段的值数组任选；不同模式同时成立。空对象表示无额外条件。

| 条件字段 | 读取位置 |
| --- | --- |
| `pos`、`sub`、`detail` | UniDic 一级、二级、三级 POS |
| `ctype`、`cform` | `cType`、`cForm` |
| `lemma` | UniDic 词元 |
| `base` | 优先 `orthBase`，缺失时取 lemma |
| `surface` | 本句实际 token 表面串 |

features 可以通过 `exclude` 排除重叠形式。普通 features 累计全部命中的 kind 并去重；文言助动词专用识别成功时直接返回对应 kind。connections 的 `when` 数组任选，各条件内部同时检查 next、last、following、状态、最近 operator、功能或关系等字段。组链动作包括 `core`、`attach`、`support`、`stop`。`stop` 明确保留成员边界。

sequences 依据当前状态、完整 token 模式和字符连续性匹配；`head` 默认为首项，指定词元及活用身份来源，`terminal_only` 限制输入结尾或允许的句末标记，`exclude_following` 为后续构式保留边界。predicate_links 在后项完成活用后核对前项 prefix、后项身份及必需的否定等操作。

features、connections、sequences、predicate_links、forms、classical_forms、classical_auxiliaries、input_recoveries、variants 和 paths 保留数组顺序。connections 选择首个有效动作，sequences 选择首个完整匹配；paths 保留全部命中。stage 与 order_rank 用于说明及投影；实际匹配优先级来自执行入口和相应有序数组。

## 一个连续词尾示例

以下为 `rules/sequences.json` 中「でした」分支的完整条目，采用紧凑排版：

```json
{
  "categories": ["名詞", "形状詞", "代名詞", "形容詞"],
  "kind": "copula_past",
  "forms": ["stem", "terminal", "attributive"],
  "tokens": [
    {
      "equals": {
        "pos": ["助動詞"],
        "surface": ["でし"],
        "ctype": ["助動詞-デス"]
      }
    },
    {
      "equals": {"ctype": ["助動詞-タ"], "pos": ["助動詞"]},
      "starts_with": {"cform": ["終止形", "連体形", "基本形"]}
    }
  ]
}
```

该条目确认前项状态及两个连续成员。`operators/judgement.json` 的 `copula_past` 另外声明 `na_adjective_judgement`、`connection_type: sequence`、`connection_rule: judgement_past` 及 concept `grammar.auxiliary.da`。

同一 operator 的 `definition.variants` 包含以下形式选择项：

```json
{
  "token": {"equals": {"surface": ["でし"]}},
  "observed_form": "でした",
  "form": "でした"
}
```

连续词尾识别、执行元数据、接续路径和显示原型分别承担成员匹配、状态变化、前接说明和词形显示。本句识别结果通过同一 kind 关联这些资源。

## 连接规则与路径

每组运行 `connection_rules` 保存 `label`、`operators` 和有序 `paths`，可选 `scope`、`source`、`ctype_mode`。`scope: inflection` 读取词内状态；普通路径读取局部前项。`ctype_mode` 默认按前缀匹配，`classical_prefix` 兼容文言活用型的「文語」前缀。

path 的基本显示字段为 `id`、`group`、`label`、`steps`；机器条件包括类别、连接形、活用型及排除、原始活用形、前序 operator、evidence、next、前项 token 模式、following、exclude_following 和 capabilities。全部命中写入 `matched_paths`，唯一命中写入 `path`。

连接器分两种引用方式：operator 的 `connector`／`connector_alternatives` 关联共享入口；具体 path 的 `connector` 和 `connector_paths` 限定自身需要的连接器及已命中路径。`te_iru` 等补助用法继承て连接器；「名词＋な＋の」的名词化路径引用な及其路径。

`connection_rule` 填写直接所属规则组；通过连接器取得路径的 operator 可填写 null。运行连接记录保留 `predecessor_range`、`connector_operator_id`、`context_range` 和 `dependency_type`。`connection_types` 描述接续结构，`dependency_type` 是本次路径的实际结构分类。

### 路径编译

显式路径按源声明编译。文言路径通过具名 input 引用 `classical/auxiliaries.json`，编译器复制类别、形式、活用型及排除条件，生成对应显示分支。谓语路径通过 predicate 引用 `predicate_links`，沿 prefix 取得已有前项路径并复制。循环引用、归属不一致及无效输入由编译器报告。

## 文言、插入及补助声明

文言 forms 按范式保存词元词尾、活用型和实际词形；auxiliaries 保存助动词完整词形及 inputs。inputs 内的类别、活用形和活用型同时成立，多组 inputs 任选。なり、たり等多用法分别声明规则身份。

插入白名单与位置由 `rules/insertion.json` 统一维护；实体判断和运行识别读取同一成员条件。位置可限制类别、形式、基本表记，或声明 connector 与 support_group。具体 UD 方向及缺少来源时的行为见 [L3](ud_alignment.md#て形后的插入补助)。

补助定义的 `support` 保存 group、bases 及可选 expanded。bases 是本体身份，expanded 为缩约展开词形和读法。定义中的 connector 指向前项连接器；后项活用与词典查询继续使用其本体。`compound_candidate` 控制相应整体查询候选。

## 副词接续资格

[build_adverb_lexicon.py](../../scripts/build_adverb_lexicon.py) 提供词元采集、资格来源整理及 `compile_reviewed`。运行配置编译直接读取 `adverbs/reviewed.json`，要求记录具备 reviewed 状态、理由和合法 flags，再生成按 lemma_id 索引的映射。

`adverbs/unidic_multi.json` 保存候选来源和同形同音关联，供资料整理使用；运行映射由 reviewed 文件编译。原始 flags 对应 none、suru、na_da、to_suru、tari，none 与正向资格互斥。词元 ID、词元和读法共同限定登记身份，运行读取方式见 [L2](lexical_input.md#基础身份与接续资格)。

## 原型和 presentation

Rust 在 `normalized_form` 保存本句原型。显示形先取首个 `observed_form` 与完整实际词面相同的 variant，此分支仅比较完整词面；随后依次取 `normalized`、首个 token 条件命中的 variant.form、`token_lemma` 指定的词元、默认 prototype。`concept` 单独选择首个 token 条件命中的 variant.concept，缺省取定义 concept。完整词面选形与 token 选概念是两次选择。

前端 `morphologyPrototype` 优先读取本句 `normalized_form`，静态条目由 `configuredPrototype` 取得默认 normalized／prototype。实际词面、音便、缩略和成员继续保存在分析对象中。

| presentation 字段 | 用途 |
| --- | --- |
| `sentence_compounds` | 句中图合并显示的完整复合条目，现用于かもしれない |
| `sentence_layers` | 句中组成的展示分组及折行依据 |
| `contraction_label` | 缩略形式文案 |
| `observed_steps` | 指定推导步骤采用实际词形 |
| `phenomena` | 依据 cForm 和证据生成音便、插入或缩略标签 |
| `form_labels`、`raw_form_labels` | 连接形及原始活用形名称 |
| `families`、`category_labels` | 活用型家族及类别名称 |
| `label_aliases` | 显示别名 |

families 与 raw_form_labels 采用有序 `{prefix, label}` 数组，首个匹配决定标签。对象键排序只影响资源组织；数组顺序保留实际语义。

## 字段编排与限定集合

本节的“限定集合”指当前编译器或消费代码支持的值；“编写约定”指本次为同类规则确定的维护规范。JSON 对象的字段顺序方便人工阅读，编译产物会按键名排序。各类规则的具体取值组合见[分类指南](morphology_rule_categories.md#各类规则的元数据组合)。

### operator 条目的固定编排

源条目统一按 `definition → execution → connection_type → connection_rule` 编排。definition 内依次写名称、概念、词形、语义候选、补助本体和连接器；execution 内依次写执行身份、输入、输出及停机信息。以下示例来自 `operators/conversion.json` 的 desire，仅调整字段排列：

```json
{
  "desire": {
    "definition": {
      "label": "愿望",
      "title": "愿望",
      "concept": "morphology.modality.desire",
      "prototype": "たい"
    },
    "execution": {
      "executor": "i_adjective_conversion",
      "stage": "desire",
      "rule_group": "主观愿望",
      "input_label": "動詞",
      "input_category_mode": "exact",
      "input_categories": ["動詞"],
      "exclude_input_ctypes": ["助動詞-マス", "助動詞-タ", "助動詞-デス", "助動詞-ダ"],
      "output_category": "形容詞",
      "terminal_mode": "continue",
      "order_rank": 40
    },
    "connection_type": "direct",
    "connection_rule": "continuative"
  }
}
```

`input_label` 的显示用语按新规范写作“动词连用形”；上例保留当前字段值，便于对照。输入连接形在此规则的 connections 和 continuative 路径中声明。新增 `input_forms` 会增加运行约束，需结合真实分词决定。

### 名称、概念与词形字段

| 字段 | 类型、必填性与用途 | 编写约定及取值范围 |
| --- | --- | --- |
| `label` | 必填字符串；保存到本句 operator.label，参与步骤投影 | 简短中文功能名，采用[统一名称](morphology_display_conventions.md#名称与文字的编写约定) |
| `title` | 必填字符串；活用词条的用法标题及部分图节点文字 | 与 label 使用同一功能名称；具体用法依据已确认的证据细分 |
| `concept` | 必填字符串；关联解说 | 引用已存在的 concept ID；编译检查有效功能摘要及例句 |
| `prototype` | 必填字符串；默认日语形式 | 记录形式或构式原型；词内活用、通用插入等现有条目允许空串 |
| `display_forms` | 可选字符串数组；登记可选显示形 | 存在非空登记表时，每个 variant.form 均须在表内 |
| `variants` | 可选有序数组；选择显示形或 concept | 每项必填 `token`、`form`；可选 `observed_form`、`concept`。具体到一般排列 |
| `token_lemma` | 可选布尔值，默认 false | 需要按本句词元展示同类本体时使用，如切る／抜く／尽くす |
| `normalized` | 可选字符串；优先于 token 选形的统一形式 | 用于ている、てしまう等规范形式；与需要保留差别的 variants 协调 |
| `candidates` | 可选字符串数组；本次可能解释 | 当前使用受身、可能、尊敬、自発等；编译器检查字符串类型，内容按具体规则规定 |
| `support` | 可选对象；声明补助本体 | 必填 `group`、非空 `bases`；可选 `expanded: [词形, 读法]`，恰好两项 |
| `compound_candidate` | 可选布尔值，默认 false | 允许相应整体词汇查询候选；按具体构词与查询要求决定 |
| `connector` | 可选 operator ID | 指向实际连接器，如 te_connection、enumerative |
| `connector_alternatives` | 可选 operator ID 数组 | 同一构式允许的其他连接器，如 ni_naru 的 adverbial_ni |
| `display_suffix` | 可选字符串 | 当前判断词干接な等形式时使用的显示词尾 |

`support.group` 当前由执行器解释的集合为 `te`、`contracted`、`continuative`、`enumerative`、`volitional`、`nominal_change`、`adjectival_change`。编译器检查组内 `(group, base)` 的唯一归属，group 本身按字符串读取；新增组名需要相应执行逻辑。普通 bases 使用 `orthBase`，缺失时采用 lemma；可能补助形式另有已确认词元的匹配入口。

### execution 的逐字段要求

| 字段 | 当前协议 | 同类规则的编写要求 |
| --- | --- | --- |
| `executor` | 必填字符串；九种实际分派值见 L4 | 与目标状态变化一致，且符合 operator 所在文件的许可 |
| `stage` | 必填字符串，编译器检查类型 | 复用同功能现有阶段；保持与实际匹配顺序的独立性 |
| `rule_group` | 必填字符串，编译器检查类型 | 表示细分功能或接续家族，如“连用形补助”“名词化”；建议名称见分类指南 |
| `input_label` | 必填字符串，可为空；仅作说明 | 新条目采用“词类＋连接形＋必要资格”的短语，如“具有サ变资格的副词词干” |
| `input_category_mode` | 必填；`any`、`exact`、`judgement_compatible` | 精确词类用 exact，判断谓语共享许可用 judgement_compatible；any 用于真正保留任意前项的规则 |
| `input_categories` | 必填字符串数组 | 从下述八类中选择；前项是组合后的当前类别 |
| `input_forms` | 可选连接形数组，默认空 | 空表示该通用检查不限连接形；sequences.forms 为另一处独立且必填的约束 |
| `exclude_input_ctypes` | 可选字符串数组，默认空 | 完整活用型名称精确排除；用于阻止特定助动词继续错误叠加 |
| `output_category` | 必填；八类或 `用言`、`保持前项` | 按执行器的实际支持填写，不能将允许字符串视为各执行器的通用能力 |
| `preserve_state` | 可选布尔值，默认 false | 复制前项完整状态；插入成分和 te_connection 是典型用法 |
| `output_form` | 可选连接形 | 在状态继承之后设置；省略时保留当前成员或执行器决定的形式 |
| `terminal_mode` | 必填；`continue`、`sentence_or_pause`、`pause_or_continue`、`preserve` | preserve 继承前项停机许可，其余结合执行器决定可停机性 |
| `order_rank` | 必填整数，编译器未限制区间 | 同功能规则复用既有等级；当前为 0、10、20、30、31、37、40、45、50、60、70、72、80 等，不作为识别优先级 |

输入类别限定为 `動詞`、`形容詞`、`形状詞`、`名詞`、`代名詞`、`助動詞`、`文語用言`、`副詞`。`用言` 和 `保持前项` 是输出端的执行器指示值。原始 token 的 `助詞`、`接尾辞` 等 POS 可以出现在 token 条件中；这些名称各自属于不同字段。

连接形限定为 `stem`、`irrealis`、`continuative`、`terminal`、`attributive`、`conditional`、`imperative`、`volitional`、`te`、`other`。文言表以日语活用形为键，当前使用 `未然形`、`連用形`、`終止形`、`連体形`、`已然形`、`命令形`；连接形中的 conditional 同时承接仮定形与已然形，显示时需要原始 cForm 区分。

当前 stage 使用 `voice`、`desire`、`polarity_politeness`、`tense`、`modality`、`te_support`、`conversion`、`boundary`、`classical`、`insertion`。该集合是既有内容的枚举，编译器按字符串校验。语法分类的统一优先通过 rule_group 和功能名称完成，stage 的取值调整需核对投影使用。

### 各执行器实际读取哪些元数据

同一个字段的协议许可范围与具体执行器的能力分别核对。下表用于判断同类规则可以采用哪些元数据组合。

| executor或分支 | 输入条件的实际读取 | 输出字段的实际作用及编写约定 |
| --- | --- | --- |
| verb_cycle | 调用通用accepts_input | 处理動詞、名詞及用言继承；按所需行为选择这三类输出，其他字符串的作用需核对实现 |
| i_adjective_conversion | 调用通用accepts_input | 最终强制为形容詞；output_category统一填形容詞 |
| verb_cycle+i_adjective_conversion | 按output_category为形容詞或前项为形容詞选择转换分支，否则采用动词分支 | negative填形容詞，past填用言；分支选择会影响实际输出 |
| na_adjective_judgement | 调用通用accepts_input | 处理形状詞、名詞、用言或保持前项等；そう的传闻分支有前项类别继承处理 |
| nominalization | 调用通用accepts_input | 最终强制为名詞并清除实体类别；output_category统一填名詞 |
| auxiliary_verb的补助本体 | 调用通用accepts_input，另核对support组、连接器和连续性 | 直接采用具体output_category；当前动词、形容词及ごらん的名词性输出分别声明 |
| auxiliary_verb的te_connection | 通用分派仅调用declared_output；前接许可来自连接规则 | 依preserve_state继承前项，output_form设置te；名义输入类别不构成该分支的通用检查 |
| adverbial_predicate | 当前专门核对SURU或TO_SURU资格，通用输入字段没有单独调用accepts_input | adverb_to继承状态并设other，其余输出動詞；填写与该实际行为一致的元数据 |
| classical_conjugation | 实际使用文言词形表和inputs | 输出文語用言；classical_adjective与ごとくなり组合另有专用处理 |
| insertion | 调用通用accepts_input，核对成员或举例证据 | 要求preserve_state为true，完整继承状态与停机许可；填保持前项和preserve |

rule_group会保存到分析结果，补助执行器还生成`connector:<rule_group>`证据；前端一处规则说明使用“rule_group：label”。统一名称时需要同时核对该处组合，采用一个功能标题及独立接续条件，防止同义名称重复。order_rank与stage同样进入本句结果，语法合法性仍由实际读取的条件决定。

### 识别条目与路径的固定编排

| 条目 | 必填字段 | 可选字段及准确含义 |
| --- | --- | --- |
| feature | `kind`、`token` | `exclude`：任一模式命中即排除本条；其他 feature 继续判断 |
| connection | `action`、`when` | `kinds`：attach 指定功能；省略或为空时读取 features。`support_group`：support 查找的本体组 |
| sequence | `kind`、`tokens`、`forms` | `head`：零起点，须小于 tokens 长度；`categories`、`operators`：前项限制；`terminal_only`、`exclude_following`、`phenomena` |
| predicate link | `kind`、`prefix`、`next`、`required_operators` | 四项均须声明；required_operators 空数组允许直接以后项核心确定构式范围 |
| 显式规则组 | `label`、`paths` | `scope: inflection`；`ctype_mode: prefix / classical_prefix`。operators 由归属生成 |
| 显式 path | `id`、`group`、`label`、`steps` | 机器条件按下表填写；显示文案按显示规范编排 |

connections 的 action 限定为 `core`、`attach`、`support`、`stop`。when 的许可字段为 `next`、`last`、`following`、`at_end`、`forms`、`categories`、`operators`、`suffixes`、`functional`、`auxiliary_relation`、`sahen`。next 是待接 token，last 是前链末 token；following 是 next 后的一个 token。`at_end: true` 要求 next 后没有来源 token；句点是一个实际 token。

`functional: true` 接受非自立 POS、GiNZA AUX、功能位置或 aux/cop 关系中的任一证据；`auxiliary_relation: true` 要求 aux/cop 两端关系；`sahen: true` 核对前项名词的サ变身份。`operators` 查询最近有效操作，插入步骤按已有许可跳过。每个 when 同时满足内部字段，多组 when 任选。

predicate 的 `required_operators` 实际表示“后项必须含列表中的任一操作”，按后项操作顺序取首个匹配来确定构式正文的结束位置。例如义务后项允许 negative、negative_ba、negative_to、negative_tewa、negative_de 中的一个；后续过去或礼貌继续单独说明。

### path 条件的范围与组合

源字段建议依次排列为 `id / group / label / steps`、`categories / exact_category / forms`、活用型、词面及前后项、连接器、证据及资格。同一条件内不同字段同时成立。

| 字段 | 匹配语义 |
| --- | --- |
| `categories`、`forms` | 数组内任选；省略或空数组不限。exact_category 强制精确类别；副词路径始终精确匹配 |
| `ctypes`、`exclude_ctypes` | 前者任选，后者任一命中即排除；受组内 ctype_mode 控制 |
| `cform_contains`、`exclude_cform_contains` | 前项原始 cForm 的包含判断；前者任选，后者排除 |
| `previous_operators`、`exclude_previous_operators` | 最近有效操作的许可与排除集合，支持回溯父链 |
| `next` | 当前 operator 成员范围内的首个来源 token；与 connection.next 的观察时点相对应 |
| `predecessors` | operator 之前指定数量的连续 token；数组顺序为正文从左到右，逐项匹配 |
| `following`、`exclude_following` | operator 完整成员之后字符紧邻的一个 token；前者要求存在且匹配，后者允许缺失或不匹配 |
| `connector`、`connector_paths` | 要求同链最近非插入操作是指定连接器，字符紧邻，且连接器的唯一 path 位于指定集合 |
| `evidence` | 数组各项均须存在于当前 operator 的 evidence；使用实际执行器产生的证据字符串 |
| `capabilities` | 正整数位组合；当前合法位为 1、2、4、8、16，1 单独使用。路径要求全部指定位同时具备 |

`TokenPattern` 的模式键限定为 `equals`、`starts_with`、`contains`，字段键限定为 `pos`、`sub`、`detail`、`ctype`、`cform`、`lemma`、`base`、`surface`。模式之间、字段之间同时成立，同字段数组任选。空对象匹配任意 token；字段值空数组匹配零个值。字符串采用原始字段值，保留日文字符、大小写和全半角差别。

`scope: inflection` 在 operator 起点等于链 anchor 时读取 state_after；普通路径读取 state_before。`group`、`label`、`steps` 是字符串，当前编译器没有规定显示词汇集合。本次规定的词类分组和条件写法见[前接规则编排](morphology_display_conventions.md#前接规则的编排)。

### 编译产生的字段与人工维护范围

operator 的 connection_rule 决定直接规则归属；编译器据此生成每组 operators 和全部 connection_types 成员。直接归属的规则必须恰好存在。经连接器继承规则的条目可用 null，并声明可解析的 connector。

文言输入的 `id` 与路径的 `input` 只在源文件使用。编译器复制 input 的 forms、ctype、exclude_ctype，按路径 categories 生成精确类别分支，并检查全部输入类别的覆盖一致性；运行 inputs 移除 id。谓语路径的 `predicate` 指向构式 kind，编译器沿该构式的 prefix 所属组复制完整 paths。`operators`、`source` 及派生条件由编译器生成。

categories、forms、token 模式、接续功能和路径说明应表达同一前接许可。编译器核对文言派生与谓语继承的一致性；一般规则的语义一致性依赖内容审阅。已经确认的冲突见[待修订清单](morphology_display_conventions.md#现有不一致与修订清单)。

## 编译、更新与验证

新增规则前先核实真实分词、词元、活用型、活用形及必要的 UD。修改所属 operator、识别规则及路径，关联 concept 和例句，再生成运行配置。新增输入类别时同步维护文言或显式路径的显示分支。

```powershell
npm run morphology:build
npm run morphology:check
```

build 生成配置并检查引用，内容相同时保留文件；check 在内存中编译并比较已保存产物的 UTF-8 字节。编译结果采用 LF、两空格缩进、固定顶层结构，对象键排序、数组顺序保留。Rust 类型和前端配置加载核对 v1 协议。

[check_morphology_connections.py](../../scripts/check_morphology_connections.py) 核对分类、唯一归属、路径、连接器、文言和谓语引用、概念摘要、例句及既有路径样例。该脚本检查资源关系，语言识别及范围判断由相应样例验证。

修改执行规则后重新编译 Rust 服务，并生成受影响指南或书籍快照；仅修改展示配置时核对相应前端消费。指南局部更新及服务命令见[开发说明](../development.md#指南及书籍静态资源)。最终验证按修改范围选用现有 Rust、前端投影或真实例句入口。

## 规则细节交接

规则分类、字段编排与显示设计已经在本次深化中整理。后续工作分为按规范修订内容、处理接续条件冲突、实现统一显示三个部分。

| 主题 | 已记录内容 | 后续细化 |
| --- | --- | --- |
| 规则细分类别 | [分类指南](morphology_rule_categories.md)按学校语法及功能整理语态、否定、时态、愿望、动作阶段、授受、判断、名词化、接续、义务、插入、副词与文言 | 依据各类规范维护新增条目 |
| 内部字段 | 本文列出必填性、限定集合、排序、执行器生效范围、派生字段和完整样例 | 修改元数据时核对所属细类及识别、执行、路径三处许可 |
| 规则名称及讲解 | [显示规范](morphology_display_conventions.md)列出统一术语、逐类名称、用途和摘要对应 | 按清单实施operators和讲解内容修订 |
| 前接规则与D3 | 已规定七个词类分组、分支标题、条件行、节点内容和显示依赖 | 处理steps文字依赖和D3排序，统一各投影入口 |
| 输入冲突 | 已定位すぎる、否定条件、可能性等元数据与路径的差异 | 依真实输入核实语法许可，实施对应规则修订 |
| 文言 | 已列五个范式、32项助动词、具名输入及完整例子 | 为额外输入形式提供逐项依据，确认同形形式的边界 |

各项修订保留真实引用，按修改范围集中使用既有例句核对。L7–L10的新模块详细设计继续按总计划暂缓。
