# 活用配置协议与存储重构方案

## 结论与实施状态

最终配置可以固定结构、字段解释和下游消费方式。采用 `readux.morphology-config.v1` 作为正式协议，`configs/unidic-nlp/morphology.json` 继续作为 Rust 与前端共用的运行资源。先完成协议与消费端修复，再按该协议开发规则编译器；编译器输出由已确定的运行模型约束。

输入文案与机器条件分离、输出状态字段归并、原子特征独立、文言活用型匹配统一、原型选择统一、规则归属索引，以及前缀表的顺序显式化均已实施。150 个 operator 已迁移至 18 个规则源文件，编译器按本文协议生成唯一运行产物。源文件索引与维护入口见 [source/README.md](../configs/unidic-nlp/morphology/source/README.md)。

## 一、结构统计与准确计数

初次分析的文件为 357,199 字节、294,039 个字符，按换行符计数为 10,493 行位置。原文件有 17 个顶层区段。修复后增加 `schema`、显式 `max_auxiliary_depth` 和 `intrinsic_features`，共 20 个顶层字段。以下数量为本次迁移的内容基准，后续新增规则按协议扩展。

| 最终字段 | 类型与当前内容 | 职责 |
| --- | --- | --- |
| `schema` | 固定字符串 `readux.morphology-config.v1` | 配置协议版本 |
| `max_auxiliary_depth` | 整数 8 | 补助构式递归上限 |
| `intrinsic_features` | 3 个特征名 | 活用、词首变化、词尾变化等原子分析特征 |
| `operators` | 150 个 ID → 定义 | 概念路由、词形、补助本体与连接器身份 |
| `rule_metadata` | 150 个 ID → 执行元数据 | 输入条件、输出状态、执行器与停机语义 |
| `connection_types` | 6 类、150 个成员 | 接续结构分类索引 |
| `connection_rules` | 89 组、435 条路径 | 路径匹配、连接器路径和接续图声明 |
| `features` | 48 条 | 单 token 功能识别 |
| `connections` | 48 条、67 组 when 条件 | 相邻成员的组链动作 |
| `sequences` | 30 条、58 个 token 模式位置 | 连续词尾识别 |
| `predicate_links` | 12 条 | 已形成谓语链之间的构式关联 |
| `forms` | 9 条 | UniDic 活用形到连接形的前缀映射 |
| `classical_forms` | 5 个范式、36 个表内词形 | 文言用言范式 |
| `classical_auxiliaries` | 32 条、193 个表内词形、44 组 inputs | 文言助动词及前接许可 |
| `insertion` | 1 个白名单条件组 | 插入成员身份 |
| `insertion_positions` | 4 组 | 插入位置与补助连接条件 |
| `input_recoveries` | 1 条 | 明确声明的分词恢复 |
| `entity_classes` | 6 类 | 实体分类说明 |
| `rule_tree` | 6 个根节点 | 实体与规则总览 |
| `presentation` | 10 个子区段 | 形式标签、现象、句中层与展示分组 |

150 个 operator 的接续分类为 direct 84、connector 30、sequence 18、predicate 12、inflection 4、insertion 2。124 个 operator 直接归属接续规则，26 个通过连接器获得路径。

89 组路径分为三种来源：31 组文言输入对应 138 条路径，12 组谓语构式对应 63 条路径，46 组显式路径对应 234 条路径。三者合计 435 条。首次迁移的 433 条路径与 v1 基准完整一致；随后回归修复增加了「用言＋だけ＋な＋の／ん」的两条路径。文言助动词共 32 条；其中 `negative_zuni` 使用显式 `zuni` 路径，包含分词恢复等专门条件，故文言派生组为 31 组。

六类实体与执行器分派值分别计数。执行元数据含 verb_cycle 45、auxiliary_verb 44、classical_conjugation 33、na_adjective_judgement 16、i_adjective_conversion 6、nominalization 2、verb_cycle+i_adjective_conversion 2、insertion 2，共八种分派值、150 条记录。

初版报告的分区字符统计误匹配了嵌套同名字段。按 JSON 顶层值的实际边界统计，原文件 `operators` 为 36,992 个字符、`connection_rules` 为 103,153 个字符、`rule_metadata` 为 43,204 个字符；`forms` 为 747 个字符、`insertion` 为 393 个字符。这些数字只统计对应 JSON 值，接续路径是最大的维护区段。

