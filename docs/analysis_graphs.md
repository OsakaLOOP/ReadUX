# D3 分析图与交互控件

更新日期：2026-11-09。本文为 M4，说明句中活用图、前接规则树及共同绘制控件。图数据的语义来源见 [L11](language_analysis/presentation.md)，三种视图的容器和定位见 [M3](explanation_layout.md)。

## 数据、布局与绘制

| 层 | 输入与输出 | 入口 |
| --- | --- | --- |
| 语义内容 | 活用父子链、operator、connection 及配置 → `ConnectionDiagram` | [morphologyBubble.ts](../src/reader/morphologyBubble.ts) |
| 图组织 | diagram、展开类别和可用高度 → 两组 layout 及视区高度 | [ReaderConnectionTree.vue](../src/components/reader/ReaderConnectionTree.vue) |
| 几何布局 | sentence／branches → `MorphologyGraphLayout` | [morphologyGraph.ts](../src/reader/morphologyGraph.ts) |
| 绘制与动画 | layout、视区和 flow → SVG、滚动、toggle／select／resize | [ReaderMorphologyGraph.vue](../src/components/reader/ReaderMorphologyGraph.vue) |
| 词形摘要 | 链摘要及步骤 → 文字形式的活用信息 | [ReaderMorphologySummary.vue](../src/components/reader/ReaderMorphologySummary.vue) |

布局结果保存 width、height、nodes、links。节点有稳定 ID、位置与尺寸、文本行、类型、命中状态及可选列表／展开动作；连线有端点、命中状态、折行标志、标签和箭头控制。`GraphLine.runs` 用于区分中文标签、日文词形及箭头文字。

## 句中活用图

`layoutSentenceChain` 按 sentence 的实际顺序布局，每个节点固定两行。第一行为功能标签及词形；原型与词面有差异时显示「原型 → 本句词形」。第二行为当前活用、音便或缩略说明，无活用成分显示“无活用”。

节点宽度按两行较宽文字加 24 px 计算，高度为 44 px。测量优先使用 canvas，缺少 canvas 的环境按字符宽度估算。文字在节点内居中，中文使用界面字体，日文词形使用日文字体。

`breakBefore` 由语义投影决定：补助层从连接器之前折行；认识单元承接谓语或补助层时按配置折行一次，内部继续排列。布局依此将各行从左向右连接，同一行间距为 18 px，折行垂直间距为 26 px。

逻辑列从 1 开始，同一行逐项增加。折行时逻辑列达到 2 则减 1，后项相对前项中心向左移 52 px；更靠左时按中心对齐。布局统一平移全部节点以容纳负向偏移。

普通连线从节点侧面连接，折行连线从前项底部指向后项顶部。折行关系文字位于连线中点右侧 9 px；当前节点的前接线及文字采用强调色，插入成分采用虚线边框。

## 前接规则树

`layoutConnectionRules` 使用 `d3-hierarchy` 的 hierarchy／tree 计算树。界面从左向右表示“前项 → 词类与接续条件 → 连接器 → 当前词条”。diagram 的 branches 和 groups 来自已展开的规则引用，matched 来自本句 `matched_paths`。

| 元素 | 当前组织方式 |
| --- | --- |
| 前项 | 取实际前项表面串，与当前命中词类标题对齐，连线不画箭头 |
| 词类分类 | 始终显示；按动词、形容词、名词、文言等顺序组织，イ／ナ形容词进一步排序 |
| 分类内路径 | 命中路径始终保留，其余路径由该类别展开状态决定 |
| 连接器 | 独立节点，て／で各有分支；如な等连接器按引用保持独立 |
| 当前词条 | 使用 diagram.target；连接形式自身可省略重复目标节点 |

同一图的分类节点采用统一宽度，依据全部路径标题与条件测量，展开前后宽度稳定。展开按钮和总路径数在左，分类标题居中。`expanded` 为单个类别 ID，同一时间显示一类的其他路径；再次点击该类收起。

分类标题下方用列表显示细分路径，竖向悬挂线从标题延伸，短横线连接条目。当前路径有强调文字和高亮引导箭头；实际音便标签由 `path.actual` 显示。

直接路径与连接器路径混排时，布局用隐藏节点预留连接器列，所有词类位于同一列；可见连线跳过隐藏节点。树的 separation 采用分类及完整列表高度。父节点按首个可见子节点顶部至末个分类标题底部居中，末类的列表继续占据布局空间。

## 两部分高度与滚动

`ReaderConnectionTree` 分为“接续方式”和“接续规则”。diagram.status 为 candidate 时，规则标题附“候选”；详细诊断继续保存在图数据中。

悬浮布局采用 split：扣除标题和分隔区域后，若内容能完整显示则采用自然高度；空间不足时优先完整显示较短部分，另一部分至少获得可用图示高度的三分之一。较短部分最多占三分之二；单部分图使用全部可用高度。

固定布局采用 flow：图示按完整高度进入外层统一滚动区，按视区宽度等比例缩小。`data-natural-width`、`data-natural-height` 向容器声明完整尺寸，`measureIntrinsicPanel` 使用该值定位悬浮面板，内部视区高度与自然高度分别计算。

split 模式根据当前命中节点联合范围缩放并横纵居中，规则区纵向以命中的具体列表条目为中心。首次定位直接设置滚动位置，后续采用 220 ms 过渡；布局和视区尺寸稳定后保留用户手动滚动。

## 选择、键盘与触摸

句中节点可点击，发出 `select(id)`；宿主由 sentence part 的 range 找到正文成分并更新当前查询。图中词类节点发出 `toggle(key)`，由 `ReaderConnectionTree` 更新展开类别。节点自身只负责操作事件，语言关系与范围由投影对象提供。

可操作节点支持 `Enter`、空格和 focus 样式，展开节点提供 `aria-expanded`；SVG 与视区有可访问名称。触摸使用 `useTouchBrush` 在句中节点间选择，消费拖动后的点击事件。overflow 区域保留滚动能力。

## 动画与实例身份

`ReaderMorphologyGraph` 按稳定节点 ID 对比新旧布局，使用 `d3-interpolate` 插值位置、尺寸和透明度，几何过渡为 280 ms。新布局首帧采用新 links，退出节点继续淡出；背景、边框、连线及规则条目使用 180 ms 颜色过渡。

宿主用首个句中节点 ID 维持同一家族的图组件实例，链内切换解释对象时更新选择与规则内容。系统启用 `prefers-reduced-motion` 时直接更新几何与定位，并停用相应颜色动画。

## 维护与扩展

规则路径、连接器、成员范围及折行内容在 L11 和配置中维护；节点尺寸、列对齐和路径几何在 `morphologyGraph.ts` 维护；滚动、缩放、绘制及动画在 Vue 图组件维护。已有检查入口为 `src/reader/morphologyGraph.test.mjs` 及指南的复杂句、39 条接续分支样例。

代表检查包括「はずなの」的独立な节点、直接路径与连接器路径混排、递归补助折行、かもしれない整体节点、分类展开后的居中，以及抽屉中的完整图示高度。变更后按相应样例集中核对。

统一名称、七组分类和节点文字的后续目标见[显示规范与修订设计](language_analysis/morphology_display_conventions.md)。当前分类顺序由文字匹配与 localeCompare 决定，“助动词”会进入动词的文字匹配分支；七组固定排序尚待实现。修改分组时同步核对 `categoryOrder`、路径展开和测量。

后续词组和依存图可复用图视区、稳定 ID、选择回调、测量、键盘与动画机制。新增模块的节点、边、方向和布局取决于其分析对象，详细设计在对应语言模块确定后进行。
