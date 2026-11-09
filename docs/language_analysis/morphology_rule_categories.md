# 活用规则分类与编写指南

更新日期：2026-11-09。本文按照学校语法和实际功能归纳规则，说明同类成员的共同要求、差别及元数据编写方式。字段的完整限定集合见 [L5](morphology_configuration.md#字段编排与限定集合)，名称、讲解和图示的统一设计见[显示规范](morphology_display_conventions.md)。

正文中的“当前”指仓库已有声明；“编写规范”规定后续维护采用的组织方式。JSON 样例摘自源文件，保留当前语义。建议调整的名字及条件冲突单独记录，便于实施时核对。

## 分类依据

学校语法的品词、语法功能和接续资格分别记录。例如「はっきり」属于副词，可以表达状态，具备する及と接续资格；「すぎる」表示过度程度，采用补助动词活用；「ない」表示否定，后续具有イ形容词性活用。三者的语义分类、前接条件和输出类别各自有依据。

| 分类角度 | 例子 | 编写时回答的问题 |
| --- | --- | --- |
| 学校语法与词法身份 | 助动词、接续助词、准体助词、接尾辞、副词 | 哪些 POS、词元和活用字段能够确认形式身份？ |
| 细分功能 | 使役、愿望、动作开始、结果存续、授受、样态、传闻 | 同类形式共享什么意义？本条与相近条目怎样区分？ |
| 前接要求 | 动词未然形、イ形容词词干、完整谓语、て连接器、词元资格 | 许可由哪些精确条件共同构成？ |
| 后续语法行为 | 动词活用、形容词活用、名词、判断词干、状态保持 | 接入后能继续出现哪些语法形式？ |

`operators/adverb.json` 等文件是维护位置；六类 entity_classes 是状态机分类。本文的细分类别可以跨越这些文件和实体类别。细分类别目前采用文档及 rule_group 描述，新增 JSON 字段需另行修改协议。

## 规则身份、变体与共享条件

同一语法身份的清浊、表记和缩略形式优先采用 variants 或明确的缩约条目。例如た／だ共用过去身份，だった／でした共用判断过去身份，てゆく保留自身表记并共享いく的 concept。缩约条目需要保存展开本体及独立的实际活用，故ちゃう等采用独立 operator。

词法身份、意义或前接规律不同的同形形式分别定义。典型例子是推量らしい与典型性接尾辞らしい，文言断定なり与传闻なり，现代列举たり与文言完了たり。样态そう和传闻そう当前共享 operator，前接路径与 evidence 区分具体用法；显示时采用相应的功能名。

规则共享要求完整输入许可一致。例如たい、ます及多数连用形补助可以复用 continuative 路径；すぎる另有形容词词干分支，使用 excessive 路径。路径 ID 是组内身份，同名的 verb 可以存在于多个组中；引用必须同时确定规则组。

## 各类规则的元数据组合

下表给出细分功能对应的编写约束。executor、输入输出采用现有协议；rule_group 列是统一设计采用的语法名称，现有条目的不同写法列于显示规范。输入约束以当前组合状态为对象。

| 细分类别 | executor 与输入规范 | 输出及专属字段 | 建议 rule_group |
| --- | --- | --- | --- |
| 使役、被动等语态 | verb_cycle；动词未然形；区分活用型及已有助动词 | 动词状态；多解释形式保存 candidates | 使役／被动等语态 |
| 普通否定 | 组合执行器；动词未然形、形容词连用形、判断连用分别声明 | 当前 negative 输出形容词性状态；表记变体与文言身份分别处理 | 否定 |
| 敬体与敬体否定 | ます接动词连用形；ません采用完整序列 | polite_negative 保持动词范畴；ません为独立词条 | 敬体／敬体否定 |
| 过去与完成 | 组合执行器；连用形按词类及清浊分支 | 继承前项类别；同一た保留过去／完成的概括解释 | 过去与完成 |
| 意向、假定、命令等词内活用 | verb_cycle；依据来源 cForm 识别 | connection_type 为 inflection；scope 为 inflection | 意向／假定／命令 |
| 主体愿望 | i_adjective_conversion；动词连用形 | たい输出形容词；保留后续否定和过去 | 愿望 |
| 愿望表现 | verb_cycle；动词连用形 | たがる输出动词，后续按动词活用 | 愿望表现 |
| 难易、典型性派生 | i_adjective_conversion；前接动词连用形或名词词干 | 输出形容词；词元差别保留在本句原型 | 难易／典型性 |
| 动作阶段、反复与完成程度 | auxiliary_verb；通常接动词连用形 | support.group 为 continuative；本体独立活用 | 动作开始／动作持续／动作结束／完成程度等 |
| 过度程度 | auxiliary_verb；动词连用形与形容词词干分支 | 本体すぎる为动词；当前输入冲突需先修订 | 过度 |
| て形后的各类补助 | auxiliary_verb；已确认的て连接器 | support.group 为 te；输出由本体决定 | 持续与状态／授受／趋向等 |
| 判断词干与词尾 | na_adjective_judgement；判断兼容模式 | 区分形状词、名词或前项类别继承；词尾指定连接形 | 断定／敬体／连体修饰／连用修饰 |
| 认识、传闻和判断态度 | 按本条选择判断或形容词转换执行器 | 依具体用法保留 evidence；输出采用本体语法行为 | 样态／传闻／推量／预期／当为 |
| 名词化 | nominalization；分别声明词干与连体前接 | 名词状态；必要时引用な连接器 | 性质名词化／准体名词化／内容名词化 |
| 接续助词 | 依现有规则采用 verb_cycle 等；核对助词小类和连接形 | 记录接续功能及输出连接形 | 并行／逆接／原因／条件／否定接续 |
| 义务、禁止 | predicate_links；前缀形式与完整后项共同确认 | connection_type 为 predicate；继承前缀路径 | 义务／禁止 |
| 提示、限定、举例插入 | insertion；白名单与位置共同限定 | preserve_state、terminal_mode: preserve | 对比提示／追加提示／限定／举例 |
| 副词的述语化和状态修饰 | adverbial_predicate；按具体词元资格区分 | する输出动词，と保存副词性状态并设置 other | 副词动词化／状态修饰 |
| 文言助动词 | classical_conjugation；许可放在具名 inputs | 文言用言；精确词形表和活用范式 | 使役／否定／完了／推量／断定等细分功能 |

rule_group 与 stage 的编排独立。细分功能可以共享 te_support 等既有 stage，具体功能依靠名称和 rule_group 说明。各子类的字段顺序统一遵循 L5，取值按下列语言条件决定。

## 语态：使役与被动等

causative 与 passive_potential 共用动词未然形前接组 irrealis，执行后继续动词活用。二者均排除ます、た、です、だ等特定助动词活用型；使役后可继续接被动形式，后续条件读取使役后的状态。

| 规则 | 形式及同类区别 | 显示和讲解规范 |
| --- | --- | --- |
| causative | せる／させる；使役关系 | 名称“使役”；说明使某人实施动作或允许实施动作 |
| passive_potential | れる／られる；同形可能表达被动、可能、尊敬、自发 | 未完成语义选择时名称“被动／可能等”，讲解列明候选；具体语义由证据支持 |

`candidates` 保存的是解释候选；词元匹配或动词输出类别本身无法选定被动义。句中图和教学名称保持相同的语义精度。`られる` 的 variant 使用 lemma 包含或实际表面串前缀，覆盖其继续活用后的词形。

## 否定、敬体与时态

| 子类 | 前接及后续行为 | 需保留的区别 |
| --- | --- | --- |
| negative | 动词未然形＋ない；形容词连用形＋ない；判断连用结构＋ない | 动词的読まない、形容词的高くない、判断的静かでない分别有路径 |
| polite_negative | 动词连用形＋ませ＋ん | 整体为敬体否定，后续可接でした；当前与普通否定共享 concept，摘要有专用读取 |
| politeness_masu | 动词连用形＋ます | 敬体表达，后续活用来自ます |
| politeness_desu | 判断词干或允许的谓语＋です | 判断词尾的敬体；与でしょう的意志推量形分开 |
| past | 连用形＋た／だ | 清浊依活用型与音便决定；词条原型为た，实际节点保留だ |
| copula_past | 名词／ナ形容词＋だった／でした；形容词的敬体过去分支 | 完整判断过去，形式区别由 variants 保存 |

ない的形容词性后续可以形成なかった、なくなる等；ません具有不同的组词形式。ぬ、ず、ん涉及现代否定表记和文言规则，须同时核对词元、cType、cForm 与完整词形。`classical_classical_nu` 是文言完了ぬ，`negative_zuni` 是文言否定ず；“ぬ”字面相同不足以合并两个身份。

过去 feature 的当前完整声明如下，exclude 为たら、たり等条件或融合形式保留自身身份：

```json
{
  "kind": "past",
  "token": {"equals": {"pos": ["助動詞"], "ctype": ["助動詞-タ"]}},
  "exclude": [
    {"starts_with": {"cform": ["仮定形", "已然形"]}},
    {"contains": {"cform": ["融合"]}}
  ]
}
```

た／だ的前接路径分别限定五段ガ・ナ・バ・マ行与其他动词类型；イ形容词连用形、ます连用形、文言和判断另有分支。名称统一为“过去”，摘要说明过去或完成；语境尚未区分时采用该概括范围。

## 词内活用与接续助词

| 对象 | 当前 kind | 输入依据 | 规范名称 |
| --- | --- | --- | --- |
| 意向形 | volitional | 来源意志推量形等活用字段 | 意向形 |
| 假定形 | conditional | 词内 cForm；本体可继续接ば | 假定形 |
| 命令形 | imperative | 词内命令形 | 命令形 |
| 条件助词ば | ba_connection | 当前 conditional＋接续助词ば | 条件接续 |
| 条件形式たら | tara_condition | 助动词タ的对应词形 | 条件接续 |
| 条件なら | nara_condition | 判断词尾なら，完整序列身份 | 条件接续 |

词内活用的 scope 使用 inflection，说明当前词自身的形式。接续助词读取前项连接形，说明该形与后续成分的关系。文言已然形与现代假定形在状态枚举中共用 conditional，讲解和节点第二行采用原始 cForm 对应名称。

## 愿望、难易与性质派生

| 子类 | kind 与成员 | 前接要求 | 后续行为和统一功能名 |
| --- | --- | --- | --- |
| 主体愿望 | desire：たい | 动词连用形 | 形容词活用；“愿望” |
| 愿望表现 | desire_outward：たがる | 动词连用形 | 动词活用；“愿望表现” |
| 易于实施 | ease：やすい | 动词连用形 | 形容词活用；“容易” |
| 难于实施 | difficulty：にくい／づらい | 动词连用形 | 形容词活用；“困难” |
| 具有典型性质 | rashii_characteristic：らしい | 名词词干＋接尾辞らしい | 形容词活用；“典型性” |

たい与たがる的差别包含愿望的呈现方式和后续活用。第一、第三人称是讲解中的常见使用条件，当前匹配规则根据词法与前接形识别。にくい与づらい共享困难规则，细微使用差异应在例句和讲解中说明。接尾辞らしい以 POS 区别于推量助动词らしい。

## 动作阶段、反复与完成程度

以下十二项当前均在 support.group: continuative 中维护。各本体建立功能子链，并保持自己的词元、活用型和后续否定、敬体、过去。表内“建议名称”同时用于 label、title 和图节点功能名。

| 细分类别 | kind | 代表本体 | 建议名称与区别 |
| --- | --- | --- | --- |
| 动作开始 | inceptive | 始める | 动作开始；着重进入动作 |
| 开始发生 | inchoative | 出す | 开始发生；着重动作或现象出现 |
| 动作持续 | continuative_aspect | 続ける | 动作持续；与ている的状态持续分开 |
| 动作结束 | terminative | 終わる | 动作结束；标明过程结束 |
| 反复实施 | iterative_aspect | まくる | 反复动作；重复或集中实施 |
| 重新实施 | corrective | 直す | 重新进行；重复并调整前项行为 |
| 进入未完过程 | incomplete | 掛ける | 动作未完；开始或接近实施而尚未完成 |
| 完全实施 | completion | 切る／抜く／尽くす | 彻底完成；关注完成程度 |
| 实施困难 | reluctance | 兼ねる | 难以实施；与词法难易接尾辞分开 |
| 实施失败 | failure | 損なう | 实施失败；动作未能如期完成 |
| 机会未能利用 | missed_opportunity | そびれる | 错失机会；时机或机会方面的未能实施 |
| 程度超过适当范围 | excessive | すぎる | 过度；动作与性质的前接形不同 |

`token_lemma: true` 可保留始める、切る等本句词元；`compound_candidate` 依整体查词需求声明。同类组中的 bases 必须按实际词法身份填写，同组同 base 只归属于一个 operator。切れる等可能形式经词元恢复确认后可按きる匹配，实际可能活用继续保留。

すぎる的接续树已有“动词连用形、イ形容词词干、ナ形容词词干”三支，当前 execution 与相邻补助入口的许可更窄。后续统一以该差异为独立语法修订项，不能仅以三支图示认定全部输入已经成立。

## て形补助的细分功能

本组共同前接要求为已确认的て连接器，实际可以为て／で。通常声明 `connector: te_connection`、`connection_rule: null`，继承连接器路径。输出类别取后项本体；“补助”描述句中角色，具体功能按下表命名。

| 细分类别 | kind | 建议功能名称及讲解范围 |
| --- | --- | --- |
| 动作进行与状态持续 | te_iru | 进行／状态持续；包括动作进行、习惯性活动及结果状态 |
| 人为结果的存续 | te_aru | 结果存续；与ている按结果及动作关系说明 |
| 完成及说话人评价 | te_shimau | 完成／遗憾／感慨 |
| 预先实施与状态保持 | te_oku | 预先处置／保持 |
| 趋近和变化的显现 | te_kuru | 趋近／逐渐显现 |
| 远离和后续发展 | te_iku、te_yuku | 远离／持续发展；保留いく／ゆく的本句形式 |
| 尝试实施 | te_miru | 尝试 |
| 对外给予行为 | te_ageru、te_yaru | 给予；敬意和人际使用条件放入讲解 |
| 由他者给予行为 | te_kureru | 受惠 |
| 接受他者行为 | te_morau | 受惠／请求 |
| 受惠的敬语表达 | te_itadaku、te_kudasaru | 受惠（自谦）、受惠（尊敬） |
| 希望他者实施 | te_hoshii | 希望；后项按形容词活用 |
| 许可与认可 | te_ii | 许可／认可；なくていい说明“不必” |
| 建议尝试 | te_goran | 建议尝试；当前本体按名词性补助表达处理 |
| 展示实施决心 | te_miseru | 决心／展示 |
| 持有某种态度 | te_kakaru | 态度／应对 |
| 感受或欲望程度强烈 | te_tamaranai | 强烈感受；后项为完整否定形式 |
| 感受自然发生且难以抑制 | te_naranai | 自发感受；与禁止构式てはならない分开 |
| 感情、愿望持续 | te_yamanai | 感情持续；与一般动作持续分开 |

てたまらない、てならない、てやまない通过完整“本体＋ない”序列识别。该构式中的否定具有整体功能，讲解说明整体意义。当前通用本体组的允许类别有宽有窄，语义适配应在每条规则的说明及例句中分别写明。

以下是 te_iru 的完整源条目，连接组、功能名称和执行元数据各有固定位置：

```json
{
  "te_iru": {
    "definition": {
      "label": "进行／状态持续",
      "title": "进行／状态持续",
      "concept": "grammar.auxiliary.iru",
      "prototype": "ている",
      "normalized": "ている",
      "support": {"group": "te", "bases": ["いる", "居る", "おる", "居る-オル"]},
      "connector": "te_connection"
    },
    "execution": {
      "executor": "auxiliary_verb",
      "stage": "te_support",
      "rule_group": "持续与状态",
      "input_label": "动词",
      "input_category_mode": "exact",
      "input_categories": ["動詞"],
      "output_category": "動詞",
      "terminal_mode": "continue",
      "order_rank": 31
    },
    "connection_type": "connector",
    "connection_rule": null
  }
}
```

本例的label与title统一为“进行／状态持续”。同一连接器可以为不同补助细分类别提供前接条件；本体之间的语义差异保留在concept、功能名及讲解中。

### 缩约、列举、意向与变化补助

| 细分类别 | kind | 内部声明及前接要求 | 编写与显示规范 |
| --- | --- | --- | --- |
| 持续形式缩约 | contracted_te_iru | contracted 组；てる／でる本体，展开为いる | 共享“进行／状态持续”，另标“缩略形式” |
| 完成形式缩约 | contracted_te_shimau | contracted 组；ちゃう／じゃう，展开为しまう | 共享“完成／遗憾／感慨”，实际节点保留缩约活用 |
| 预置形式缩约 | contracted_te_oku | contracted 组；とく／どく等，展开为おく | 共享“预先处置／保持” |
| 列举的述语化 | tari_suru、tari_dekiru | enumerative 组，前项最近有效操作为enumerative | “列举”“列举（可能）”；本体采用する／できる的自身活用 |
| 意向后的尝试或即将 | volitional_to_suru | volitional 组，前项经volitional_to_connection | “尝试／即将”；と作为独立连接器保留 |
| 名词性变化 | ni_naru | nominal_change 组，接受nominal_change_ni或adverbial_ni | “变化”；名词与ナ形容词前项各用自己的路径 |
| 性质变化 | adjectival_naru | adjectival_change 组，前项为形容词性连用形 | “变化”；高くなる与なくなる说明性质或状态变化 |

缩约条目的 support.expanded 使用 `[本体辞书形, 读法]`。contracted_te_shimau 的声明为：

```json
{
  "group": "contracted",
  "bases": ["ちゃう", "じゃう"],
  "expanded": ["しまう", "シマウ"]
}
```

`読む→読んじゃった` 的词汇查询、补助本体查询、词条原型和实际缩约输出分别保留。缩约身份由 group 确定，中文功能名沿用展开形式。

## 判断词尾与认识、传闻

### 断定、敬体及修饰词尾

copula、politeness_desu、copula_past、copula_de、copula_aru 分别表示断定、敬体、断定过去、断定连用、断定；attributive_na 与 adverbial_ni 分别表示连体修饰和连用修饰。名词谓语继续保留名词类别，ナ形容词以形状词类别保存；前接树据此分组。

名词或ナ形容词接过去判断时，完整序列为だった／でした。copula_past 的 variant 使用完整 observed_form 选择“でした”。连续序列的 head 默认 0，成员范围包含两个 token；图中的各成员活用仍分别读取来源字段。

「な」至少需要区分普通连体修饰、名词化前接、限定成分后的名词化三个路径用途。当前名词化前接路径如下：

```json
{
  "id": "noun_nominalization",
  "group": "名词",
  "label": "名词化接续",
  "steps": ["判断词干"],
  "categories": ["名詞", "代名詞"],
  "forms": ["stem", "terminal", "attributive"],
  "exact_category": true,
  "following": {"equals": {"pos": ["助詞"], "lemma": ["の"], "surface": ["の", "ん"]}}
}
```

普通名词分支使用相同后项的 exclude_following，使两个分支互斥。形式相同且前后接条件不同的用途以路径细分；中文名称应说明当前用途。显示设计将此路径归入“名词／代词”，条件写为“词干；后接の／ん”。

### 样态、传闻、推量、预期与当为

| 子类 | kind | 前接共同点与差别 | 输出及建议名称 |
| --- | --- | --- | --- |
| 外观或迹象判断 | sou_modality 的appearance路径 | 动词连用形；イ形容词、ナ形容词词干 | 形成判断词干；“样态” |
| 转述信息 | sou_modality 的hearsay路径 | 终止形／连体形；名词需具有判断谓语状态 | 保留所承接谓语类别的专门处理；“传闻” |
| 比况及依据情况推量 | you_modality、mitai_modality | 用言连体／终止；名词的よう通过の，みたい有直接前接 | 判断词干；“比况／推量” |
| 基于迹象推量 | rashii_modality | 名词、ナ形容词词干，或用言完整谓语 | 形容词活用；“推量” |
| 预期判断 | hazu_modality | 用言连体形、ナ形容词な形、名词の形 | 名词词干；“预期” |
| 规范性判断 | obligation | 现代べき后接だ／です等判断词尾 | 判断词干；“当为” |
| 判断的推量词尾 | darou_modality、deshou_modality、darou_question | だ／です的意志推量形；疑问序列后接か | “推量”“推量疑问”；敬体差别单独标明 |
| 可能性判断 | kamoshirenai | 完整か＋も＋しれ＋ない序列 | 形容词活用；“可能性” |
| 否定推量及否定意志 | negative_volitional | まい，按活用型与前项形式分别限定 | “否定推量／否定意志” |

そう的规则必须形成“前接形—用途—讲解”一致的对应。雨が降りそうだ解释为样态，雨が降るそうだ解释为传闻；当前保存 judgement_usage:appearance／hearsay。名词的だそうだ具有完整判断前项，词干直接接そう的资格按具体规则核对。

よう、みたい、らしい都可能表达推量，三者的前接与后续活用存在差别。はず是形式名词，输出名词状态，后续なの／だった等连接以该状态为依据。べき的当为判断与なければならない的义务构式分别维护。语义名称相近时保留这些结构差别。

## 名词化、限定与程度

| 细分类别 | kind | 前接、同形区别及输出 | 建议名称 |
| --- | --- | --- | --- |
| 性质或程度名词化 | nominalization 的さ分支 | イ形容词／ナ形容词词干＋接尾辞さ；输出名词 | 性质名词化 |
| 准体名词化 | nominalization 的の分支 | 用言连体形，或名词／ナ形容词经な；の必须有对应助词身份 | 准体名词化 |
| 内容、事件名词化 | koto_nominalization | 连体形＋こと／事；输出名词词干 | 内容名词化 |
| 程度、约量和最低标准 | degree | 名词、数量或完整谓语＋副助词くらい／ぐらい | 程度／约量 |
| 唯一选择 | shika_nai | 完整しか＋ない序列；当前允许名词或完整谓语等前项 | 唯一选择 |

准体の与句末终助词の需要依据词性小类、后续だ／です以及输入边界区分。当前连接规则先处理可确认的解释结构，再处理终助词边界，最后处理一般准体名词化。`なの／なん` 的な具有独立连接器身份。

以下是「名词＋な＋の」的名词化路径完整声明。connector_paths 引用前节的 noun_nominalization，明确要求同一名词化用途：

```json
{
  "id": "no_noun_na",
  "group": "名词",
  "label": "经连体修饰な",
  "steps": ["な"],
  "categories": ["名詞", "代名詞"],
  "forms": ["attributive"],
  "exact_category": true,
  "connector": "attributive_na",
  "connector_paths": ["noun_nominalization"],
  "next": {"equals": {"pos": ["助詞"], "lemma": ["の"], "surface": ["の", "ん"]}}
}
```

degree 输出名词状态便于后续判断接续，其语法功能是程度／约量。当前图示将该规则归入“名词化”的文字需要修订。shika_nai 的否定属于完整唯一选择构式，后续可以继续なかった、なくなる等形容词性变化。

## 接续助词的细分类别

| 细分类别 | kind | 前接条件 | 讲解范围与区别 |
| --- | --- | --- | --- |
| 连用连接及补助入口 | te_connection、te_form、de_form | 对应连用形或判断连用分支；清浊取实际形式 | 统一名称“て形接续”；具体て／で作为形式 |
| 同时或伴随 | nagara_connection、tsutsu_connection | 连用形 | ながら为“伴随”，つつ为“伴随／持续”；具体逆接用法需另有条件 |
| 逆接 | kedo_connection、keredomo_connection、ga_connection | 终止形／连体形；が要求接续助词身份 | “逆接”；格助词が另属自身词条 |
| 文言逆接 | concessive_connection | 已然／假定状态＋ど／ども | “逆接”；连接形用原始字段说明 |
| 原因 | kara_connection、node_connection | 完整谓语；ので可由の＋で组成 | “原因”；格助词から按词法身份区分 |
| 条件 | ba_connection、tara_condition、to_condition、nara_condition | 分别为假定形、たら形式、完整谓语＋接续助词と、なら形式 | “条件接续”；具体成立条件在讲解中展开 |
| 逆接与用途评价 | noni_connection | 连体前项＋のに；单体或の＋に序列 | “逆接／用途与评价”；已有形式证据仍不足以直接选定其中一义 |
| 否定条件 | negative_ba、negative_tewa、negative_to | なければ／なきゃ、なくては／なくちゃ、ないと | “否定条件”；与后项义务构式分别维护 |
| 否定接续 | negative_de、negative_zuni | ないで；文言ず及相应に接续 | “否定接续”“文言否定”；限定对应否定形与助词身份 |
| ては条件 | tewa_condition | 连用形＋て／で＋は，或ちゃ／じゃ | “ては条件”；后续可能组成禁止 |
| 列举 | enumerative | たり／だり；兼容所声明的助动词、接续助词、副助词分词 | “列举”；后续可接する／できる |
| 意向连接 | volitional_to_connection | 动词意向形＋格助词と，且后接する | “意向接续”；普通引用と按自己的身份处理 |
| 名词性变化连接 | nominal_change_ni | 名词性词干＋格助词に，后接なる | “变化接续”；与判断连用に及普通格助词区分 |

相同表面形式的细分类别通过 POS、前接形、最近操作和后续成员共同限定。`negative_de` 的 forms 描述“ないで”序列之前的动词或形容词状态；序列内ない的 cForm 另由 tokens 限定，两个位置分别填写。

## 义务与禁止构式

义务由否定条件前缀加后项なる／いける的否定形式，或だめ形成；禁止由ては前缀加同类后项形成。当前共有九项义务、三项禁止：

| 细分类别 | kind 的完整组成 | 形式 |
| --- | --- | --- |
| なければ义务 | necessity_negative_ba_naru、necessity_negative_ba_ikeru、necessity_negative_ba_dame | なければならない／いけない／だめ |
| なくては义务 | necessity_negative_tewa_naru、necessity_negative_tewa_ikeru、necessity_negative_tewa_dame | なくてはならない／いけない／だめ |
| ないと义务 | necessity_negative_to_naru、necessity_negative_to_ikeru、necessity_negative_to_dame | ないとならない／いけない／だめ |
| ては禁止 | prohibition_tewa_condition_naru、prohibition_tewa_condition_ikeru、prohibition_tewa_condition_dame | てはならない／いけない／だめ |

后项なる或いける必须带任一允许的否定操作。だめ以自身词汇身份表达禁止或不可行，required_operators 为空。后项礼貌、过去、进一步条件等继续保留。当前示例：

```json
{
  "kind": "necessity_negative_ba_naru",
  "prefix": "negative_ba",
  "next": {"equals": {"pos": ["動詞"], "base": ["なる", "成る"]}},
  "required_operators": ["negative", "negative_ba", "negative_to", "negative_tewa", "negative_de"]
}
```

对应的路径源只需声明引用：

```json
{
  "necessity_negative_ba_naru": {
    "label": "义务",
    "predicate": "necessity_negative_ba_naru"
  }
}
```

该路径继承 negative_ba 的完整输入条件。编写规范要求义务与禁止分别引用自己的前缀；省略后项的ないと保留“否定条件”，其识别证据所支持的范围和完整义务构式不同。动词禁止prohibitive为终止形＋终助词な，名称同为“禁止”，前接规则单独维护。

## 提示、限定与举例成分

| 功能细类 | 当前白名单成员 | 文案及元数据要求 |
| --- | --- | --- |
| 话题或对比提示 | は | 功能讲解依具体词条，插入操作保存前项状态 |
| 追加提示 | も | 与て形固定构式中的成员身份区分 |
| 强调或极端举例 | さえ、こそ、すら | 词元及副助词／係助詞身份共同限定 |
| 范围限定 | だけ、まで、しか | 限定的语义范围保留；しかない等完整构式有独立身份 |
| 举例或概括 | なんて、など、とか | とか兼容单体与と＋か；连续形式使用exemplification |

当前普通成员统一使用 insertion，完整とか使用exemplification。上表是语法功能细分；insertion 的 title 暂不能为每个成员单独选取对比、限定等名称，功能讲解继续关联具体助词词条。显示层采用“插入成分”作为结构身份，正文形式显示は、だけ等。

四组位置分别是て形补助间、谓语末尾、名词词干后、连用形焦点位置。每组进一步限定 bases。`たりとか` 的连用位置由最近有效enumerative许可；普通読みとか与読んでとか按当前边界处理。て形后存在GiNZA来源时，连接器、每个插入成员和补助词须满足 [L3](ud_alignment.md#て形后的插入补助) 的定向依存条件。

## 副词的语法细类与接续资格

副词按学校语法和实际作用分为下列细类；情态副词中的拟声、拟态表达可以继续按修饰对象区分。某些词具有多个用法，分类以具体用法为单位。

| 细类 | 典型作用和语例 | 规则编写的关注点 |
| --- | --- | --- |
| 情态／状态副词 | ゆっくり、はっきり等，说明动作方式或状态 | 区分副词修饰、する述语化及と标记；接续资格按词元登记 |
| 拟声、拟态表达 | ふわふわ、どきどき等，描写感受、动作或状态 | 重叠形式与接续能力分别确认；词长和表面重复本身不能确定する资格 |
| 程度副词 | とても、かなり等，限定性质或状态程度 | 修饰功能与する／な／と等资格分别记录 |
| 陈述／呼应副词 | 決して、もし、たぶん等，与否定、条件、推量等呼应 | 记录对应谓语形式；现有活用资格表仅表达词元接续能力，呼应关系归后续语法分析 |
| 时间、频率及数量相关用法 | すでに、しばしば等，限定时间或次数 | 以来源词性核实具体词元；语义近似的名词用法保留自身身份 |
| 指示副词的谓语用法 | ああ、こう等，说明所指方式或状态 | 可接判断词尾的资格与普通修饰用法区分 |

学校语法细类属于意义与用法分类；flags 的五个名字是运行许可集合。应逐词元审定，实际登记可以同时具备多个正向资格。

| 资格 | 数值 | 实际语法要求 | 对应元数据及显示 |
| --- | --- | --- | --- |
| none | 1 | 已审定的无这些静态接续资格 | 普通副词身份；本句显式判断另有上下文路径 |
| suru | 2 | 词干可直接接する | adverb_suru；input_forms: stem；输出動詞；名称“副词动词化” |
| na_da | 4 | 可接な／だ等判断词尾 | 复用判断类；副词前项单独分组，原始身份继续保留 |
| to_suru | 8 | 可采用と标记，并在许可时接する | adverb_to输出other；adverb_to_suru输出動詞；名称“状态修饰”“状态述语化” |
| tari | 16 | 可按タリ相关接续使用 | 复用相应タリ规则与本体范式；前接树明确副词类别 |

none 与其他标志互斥；其他四位可组合。当前形状词タリ的明确POS默认提供tari与to_suru，形状词一般默认提供na_da；审定映射优先。`Capabilities::has` 为任一位命中，路径 capabilities 要求全部位命中；各处取值应与所需许可对应。

以下是 reviewed.json 中的一项完整记录，属于sources[].entries[]：

```json
{
  "lemma_id": "68",
  "lemma": "ああ",
  "reading": "アア",
  "status": "reviewed",
  "reason": "指示副词，可作谓语「ああだ」及连体「ああな」使用，不直接接する。",
  "flags": ["na_da"]
}
```

词元ID使用十进制字符串，reading核对lForm。编译移除理由和审定状态，保留ID索引下的lemma、reading和数值flags。新词应记录实际词元、读法、许可形式和理由；候选资料中的同形词或其他词元资格需分别审定。

## 文言规则的细分类别

### 用言范式与词形表

classical_adjective 汇总文言用言自身的活用识别。当前五项范式如下，前三列属于词形声明，语义分类由具体词汇说明。

| 代表词元 | 范式 | lemma_suffix | 表内词形特点 |
| --- | --- | --- | --- |
| よし | ク活用 | し | よ＋く／から／かり／し／き／かる／けれ／かれ |
| 美し | シク活用 | し | 美＋しく／しから／しかり／し／しき／しかる／しけれ／しかれ |
| 静かなり | ナリ活用 | なり | 静か＋なら／なり／に／なる／なれ |
| 堂々たり | タリ活用 | たり | 堂々＋たら／たり／と／たる／たれ |
| ごとし | ク活用 | し | ごと＋く／し／き；表内仅声明四种活用形 |

classical_forms 的 forms 值为词干之后的词尾数组；classical_auxiliaries 的 forms 值为助动词完整实际词形数组。两张表的写法分别遵循这个范围。范式识别同时核对词元或cType、lemma_suffix和实际词面。

ク活用范式的完整示例：

```json
{
  "lemma": "よし",
  "lemma_suffix": "し",
  "ctype": ["文語形容詞-ク"],
  "class_name": "ク活用",
  "forms": {
    "未然形": ["く", "から"],
    "連用形": ["く", "かり"],
    "終止形": ["し"],
    "連体形": ["き", "かる"],
    "已然形": ["けれ"],
    "命令形": ["かれ"]
  },
  "connection": ["用言の連用形", "体言"]
}
```

`class_name` 和 `connection` 是现有的文字说明字段；接续许可使用具名inputs。编写时将class_name限制在该条的真实范式名称，将connection统一为与inputs对应的简短中文条件。文言表forms键采用L5列出的六种活用形，省略该助动词没有的形式。

### 助动词按功能和前接细分

下表完整列出32项助动词声明。前接列概括当前inputs；“普通终止组”指终止形及对所列ラ变类cType改用连体形的分支，具体cType集合以源文件为准。负面前接限制和来源兼容分支必须写入机器条件，讲解说明其语法依据。

| 功能细类 | kind | 原型 | 当前inputs的区别 |
| --- | --- | --- | --- |
| 被动、可能、自发、尊敬 | classical_ru | る | 四段、ナ变、ラ变未然形 |
| 被动、可能、自发、尊敬 | classical_raru | らる | 排除前述三类；当前登记未然形及连用形 |
| 使役 | classical_su | す | 四段、ナ变、ラ变未然形 |
| 使役、尊敬 | classical_sasu | さす | 未然形，排除前述三类 |
| 使役、尊敬 | classical_shimu | しむ | 未然形 |
| 过去 | classical_ki | き | 当前登记连用形及终止形 |
| 过去、咏叹 | classical_keri | けり | 当前登记连用形及终止形 |
| 完了、强意 | classical_tsu | つ | 连用形 |
| 完了、自然成立 | classical_classical_nu | ぬ | 当前登记连用形及未然形 |
| 完了、存续 | classical_tari_perfect | たり | 用言连用形 |
| 断定 | classical_tari_copula | たり | 名词、代词、形状词、副词的词干 |
| 完了、存续 | classical_ri | り | サ变未然形或四段已然形 |
| 推量、意志、婉曲 | classical_mu | む | 未然形 |
| 推量、意志 | classical_muzu | むず | 未然形 |
| 反实、愿望 | classical_mashi | まし | 未然形 |
| 过去推量、传闻 | classical_kemu | けむ | 连用形 |
| 现在推量、传闻 | classical_ramu | らむ | 普通终止组 |
| 推定 | classical_rashi | らし | 普通终止组 |
| 过去推定、咏叹 | classical_kerashi | けらし | 连用形 |
| 推定、婉曲 | classical_meri | めり | 普通终止组 |
| 推量、当然等 | classical_beshi | べし | 普通终止组；连体分支额外列出ズ／ヌ型 |
| 推量 | classical_beranari | べらなり | 普通终止组 |
| 否定 | negative_zuni | ず | 形容词未然形，或动词／文言用言的未然形及连用形 |
| 否定推量、否定意志 | classical_ji | じ | 未然形 |
| 否定推量、禁止等 | classical_maji | まじ | 当前普通分支登记终止形及连用形；ラ变类用连体形 |
| 愿望 | classical_tashi | たし | 连用形 |
| 愿望 | classical_mahoshi | まほし | 未然形 |
| 断定、存在 | classical_nari_copula | なり | 体言词干或用言连体形 |
| 传闻、推定 | classical_nari_hearsay | なり | 普通终止形；当前ラ变类分支登记连用形 |
| 比况 | classical_gotoshi | ごとし | 体言词干或用言连体形；名词の接续另有序列 |
| 比况 | classical_gotokunari | ごとくなり | 体言词干或用言连体形；另支持ごとく＋なり组合 |
| 比况、例示 | classical_yaunari | やうなり | 当前登记名词、代词、形状词词干 |

同形なり、たり的规则应当同时区分前接、功能名和concept。输入集合中允许的每个形式都需要具体依据。上表所列らる的连用形、き／けり的终止形、完了ぬ的未然形、まじ的连用形以及传闻なり的ラ变连用分支，应逐项区分来源词形兼容与语法许可，作为文言规则审阅项保留；其依据尚未在源字段中说明。扩展规则时采用已经解释清楚的前接组。

### 具名输入与对应前接树

classical_ri 的完整助动词声明展示了同一功能内两种不同前接。输入中的id是稳定引用，词类、连接形和活用型同时限制前项：

```json
{
  "lemma": "り",
  "rule": "classical_ri",
  "class_name": "ラ変型",
  "ctype": ["助動詞-リ"],
  "forms": {
    "未然形": ["ら"], "連用形": ["り"], "終止形": ["り"],
    "連体形": ["る"], "已然形": ["れ"], "命令形": ["れ"]
  },
  "connection": ["サ変の未然形と四段の已然形"],
  "inputs": [
    {"id": "input0", "categories": ["文語用言", "動詞"], "forms": ["irrealis"], "ctype": ["文語サ行変格", "サ行変格"]},
    {"id": "input1", "categories": ["文語用言", "動詞"], "forms": ["conditional"], "ctype": ["文語四段", "四段"]}
  ]
}
```

路径布局以input选择前接条件，以categories选择显示分组。两组输入均含两个类别，共有四个当前分支：

```json
{
  "classical_ri": {
    "label": "存续／完了",
    "auxiliary": "classical_ri",
    "paths": [
      {"id": "input0_classical", "input": "input0", "categories": ["文語用言"], "group": "文言用言", "label": "未然形 · 文語サ行変格／サ行変格", "steps": ["未然形 · 文語サ行変格／サ行変格"]},
      {"id": "input0_verb", "input": "input0", "categories": ["動詞"], "group": "动词", "label": "未然形 · 文語サ行変格／サ行変格", "steps": ["未然形 · 文語サ行変格／サ行変格"]},
      {"id": "input1_classical", "input": "input1", "categories": ["文語用言"], "group": "文言用言", "label": "已然形 · 文語四段／四段", "steps": ["已然形 · 文語四段／四段"]},
      {"id": "input1_verb", "input": "input1", "categories": ["動詞"], "group": "动词", "label": "已然形 · 文語四段／四段", "steps": ["已然形 · 文語四段／四段"]}
    ]
  }
}
```

编译后forms、ctypes从input继承，exact_category为true，组内ctype_mode为classical_prefix。显示设计将标签统一为“サ变”“四段”，条件分别写“未然形”“已然形”，避免同一内容在标题和条件行重复。现代动词与文言用言的两个分组继续依据真实状态分别保存。

## 同类规则的内部编排规范

每个细分类别先确定共同前接与共同输出，再列各成员差异。推荐按以下顺序维护一组规则：

1. 明确品词与功能，例如“接续助词—原因”“补助用言—授受”“副词—陈述用法”。语法细类写入文档与rule_group的统一用语；词元和POS写入机器条件。
2. 记录完整许可：当前类别、连接形、必要cType、前后项、词元资格和来源要求。前接说明与机器条件逐项对应。
3. 定义输出类别、连接形和继承方式。普通动词、形容词、名词化、判断词干、插入语各使用对应的元数据组合。
4. 确定独立operator、variant或共享路径。仅前接显示分支不同的规则优先复用同一功能身份；同形不同功能保留可区分的证据或独立kind。
5. 统一label、title、rule_group、input_label、路径group／label／steps及概念讲解。各字段采用自己的文案职责，格式见显示规范。

数组顺序按实际用途编排：特指形式与明确排除放在相邻一般规则之前；同一语法家族的分词变体相邻；连续词尾优先处理完整形式；前接路径在各词类下按一般活用、音便、特殊前置操作、上下文用途排列。路径须尽量互斥，运行会保留全部命中并报告冲突。

各类规则的内部一致性至少涵盖：识别成员与POS一致、连接许可与execution输入一致、输出类别与后续活用一致、前接树与实际许可一致、中文名称与具体功能一致。当前编译器只能检查其中一部分，显示规范列出已定位的内容差异及实施位置。

## 内容依据与代表样例

| 内容 | 直接依据 |
| --- | --- |
| 全部operator、功能与执行元数据 | [operators目录](../../configs/unidic-nlp/morphology/source/operators/) |
| 同形识别、固定词尾和构式前后项 | [features](../../configs/unidic-nlp/morphology/source/rules/features.json)、[connections](../../configs/unidic-nlp/morphology/source/rules/connections.json)、[sequences](../../configs/unidic-nlp/morphology/source/rules/sequences.json)、[predicate_links](../../configs/unidic-nlp/morphology/source/rules/predicate_links.json) |
| 输入许可和显示分支 | [显式路径](../../configs/unidic-nlp/morphology/source/paths/explicit.json)、[文言助动词](../../configs/unidic-nlp/morphology/source/classical/auxiliaries.json)、[文言路径](../../configs/unidic-nlp/morphology/source/paths/classical.json) |
| 实际字段语义 | [morphology_config.rs](../../crates/readux-nlp/src/morphology_config.rs)、[morphology_machine.rs](../../crates/readux-nlp/src/morphology_machine.rs)、[adverb.rs](../../crates/readux-nlp/src/adverb.rs) |
| 既有例句及当前预期 | [活用重构样例](../../crates/readux-nlp/tests/morphology_reconstruction.rs)、[文言样例](../../crates/readux-nlp/tests/morphology_stages_5_6.rs)、[谓语集成样例](../../crates/readux-core/tests/p4_language.rs)、[指南路径样例](../../data/validation/morphology_stage11_cases.json) |

当前样例包括“読みたい→読みたくない／読みたかった”“諦めてくれ”“読んではいる”“気付きたくなかっただけなのだ”“キャンセルになるかもしれなくなってしまった”等。同一细类的内容审阅同时核对一个正例和相邻边界：如样态そう／传闻そう、接尾辞らしい／推量らしい、名词化の／终助词の、てならない／てはならない。指南样例用于实际范围和显示核对，规则的语言依据与限定集合由上述分类和字段契约说明。
