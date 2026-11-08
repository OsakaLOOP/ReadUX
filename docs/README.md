# 文档索引

更新日期：2026-11-09。项目目标、架构和开发入口已按当前实现及用户说明重新整理。其他现有模块文档按[文档更新总计划](documentation_update_plan.md)继续重写和整合。

## 从这里阅读

| 文档 | 内容 |
| --- | --- |
| [开发目标与功能范围](goals.md) | 活用核心成果、当前入口、后续语言分析与应用方向 |
| [总体架构](architecture.md) | 实际分析链路、规则与词条、说明视图、D3 图、静态资源及数据 |
| [开发说明](development.md) | 环境、运行、配置与词条维护、快照制作、构建及针对性验证 |
| [文档更新总计划与追踪](documentation_update_plan.md) | 交接要求、目标文档名称、内容概要、代码范围、现有资料去向和进度 |

接替 agent 先阅读上述四篇。当前任务是整理现有成果，待设计模块保留已确定方向和问题；research 原文保持现状。详细要求及状态以总计划为准。

## 现有模块资料

下表提供现存文档入口。各篇处于待整理状态，其中的目标、实施阶段和历史统计按总体文档及实际代码核对。

| 范围 | 现有文档 |
| --- | --- |
| 活用分析 | [状态机设计](morphology_state_machine_design.md)、[配置维护](morphology_configuration.md)、[配置协议](morphology_storage_refactor_analysis.md) |
| 活用展示与审阅 | [气泡及接续展示](morphology_stage11_bubble_display.md)、[谓语关系审阅](morphology_predicate_relations_review.md)、[规则覆盖清单](morphology_operator_inventory.md)、[阶段记录](morphology_refactor_execution_plan.md) |
| 说明界面 | [气泡、侧栏与抽屉布局资料](explanation_layout.md) |
| 分析输入与流程 | [来源与对齐](nlp_sources.md)、[语言分析资料](language_analysis.md) |
| 查询与词典 | [词典资料](dictionary.md)、[查询及投影问题记录](lookup_issue_tracking.md) |
| 书架及阅读 | [书库与阅读器](reader.md)、[文档会话及分析调度](reader_engine.md) |
| 指南内容 | [展示及内容制作资料](grammar_guide_display.md)、[当前 2.1 课程的原内容规格](grammar_guide_1_1.md) |
| 数据与导出 | [收藏、笔记及输出资料](user_state.md) |
| 验证与发行 | [质量验证资料](quality.md)、[桌面发行资料](standalone_distribution.md)、[静态书库](static_library.md) |

语言分析后续统一组织在 `docs/language_analysis/`，包括基础输入、UD 对齐、状态机、规则配置、词条和结果投影。该目录及其他待编写文档的完整清单见总计划，随实际交付更新链接。

## 研究入口

研究资料位于 [research](research/)，本轮保留原文。范围和用途在[开发目标](goals.md#研究内容)、[总体架构](architecture.md#后续方向及研究)及[开发说明](development.md#研究与文档交接)中简述。

外部资料来源与标题索引继续维护在 [sources.md](../sources.md)。
