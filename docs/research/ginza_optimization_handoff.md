# GiNZA BERT 优化交接与流程模型

## 本轮结论与范围

2026-10-13，本机 GiNZA BERT 完整推理优化阶段按用户确认完成。六路 CPU 解码、共享一份 FP16 BERT、双批次阶段重叠，首次三次测量中位数为 **2847 token/s**，较同语料 FP16 默认管线的 808 token/s 提升 3.52 倍。后续同一进程池内的阶段对照为 **2188 token/s**，包含输出字段转换及 JSON 编码为 **2019 token/s**。实体识别、依存、词性、文节、小句、词界及词典形态字段通过对照。

用户接受本阶段成果，并要求停止增加实验与实现。下一 agent 的任务是分析现有流程模型，面向 **RTX 4060 Ti 16 GiB 或更高平台**设计 GPU 并行与紧密编排。本轮范围仅为 BERT 实验核心。

验收目标为完整管线至少 1500 token/s. 现有 CPU 等待与 CUDA event 区间只是阶段诊断指标，纯 GPU 活跃时间与完整 CPU 关键路径需要下一阶段精确归因。

## 环境、输入与测量口径

硬件为 Ryzen 7 5800H、RTX 3050 Laptop 4 GiB，Windows；Python 3.11.9、PyTorch 2.7.1+cu128、spaCy 3.8.16、GiNZA/ja_ginza_bert_large 5.3.0、Cython 3.0.12、threadpoolctl 3.6.0。CPU 优化采用通用连续数组、共享内存和 spawn 进程池，线程数与 worker 数为运行参数。Linux 扩展构建参数已提供，实际编译及推理证据来自 Windows。

完整语料为 P01–P10 与小说 32 单元，共 42 单元、14772 字符、9068 GiNZA token。输入摘要为 `dd9b8bf790802a9154f140d8e65f7d0f57b92fdce736b4b2c29f2504c23e90c0`。本机快照为 `.agents/analysis/ginza-opt/units.json`；命令支持 `--input`，可使用相同结构的其他语料。

`tokens_per_second` 使用已预热的完整分析 wall time，计入 Sudachi 分词、Transformer、结构头、GPU→CPU 传输、CPU 解码、共享内存复制、进程通信及 Doc 重建。模型加载和 worker 初始化独立记录。`export_tokens_per_second` 进一步计入 token/实体等字段转换与 UTF-8 JSON 编码。文件写入与网络服务属于另一个端到端测量范围。

完整输出核对包括 token 范围、表面串、lemma、norm、UPOS、tag、morph、dependency、head、实体 IOB/类别、文节位置、句界、实体范围、文节范围、小句成员与 sub-token。重复测量中的输出一致性由最终 benchmark 与 pipeline suite 检查。FP32/FP16 的既有全量精度对照见 [CUDA 报告](ginza_cuda_measurement.md)。

## 完整管线对照

| 测量                 | 组织方式                     | 中位耗时 | GiNZA token/s | 含 JSON 编码 token/s | 输出差异                               |
| -------------------- | ---------------------------- | -------: | ------------: | -------------------: | -------------------------------------- |
| baseline-fp16，2 次  | 默认完整管线，batch 16       | 11.219 s |           808 |           本组未采集 | 参考输出                               |
| fused-1，2 次        | 初版融合解码，单进程         | 10.869 s |           834 |           本组未采集 | 0                                      |
| fused-w4，2 次       | 初版融合解码，4 worker，重叠 |  3.718 s |          2439 |           本组未采集 | 0                                      |
| vector-w6，3 次      | 连续数组内核，6 worker，重叠 |  3.185 s |          2847 |           本组未采集 | 0                                      |
| vector-w8-b16，2 次  | 连续数组内核，8 worker，重叠 |  3.127 s |          2900 |           本组未采集 | 0                                      |
| vector-w6-b8，2 次   | 6 worker，batch 8，重叠      |  4.221 s |          2148 |           本组未采集 | novel-14 实体、novel-15 token 字段变化 |
| final-overlap，5 次  | 6 worker，batch 16，重叠复测 |  4.901 s |          1850 |                 1601 | 0，重复结果一致                        |
| suite 原版解码，3 次 | 6 worker，阶段串行           |  5.938 s |          1527 |                 1445 | 0                                      |
| suite 融合解码，3 次 | 6 worker，阶段串行           |  6.103 s |          1486 |                 1416 | 0                                      |
| suite 融合解码，3 次 | 6 worker，阶段重叠           |  4.144 s |          2188 |                 2019 | 0                                      |

前几组与复测具有明显的本机性能波动, 大概率是发热降频了。同一进程池内的 suite 是判断原版/融合解码与阶段重叠效果的主要证据：阶段重叠提升约 47%，融合内核的独立收益尚未成立。下一 agent 应优先比较原版解码加重叠，决定生产核心采用哪一个内核。

