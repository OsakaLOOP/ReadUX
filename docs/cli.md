# CLI 与本机请求协议

更新日期：2026-11-09。本文为 E3，依据 [Request 枚举](../crates/readux-core/src/analysis.rs) 和 [CLI 入口](../crates/readux-core/src/bin/readux-nlp.rs) 说明当前命令、资源及逐行请求。分析阶段见 [L1](language_analysis/pipeline.md)，运行环境见[开发说明](development.md)。

## 命令与输出

```powershell
npm run morphology:build
cargo build -p readux-core --bin readux-nlp
.\target\debug\readux-nlp.exe inspect cwj "本を読んでいる。"
.\target\debug\readux-nlp.exe repl
.\target\debug\readux-nlp.exe stdio
```

| 命令 | 输入与行为 | 输出 |
| --- | --- | --- |
| `inspect cwj|csj 文本` | 第三个参数为完整文本；`csj` 选择口语，其余值按当前代码选择 CWJ | 成功时输出格式化的基础文档 JSON，失败通过进程错误返回 |
| `repl` | 每行正文执行 CWJ 基础分析，空行退出 | 每行一个 `Response` |
| `stdio` | 每行一个完整 JSON 请求，持续复用同一服务 | 每行一个 `Response`，写出后刷新 stdout |

`inspect`、`repl` 调用 `Analyze`，得到基础 UniDic 与统一分析结果。需要 GiNZA 时，在同一 stdio 进程内对返回的文档 ID 调用 `enrich`。完整文档会话的单元处理包含 GiNZA 及查询目标构建。

## 逐行协议

请求用 `command` 区分操作，名称采用 snake_case，字段严格按对应枚举变体读取，额外字段产生反序列化错误。JSON 内的文本换行使用 `\n`；每个请求占一个物理行。响应格式为：

```json
{"result":{"示例":"操作返回值"},"error":null}
{"result":null,"error":"具体错误信息"}
```

stdout 承载协议 JSON，诊断信息使用 stderr。协议按请求顺序返回，客户端维护请求与响应队列；响应没有独立 request ID。stdio 的输入线程提前处理 `cancel_external` 的取消代次，再由服务顺序消费请求。

基础检查可逐行输入：

```json
{"command":"status"}
{"command":"provider_status"}
{"command":"check_providers"}
{"command":"analyze","text":"本を読んでいる。","register":"cwj"}
```

基础分析响应的 `result.id` 用于 `analysis_id`；`enrich` 响应的 `result.document` 包含更新后的文档。服务内对象引用由当前进程保存，后续请求使用最近响应中的实际身份。

```json
{"command":"enrich","analysis_id":"<result.id>"}
{"command":"lookup_targets","analysis_id":"<最新文档 id>","range":[0,8]}
{"command":"query_target","analysis_id":"<最新文档 id>","target_id":"<目标 id>"}
```

示例中的范围对应「本を読んでいる。」的准备后全文；替换文本时同步使用实际字符范围。基础分析文本上限为 20,000 个 Unicode scalar，长文采用文档会话。

## 当前请求目录

下表覆盖 `Request` 枚举全部操作。会话引用简写为 `session_id`、`text_version`、`generation`；单元引用再增加 `unit_id`、`artifact_revision`。表中 `selected_form` 均可省略。

| 请求 | 主要输入与用途 |
| --- | --- |
| `status` | 服务、词典等当前状态 |
| `provider_status`、`check_providers` | 配置与进程状态；后者检查 provider 并使会话来源状态失效 |
| `configure_providers` | `settings`；保存配置并刷新来源状态 |
| `set_analysis_timing` | `enabled`；设置耗时记录 |
| `performance_stats` | 可选 `clear`，默认 false；读取性能统计 |
| `analyze` | `text,register`；`register` 为 `cwj` 或 `csj` |
| `analyze_routed` | `text,policy`；policy 为 `auto`、`cwj` 或 `csj` |
| `analyze_with_artifacts` | `text,register,artifacts`，可选 `grammar,expression`；附带来源或覆盖产物 |
| `enrich` | `analysis_id`；追加 GiNZA 并重新统一 |
| `cancel_external` | 请求中止外部 provider 工作 |
| `open_document` | `text`，可选 `document_id,policy,initial_offset`；policy 默认 auto，offset 默认 0 |
| `request_range` | 会话引用、`range`，可选 `after_revision`；请求全文范围附近单元 |
| `continue_document` | 会话引用，可选 `after_revision`；继续处理 |
| `cancel_document` | 会话引用；取消当前会话任务 |
| `retry_document` | 会话引用，可选 `unit_id`；重试指定单元或失败任务 |
| `close_document`、`sync_document` | `session_id`；关闭或读取当前快照 |
| `poll_document` | 会话引用、`after_revision`；读取增量更新 |
| `query_document` | 单元引用、`token_id`，可选 `selected_form`；查询单个 morpheme |
| `lookup_document` | 单元引用、局部 `range`；构建范围查询目标 |
| `query_lookup_document` | 单元引用、`target_id`，可选 `selected_form`；查询单元已保存目标 |
| `query` | `analysis_id,token_id`，可选 `selected_form` |
| `query_candidate` | `analysis_id,candidate_id`，可选 `selected_form` |
| `lookup_targets`、`lookup_report` | `analysis_id`，前者增加 `range`；目标组或查询诊断报告 |
| `query_target` | `analysis_id,target_id`，可选 `selected_form` |
| `query_entry` | `entry_key`；取得源记录完整内容，结果为记录数组 |
| `search` | `word`；1 至 100 字符的主动词典搜索 |
| `list_rules` | 当前通用规则快照 |
| `preview_rule` | 单元引用及 `rule`；校验规则后执行预览 |
| `save_rule`、`set_rule_enabled`、`delete_rule` | 分别使用 `rule`、`id,enabled`、`id` 管理通用规则 |