## 二、已实施的结构与消费修复

| 对象 | 原因与具体表现 | 修复后的契约 |
| --- | --- | --- |
| 原子特征 | `presentation.inflections` 同时决定 Rust occurrence、词典目标与前端展示 | 独立为 `intrinsic_features`，Rust 与前端共用该字段 |
| 输入类别 | `input_category` 同时包含“用言”等机器解释和文言接续说明 | `input_label` 仅用于说明；判断读取 `input_category_mode` 与 `input_categories` |
| 输出状态 | 定义和元数据同时声明 output_category；はず两处分别为名词与形状词 | 输出词类统一由元数据声明；はず为名词；output_form 与 preserve_state 同步移至元数据 |
| 文言活用型 | 文言执行器兼容“文語”前缀，路径匹配采用原始 starts_with | 31 个文言派生组显式采用 classical_prefix，复用文言执行器的匹配函数 |
| 本句原型 | 前端再次执行 variants，却没有提供 base、ctype、cform；てよい使用 base 条件 | Rust 确定 normalized_form，气泡与推导共用该结果；静态原型用于无本句结果的展示 |
| 规则归属 | Rust 和前端反复扫描规则列表查找 operator 的所属组 | 两端加载时建立 operator → 规则、operator → 分类索引 |
| 前缀标签 | JavaScript 对象枚举顺序承担首次匹配优先级 | families、raw_form_labels 改为有序的 prefix/label 数组 |
| 协议边界 | 文件缺少自身版本，递归深度隐含于 Rust 默认值 | schema 与 max_auxiliary_depth 显式进入配置，两端按 v1 加载 |

`intrinsic_features` 的三个名字为 conjugation、initial_alternation、final_alternation，由 `morphology.rs` 直接创建，属于分析产物的原子特征。接续分类中的四项 inflection 为 volitional、conditional、imperative、classical_adjective，属于 150 个配置 operator。两组数据拥有不同的成员集合和用途。

## 三、最终字段与执行契约

### 3.1 定义与执行元数据

`operators[id]` 保留 label、title、concept、prototype、variants、token_lemma、normalized、candidates、support、compound_candidate、connector、connector_alternatives、display_suffix。词形规则和概念路由由 Rust 读取；补助关系和展示信息按现有职责消费。

`rule_metadata[id]` 保存 stage、executor、rule_group、input_label、input_category_mode、input_categories、input_forms、exclude_input_ctypes、output_category、output_form、preserve_state、terminal_mode、order_rank。输入输出状态字段在此表内具有唯一归属，operator 定义通过相同 ID 关联。

`input_category_mode` 的解释固定如下：

| 值 | 判断 |
| --- | --- |
| any | 接受任意前项词类，继续检查活用形和排除条件 |
| exact | 前项原始词类属于 input_categories |
| judgement_compatible | 接受集合中的词类；已形成判断谓语的名词、代词可以共享形状词路径 |

exact 配合空集合表示通用输入判断不接受任何词类。文言助动词的实际许可由其专门的 `classical_auxiliaries.inputs` 声明；输入说明中的“未然形”等文字单独保留在 input_label。原有“用言”规则已展开为五类词类集合并采用 judgement_compatible，保留对名词判断谓语的处理。

`input_forms` 为空表示通用输入判断对活用形不作额外限定。`exclude_input_ctypes` 使用完整活用型名称的精确匹配。普通 TokenPattern 的 starts_with、普通路径的 cType 前缀匹配和文言前缀兼容各有明确的调用位置。

`output_category` 的具体词类采用 UniDic 名称，例如 `動詞`、`名詞`、`形容詞`。`用言` 和 `保持前项` 沿用执行器中已经定义的前项继承语义。通用状态准备先按 preserve_state 继承状态，再应用 output_form；词类变更与实体身份由所属执行器完成。补助用言继续保留自身的活用型与活用形，て连接器继续保留前项词类。そう的样态／传闻区分和文言词形转换由现有专用执行器处理。

stage、rule_group、order_rank 用于说明与投影；匹配优先级由有序规则数组决定。分析产物保留既有 input_category 字段作为展示投影，其值来自 input_label；state_before、state_after 保存实际词类和活用状态。

### 3.2 组链规则与路径规则