我的建议是, 生产版应当最终采取fp16, 且后续改进的本机测试受诸多瓶颈影响, 对生产的设计没有约束作用, 比较也不一定成立. 请针对4060ti平台的实际硬件参数和性能预期, 设计理论最合理的架构并完成.

首次六路结果的 PyTorch 峰值分配为 1042 MiB、保留为 1658 MiB，主进程及子进程 RSS 总和峰值约 5527 MiB。RSS 总和会重复计入共享页。CPU worker 排除 Transformer 组件，BERT 权重仅在 GPU 进程加载；Python/spaCy 插件依赖仍参与每个 worker 的导入，初始化优化留待后续。

## 已完成的执行组织

`scripts/ginza_bert_engine.py` 中的 `BertEngine` 实现单 GPU 生产者与常驻 CPU 消费者。外层 batch=16、Transformer max_batch_items=4096，保留原来的 128/96 分窗配置。

```mermaid
flowchart LR
  A[有序文本单元] --> B[Sudachi 分词与按 token 长度排序]
  B --> C[共享 FP16 BERT]
  C --> D[GPU 结构头特征预计算与词性预测]
  D --> E[连续 CPU 数组与共享内存]
  E --> F[CPU 进程池：依存和 NER 解码]
  F --> G[词性写回、文节和小句]
  G --> H[按输入顺序重建 Doc]
  H --> I[字段转换与 JSON 编码]
```

GPU 生成 parser/NER 的完整预计算数组，含一行 padding，按当前批次实际 token 数分配。每个 worker 接收自己的连续文档子组及 token offset，通过共享内存读取同一批次特征；进程通信仅传 Doc、偏置、词性结果和数组描述。worker 完成后释放该批次共享内存。

阶段重叠最多保留两个在途批次：CPU 解码 A 时 GPU 准备 B，提交 B 后等待 A，随后 GPU 准备 C。GPU 准备中的传输为同步 `.get()`；主进程先取得 NumPy 数组，再复制到共享内存。现有组织已经实现计算阶段重叠，异步 D2H、固定双缓冲与缓冲区复用属于下一阶段。

CPU 融合内核 `scripts/ginza_decode.pyx` 复用 spaCy 的 StateC、合法动作函数和动作更新，保持逐特征浮点相加顺序、maxout 与较早动作优先的同分规则，复用上下文及评分数组；不采用 AVX 型号专用指令。内核前提为 use_upper=false、无 unseen classes，BertEngine 初始化时验证。完整语料验证了语义一致性，同一进程池对照显示其速度与原版接近。

## 已有流程成本模型

### 时间变量与实测阶段

对第 b 个批次，定义 n_b 为正式 GiNZA token 数，w_b 为有效 WordPiece 数，s_b 为实际分窗序列数。GPU benchmark 的 WordPiece/s 与完整管线的 GiNZA token/s 分别统计。

| 变量     | 定义                                          | 对应字段                                      |
| -------- | --------------------------------------------- | --------------------------------------------- |
| T        | 文本分词与批次准备的 CPU 时间                 | tokenizer_seconds                             |
| G_b      | GPU 生产阶段区间：BERT 加结构头               | gpu_compute_seconds、gpu_event_seconds        |
| B_b、H_b | BERT 区间、结构头区间                         | bert_interval_seconds、heads_interval_seconds |
| P_b      | GPU 生产者完整准备时间，包含传输与 Doc 序列化 | prepare_finished−prepare_started              |
| D_b      | 特征 D2H wall time                            | d2h_seconds                                   |
| S_b      | 共享内存复制与 worker 提交                    | submit_seconds                                |
| C_b,j    | 第 b 批第 j 个 CPU 子任务完整服务时间         | task_finished−task_started                    |
| C_sum    | 所有 CPU 子任务 wall time 之和                | cpu_decode_seconds                            |
| W        | 主线程收集结果时尚未完成的等待                | cpu_wait_seconds                              |
| R        | 最终 Doc 重建                                 | reconstruct_seconds                           |
| E        | 字段转换与 JSON 编码                          | export_seconds 或 conversion_runs_seconds     |

CUDA event 的区间包含该 stream 上两事件之间的空闲和同步，BERT 区间还覆盖 Hugging Face 分词、对齐及 shim 的调度等待；字段名 `gpu_compute_seconds` 目前表示 event 区间。纯 GPU 活跃计算时间应通过 kernel trace 另行建立。CPU 子任务累计 wall time包含并行执行和调度争用，其数值适用于容量分析；关键路径等待由 W 描述。

