# 活用链合成样本实验（2026-10-26）

## 实验范围

本实验使用真实 UniDic 分词和 readux-nlp stdio，观察活用链重构后的六类执行器在合成句中的组合结果。实验只记录语言分析输出，不进入代码排查。

原始结果：

- data/validation/synthetic_morphology_raw.jsonl：第一轮 14 个样本。
- data/validation/synthetic_morphology_round2.jsonl：第二轮 20 个相似样本。
- data/validation/synthetic_morphology_round3.jsonl：第三轮 23 个边界样本。

三轮共 57 个样本、101 条形态链。统计字段为 morphology.chains[*].status、terminal、operators 和 morphology.diagnostics。

## 结果

読みたくなかった形成 conjugation → desire → negative → past，状态为 resolved、terminal=true。読ませられなかった形成 conjugation → causative → passive_potential → negative → past，受身／可能候选保留但不阻断终止。読みました、読みません、読みたいです、読ませられました同样收束。

読んでしまった、読んでいなかった、読んでおいた、読み始めた、読みすぎた均命中连接器和补助词规则，但出现 support_ownership_candidate，前项和补助词分别成为局部链，补助链状态为 candidate。読んでいられない进一步出现 rule_order_conflict。

読みませんでした稳定产生 na_judgement_input_state 和 rule_order_conflict；静かではありませんでした也将候选链单独保留。静かだったらしい出现判断过去后接らしい的顺序冲突。

行かざるを得なかった的 negative_zuni 命中并正常继续。行かぬ命中 classical_nu 但为 candidate；ごときものなり、ごとくなり命中 classical_gotoshi、classical_adjective 但保持 pending 并生成 unattached_auxiliary；べきだった、べきではない也保持 pending。

読まないたい末尾たい为 pending 并产生 unattached_auxiliary；読みたせる和読みましたい被分成多个独立链。

## 统计

| 指标 | 数量 |
| --- | ---: |
| 样本 | 57 |
| 形态链 | 101 |
| resolved | 75 |
| candidate | 17 |
| pending | 9 |
| 终止链 | 52 |
| 非终止链 | 49 |
| 产生诊断的样本 | 20 |

| 诊断 | 次数 | 代表样本 |
| --- | ---: | --- |
| support_ownership_candidate | 12 | 読んでしまった、読んでいなかった、読み始めた |
| rule_order_conflict | 8 | 読みませんでした、静かだったらしい、高くなくなった |
| unattached_auxiliary | 9 | ごときものなり、ごとくなり、べきだった |
| na_judgement_input_state | 3 | 読みませんでした、静かではありませんでした |
| classical_input_state | 1 | 行かぬ |

## 模块判断

1. 动词循环类短链稳定，使役、受身／可能、否定、礼貌、过去和条件可组合。
2. イ形容词转换类的たい、ない、过去和插入语组合稳定；高くなくなった存在变化构式边界冲突。
3. 补助动词本体、て／で连接和递归规则能命中；父子链合并是最集中问题，影响查询范围和终止状态。
4. ナ形容词判断类短链和部分长链可用；判断继续接推量或礼貌过去时的输入约束不足。
5. 文言专用规则能命中；ぬ、ごとし／ごとく、べし常停留在 candidate 或 pending。
6. 插入语白名单在高くはなかった、読むだけだ和长链名词化中保持状态有效；补助父子链限制整体投影。

## 后续复测重点

1. 読んでしまった／読んでいなかった／読んでおいた／読み始めた：确认父链和补助链是否合并为 resolved 递归结构。
2. 読みませんでした／静かではありませんでした：确认礼貌否定、です和过去的顺序及输入类别。
3. 高くなくなった／読んでいなくなった：确认イ形容词转换与 なる 的层级边界。
4. 行かぬ／ごときものなり／べきだった：确认文言词形、现代判断和独立终止的连接。
5. 読まないたい／読みたせる／読みましたい：确认逆序组合进入候选诊断。
