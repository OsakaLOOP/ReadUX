"""固定完整语料，对照 BERT 精度、原版解码、融合解码及并行编排。"""
from __future__ import annotations

import argparse
import hashlib
import importlib.metadata as metadata
import json
from pathlib import Path
import statistics
import threading
import time

from compare_ginza_precision import samples, snapshot
from ginza_bert_engine import BertEngine, load_gpu


def complete_snapshot(doc):
    import ginza
    result = snapshot(doc)
    result["clauses"] = {str(key): value for key, value in doc.user_data["clauses"].items()}
    result["bunsetsu_spans"] = [[s.start_char, s.end_char] for s in ginza.bunsetu_spans(doc)]
    result["sub_tokens"] = [None if part is None else
                            [[list(token) for token in mode] for mode in part]
                            for part in doc.user_data["sub_tokens"]]
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", choices=["baseline", "original"], default="original")
    parser.add_argument("--precision", choices=["fp32", "fp16", "amp"], default="fp16")
    parser.add_argument("--threads", type=int, default=1)
    parser.add_argument("--workers", type=int, default=0)
    parser.add_argument("--batch", type=int, default=32)
    parser.add_argument("--max-batch-items", type=int, default=8192)
    parser.add_argument("--buffer-tokens", type=int, default=8192)
    parser.add_argument("--sync-d2h", action="store_true")
    parser.add_argument("--serial", action="store_true")
    parser.add_argument("--unsorted", action="store_true")
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--limit", type=int)
    parser.add_argument("--reference", type=Path)
    parser.add_argument("--input", type=Path, help="有序 JSON 单元列表，每项包含 id 和 text")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    started = time.perf_counter()
    nlp = load_gpu(args.precision, args.threads, args.max_batch_items)
    import torch
    import psutil
    torch.cuda.synchronize()
    load_seconds = time.perf_counter() - started
    started = time.perf_counter()
    engine = None if args.mode == "baseline" else BertEngine(
        nlp, args.workers, args.threads, args.mode, not args.sync_d2h, args.buffer_tokens)
    initialize_seconds = time.perf_counter() - started
    cases = (json.loads(args.input.read_text(encoding="utf-8")) if args.input else samples())[:args.limit]
    texts = [case["text"] for case in cases]
    def operation():
        with torch.inference_mode():
            if engine:
                return engine.analyze(texts, args.batch, not args.serial, not args.unsorted)
            return list(nlp.pipe(texts, batch_size=args.batch))
    started = time.perf_counter()
    warmup = operation()
    torch.cuda.synchronize()
    warmup_seconds = time.perf_counter() - started
    del warmup
    torch.cuda.reset_peak_memory_stats()
    runs = []
    stages = []
    conversions = []
    repeated_outputs = []
    resources = []
    stop = threading.Event()
    process = psutil.Process()
    def sample():
        while not stop.is_set():
            processes = [process] + process.children(recursive=True)
            infos = [p.memory_info() for p in processes]
            resources.append({"rss_mib": sum(x.rss for x in infos) / 2**20,
                              "cpu_seconds": sum(sum(p.cpu_times()[:2]) for p in processes)})
            stop.wait(.2)
    sampler = threading.Thread(target=sample, daemon=True)
    sampler.start()
    try:
        for _ in range(args.repeats):
            torch.cuda.synchronize()
            started = time.perf_counter()
            docs = operation()
            torch.cuda.synchronize()
            runs.append(time.perf_counter() - started)
            if engine:
                stages.append(dict(engine.metrics))
            conversion_start = time.perf_counter()
            outputs = [{"id": case["id"], **complete_snapshot(doc)} for case, doc in zip(cases, docs)]
            encoded = json.dumps(outputs, ensure_ascii=False).encode("utf-8")
            conversions.append(time.perf_counter() - conversion_start)
            repeated_outputs.append(outputs)
            del docs
    finally:
        stop.set()
        sampler.join()
        if engine:
            engine.close()
    reference = json.loads(args.reference.read_text(encoding="utf-8"))["outputs"] if args.reference else outputs
    differences = [{"id": a["id"], "fields": [key for key in a if a[key] != b.get(key)]}
                   for a, b in zip(outputs, reference) if a != b]
    assert len(reference) == len(outputs)
    tokens = sum(len(x["tokens"]) for x in outputs)
    seconds = statistics.median(runs)
    result = {"schema": "readux.ginza-optimization.v1", "options": vars(args) | {"output": str(args.output), "reference": str(args.reference)},
              "load_seconds": load_seconds, "initialize_seconds": initialize_seconds, "warmup_seconds": warmup_seconds,
              "characters": sum(map(len, texts)), "tokens": tokens, "units": len(cases),
              "input_sha256": hashlib.sha256(json.dumps(cases, ensure_ascii=False).encode("utf-8")).hexdigest(),
              "seconds": seconds, "runs_seconds": runs, "tokens_per_second": tokens / seconds,
              "conversion_runs_seconds": conversions, "output_bytes": len(encoded),
              "export_tokens_per_second": tokens / statistics.median([a+b for a,b in zip(runs, conversions)]),
              "repeat_output_consistent": all(item == outputs for item in repeated_outputs),
              "characters_per_second": sum(map(len, texts)) / seconds, "stages": stages,
              "peak_allocated_mib": torch.cuda.max_memory_allocated()/2**20,
              "peak_reserved_mib": torch.cuda.max_memory_reserved()/2**20,
              "peak_rss_mib": max(x["rss_mib"] for x in resources),
              "cpu_seconds": resources[-1]["cpu_seconds"] - resources[0]["cpu_seconds"],
              "versions": {key: metadata.version(key) for key in ("torch", "spacy", "ginza", "ja-ginza-bert-large", "Cython", "numpy", "threadpoolctl")},
              "differences": differences, "outputs": outputs}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2, default=str)+"\n", encoding="utf-8")
    print(json.dumps({key: value for key, value in result.items() if key not in ("outputs", "stages", "versions")}, ensure_ascii=False, default=str), flush=True)
    assert not differences, differences[:3]
    assert result["repeat_output_consistent"]


if __name__ == "__main__":
    main()
