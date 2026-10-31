# 活用规则源文件

本目录维护活用规则，运行产物为 `configs/unidic-nlp/morphology.json`，协议为 `readux.morphology-config.v1`。编译入口为 `scripts/build_morphology_config.py`。

```powershell
npm run morphology:build
npm run morphology:check
```

生成命令编译规则并检查引用；内容相同时保留产物文件。检查命令在内存中编译，要求结果与已保存产物的 UTF-8 字节一致，同时检查概念正文、例句与既有路径引用。前端开发、预览和构建入口自动生成配置，发布 CI 执行检查命令。

## 文件索引

| 文件 | 内容 |
| --- | --- |
| `base.json` | 递归深度、三个原子特征、活用形映射与输入恢复 |
| `operators/verb.json` | 动词循环与组合执行器，共 47 条 |
| `operators/conversion.json` | イ形容词转换与名词化，共 8 条 |
| `operators/judgement.json` | 判断执行器，共 16 条 |
| `operators/auxiliary.json` | 补助执行器与连接器，共 44 条 |
| `operators/classical.json` | 文言执行器，共 33 条 |
| `operators/insertion.json` | 插入执行器，共 2 条 |
| `rules/features.json` | 48 条单 token 特征 |
| `rules/connections.json` | 48 条相邻连接规则 |
| `rules/sequences.json` | 30 条连续词尾 |
| `rules/predicate_links.json` | 12 条谓语构式 |
| `rules/insertion.json` | 插入成员白名单与四组允许位置 |
| `classical/forms.json` | 5 个文言范式 |
| `classical/auxiliaries.json` | 32 条文言助动词及 44 组具名输入 |
| `paths/explicit.json` | 46 组显式路径 |
| `paths/classical.json` | 31 组文言输入引用与路径展示声明 |
| `paths/predicates.json` | 12 组谓语前缀引用 |
| `presentation.json` | 六类实体、规则树与展示配置 |

## 维护约定

每个 operator ID 在一个领域文件中保存 `definition`、`execution`、`connection_type` 和 `connection_rule`。执行器决定领域归属。`connection_rule` 为直接所属规则 ID；通过连接器继承路径时填写 null，并在 definition 声明 connector。

输入文案位于 execution.input_label；机器条件使用 input_category_mode、input_categories、input_forms。输出状态字段统一位于 execution。definition 中的 variants 按顺序匹配，Rust 将本句结果写入 normalized_form。

文言 inputs 的 id 是稳定引用。路径布局通过 input 指向该声明，通过 categories 选择显示分支；名词与代词可以共享分支。forms、ctypes、exclude_ctypes 从输入声明取得。新增输入类别时同步声明其显示分支。

谓语路径通过 predicate 指向对应构式，编译器按构式 prefix 查找前项并复制完整路径。显式路径保存自身的条件与显示字段。连接器路径继续使用 connector 和 connector_paths 引用共享规则。

features、connections、sequences、predicate_links、forms、classical_forms、classical_auxiliaries、variants 和 paths 的数组顺序参与执行或展示，维护时保留其顺序。对象键按字典序生成；分类成员和规则归属成员按 operator ID 排序。

完整协议、数量基准与验证记录见 [活用配置协议与存储重构](../../../../docs/morphology_storage_refactor_analysis.md)。