suite 六路融合重叠的阶段中位数：T=0.347 s，P=2.005 s，G=1.558 s，B=1.441 s，H=0.117 s，D=0.100 s，S=0.181 s，C_sum=13.188 s，W=1.358 s，R=0.080 s。各字段独立取中位数，批次时间线保留逐次原值。完整 wall time为 4.144 s，含输出为约 4.491 s。

首次六路稳定结果 W/G≈67.6%；suite 重叠 W/G≈87.2%。加入 tokenizer、复制与结果处理后，CPU 关键路径的比例更高。50% 目标尚待下一阶段完成精确计时与优化。

### 吞吐与调度关系

短任务延迟按实际批次事件计算：

```text
T_job = T + GPU生产准备 + worker提交 + 未被重叠覆盖的CPU等待 + R + 调度余项
T_export = T_job + E
Q_job = 总GiNZA token / T_job
```

持续运行的流水吞吐受各阶段服务能力约束。采用同一组语料的规模 N，可建立：

```text
Q_gpu_producer = N / Σ(P_b + S_b)
Q_cpu_pool = k × e_k × N / C_single
Q_pipeline ≤ min(Q_tokenizer, Q_gpu_producer, Q_cpu_pool, Q_output)
```

k 为 CPU worker 数，e_k 为该硬件与负载下测得的并行效率，C_single 为可比较的单 worker 服务时间。现有 C_sum 来自并发争用状态，下一 agent 需要独立服务时间和 worker 利用率来拟合 e_k。首批启动、尾批收集、长度分组与子任务不均衡影响有限任务延迟。

批次事件的现有约束是 `GPU准备(b+2)` 等待 `CPU结果(b)` 收集。每批 CPU 子任务由 FIFO 进程池执行，当前按文档数量均分，组内 token 数可不同。最终 suite JSON 保存 `batches` 与 `decode_tasks`，后者有 batch_index、起止时刻、token 数和内核时间。使用这些事件可建立生产者/消费者时间线与队列模型。

### GPU 并行实测

`benchmark_ginza_gpu_concurrency.py` 捕获真实 GiNZA BERT 输入，使用同一份 FP16 权重，统一 padding 后比较单流合批和 2/4 个 CUDA stream。捕获结果为 107 条序列，padding 长度 187，有效 WordPiece 13880，attention=sdpa。测量覆盖 PyTorch BERT 执行与同步，结构头和 CPU 解码属于独立完整管线实验。

| 序列 batch | stream 数 | 中位耗时 | 有效 WordPiece/s | 相对 batch 8 单流 |
| ---------: | --------: | -------: | ---------------: | ----------------: |
|          8 |         1 |  1.225 s |            11331 |            1.000× |
|         16 |         1 |  1.152 s |            12048 |            1.063× |
|         32 |         1 |  1.161 s |            11954 |            1.055× |
|         64 |         1 |  1.150 s |            12067 |            1.065× |
|          8 |         2 |  1.187 s |            11692 |            1.032× |
|          8 |         4 |  1.190 s |            11666 |            1.030× |
|         16 |         2 | 16.078 s |              863 |  异常退化，待归因 |

最后一项在此前另一轮同配置中约 1.145 s；变更前后的线程提交方式和本机资源状态均需检查。四流的一次测量同样出现 1.878 s。原始数据完整保留，GPU 多流的稳定收益尚未确立。

同 batch 的 8×1/2/4 stream 张量逐值一致。改变 batch 时有效位置最大绝对差为 0.02734375；完整管线 batch 8 对 batch 16 的两处离散输出变化需在同 batch 基准中进一步归因。GPU 单流 batch 16–64 在本机接近吞吐平台，计算核心占用和显存带宽尚无 profiler 证据。

## 面向 4060 Ti 的分析输入与剩余任务

用户提供的目标硬件倍率：频率 1.5×、计算核心 2.1×、Tensor Core 2.1×、FLOPS 2.2×，CPU 更快但具体倍率待确定；期望 GPU 部分总体提升超过 2×。各倍率记录为分析输入，GPU 总算力使用 FLOPS 倍率，频率和核心数用于解释硬件构成。16 GiB 显存提供批次与缓冲容量；计算、内存带宽、传输和 host 调度分别进入阶段模型。

GPU 目标成本采用分项表达，参数由下一 agent 建立：

```text
G_target = G计算 / s计算 + G内存访问 / s带宽 + G提交等待 / s_host
P_target = G_target + D2H_target + 输入与结果准备_target
```

本轮完成了共享模型、CPU 并行、长度排序、共享内存交付、两个在途批次、完整输出对照及 GPU 多流原型。下列事项集中属于 GPU 并行和紧密编排的后续工作：

