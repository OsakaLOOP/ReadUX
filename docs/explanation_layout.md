# 气泡、侧栏与抽屉

更新日期：2026-11-09。本文为 M3，说明阅读器和语法指南共用的说明内容、布局状态、测量及交互。正文范围与内容映射见 [L11](language_analysis/presentation.md)，D3 图及滚动视区见 [M4](analysis_graphs.md)，词典状态见 [M5](dictionary.md)。

## 内容与宿主

说明界面围绕当前查询对象组织词典、活用和接续三个内容区。词典显示整体词及当前位置的内部词，活用显示当前身份、实际活用、词形变化、功能摘要和例句，接续显示完整句中组成及当前规则的前接树。

| 内容或状态 | 当前所有者与协作 |
| --- | --- |
| 正文查询对象、内部对象、详情和锚点 | 阅读器 `ReaderDocumentView.vue`；指南 `GrammarGuideExample.vue` 各自维护 |
| 词典加载、错误、历史与响应代次 | 对应宿主的整体／内部查询通道分别维护 |
| 活用词条与图数据 | `lookupPresentation.ts`、`morphologyBubble.ts` 从分析及配置生成 |
| 词典内容视图 | `TooltipPanel.vue` 及 `components/dictionary/` |
| 活用与接续容器 | `ReaderExplanationBubble.vue`、`ReaderMorphologyEntry.vue`、`ReaderConnectionTree.vue` |
| 布局模式、激活及内容 key | `morphologyLayout.ts`，与当前词典结果分别管理 |

阅读器直接组合说明组件，指南经 `ExplanationPopover.vue` 组合。两者分别管理查询生命周期，共用投影、测量和图组件。`useExplanationSession.ts`、`useExplanationInteraction.ts` 当前由 `QualityAuditPane.vue` 引用，属于保留的质量界面调用；主阅读器和指南例句采用各自的查询状态。

## 三种视图与激活

顶部工具栏使用一个 `MorphologyLayoutToggle` 切换悬浮和固定布局。状态创建时 `docked=true`、`active=false`，宽度由 media query 更新。

| 条件 | mode | 当前行为 |
| --- | --- | --- |
| 悬浮布局 | `modal` | 正文悬浮或点击打开气泡，词典与活用／接续分别定位 |
| 固定布局，宽度大于 960 px | `panel` | 点击后展开独立右侧栏 |
| 固定布局，宽度不超过 960 px | `sheet` | 点击后展开底部抽屉 |

`docked` 表示布局选择，`active` 表示当前说明是否激活，`contentKey` 表示所选内容。切换词条会在原位置更新内容，布局切换沿用当前对象。布局选择保存在组件状态中，页面创建采用默认值。

指南面板提供共享 layout state，各例句注入同一对象，并通过 `readux:guide-lookup` 协调当前激活例句。固定内容使用延迟 Teleport 进入始终存在的工作区容器；例句激活时创建说明内容。

## 悬浮气泡与定位

正文悬浮使用 60 ms 意图延迟；离开正文及说明区域后采用 140 ms 关闭宽限，同一次离开过程保留既有计时。进入面板取消关闭，选择新对象时更新相应通道，迟到响应按请求代次处理。滚动、对象失效与页面退出由宿主结束或更新当前说明。

[geometry.ts](../src/explanation/geometry.ts) 的 `snapshotRect` 保存屏幕锚点，`measureIntrinsicPanel` 测量自然尺寸。词典双面板使用 `placeExplanationPanels`，活用与接续组合使用 `placeMorphologyPanels`；阅读器和指南调用相同几何接口。

活用气泡靠近当前悬浮词条。设同向可用高度为 F、反向为 R、活用气泡的独立显示高度为 E：当 `R > F - E` 时，接续气泡位于词条反向；其他情况沿活用气泡外侧堆叠。页面边距为 12 px，词条与面板的上下间距为 10 px；省略词条气泡时按单个接续面板定位。

面板宽度按内容和视口分配，高度受自然高度及可用空间约束。图通过 `data-natural-width/height` 提供完整尺寸，内部滚动区域保持独立测量。ResizeObserver 在内容、图布局及尺寸变化后触发定位。

## 固定侧栏

