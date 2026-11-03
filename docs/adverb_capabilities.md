# 副词接续资格与活用链

## 问题与归因

UniDic 2025.12 的副词记录缺少完整的述语化资格。「そう」「どきどき」「もっと」共享 right_id=66，后接相同词条时的矩阵代价完全相同。「しっかり」的右 ID 差异对应「シ濁」词首变化。连接代价无法独立完成分类。

同词元的词性记录提供部分资格。「ふわふわ」的副词与一般形状词共享词元 ID，「堂々」的副词与タリ形状词共享词元 ID。仅有副词记录的「はっきり」「どきどき」需要审定。词形规则存在「度々」「益々」「すっかり」「てっきり」等反例，作为候选线索保存。

## 资格与审定状态

主分类为可活用／不可活用，内部标志可叠加，原始 POS 保留。

| 标志 | 数值 | 接续资格 |
| --- | --- | --- |
| none | 1 | 明确不可活用，与其他标志互斥 |
| suru | 2 | 直结する及其活用形 |
| na_da | 4 | だ、な、で、に、なら及です |
| to_suru | 8 | 状态修饰と及とする |
| tari | 16 | タリ词尾たる、たり、と等 |

未审定使用空值，与明确 none 分开。任一正向资格成立即为可活用。to_suru 同时许可状态修饰と和とする，审定时核对二者。普通格助词の不纳入本轮活用资格。链内 contextual_judgement 证据标记用于本句恢复的判断资格，局部结果不写回静态映射。

## 数据处理

`scripts/build_adverb_lexicon.py export` 单次顺序读取 CWJ lex.csv，按词元 ID 去重副词，保存词元、读法、全部表记、同词元词性。扫描时保存相关词性记录，聚合后筛出副词。词元 ID 使用字符串。

同词元一般形状词生成 na_da 候选，タリ形状词生成 tari、to_suru 候选，サ变可能名词生成 suru 候选。表记与读法相同的其他词元单列候选关联，保留词元身份、词性与候选资格，供人工筛查同音异义。关联内容随词条表保存，后续合并无需再次读取 UniDic。

`merge` 合并多个来源文件；每条提案含词元身份、资格、审定状态、理由和诊断例句。一致的已审定来源可以发布，冲突记录和未审定记录保留在待审结果。文件顺序不构成覆盖优先级。

映射表分 UniDic 资料提取、LLM 分类、人工复核与运行映射整合。UniDic 提供词元 ID、词元、读法、表面形、同词元词性及同形同音 lemma 资料。LLM 按 `suru`、`na_da`、`to_suru`、`tari`、`none` 分类，返回资格、状态和简短理由；BCCWJ 词频命中的候选再按人工规则复核，确认后标为 reviewed。诊断例句属于可选审计材料。

统一 LLM 分类入口为 `scripts/classify_adverbs_llm.py`。`prepare` 从 UniDic catalog 提取词元 ID、词元、读法、全部表面形、词性、同形同音 lemma 的表面形和词性，并按每批 20 条写入 `data/adverbs-llm/llm_batches.json`。`classify --batch 1,3,8-10` 只处理指定批次，结果持续写入 `data/adverbs-llm/llm_results.json`；`html` 始终只生成一个 `data/adverbs-llm/llm_review.html` 审核页，分类后自动更新。脚本顶部预留 `API_KEY`，请求使用 `https://api.deepseek.com/responses` 的 `deepseek-flash` 模型，通过 `reasoning.effort` 设置思考强度，通过 `stream=true` 实时读取 SSE，并通过 Responses `text.format.type=json_schema` 强制输出 `adverb_capability_batch` schema。输出字段限于词元身份、资格、状态和简短理由。HTTP 错误、流中断和 JSON 校验失败均写入结果文件并显示在同一审核页，批次继续处理。

UniDic 提取和运行映射整合命令如下：

```powershell
python -X utf8 scripts/build_adverb_lexicon.py synthesize --catalog .agents/analysis/adverbs/catalog.json --output configs/unidic-nlp/morphology/source/adverbs/reviewed.json
python -X utf8 scripts/build_adverb_lexicon.py fill-unidic --catalog .agents/analysis/adverbs/catalog.json --output configs/unidic-nlp/morphology/source/adverbs/unidic_multi.json
python -X utf8 scripts/build_adverb_lexicon.py integrate-llm --results data/adverbs-llm/llm_results.json --frequency data/adverbs-llm/candidate_frequency_global.json --output configs/unidic-nlp/morphology/source/adverbs/reviewed.json
```

正式映射位于 `configs/unidic-nlp/morphology/source/adverbs/reviewed.json`。`integrate-llm` 汇总完整 LLM reviewed 结果，并将 BCCWJ global 词频命中的 candidate 按复核规则转为 reviewed 后写入同一运行来源。运行资格以词元 ID 为主，并核对词元和读法；没有 ID 的构造输入采用词元与读法精确匹配。诊断例句为可选审计材料，不影响运行资格编译。

