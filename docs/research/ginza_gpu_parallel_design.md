# GiNZA GPU 并行设计

## 结论

生产执行核心采用一份 FP16 BERT、单计算流、独立 non-blocking D2H 流、两个 pinned host 与 shared memory 缓冲槽和八个 CPU 解码 worker。跨请求合批使用长度分桶与请求轮转，批次身份携带 `request_id`、`unit_id` 和原始顺序。结构解码调用 spaCy 原版状态机；实验性的融合 Cython 解码已从生产路径移除。

4060 Ti 16 GiB 的官方参数为 4352 CUDA cores、2.54 GHz boost、16 GB GDDR6 和 128-bit memory interface。按本轮硬件输入，GPU 计算能力按 2.2×、CPU 按 1.5× 建模。理论 FP32 峰值约为 `4352 × 2 × 2.54 GHz = 22.1 TFLOPS`；实际吞吐由 Tensor Core、序列长度、attention、kernel occupancy 和 host 供给共同决定。

## 已有证据

RTX 3050 的完整 trace 使用固定 9068 GiNZA token 语料。一次完整任务 wall time 为 5.44 秒，CUDA kernel 活跃 1.19 秒，包含 GPU 传输的设备忙碌时间为 1.23 秒，D2H 传输独占约 0.032 秒，host 暴露时间约 4.22 秒。BERT kernel 占 GPU kernel 活跃时间约 1.15 秒，结构头约 0.042 秒。GPU 只占完整关键路径的一小部分，CPU 解码、结果重建和字段编码是当前限制。

同一模型和进程池的对照显示，原版解码阶段重叠为 1856 token/s，融合解码阶段重叠为 1830 token/s；两者与 FP16 基准字段一致。融合解码没有带来收益，生产核心已固定为原版解码。

GPU 并发实验的稳定部分为单流 batch 8/16/32/64，WordPiece 吞吐约 11331/12048/11954/12067；batch 8 使用 2/4 stream 约 11692/11666。batch 16 双流出现 863 的异常值，说明多计算流尚未形成可靠收益。设计只保留独立 D2H 流，多计算流继续作为目标设备上的受控实验。

## 显存与批次预算

模型 FP16 权重约 644 MiB。实际缓冲按 8192 GiNZA token 上限、两个槽和 parser/NER 连续特征估算，3050 trace 中双槽共享缓冲约 707 MiB；模型、PyTorch allocator、Transformer 激活和运行时余量单独计入显存预算。4060 Ti 16 GiB 采用 12 GiB 运行预算，至少保留 4 GiB 给 allocator 碎片、长序列临时张量和未来 stream 配置。

生产批次参数：外层 batch 32，Transformer `max_batch_items=8192`，单槽上限 8192 token，两个在途槽，八个 CPU worker。长度分桶将 64 token 内的单元放入相近批次；跨请求队列按请求轮转选取同桶单元，防止长请求独占 GPU。若 trace 显示显存低于 70% 且 Tensor Core occupancy 仍低于目标，依次将外层 batch 调整为 48、64，再将 `max_batch_items` 调整为 12288；每次调整都需通过输出差分和峰值显存检查。

## 流程与并行边界

GPU 默认流执行 tokenizer 后的 BERT 和结构头；独立 non-blocking copy stream 在 GPU event 后执行 D2H。D2H 目标必须写入 page-locked host memory，随后复制到对应 shared memory 槽。CPU worker 只读取已发布槽，槽在所有 future 完成后复用。队列深度为 2，避免无限积压和跨批次覆盖。

多计算流只有满足以下条件才启用：相同 batch 的结果逐值一致；连续三次测量的吞吐提升至少 10%；显存峰值低于 12 GiB；没有 host submit 阻塞或异常长尾。否则保持单计算流，使用跨请求合批提高单次 kernel 的有效工作量。多流重点服务短批次和尾延迟，不能假设它会提升单个大批次的峰值吞吐。

## 4060 Ti 性能模型

使用 3050 trace 的阶段成本作为基线，GPU 阶段按 2.2× 缩放，CPU 阶段按 1.5× 缩放，解码并行效率先取实测值再更新。当前队列模型的目标结果如下：

| CPU worker | 预测完整吞吐 |
| ---: | ---: |
| 4 | 约 2740 token/s |
| 6 | 约 3650 token/s |
| 8 | 约 3690 token/s |
| 10 | 约 3920 token/s |

八个 worker 作为默认值，十个 worker 只在 CPU 利用率和尾延迟均满足条件时启用。模型是硬件倍率与现有事件成本的外推，目标设备部署后需要用相同语料重新测量。

## 验证矩阵

目标设备按 `batch ∈ {16, 32, 48, 64}`、`max_batch_items ∈ {4096, 8192, 12288}`、`streams ∈ {1, 2}` 和 `workers ∈ {6, 8, 10}` 组合测量。每组预热一次、正式测量五次，记录 GPU kernel active/device busy、Tensor Core occupancy、显存峰值、D2H overlap、CPU queue wait、完整 wall time 和 JSON export time。输出逐字段比较 FP16 基准，混合请求额外比较 request/unit 顺序、公平性和 p95 延迟。

验收条件是完整管线至少 1500 token/s，生产目标按 3000 token/s 规划；GPU 阶段相对 3050 基线达到 2.2× 的硬件预期，CPU 阶段达到 1.5× 的硬件预期，组织优化产生的额外收益单独报告。

本阶段报告保存在 `data/validation/ginza_gpu_pipeline_report.json`，原始 trace 保存在 `.agents/analysis/ginza-next/pipeline-trace.json`，跨请求 smoke 结果验证了 unit 顺序和双槽轮转。
