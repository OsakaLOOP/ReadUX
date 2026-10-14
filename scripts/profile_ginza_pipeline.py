"""预热后记录一次完整 FP16 分析及字段编码的 CPU/CUDA trace。"""
import argparse
import json
from pathlib import Path
import time

from analyze_ginza_pipeline import summarize_trace
from benchmark_ginza_optimized import complete_snapshot
from ginza_bert_engine import BertEngine, load_gpu


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--workers", type=int, default=6)
    parser.add_argument("--batch", type=int, default=32)
    parser.add_argument("--max-batch-items", type=int, default=8192)
    args = parser.parse_args()
    units = json.loads(args.input.read_text(encoding="utf-8"))
    texts = [unit["text"] for unit in units]
    nlp = load_gpu(max_batch_items=args.max_batch_items)
    engine = BertEngine(nlp, args.workers)
    import torch
    try:
        engine.analyze(texts, args.batch)
        torch.cuda.synchronize()
        with torch.profiler.profile(activities=[torch.profiler.ProfilerActivity.CPU,
                                                torch.profiler.ProfilerActivity.CUDA]) as profile:
            started = time.perf_counter()
            with torch.profiler.record_function("ginza.job"):
                docs = engine.analyze(texts, args.batch)
                outputs = [{"id": unit["id"], **complete_snapshot(doc)} for unit, doc in zip(units, docs)]
                json.dumps(outputs, ensure_ascii=False).encode("utf-8")
                torch.cuda.synchronize()
            elapsed = time.perf_counter() - started
        args.output.parent.mkdir(parents=True, exist_ok=True)
        profile.export_chrome_trace(str(args.output))
        summary = summarize_trace(args.output)
        summary.update(gpu=torch.cuda.get_device_name(), wall_seconds=elapsed, stages=engine.metrics,
                       tokens=sum(len(doc) for doc in docs), profiler_overhead_included=True)
        args.output.with_suffix(".summary.json").write_text(
            json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(json.dumps({key: value for key, value in summary.items() if key != "stages"}), flush=True)
        if not summary["kernel_count"]:
            raise RuntimeError("trace 未采集到 CUDA kernel；请在具备 CUPTI 的环境执行同一命令")
    finally:
        engine.close()


if __name__ == "__main__":
    main()