执行流程固定为：读取 token 与状态 → 按顺序识别连续词尾及相邻接续 → 由对应执行器更新链 → 关联谓语构式 → 识别具体接续路径 → 生成 occurrence 和前端投影。

`features`、`connections`、`sequences` 与 `predicate_links` 分别承担识别和组链职责。`connection_rules` 在链形成后匹配具体路径，并决定命中分支、前项范围、连接器上下文和候选诊断。路径匹配影响结果状态，因此属于运行协议。

每个 connection rule 固定包含 label、operators、paths；scope、source、ctype_mode 为可选字段。scope 默认普通前接，inflection 读取词内状态；source 记录数据来源；ctype_mode 默认 prefix，classical_prefix 兼容“文語”前缀。数据来源说明与机器匹配模式分别保存。

每条 path 由 id、group、label、steps 及已有条件字段组成：categories、forms、ctypes、exclude_ctypes、cform_contains、exclude_cform_contains、previous_operators、exclude_previous_operators、exact_category、evidence、next、predecessors、connector、connector_paths、following、exclude_following。条件字段按现有 Rust 类型解释，显示字段不参与条件判断。evidence 是需要满足的分析证据，属于机器条件。

路径匹配保留全部命中项；唯一命中形成 path，多项或零项命中形成候选诊断。路径数组的顺序同时决定 matched_paths 和图示排列。连接器继承实际分析结果中的 rule、path 和范围；本句的 dependency_type 可以因具体路径成为 connector。

### 3.3 统一消费入口

| 层级 | 入口与责任 |
| --- | --- |
| Rust 配置 | `morphology_config.rs`：v1 类型、一次解析、operator 定义／元数据／规则／分类查询 |
| 状态机 | `morphology_machine.rs`：有序匹配、专用执行器、路径识别与结果生成 |
| 原子特征 | `morphology.rs`：构造三个原子特征，使用统一状态和概念接口 |
| 前端配置 | `src/reader/morphologyConfig.ts`：v1 加载、规则归属索引、原子特征判断、原型读取 |
| 展示消费 | `morphologyBubble.ts`、`lookupPresentation.ts`：消费本句结果与静态规则树 |
| 词典查询 | `dictionary/targets.rs`：通过配置模块读取定义与原子特征分类 |
| 缓存 | `analysis_engine.rs`：基础分析与完整分析均继续将 SOURCE 纳入既有缓存标识 |
| 配置维护 | `check_morphology_connections.py`：保留已有分类、路径、概念和例句关系检查 |

前端接续图展开规则树，命中信息采用 Rust 的 matched_paths。词形分支、concept 选择和实际状态由分析阶段决定。图示布局与语言匹配分别维护。

## 四、确定性约定

“结构一致”指同一个协议版本具有固定字段、类型、默认值、引用范围和执行语义；条目内容及数量可以按规则维护需求扩展。“生成一致”指相同源内容产生完全相同的 UTF-8 字节。

| 数据类型 | 顺序与生成规则 |
| --- | --- |
| 顶层字段 | 按本文结构表顺序输出全部 20 个字段 |
| operators、rule_metadata、connection_rules | ID 采用稳定字典序；对象顺序不决定规则优先级 |
| connection_types 成员及每组 operators | 属于归属集合，输出按 operator ID 排序 |
| features | 保留声明顺序，决定多特征结果的排列 |
| connections、sequences、predicate_links | 保留声明顺序，首次有效匹配按现有执行流程生效 |
| forms、classical_forms、classical_auxiliaries、input_recoveries | 保留声明顺序，维护首次匹配语义 |
| variants、support.bases、connector_alternatives | 保留声明顺序；bases 的首项参与补助词头展示 |
| paths、rule_tree、phenomena、前缀标签数组 | 保留声明顺序，维护命中列表与展示排列 |
| 其他数组 | 保留源文件顺序，编译器不进行通用数组排序 |
| 可选字段 | 按字段默认规则解释；同一源内容固定产生相同的省略或显式值 |
| 文本输出 | UTF-8、LF、固定两空格缩进、保留 Unicode 字符、文件末尾一个换行 |

配置内容只依赖明确列出的源文件。文件修改时间、绝对路径、运行时间、目录枚举顺序与系统区域设置均不进入产物。当前 support 的 group/base 组合具有唯一归属；编译器延续这一约束，使补助查找与 operator 键排序无关。

