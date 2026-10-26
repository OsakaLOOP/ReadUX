# 阶段 9：旧活用 operator 迁移审计

本记录对应执行计划第 9 阶段。审计对象是配置中仍将正式 operator 映射到 `morphology.chain` 的 20 个实际规则；执行计划中的 27 是历史统计，当前配置清单以运行时配置为准。

## 重要度与归因

问题重要度为中高。operator 已经能改变状态、范围和停机语义，但统一的 `morphology.chain` 概念让规则树、语法目录、搜索索引和阅读查询无法区分条件、列举、程度、禁止等不同功能。查询层只能看到“活用链”，无法准确进入正式词条，也无法为构式提供接续条件和边界说明。

根因分为三类：一是已有正式 concept 尚未接入 operator；二是配置把不同语法功能压缩为一个旧概念；三是 `insertion` 根据实际语素承载多个词条，不能用固定单一概念表达。

## 真实组合样例

`data/validation/morphology_stage9_before.jsonl` 保存了本轮 20 条 CWJ UniDic 实验的压缩结果。已经成立的组合包括 `食べたら`、`歩きながら`、`考えつつ`、`ここで騒ぐな`、`読んだり`、`読むとか`、`高いくらい`、`早く行け`、`静かなので`、`食べないで`、`行きたがる`、`学生である` 和 `静かさ`。普通格助詞 `が` 未误入连接规则，说明连接条件与边界停机仍然有效。

`春なら` 暴露名词＋なら的现有输入缺口；三条 `てはならない／てはいけない／てはだめ` 样例没有形成预期的整条构式，属于独立的构式识别问题，不能借迁移概念掩盖。 `insertion` 保留父链状态的执行结果正确，但其概念必须随具体插入语归属。

## 迁移目标

| operator | 迁移目标 | 处理判断 |
| --- | --- | --- |
| `desire_outward` | `grammar.modality.tagaru` | 新建外向愿望构式，区别 `たい` |
| `copula_aru` | `grammar.functional.aru` | 复用 `ある` 正式功能词条 |
| `nominalization` | `grammar.functional.sa` | 新建 `さ` 名词化词条，并保留 `の` 的既有功能词条 |
| `ba_connection` | `grammar.particle.ba` | 复用正式 `ば` 概念 |
| `tara_condition` | `morphology.mood.conditional` | 归入假定形概念，规则保留 `たら／だら` |
| `nagara_connection` | `grammar.connective.nagara` | 新建普通伴随接续，区别 `ながらも` |
| `tsutsu_connection` | `grammar.connective.tsutsu` | 新建持续／伴随接续 |
| `concessive_connection` | `grammar.connective.keredomo` | 新建逆接接续，兼容文言 `ど／ども` |
| `prohibitive` | `grammar.sentence_particle.na` | 新建句末禁止 `な` |
| `enumerative` | `grammar.connective.tari` | 新建列举接续，保留 `たり／だり` |
| `insertion` | `grammar.morphology.insertion` | 表示执行器的范围扩展；实际词项仍由其正式词条提供查询 |
| `exemplification` | `grammar.connective.toka` | 新建举例接续 |
| `degree` | `grammar.particle.kurai` | 新建程度／约量副助词 |
| `imperative` | `morphology.form.imperative` | 复用正式命令形概念 |
| `ga_connection` | `grammar.particle.ga` | 复用正式 `が` 概念 |
| `node_connection` | `grammar.connective.node` | 新建 `ので` 原因接续 |
| `negative_de` | `grammar.connective.naide` | 新建 `ないで` 否定接续 |
| `prohibition_tewa_condition_naru` | `grammar.prohibition.tewa_naranai` | 新建整条禁止构式 |
| `prohibition_tewa_condition_ikeru` | `grammar.prohibition.tewa_ikenai` | 新建整条禁止构式 |
| `prohibition_tewa_condition_dame` | `grammar.prohibition.tewa_dame` | 新建整条禁止构式 |

修复将保持执行器的状态转换、输入限制和停机属性，只迁移语义归属，并为缺少正式资源的构式建立独立 catalog 条目。 `insertion` 的实际语素归属在查询层继续优先使用具体词项的正式 concept。

当前阶段状态：审计完成，迁移实现待提交。
