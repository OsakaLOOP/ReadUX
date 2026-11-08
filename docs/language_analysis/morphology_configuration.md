# 活用规则与配置维护

更新日期：2026-11-09。本页说明当前源文件、运行协议、规则语义及维护方法。执行过程见 [L4](morphology.md)，词条和解说见 [L6](entries.md)。逐条复杂规则的 JSON 和语言依据按文末清单后续细化。

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

`operators` 按执行器分文件，编译器核对归属。规则数组集中维护完整顺序；路径源文件分别承担显式条件、文言输入引用和谓语前缀继承。

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

`input_forms` 为空时不增加活用形限制。`exclude_input_ctypes` 按完整名称排除。文言 inputs 使用专门许可，通用 exact 空集合可表示由专用执行器决定输入。

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

features 可以通过 `exclude` 排除重叠形式。connections 的 `when` 数组任选，各条件内部同时检查 next、last、following、状态、最近 operator、功能或关系等字段。组链动作包括 `core`、`attach`、`support`、`stop`。`stop` 明确保留成员边界。

sequences 依据当前状态、完整 token 模式和字符连续性匹配；`head` 默认为首项，指定词元及活用身份来源，`terminal_only` 限制输入结尾或允许的句末标记，`exclude_following` 为后续构式保留边界。predicate_links 在后项完成活用后核对前项 prefix、后项身份及必需的否定等操作。

features、connections、sequences、predicate_links、forms、classical_forms、classical_auxiliaries、input_recoveries、variants 和 paths 保留数组顺序。stage 与 order_rank 用于说明及投影；实际匹配优先级来自执行入口和相应有序数组。

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

Rust 依据完整来源字段选择 variants，并在 `normalized_form` 保存本句原型。完整 observed_form 优先映射，随后使用 normalized 或 token 变体及默认 prototype。`variants.concept` 可选择不同概念；显示形与概念选择各自保留。

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

L0–L6 本轮整理完成。下表是用户明确留给下一位 agent 的逐规则深化工作，详细范围和正文组织可根据实际规则调整。

| 主题 | 已记录内容 | 后续细化 |
| --- | --- | --- |
| 组链与执行 | 字段语义、数组顺序、执行器及状态变化 | 按语法家族解释完整 definition／execution／识别规则 JSON，给出对应输入和输出 |
| 判断与名词化 | でした示例、词干身份、な连接器与の／こと | 同形の、なの／なん、だけなの等具体分支及条件对照 |
| 补助与缩约 | support 分组、父子链、连接器及原型 | て／で、连用形、たり、意向、变化及缩约的逐条 JSON、范围和递归例句 |
| 插入与 UD | 四组位置、标签方向及来源条件 | 全部白名单、单体とか与と＋か、连续词尾竞争及正反例 |
| 谓语构式 | 后项完整活用与 prefix 路径继承 | 义务、禁止、省略后项及后续否定／礼貌／过去的完整条件 |
| 文言 | 范式、具名 inputs、现代语边界 | 各范式和多用法助动词的完整 JSON、活用表及边界例句 |
| 副词资格 | reviewed 到运行映射及 flags | 词元判定依据、候选审定流程、各资格的分支和样例 |
| 路径与显示 | matched_paths、连接器上下文及 presentation | 逐条路径到句中图、规则树和高亮的对应，衔接 L11 |

细化时保留现有规则的真实引用，正例和相邻边界对照可优先采用既有测试及指南快照。L7–L10 的新模块详细设计继续按总计划暂缓。