`open_document` 返回 `plan` 和初始 `update`，后续控制使用 `update` 的会话身份；单元响应中的 `artifact_revision` 用于查询有效期检查。`request_range.range` 使用全文准备后坐标；`lookup_document.range` 使用所属单元文档坐标。范围均为半开区间，转换见 [L2](language_analysis/lexical_input.md)。

通用规则管理保留当前接口。普通分析未调用 `language_analysis::apply`，规则保存后的 `refresh_language` 当前复制并保存文档；规则预览通过 `language_analysis::preview` 执行。活用规则在 source 配置中维护，见 [L5](language_analysis/morphology_configuration.md)。

## 资源、配置与错误

CLI 根据编译时仓库路径使用 `ResourcePaths::development`。默认词典为 `experiments/unidic-source/` 下的 CWJ／CSJ Vibrato 文件；数据目录默认 `data/`，包含 `dict-sources/`、`dicts/`、`providers.local.json`、`provider-logs/` 和分析缓存。环境覆盖为 `READUX_UNIDIC_CWJ`、`READUX_UNIDIC_CSJ`、`READUX_DATA_DIR`。

provider 默认使用 `experiments/ginza311/Scripts/python.exe` 和 `ja_ginza`。在 stdio 中提交实际本机路径：

```json
{"command":"configure_providers","settings":{"ginza":{"python":"D:/path/to/python.exe","model":"ja_ginza","enabled":true,"timeout_seconds":120,"dictionary":""},"analysis_timing_enabled":true}}
{"command":"check_providers"}
```

解释器、模型、词典或来源错误通过响应和 provider 日志定位；文档会话中的单元错误记录在对应更新中。分析 ID 无效时重新分析，单元产物更新后使用最新引用。前端请求适配将 `error` 转换为页面错误，见 [nlp.ts](../src/services/nlp.ts)。

## bridge 与桌面命令

`npm run acceptance` 构建 debug CLI 并启动 1420 端口 Vite。[nlp_bridge.mjs](../scripts/nlp_bridge.mjs) 复用 `readux-nlp.exe stdio`，将 `POST /api/nlp` 的 JSON 请求按序转发并返回原响应。进程结束后需重新启动该开发入口。

Tauri 的 `nlp_request {request}` 调用同一 `AnalysisService`。书架导入、进度、收藏等 `reader_*` 命令在 Tauri 独立注册，静态浏览器由适配器提供相应子集；CLI 请求目录与这些宿主命令分别维护。浏览器静态模式的 `nlpRequest` 直接读取快照，能力见 [E2](static_library.md)。

## UniDic/Vibrato 构建工具

[unidic-vibrato-build.rs](../crates/readux-core/src/bin/unidic-vibrato-build.rs) 从 UniDic 源目录编译 Vibrato 词典：

```powershell
cargo run --release -p readux-core --bin unidic-vibrato-build -- --input unidic-cwj-202512 --output experiments/unidic-source/unidic-cwj-202512.vibrato.dic
```

输入目录需包含 `lex.csv`、`char.def`、`unk.def`、`feature.def`、`right-id.def`、`left-id.def`、`model.def`。工具在输出旁创建 `.vibrato-work` 工作目录，生成紧凑连接模型后写出词典；输出父目录需已存在。CSJ 使用相应源目录及输出路径。源字段、语域与使用方式见 [L2](language_analysis/lexical_input.md)。