本协议由 Rust 配置类型、前端消费类型和本文共同定义。schema 标识在两端加载时核对。字段解释或数组语义发生改变时升级协议；常规规则增删继续沿用 v1。

## 五、分文件源结构

源文件位于 `configs/unidic-nlp/morphology/source/`，生成目标保持 `configs/unidic-nlp/morphology.json`。

```text
source/
  base.json                     # 递归深度、原子特征、forms、input_recoveries
  operators/
    verb.json                   # verb_cycle 与组合分派，共 47 条
    conversion.json             # イ形容词转换与名词化，共 8 条
    judgement.json              # 判断执行器，共 16 条
    auxiliary.json              # 补助执行器及其连接器，共 44 条
    classical.json              # 文言执行器，共 33 条
    insertion.json              # 插入执行器，共 2 条
  rules/
    features.json               # 完整有序数组
    connections.json            # 完整有序数组
    sequences.json              # 完整有序数组
    predicate_links.json        # 完整有序数组
    insertion.json              # 白名单与四组位置
  classical/
    forms.json                  # 5 个范式
    auxiliaries.json            # 32 条助动词及 inputs
  paths/
    explicit.json               # 46 组显式路径
    classical.json              # 31 组输入引用与路径展示声明
    predicates.json             # 12 组构式前缀引用
  presentation.json             # entity_classes、rule_tree、presentation
```

每个有执行优先级的规则数组集中在一个源文件内维护，保留完整顺序。operator 文件按明确的执行器归属划分，每个 ID 只有一个维护位置。编译器内使用固定文件清单，目录路径承担组织作用。

每个 operator 源条目采用以下完整结构，外层键为现有 operator ID：

```json
{
  "definition": {},
  "execution": {},
  "connection_type": "connector",
  "connection_rule": null
}
```

definition 保存运行时 operators 的全部定义字段，execution 保存 rule_metadata 的全部字段；connection_type 生成分类索引；connection_rule 保存直接所属规则 ID，连接器继承条目使用 null 并在 definition 中保留 connector 声明。空对象仅用于示意字段边界，正式源条目保存完整内容。

生成器由同一条源记录产生 operators[id]、rule_metadata[id] 和分类索引。共享路径组的 operators 列表由 connection_rule 归属生成。运行资源中的这些表是按消费职责组织的索引；人工维护集中在源条目。

## 六、消除路径重复维护

31 组文言路径从对应助动词 inputs 取得 categories、forms、ctype 和 exclude_ctype。源 inputs 使用稳定 ID，路径源记录保存既有 path ID、input 引用、类别分支和 group／label／steps 等展示信息。编译时将引用展开为完整路径，写入 exact_category 与 classical_prefix。已有 input0_* 等路径 ID 作为稳定名字保留，输入数组调整时沿用该名字。

12 组谓语路径通过 predicate_links 的 kind 定位构式，通过 prefix 定位前项 operator 及其所属规则。编译器复制该前项的完整 paths，形成现有 63 条展开路径。完整展开保留独立规则 ID 和既有分析结果的路径引用。

46 组显式路径原样维护其条件与显示字段。zuni 的四条路径继续在此维护，保留文言前接、形容词未然形和分词恢复的差别。连接器路径通过 connector 与 connector_paths 引用共享路径，保持现有共享结构。

生成结果包含 89 组和 435 条展开路径，源文件中具有同一来源的机器条件只维护一次。语法概念正文继续位于 grammar source bundles，通过 concept ID 关联。

限定后名词化由两条相互引用的显式路径构成：`judgement_attributive/restriction_nominalization` 接受动词、形容词或助动词的终止／连体状态，并要求紧邻的「だけ」及后续「の／ん」；`nominalization/no_restriction_na` 引用该连接器路径。一般用言名词化路径排除已存在的 attributive_na，使两种结构分别命中。

「ごとく＋なり」复合结果在文言执行器中继承ごとし的前接状态，operator 的范围覆盖完整复合词条。路径匹配继续读取已有的ごとくなり输入规则，分析结果保留ごとし与ごとくなり两个 operator 及复合证据。

## 七、编译器职责与结果固定

入口固定为 `scripts/build_morphology_config.py`。处理顺序为读取明确源文件、展开 operator 索引、建立规则归属、展开文言与谓语路径、组装 20 个顶层字段、稳定序列化。三个路径来源分别使用明确的转换函数。

