# 文档索引

更新日期：2026-11-09。总体文档、语言分析 L0–L6、L11 当前投影、E1–E3 和 M1–M5 已按当前实现整理。其余模块及后续事项见[文档更新总计划](documentation_update_plan.md)。

## 从这里阅读

| 文档 | 内容 |
| --- | --- |
| [开发目标与功能范围](goals.md) | 活用核心成果、当前入口、后续语言分析与应用方向 |
| [总体架构](architecture.md) | 实际分析链路、规则与词条、说明视图、D3 图、静态资源及数据 |
| [开发说明](development.md) | 环境、运行、配置与词条维护、快照制作、构建及针对性验证 |
| [文档更新总计划与追踪](documentation_update_plan.md) | 交接要求、目标文档名称、内容概要、代码范围、现有资料去向和进度 |

接替 agent 先阅读上述四篇。当前任务是整理现有成果，待设计模块保留已确定方向和问题；research 原文保持现状。详细要求及状态以总计划为准。

## 现有模块资料

语言分析的正式入口为 [L0 总览](language_analysis/README.md)，依次包括 [L1 流程](language_analysis/pipeline.md)、[L2 基础输入](language_analysis/lexical_input.md)、[L3 UD 对齐](language_analysis/ud_alignment.md)、[L4 活用状态机](language_analysis/morphology.md)、[L5 规则配置](language_analysis/morphology_configuration.md)、[L6 词条解说](language_analysis/entries.md)。L5配套[规则分类与编写指南](language_analysis/morphology_rule_categories.md)和[显示规范与修订设计](language_analysis/morphology_display_conventions.md)，进一步说明语法细类、字段编排、样例、统一名称、讲解及D3前接树。

当前结果的正文范围、查询、解说及高亮见 [L11 界面投影](language_analysis/presentation.md)。下表列出正式模块及其余资料，阶段记录保留追溯用途。

| 范围 | 现有文档 |
| --- | --- |
| 运行入口 | [E1 桌面端](desktop.md)、[E2 静态 Web 与部署](static_library.md)、[E3 CLI 与本机请求](cli.md) |
| 活用分析 | [状态机](language_analysis/morphology.md)、[配置维护](language_analysis/morphology_configuration.md)、[词条解说](language_analysis/entries.md)；原[配置迁移记录](morphology_storage_refactor_analysis.md) |
| 活用展示与审阅 | [L11 结果与解说投影](language_analysis/presentation.md)；原[阶段 11 展示记录](morphology_stage11_bubble_display.md)、[谓语审阅](morphology_predicate_relations_review.md)、[规则覆盖](morphology_operator_inventory.md)及[阶段记录](morphology_refactor_execution_plan.md) |
| 说明与图组件 | [M3 气泡、侧栏与抽屉](explanation_layout.md)、[M4 D3 分析图与交互控件](analysis_graphs.md) |
| 分析输入与流程 | [流程](language_analysis/pipeline.md)、[基础输入](language_analysis/lexical_input.md)、[UD 对齐](language_analysis/ud_alignment.md)；原[语言分析资料](language_analysis.md) |
| 查询与词典 | [M5 词典、查询目标与聚合](dictionary.md)、[查询及投影问题记录](lookup_issue_tracking.md) |
| 书架及阅读 | [M1 书架、导入与书库](library.md)、[M2 阅读器与替换方向](reader.md)；[会话与调度资料](reader_engine.md)待 M9 整理 |
| 指南内容 | [展示及内容制作资料](grammar_guide_display.md)、[当前 2.1 课程的原内容规格](grammar_guide_1_1.md) |
| 数据与导出 | [收藏、笔记及输出资料](user_state.md) |
| 验证与发行 | [质量验证资料](quality.md)、[桌面发行资料](standalone_distribution.md)、[静态书库](static_library.md) |

语言分析统一组织在 [language_analysis](language_analysis/README.md)。现有核心成果文档已完成；[显示规范](language_analysis/morphology_display_conventions.md)列出的修订继续按清单实施。M6–M9、M13、M14 待整理，L7–L10 及阅读器替换等详细设计按计划暂缓。完整状态见总计划。

## 研究入口

研究资料位于 [research](research/)，本轮保留原文。范围和用途在[开发目标](goals.md#研究内容)、[总体架构](architecture.md#后续方向及研究)及[开发说明](development.md#研究与文档交接)中简述。

外部资料来源与标题索引继续维护在 [sources.md](../sources.md)。
