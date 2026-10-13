# 首个任务：Serverless GPU API 与镜像闭环

## 任务目标

本地 Python 脚本直接调用 RunPod，完成 GiNZA BERT large 全量分析。交付可部署镜像、简易 API 输入输出、本地实验工具和实际部署报告，覆盖延迟、耗时、性能、资源占用与实际费用。

本文定义 E1 部署与基线实验。正式上线使用 E2 优化后的 GiNZA 管线，E2 完成后再接入收费产品。

## 实验组成

本地脚本负责输入、提交、状态查询、下载和报告；RunPod 负责异步任务 API 和 worker 启停；镜像包含模型推理与结果序列化。实验结果和任务记录保存在本地。

第一版使用 RunPod queue-based endpoint。提交采用 /run，查询采用 /status/{job_id}，完成后将完整结果保存到本地。客户端轮询即可跟踪任务，GPU worker 按任务生命周期执行。

## API 协议

输入示例：

~~~json
{
  "input": {
    "schema": "kotoclip.cloud-analysis.v1",
    "request_id": "book-001",
    "units": [
      {"id": "chapter-1-unit-1", "text": "銀座でランチをご一緒しましょう。"}
    ],
    "options": {"batch_size": 16, "precision": "fp16", "cpu_decode_processes": 1}
  }
}
~~~

一个任务接收一组有序单元。小说输入由本地按现有文档单元策略准备，worker 使用 nlp.pipe 批处理。API 输出包含相同 request_id、模型清单、每个单元的 id 和来源 artifact，以及任务指标。

来源 artifact 沿用 kotoclip.source-analysis.v1。统计信息单独放在 metrics，包含模型初始化、推理、结果转换耗时，输入规模、处理数量、显存和内存峰值。平台返回的任务状态、延迟、执行时间和 worker 信息完整保留在本地原始响应。

首次实验采用完整 JSON 返回。全书任务的输入输出大小与传输耗时写入报告；平台大小限制通过实际部署选定单任务书籍或章节规模，产品结果存储在后续阶段接入。

## 代码和镜像组织

| 计划入口 | 职责 |
| --- | --- |
| cloud/inference/engine.py | 模型初始化、批处理、指标 |
| cloud/inference/handler.py | RunPod SDK 入口 |
| cloud/inference/requirements.txt | Python 依赖 |
| cloud/inference/Dockerfile | Linux amd64 GPU 镜像 |
| cloud/inference/prefetch_model.py | 模型文件准备与路径记录 |
| scripts/cloud_gpu_experiment.py | 本地提交、查询、下载和并发实验 |
| scripts/cloud_gpu_report.py | 性能与费用汇总 |
| docs/cloud_gpu_experiment_report.md | 实际结果与推荐配置 |

现有 scripts/nlp_adapters.py 提供完整来源序列化，实施时提取“已分析 Doc 转 artifact”的公共函数，单文本和批处理共用。scripts/nlp_resources.py 提供资源身份记录，云端清单增加实际 GPU、模型仓库与 revision。保持现有桌面调用兼容。

共用 engine 接收文本单元并生成结果；handler 仅负责平台输入输出。后期 HTTP 服务直接调用 engine，同镜像使用另一启动命令。

镜像采用 Python 3.12 与匹配的 PyTorch/CUDA/CuPy 运行环境，安装 GiNZA 5.3.0、ja_ginza_bert_large 5.3.0、RunPod SDK、资源采集依赖。构建后保存完整依赖清单和镜像 digest。

## 模型缓存与启动

模型缓存方案为两组：RunPod cached models 与构建时预下载到镜像。缓存覆盖 GiNZA 实际使用的 Transformer 和 tokenizer，加载路径在模型清单中记录。

正式仓库为 megagonlabs/transformers-ud-japanese-bert-large-ginza，revision 为 7683285d102ba2098cf5f2641388cd69c7406241，权重文件约 1.350 GB。FP16 权重占 643.6 MiB，Transformer 输出转为 FP32 后交给 spaCy 分析头。本机完整管线 FP16 峰值 989 MiB，且输出与 FP32 一致，首版直接采用 FP16 并保留 NER。

进程启动先选择 GPU、加载模型，随后启动 handler。每个任务复用已加载的模型。GPU 模式明确在 Python 入口启用；启动时保存模型加载时间、进程标识和 GPU 名称。

比较首次部署、零 worker 后再次调用和已有 worker 连续调用。报告分别呈现模型文件准备、worker 初始化和任务执行的耗时，给出适合该模型的缓存交付方式。

## Endpoint 初始配置

采用用户确定的 16 GB、0.58 美元/小时 GPU 档，最少 worker 为 0，单请求实验最多为 1，并发实验最多为 10，每个 worker 同时执行一个任务。初始空闲保留设为 10 秒，再比较 0、10、30 秒。

并发测试通过同一 endpoint 提交 1、6、10 个任务，平台分别分配独立 worker，比较冷启动、每请求完成时间和总 worker 计费时长。平台分配的 GPU 型号写入每次报告；不同型号分别汇总。

## 性能预算与 GiNZA 优化边界

本机纯 Transformer FP16 为 12900–15944 token/s，关闭 NER 的 AMP 管线为 4705 token/s，完整 FP16 为 810 token/s。NER 占分阶段耗时 65.1%。E1 单进程为零优化对照，E2 优化后的 serverless 是正式上线版本。

