# 活用链配置

唯一规则入口为 `configs/unidic-nlp/morphology.json`。Rust 编译时嵌入配置，前端直接导入同一文件；配置内容参与分析缓存标识。修改后重新构建应用即可更新分析和展示。

`operators` 按形式身份定义概念目录路由、标题、原型、候选解释及输出状态。`variants` 按顺序选择原型；`normalized` 用于具有固定展开形式的缩约和补助结构。原始表记、成员及来源字段保存在分析对象中。补助用言的 `support` 声明词元和连接组，`connector` 指向前项接续形式，`compound_candidate` 控制整体查词候选。

`features` 从单个 UniDic token 识别形式，`connections` 决定相邻成员的核心归属、附属或补助链关系。连接规则按顺序执行；一条规则内的 `when` 为任选条件，每个条件内的字段同时成立。`equals`、`starts_with`、`contains` 的同字段取值为任选。已知形式应明确规定前接形及词类，来源关系用于辅助判断归属。字符不连续、空白和标点由执行器统一限制。

`forms` 映射 UniDic 活用形，`insertion` 定义补助连接中容许的插入成分；实际跨成分连接仍须 GiNZA 结构确认。`presentation` 集中定义活用名称和原型接续显示。新增形式时优先扩展既有字段；状态机负责遍历和证据关联，前端负责投影。

先采集真实分词，核对基本表记、词元、活用型与活用形，再编写规则。用正例和相邻边界的对照验证识别、父链、命中范围及查词形式；同一范围的复合形式采用独立身份，避免重复展示其内部功能。主要验证入口为 `morphology_reconstruction.rs`、`p4_language.rs` 和 `lookupPresentation.test.mjs`。
