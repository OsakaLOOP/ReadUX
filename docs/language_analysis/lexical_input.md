# 正文、分词与基础身份

更新日期：2026-11-09。本页定义当前分析使用的正文、字段和词法身份。来源对齐见 [L3](ud_alignment.md)，状态转换见 [L4](morphology.md)。

## 正文准备

[prepare_text](../../crates/readux-nlp/src/prepare.rs) 接收输入原文，返回 `PreparedText { text, annotations, mapping }`。准备协议为 `readux.prepare-text.v4`。

准备阶段逐字符转换 kyujitai.js 表内的旧字体，保留空白、换行、标点及字符顺序；识别 Markdown 图片标记并移除，保存删除范围和正文锚点；识别「漢字《かな》」形式的作者注音，正文保留基础文字，读法进入 `RubyAnnotation`。注音括号内要求为有效假名，基础范围要求包含汉字。

| 对象 | 保存内容 | 用途 |
| --- | --- | --- |
| `text` | 供 UniDic 与 GiNZA 共同分析的正文 | 词元、结构和活用的共同坐标 |
| `annotations` | 基础文字、作者读法和正文范围 | 作者 ruby、读音核对及查询证据 |
| `mapping.source_text` | 完整输入原文 | 原始内容及书籍位置回溯 |
| `mapping.origins` | 每个正文字符在输入中的位置 | 正文范围投影回原文 |
| `mapping.removed` | 删除的 ruby／图片标记范围及正文位置 | 标记恢复与位置解释 |
| 输入及正文摘要 | 两种文本身份 | 文档及缓存区分 |

旧字转换调用 [kyujitai.rs](../../crates/readux-nlp/src/kyujitai.rs)，数据来自仓库内 `vendor/kyujitai.js/dist/kyujitai.json` 的 `kyuji` 字符表，采用旧字到新字的一对一映射。词语级 `douon` 规则保留给其他词汇处理。分析正文保存转换后的字形，输入原文保存在映射中。

## 正文坐标与来源引用

字符区间统一为 `[start, end)`，计量单位是 Unicode scalar。Rust 使用 `chars()`，Python 使用字符串字符位置，前端在分析范围操作中使用 `Array.from`。UTF-8 的 `byte_range` 另行记录；JavaScript 原生字符串索引和 DOM 文本偏移采用 UTF-16，需要在正文定位接口中转换。

`ProviderToken.char_range`、`MorphemeToken.char_range` 和活用范围都相对于所属分析正文；`ProviderToken.byte_range` 相对于同一正文的 UTF-8 字节。`PreparationMap.source_ranges` 根据 `origins` 返回原文中的一个或多个区间，作者注音标记的删除会使原文范围分离。

