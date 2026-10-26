# 阶段 10：て构式重复实体审计

本记录对应执行计划第 10 阶段。审计范围包括旧 configs/grammar/teConstructions.json、现代活用配置中的 te_* 与 contracted_te_* operator、正式 grammar source/catalog，以及运行时查询入口。

## 重要度与归因

问题重要度为中高。旧文件同时保存「ある、しまう、おく」等补助动词组件词条和 te_* 构式规则，形成“て开头词条”和“本体词条”并存的历史实体层。Rust 运行时实际使用 configs/unidic-nlp/morphology.json 的规则，但前端词条入口仍直接读取旧文件，资源扫描和人工维护也会继续将其误认为正式词条来源。

根因是早期资源把接续形式作为词条建模，后续重构已经将补助动词本体、te_connection 连接器、构式 rule 和正式 concept 分开，但旧资源只完成了内容迁移，尚未完成实体清理。

## 运行时证据

data/validation/morphology_stage10_before.jsonl 保存了 18 个代表样例。运行时的 te_iru、te_shimau、te_oku、te_iku、te_yuku、te_kuru、te_miru、te_kudasaru、te_morau、te_hoshii、te_miseru、te_yamanai 均将 concept 指向补助本体或已有正式构式；三条 contracted_te_* 只承担缩约规则入口，也指向本体 concept。旧文件中的 component ID 没有进入运行时查询。

te_goran、te_naranai 的代表输入在本轮未命中，te_tamaranai 形成候选并记录连接顺序冲突。这些是接续规则或分词边界问题，不能通过保留旧实体解决，后续单独审计。

## 清理方案

保留以下正式层：

- 补助动词本体 concept，例如 grammar.functional.iru 和 grammar.auxiliary.iru；
- morphology.form.te 与 te_connection 连接器；
- te_*、contracted_te_* 作为规则 operator，指向本体 concept；
- 已编译 grammar source 中的正式构式 realization。

删除 configs/grammar/teConstructions.json 这个重复实体资源，新增 configs/grammar/migrations/te_construction_ids.json 保存 19 个 component、18 个 rule 和 3 个缩约 operator 的历史 ID 到正式 concept 或 operator 的迁移关系。前端规则展示读取迁移文件中的规则元数据，查询归属仍来自正式 operator/concept；迁移文件不进入 grammar catalog 或搜索索引。

## 当前阶段状态

修复后组合证据见 `data/validation/morphology_stage10_after.jsonl`。新增验证覆盖 `te_goran`、`te_tamaranai`、`te_naranai`、`te_yamanai` 和愿望／否定后的 `て` 连接；全部样例为 `resolved` 且无诊断。连续后项由 sequence 先合并，再交给补助执行器处理完整范围。

当前阶段状态：重复资源清理、前端迁移和组合查漏已完成。