右侧栏占据独立网格列，宽度为 `clamp(340px, 32vw, 400px)`，列宽从零展开或收回。正文文本宽度预先按展开后的阅读区约束，动画期间保持行宽；外围工作区随列宽变化。

固定内容依词典、活用、接续顺序排列，采用分区标题、20 px 内边距及横向分隔。整组内容使用一个纵向滚动区，接续图采用 flow 模式按完整高度显示。活用表头的分隔线随实际内容显示，名称、词形与当前活用信息分别占据相应区域。

## 底部抽屉

抽屉左右各留 12 px，底边贴合屏幕，顶部圆角。22 px 高的把手与内容滚动区分离，触摸板及内容滚动保持在内容区。把手正上方留 32 px 的交互保护区域。

[useExplanationSheet.ts](../src/composables/useExplanationSheet.ts) 读取视口高 V、首个词典面板高 D 和整组内容高 C，调用 `morphologySheetSizes`：

| 高度 | 当前计算 |
| --- | --- |
| 最低 | 80 px |
| 最高 | `max(80, min(V - 96, C + 22))` |
| 默认停靠 | `min(最高, max(240, min(V × 0.55, D + 22)))` |

激活或切换内容后清除手动高度标志，重新测量并回到内容顶部；低于默认位置时展开，高于默认位置时保留现有高度并受最高位置约束。首次激活由零高度进入，异步加载期间保留已有高度，内容完成后再测量。活用对象在固定布局中可进一步滚动到活用区，宿主负责选择相应内容位置。

把手拖动直接改变高度，释放时吸附至最低、默认或最高位置。`ArrowUp` 逐级展开，`ArrowDown` 逐级收纳，`Home` 至最低、`End` 至最高，`Enter` 或空格切换高低位置。内容更新后重新按当前词典和完整内容决定停靠位置。

抽屉展开或切换对象时，宿主根据正文锚点及抽屉顶部滚动阅读区，使当前词条保持可见。

## 关闭、拖选与触摸

固定布局通过点击正文查询对象激活，点击其他对象原位更新；点击说明和当前查询区域外部或按 `Esc` 关闭。modal 和 panel 的退出判断使用实际元素边界，sheet 的保护区围绕把手设置。

阅读器鼠标位移达到 5 px 后进入拖选，关闭查询并捕获指针；普通点击保持对象切换行为。指南及正文触摸由 `useTouchBrush` 协调成分选择与滚动。句中图的触摸、键盘和选择事件见 [M4](analysis_graphs.md#选择键盘与触摸)。

助词等内置词条由 `grammarMorphemeLookup` 生成，指南使用 `ParticleGrammarPopover.vue` 显示对应内容，阅读器组合相同内置查询结果；其入口和活用详情共同依正文身份路由。

## 正文与图的同步

正文激活先从已生成的详情选择 sentence part，取得 `unit`、完整范围和内部颜色分段；蓝色词干与黄色活用可共享一个背景对象。背景按实际字符宽度绘制渐变。图中选择同一 sentence part 后，宿主定位对应正文成分并重新打开查询。

内容路由、词典异步加载和 SVG 绘制各有输入。范围与高亮由同步投影提供，具体拆分及命中优先级见 [L11](language_analysis/presentation.md#图节点正文与高亮)。

## 维护入口与后续接入

布局状态见 [morphologyLayout.ts](../src/explanation/morphologyLayout.ts)，几何见 [geometry.ts](../src/explanation/geometry.ts)，阅读器集成见 [ReaderDocumentView.vue](../src/components/reader/ReaderDocumentView.vue)，指南集成见 [GrammarGuidePanel.vue](../src/components/grammar/GrammarGuidePanel.vue)、[GrammarGuideExample.vue](../src/components/grammar/GrammarGuideExample.vue) 和 [ExplanationPopover.vue](../src/components/explanation/ExplanationPopover.vue)。

现有检查包含 geometry、morphologyLayout、closeGrace 和正文投影样例；界面核对覆盖 960 px 两侧、布局切换、抽屉拖动与换词、气泡边缘定位及复杂句高亮。文档更新集中核对调用与范围，UI 变更按受影响行为选择最终验证。

后续分析结果可沿用当前激活对象、锚点、内容分区和三种容器。新模块需要提供整体与组分范围、说明内容及可选择的图节点，并与词典查询目标协作；具体对象和接口按对应语言分析方案讨论。