- [x] 从批次与 CPU 子任务时间线建立可验证的队列模型，分析首尾批次、任务均衡、worker 并行效率及背压。模型入口为 `scripts/analyze_ginza_pipeline.py`，可重放已保存的 batch/task 事件。
- [x] 使用 kernel trace 分离 BERT/结构头的 GPU 活跃时间、host 空档、传输与同步，统一 CPU 50% 的正式验收口径。3050 trace 中 kernel active 为 1.19 s，完整 wall time 为 5.44 s，host 暴露时间为 4.22 s，当前 CPU 关键路径未达到 50% 目标。
- [x] 分析原版解码加重叠与融合内核的选择；同一进程池中原版重叠为 1856 token/s，融合重叠为 1830 token/s，生产路径固定使用原版状态解码。
- [x] 归因 batch 16 双流异常退化，建立单流合批与双流基准。单流 batch 16–64 约 12.0k WordPiece/s，batch 8 的 2/4 stream 仅约 3% 提升，batch 16 双流出现异常退化；生产路径采用单计算流。
- [x] 面向 4060 Ti 16 GiB 设计序列长度分桶、batch/max_batch_items、worker 数和共享缓冲预算。配置与调整条件见 [GPU 并行设计](ginza_gpu_parallel_design.md)。
- [x] 设计固定缓冲区复用、pinned memory 与异步 D2H，用 event 交付对应批次；阶段事件保存于 `.agents/analysis/ginza-next/pipeline-trace.summary.json`。
- [x] 为跨请求合批建立 request/unit 身份与结果顺序、长度分桶和请求轮转调度，入口为 `BertEngine.analyze_requests` 与 `scripts/ginza_batching.py`。
- [ ] 在 4060 Ti 或更高平台验证共享 BERT、多流、批次扩大、CPU pool 和完整输出；形成云端实际速度与资源表。

目标平台尚未提供，本项保留为部署阶段验收。4060 Ti 的理论参数、显存预算、GPU/CPU 倍率模型和验证矩阵见 [GPU 并行设计](ginza_gpu_parallel_design.md)。

## 文件入口与复现

| 入口                                            | 用途                                                    |
| ----------------------------------------------- | ------------------------------------------------------- |
| scripts/ginza_bert_engine.py                    | 共享 BERT、GPU 特征预计算、共享内存、CPU pool、阶段指标 |
| scripts/ginza_decode.pyx、build_ginza_decode.py | 历史融合内核实验及本机编译记录                           |
| scripts/benchmark_ginza_optimized.py            | 单配置完整测量、输出对照、编码耗时                      |
| scripts/benchmark_ginza_pipeline_suite.py       | 同模型同池的原版串行与阶段重叠对照                       |
| scripts/analyze_ginza_pipeline.py                | 队列重放、worker 容量外推与 kernel trace 汇总             |
| scripts/profile_ginza_pipeline.py                | 完整管线 CUDA/CPU trace 采集                             |
| scripts/benchmark_ginza_gpu_concurrency.py      | 共享 BERT、多 CUDA stream、序列 batch 对照              |
| scripts/summarize_ginza_optimization.py         | 生成本轮机器摘要                                        |
| data/validation/ginza_optimization_summary.json | 全部测量摘要、suite 时间线、GPU 并发结果                |
| data/validation/ginza_gpu_pipeline_report.json | kernel trace、队列重放和 4060 Ti 外推结果                 |
| .agents/analysis/ginza-opt/                     | 原始完整字段、输入快照与全部测量                        |
| .agents/analysis/ginza-optimization-records.zip | 本轮本机完整归档                                        |
| experiments/ginza-optimized/                    | 编译得到的扩展模块                                      |

本机 PowerShell 复现命令，未来实验按下一阶段方案执行：

```powershell
$python = 'experiments/ginza-cuda/Scripts/python.exe'
& $python -m pip install Cython==3.0.12 threadpoolctl==3.6.0
& $python -X utf8 scripts/build_ginza_decode.py
& $python -X utf8 scripts/benchmark_ginza_optimized.py --mode baseline --precision fp16 --input .agents/analysis/ginza-opt/units.json --output .agents/analysis/ginza-opt/baseline-fp16.json
& $python -X utf8 scripts/benchmark_ginza_pipeline_suite.py --workers 6 --repeats 3 --input .agents/analysis/ginza-opt/units.json --reference .agents/analysis/ginza-opt/baseline-fp16.json --output .agents/analysis/ginza-opt/pipeline-suite.json
& $python -X utf8 scripts/benchmark_ginza_gpu_concurrency.py --output .agents/analysis/ginza-opt/gpu-concurrency.json
& $python -X utf8 scripts/summarize_ginza_optimization.py --folder .agents/analysis/ginza-opt --output data/validation/ginza_optimization_summary.json
```

原始小说文本路径属于本机资料，已保存的 units.json 支持后续复现。源码核对与链接见根目录 [sources.md](../sources.md)。本轮没有执行云端调用，也没有生成 4060 Ti 实际性能结论。