整书分析的单元范围相对于全文，单元内分析对象使用局部正文坐标。查询和高亮同时需要单元身份及其 context 起点，转换流程见 [L1](pipeline.md#文档单元与坐标)。

来源词元序号、`m<index>`、活用链 ID、operator 实例 ID 和 GiNZA 节点 ID 均在相应文档或来源中解析。相同文字在书中多次出现时，每次出现保留自己的位置。

## CWJ、CSJ 与语域

[UniDicProvider](../../crates/readux-nlp/src/sources.rs) 通过 Vibrato 读取 2025.12 的 CWJ 或 CSJ 词典。词典默认位置与环境覆盖见[开发说明](../development.md#环境与资源准备)。provider 保存版本、词典身份和 `unidic-node-29.v1` 字段协议。

[routing.rs](../../crates/readux-nlp/src/routing.rs) 提供 `auto`、`cwj`、`csj` 三种策略，版本为 `readux.register-routing.v2`。

| 策略 | 当前行为 |
| --- | --- |
| `cwj`、`csj` | 整个非空输入采用指定词典，原因为 `explicit_selection` |
| `auto` 叙述 | 默认采用 CWJ，原因为 `narration` |
| `auto` 引号段 | 维护「」及『』的嵌套；闭合段满足 `结束索引－开头索引 > 11` 时整段采用 CSJ |
| `auto` 未闭合引号 | 剩余范围满足代码中的长度条件时采用 CSJ，原因为 `open_quotation` |

短引号段随所在 CWJ 叙述处理。`RegisterRouting.runs` 保存各段范围、实际语域和原因，所有段相同时 `selected` 为该语域，混合时为空。`slice` 从全文路由截取单元，保留起点之前已经确定的引号上下文。

## 分词及来源 token

Vibrato 配置为 `max_grouping_len(10)` 和 `ignore_space(true)`。provider 按物理行调用 tokenizer，在一次请求中复用 worker，补回各行字符及字节偏移；单行上限为 8,192 字符。统一层继续从正文提取 token 间及末尾的 `TextGap`。

`ProviderToken` 保存表面串、字符和字节范围、词典类型、左右连接 ID、词成本、累计成本、原始 CSV 与解析后的字段。系统词要求 29 列；未知词接受 6 列或 29 列，结果统一形成 29 个 `FeatureField`。原始空字符串和 `*` 保存在 `raw` 中，解析值为 `None`；缺失的未知词后续字段保留空值。

统一层逐一检查来源 token 与正文切片、顺序和范围。`MorphemeToken.source_index` 回指原始 token，`id` 为 `m<index>`，成员顺序沿用来源序列。

## UniDic 字段

下表列号采用源码中的零起点。

| 列号 | 字段 | 含义与主要消费 |
| --- | --- | --- |
| 0–3 | `pos1`、`pos2`、`pos3`、`pos4` | 四级词性；功能身份、非自立、接尾辞及词典约束 |
| 4 | `cType` | 精确活用型；执行器和接续条件 |
| 5 | `cForm` | 本次出现的活用形及音便；状态和显示标签 |
| 6–7 | `lForm`、`lemma` | 词元读法与词元表记 |
| 8–11 | `orth`、`pron`、`orthBase`、`pronBase` | 出现表记、出现发音、基本表记、基本发音 |
| 12 | `goshu` | 语种 |
| 13–18 | `iType`、`iForm`、`fType`、`fForm`、`iConType`、`fConType` | 词首／词尾变化及结合类型；原子变化证据 |
| 19 | `type` | 词类补充 |
| 20–23 | `kana`、`kanaBase`、`form`、`formBase` | 出现假名、基本假名、语形、基本语形 |
| 24–26 | `aType`、`aConType`、`aModType` | 重音信息，原样保存 |
| 27–28 | `lid`、`lemma_id` | 来源词条与词元 ID，以字符串传输 |

`cType` 与 `cForm` 分别表示活用范式和当前词形；「下一段」与「連用形-一般」属于不同字段。POS 是来源分类，状态机可在组合过程中形成新的类别，来源字段继续保留。

读法按照用途取用。例如「向かっ」的 `kana` 对应出现形式，`kanaBase` 对应基本形式，`lForm` 对应词元。`pron` 保存出现发音，作者 ruby 保存作者指定读法。`MorphemeToken.reading` 当前取 `kana`。

## 查询形式与词元恢复

`unify.rs` 顺序生成 `QueryForm`：出现表面串配 `kana`，`orthBase` 配 `kanaBase`，`lemma` 配 `lForm`。基础生成按相同表记去重，保留首个形式的读音及来源字段。每个形式保存 `kind`、`form`、`reading`、`reading_field`。

可能动词由 [potential_lemma](../../crates/readux-nlp/src/morphology.rs) 检查动词 POS、下一段活用，以及词元或读法与基本表记之间的五段可能对应关系。成立时插入 `potential_lemma` 查询形式，并在活用中保存可能证据。普通下一段动词仍按自身字段处理。

活用链分别保存 `surface_form`、`dictionary_form`、`lemma_form`、`lookup_form` 和 `display_form`。缩约补助等结构进一步生成规范查询形式；规则变体所选的本句原型保存在 operator 的 `normalized_form`，详见 L4、L5。

## 基础身份与接续资格

活用原子以助词、助动词作为功能角色，其他词类初始为词汇角色；补助执行器可根据真实接续建立功能子链。基础功能候选另外收集助词、助动词及二级 POS 以「非自立」开头的词元。

状态机初始化读取 `cForm`、文言表和原始 POS。名词性接尾辞在连接状态中按名词处理，原始来源仍为接尾辞。实体类别用于执行和接续；名词、代词等词干的实际查询与显示继续参考原始 POS。

副词、形状词及部分名词可具有静态接续资格，入口为 [adverb.rs](../../crates/readux-nlp/src/adverb.rs)。配置按 `lemma_id` 关联并核对词元和读法；缺少 ID 时使用词元与读法查找。没有登记项时，形状词和サ变名词的明确 POS 可提供资格。

| 标志 | 数值 | 用途 |
| --- | --- | --- |
| `NONE` | 1 | 已审定的无上述接续资格 |
| `SURU` | 2 | サ变接续 |
| `NA_DA` | 4 | な／だ判断接续 |
| `TO_SURU` | 8 | と及相关动词接续 |
| `TARI` | 16 | タリ接续 |

资格按位组合，未审定时为 `None`。本句上下文可补充判断资格并保存 `adverb:contextual_judgement` 证据。资格影响状态机和显示，维护源及编译关系见 [L5](morphology_configuration.md#副词接续资格)。

## 作者 ruby 的核对

[ruby.rs](../../crates/readux-nlp/src/ruby.rs) 先合并相邻注音，再根据 UniDic 出现假名和假名锚点投影读法。省略明确左边界的 ruby 可在候选范围内寻找能够匹配的后缀，保留修正前候选与确认后的范围。

| 状态 | 当前含义 |
| --- | --- |
| `matched` | 读法一致，或仅存在比较规则允许的大小假名差异 |
| `variant` | 可投影读法与作者读法不同 |
| `unavailable` | 缺少读法或投影有歧义 |
| `unmatched` | 作者范围无法覆盖来源 token |

结果同时保存作者读法、观测读法、原始候选范围、实际正文范围、token 范围及原文区间。读法比较使用统一片假名形式，大小假名兼容另设比较函数；原始字段和作者读法继续按各自用途保存。

## 维护定位

字段定义在 [model.rs](../../crates/readux-nlp/src/model.rs)，来源解析在 [sources.rs](../../crates/readux-nlp/src/sources.rs)，正文准备在 [prepare.rs](../../crates/readux-nlp/src/prepare.rs)，语域在 [routing.rs](../../crates/readux-nlp/src/routing.rs)。前端协议位于 [nlp.ts](../../src/types/nlp.ts)。后续规则的逐条字段组合与具体 JSON 留在 L5 的细化清单中。