```powershell
python -X utf8 scripts/build_adverb_lexicon.py export --lexicon experiments/unidic-source/cwj-full/lex.csv --output .agents/analysis/adverbs/catalog.json
python -X utf8 scripts/build_adverb_lexicon.py merge --catalog .agents/analysis/adverbs/catalog.json --input 待审来源.json --output .agents/analysis/adverbs/merged.json --mapping-output .agents/analysis/adverbs/reviewed.json
npm run morphology:build
```

LLM 审定映射与 BCCWJ 高频候选已经接入；UniDic 同词元及同形同音关联作为审核资料保留在独立来源文件。

## 状态机

链保存词干资格，后续词尾采用自身活用状态。一般形状词、タリ形状词、サ变可能名词从原始 POS 恢复已有资格。

直结する要求 suru；とする要求前一步为状态修饰と且具备 to_suru。资格检查先于词汇合并和外部词界建议，失败时结束前链。后面的する独立建立动词链；「まだしていない」「すぐする」「もっとする」保持独立查询。普通动词「ふわふわ浮かぶ」保持词界。

新增 adverb_suru、adverb_to、adverb_to_suru 使用专用执行器。する接入后输出动词状态，继续接否定、礼貌、时态、て形和补助。词干保留自身词典查询形，する作为语法成分提供查询。

副词词干保持「副詞」类别。紧接 UniDic 明确标出的判断助动词时，允许本句恢复判断接续资格；首个词尾的前项状态与命中路径均保留副词身份。判断词尾接入后输出形状词性整体状态，否定、过去、名词化和补助等后续规则读取整体状态。明确 none 的词条维持不可活用标签，并使用「无活用副词的句中判断接续」分支；局部恢复的资格仅影响本句。本机 CWJ 将「とてもではない」中的で标为格助词，该实际输入保留原始边界。

だ、な、で、に、です、だった／でした、なら、だろう／でしょう的接续树均有「副词」前接分支，分支条目说明对应的判断活用方式。运行条件复用判断兼容机制，路径识别严格核对副词类别与资格；图示中的副词路径与ナ形容词路径分别命中。な后接の／ん沿用名词化规则，连接器引用同时接受副词的な路径。タリ词尾按资格或原始形状詞‐タリ身份接入，文言断定树包含副词的タリ活用分支。

## 查询与图示

新增规则保存 operator、concept、前接路径与实际命中。接续树声明所需资格；とする引用状态修饰と的路径。する节点的活用信息取自当前 token；副词词干显示「词干 · 可活用」，并列出サ变、判断、トス、タリ中已具备的活用方式。判断资格使用「判断活用（だ・な・で・に）」标签。明确不可活用词干继续显示无活用。

资格随分析链传递，前后端共用数值定义。查询窗口、词干节点、活用节点读取同一资格。编译映射参与既有缓存标识。旧快照缺少资格时沿用原始 POS 和链内状态。

## 最终验证

覆盖标志组合与 none 互斥、来源冲突、单次词表导出、无映射默认边界、する／とする正反例、判断收尾、タリ词尾，以及词条和接续树。正式表保持空表，受控资格由验收夹具注入。

2026-11-02 验证结果：单次 CWJ 扫描导出 4,858 个副词词元；同词元候选、跨词元关联、来源冲突、未审定筛查和 none 互斥通过。Rust 库测试 58 项及活用重构测试 15 项通过，19 条真实 UniDic 输入覆盖新增接续与词界反例。`scripts/verify_adverb_capabilities.mjs` 读取同一验收产物，核对新增规则的正文、例句、查询入口、命中路径、连接器、词干标签和布局坐标。TypeScript 检查、正式分析服务构建及两项配置编译检查通过。

正式服务确认「そうなんだから」的な／ん／だ／から、「そうでない」的で／ない、「堂々たる」的たる均产生语法查询目标。「まだしていない」「すぐする」保留独立する链。正式映射为空时，副词直结する保持默认边界。

原有 morphologyGraph.test.mjs 共 15 项，13 项通过，两项布局断言失败；对 HEAD 中原有投影代码复核，断言差异完全相同，分别为句中折行中心 `209.44 / 157.44` 和前接节点中心 `130.25 / 95.5`。本轮新增接续的图示专项验证通过。

副词前接分支调整后的专项验收扩展为 32 条真实分词记录及 2 条受控词性变体，核对 19 个副词判断／タリ分支。だ、な、で、に、です、过去、条件和推量均命中独立副词分支；判断词尾后输出新的整体状态。延伸样例覆盖「そうでなかったのだ」「そうでなくていいのだ」「そうであったのだ」「そうでしたから」「いろいろになっている」。后续否定、过去、许可、名词化、理由和补助继续复用原有接续条件。Rust 的 58 项库测试及 15 项活用重构测试、TypeScript 和配置检查通过。

```powershell
$env:READUX_ADVERB_ACCEPTANCE_REPORT = (Join-Path (Get-Location) '.agents/analysis/adverbs/acceptance.json')
cargo test -p readux-nlp --lib --target-dir target-lookup-review
node --experimental-strip-types scripts/verify_adverb_capabilities.mjs
```