| 状态 | 完整吞吐预算 | 十万 token 暖机完成 | 缓存冷启动完成 | 每本 GPU 费用 |
| --- | ---: | ---: | ---: | ---: |
| Serverless E1 基线 | 900 token/s/worker | 116 s | 133 s | 0.166 元 |
| Serverless E2 上线版 | 3000 token/s/worker | 38 s | 68 s | 0.091 元 |
| 常驻单卡基线 | 900 token/s/卡 | 116 s | 服务启动时初始化 | 按月租 |
| 常驻单卡正式版 | 3000 token/s/卡 | 38 s | 服务启动时初始化 | 按月租 |

E1 模型加载预算 17 s；E2 的模型、CPU pool 初始化与预热合计 30 s。请求与结果处理预留 5 s，serverless 空闲保留 10 s。E2 的 9068 token 正文暖机/缓存冷启动为 8/38 s，119702 字符材料约 73500 token，对应 30/60 s。并发 6–10 个 serverless 请求时，平台分配独立 worker，各请求保持该单次时间预算，总 GPU 费用按所有 worker 秒数累加。

E2 在每个 serverless worker 内采用一份共享 FP16 BERT、一个 GPU 推理进程和六个 CPU 解码进程，资源为六个有效 CPU 核心、24 GiB 系统内存、16 GB GPU。必须实现 NER 独立单元并行、状态特征与 maxout 数组计算优化、合法动作筛选与 argmax 循环合并、特征批量传输及共享缓冲、GPU/CPU 双缓冲重叠、长度分组与 batch/max_batch_items 配置、完成单元的 Doc 和特征释放。保留 BILUO、greedy 及同分规则，完整输出与 FP32/FP16 基准逐字段对照。

P4 常驻单卡复用 E2 执行核心，增加跨请求的单元合批和优先顺序。十个十万 token 请求在 serverless 优化版上各约 38 s 完成；在常驻单卡上共享 3000 token/s，整组约 6.4 分钟完成。上线版本、扩缩容并发与单卡共享容量分别验收。

## 实验内容与报告

| 项目 | 实验内容 | 报告结果 |
| --- | --- | --- |
| 部署闭环 | 构建、推送、建立 endpoint、真实请求 | 镜像 digest、配置、任务 ID、完整输出 |
| 语言与协议 | 复用 P4 样本，核对关键复杂词和语法、Unicode、单元顺序 | 来源输出、字符坐标、下游可消费情况 |
| 冷启动 | 首次部署和 worker 缩至零后的再次调用 | 提交至开始、加载模型、提交至完成 |
| 连续调用 | 已加载 worker 连续处理 | 每任务耗时、吞吐、结果下载耗时 |
| 全量输入 | 短段落、现有 119702 字符语料和完整书籍 | 总耗时、结果大小、完整性 |
| Serverless 并发 | 上限 10 worker，提交 1、6、10 个任务 | 每请求耗时、聚合吞吐、总 worker 秒数与费用 |
| 批处理 | 在 16 GB 卡上比较几个批大小 | 速度、显存、推荐批大小 |
| 计算精度 | FP16 默认配置与 FP32 结果基准 | 云端结果一致性和实际显存 |
| 资源占用 | worker 运行期间采集 CPU、RAM、GPU 显存和利用率 | 峰值和使用曲线 |
| 缓存与保留 | 两种模型缓存和 0/10/30 秒保留 | 启动速度、连续体验和账单 |
| 实际费用 | 实验开始至结束的全部资源支出 | GPU、初始化/保留、模型存储和总额 |

报告同时保留本地端到端耗时、worker 内耗时和平台任务指标。GPU 耗时在 CUDA 完成后记录；资源采样贯穿 worker 初始化和任务处理。

费用使用 RunPod 账单与实验期间的消费记录，按任务耗时计算的金额另列为模型估算。开始时记录余额/消费基线，结束后等待本轮账单更新，再汇总总差额与平台费用明细。实验 endpoint 单独使用，结果保存到本地后缩至零，记录缓存存储的持续费用。

将实测每本书费用代入 15/19/29/39 元月价，更新 100 人阶段的平衡点及与 3090/4090 常驻租金的相等点。首个实验输出真实数字和后续优化顺序。

## 执行顺序与完成条件

先完成共用推理核心和来源转换，再构建镜像并推送仓库，随后部署 endpoint 并完成一次短文本闭环；之后执行全量、冷暖启动、并发与费用实验，最终整理报告和推荐配置。

E1 完成时具备可复用基线镜像、本地单命令调用入口、实际 endpoint 配置、完整分析结果、性能资源报告和实际账单汇总。E2 随后交付优化镜像：完整管线达到每 worker 3000 token/s，完整 NER 和来源结果通过对照，6–10 worker 并发与真实计价通过验收，作为 P1/P2 的上线前置条件。模型缓存、worker 保留时长和后期单卡切换采用报告中的结果确定。

## 前置要求

1. 已注册并充值的 RunPod 账号，可创建上述 16 GB serverless endpoint。
2. 本机配置 RUNPOD_API_KEY，endpoint ID 由部署后写入实验配置。
3. 可供 RunPod 拉取的镜像仓库，以及 Linux amd64 Docker 构建环境；可以使用本地 Docker 或 CI 构建。
4. 可访问模型仓库的构建或运行环境，选用 cached models 时配置对应缓存。
5. 现有语言样本、119702 字符语料和一份完整书籍文本。
6. 本次实验预算与测试地域；建议首轮预算 50 元，费用报告按实际平台消费填写。

## 来源

RunPod endpoint、worker、cached models 与计费文档入口统一登记在根目录 [sources.md](../sources.md)，实施时阅读对应当前文档。
