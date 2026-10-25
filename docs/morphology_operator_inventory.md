# 活用规则逐词条覆盖清单

本清单是当前 108 个活用 operator 的架构映射基线。六类、执行器、阶段和排序来自 configs/unidic-nlp/morphology.json 的实际配置；后续逐词条升级时，以此表为入口补充正式 grammar concept、词条正文、接续例句和反例。

## 当前已具备正式执行入口的代表规则

| 规则 | 六类 | 执行器 | 阶段 | 代表形式 | 当前状态 |
| --- | --- | --- | --- | --- | --- |
| causative / passive_potential | 动词循环类 | verb_cycle | voice | せる／れる | 有限动词转移，已参与顺序检查 |
| te_iru / te_shimau / te_oku / te_yuku | 补助动词类 | auxiliary_verb | te_support | ている／てしまう／ておく／てゆく | 父子链、插入语回溯和递归深度已接入 |
| desire | イ形容词转换类 | i_adjective_conversion | desire | たい | 输出形容词性整体 |
| negative | イ形容词转换类与动词循环类 | verb_cycle+i_adjective_conversion | polarity_politeness | ない | 否定和形容词性输出共用规则 |
| ease / difficulty | イ形容词转换类 | i_adjective_conversion | desire | やすい／にくい／づらい | 已建立正式 grammar concept、接续条件、例句和反例；后续按イ形容词活用继续连接 |
| excessive / inceptive / continuative_aspect / terminative / iterative_aspect / inchoative / corrective / incomplete / completion / reluctance / failure / missed_opportunity | 补助动词类 | auxiliary_verb | te_support | すぎる／始める／続ける／終わる／まくる／出す／直す／掛ける／切る／兼ねる／損なう／そびれる | 已统一为连用形补助规则，建立正式 grammar concept、接续条件、例句和反例；父子链递归与同层组合已接入 |
| sou_modality / you_modality / mitai_modality / rashii_modality / obligation / hazu_modality | ナ形容词判断类 | na_adjective_judgement | modality | そうだ／ようだ／みたいだ／らしい／べきだ／はずだ | 已建立独立 grammar concept、sense、接续条件、例句和反例；状态机输出形状词状态仍需补充后续收尾验收 |
| nominalization | 名词化转换 | nominalization | conversion | さ／の | 输出名词状态，已修复名词化回归 |
| negative_zuni | 文言活用类 | classical_conjugation | classical | ずに | 已有文言入口，文言活用表覆盖第一条 |
| は／も／だけ／さえ／こそ等 | 插入语类 | insertion | insertion | 范围保持与对比提示 | 只对声明白名单生效，普通格助词不参与分类 |

## 仍需逐词条完善的内容

1. morphology.chain 概念仍需按语义组迁移到正式 grammar concept；判断类六条已完成第一批资源迁移，待补充判断词尾连续活用和真实链状态验收。
2. 形容词转换规则的第一批状态验收已覆盖 たい→たくない／たかった、ない→なかった、やすい→やすかった；仍需继续覆盖中顿、条件和礼貌收尾。
3. 连用形补助的第一批 12 条已迁移并接入补助执行器；仍需继续验证每条规则的独立词汇边界、后续否定／礼貌／时态和中顿停机。
4. 文言活用需要补齐ク活用、シク活用、ナリ活用、タリ活用和ごとし／ごとき的词条与代表例句。
5. 插入语需要继续区分名词化、のだ解释结构和范围保持三种规则，避免把它们合并为单一展示标签。
6. 句末停机需要补充 terminal_mode 的实际行为测试：句末、从句末、中顿和继续接续。

## 验收方法

每个规则至少需要一条正例、一条边界或反例，以及一条链状态断言。断言内容包括输入类别、前接形、规则阶段、输出类别、父链关系、停机方式和词条 concept。规则完成迁移后，再从本清单移入“已完成”区，并在模块文档中记录日文术语与中文说明。