编译过程检查直接决定产物可解释性的关系：ID 唯一、定义与元数据一一对应、接续分类唯一、直接规则归属唯一、support 的 group/base 归属唯一、连接器与路径引用存在、引用无循环、专用模式属于协议枚举，以及跨文件顺序与声明一致。概念正文和例句检查继续复用现有检查入口。

命令提供两种行为：

| 命令 | 行为 |
| --- | --- |
| `python -X utf8 scripts/build_morphology_config.py` | 生成唯一运行产物；内容相同时保留现有文件 |
| `python -X utf8 scripts/build_morphology_config.py --check` | 在内存中编译，按字节与已提交产物比较，并汇总既有引用检查 |

生成产物进入 Git，普通 Rust 编译直接读取它。开发者修改源规则后执行生成命令，Vite 监听生成文件完成更新；前端构建和正式打包在各自入口生成配置。CI 在构建前执行一次 --check，确认提交中的源文件与产物对应。

首次分文件迁移以本轮修复后的 v1 配置作为内容基准。迁移比较保留全部机器条件、原型、概念、范围声明、路径 ID 和有序数组；分类成员及规则归属按集合比较，格式与对象键顺序按规定统一。首次产物固定后，后续 --check 直接使用字节比较。条目总数作为迁移清单记录，常规新增规则通过更新源文件与对应产物交付。

## 八、实施顺序与交付边界

| 阶段 | 具体交付 | 状态 |
| --- | --- | --- |
| 1. 修复消费语义 | 状态字段唯一归属、输入条件显式化、原子特征独立、原型统一、文言匹配一致 | 已实施 |
| 2. 固定协议 | v1 版本、20 个顶层字段、消费接口、顺序语义、序列化与引用规则 | 本文与当前实现 |
| 3. 迁移源文件 | 按领域集中 operator，保持有序规则数组，表达三类路径来源 | 已完成 |
| 4. 开发编译器 | 只输出 v1，完成固定展开与单一 --check 入口 | 已完成 |
| 5. 接入维护流程 | 开发生成命令、构建入口、CI 与配置维护文档 | 已完成 |

存储拆分的可读性收益来自规则与执行元数据集中维护，以及重复路径条件按来源生成。运行时仍解析一份配置；新增的归属索引将每次遍历 89 组路径或分类成员改为按 operator 查找。规则识别、GiNZA 依赖和补助递归继续由专用执行器承担，编译器负责将可维护源结构转换成已经确定的运行协议。

## 九、迁移与回归验证（2026-10-31）

首次迁移产物与提交 `8405677` 的 v1 配置逐项比较，150 个 operator、89 个规则组、433 条路径、状态条件、概念及全部有序数组一致。分类与规则归属成员按集合核对。迁移提交为 `a50753d`；之后的回归修复单独维护两条限定名词化路径和文言复合范围。

| 检查 | 结果 |
| --- | --- |
| 重复编译、源对象键顺序反转 | 输出字节一致 |
| 修改源定义后的传播 | 产物按源内容变化 |
| 无效输入模式、缺失连接器、缺失规则 | 编译器均报告错误 |
| `npm run morphology:check` | 150 个 operator、89 组、435 条路径通过字节与引用检查 |
| 既有路径、概念、例句检查 | 全部通过，124 个直接归属、26 个连接器继承条目 |
| `cargo test -p readux-nlp` | 80 项通过 |
| `cargo test -p readux-core --test p4_language` | 14 项通过 |
| core 库单元测试 | 55 项通过 |
| `npm run build` | TypeScript 检查与 Vite 生产构建通过 |

验证同步了历史样例中的 concept 和状态预期：である采用 `grammar.copula.de_aru`，接续助词が采用 `grammar.connective.ga`，でしょう采用 `grammar.modality.deshou`，はず输出名词；前端条件词尾样例提供本句 normalized_form。既有文言复合回归增加了完整成员范围的检查。

扩展前端检查为 109/116 项通过。六项胶囊分组与一项图布局失败已在提交 `8405677` 的独立临时基线中复现，失败集合一致。扩展的 `p5_lookup` 为 9/12 项通过，三项失败分别涉及全句查询表记一致性、整体词条目标，以及「ひとつ→一つ」规范形式；这些结果单独记录为词典查询检查结果。

生成文件通过 `.gitattributes` 固定为 LF，Windows 的 `core.autocrlf=true` 保留相同的编译字节。

