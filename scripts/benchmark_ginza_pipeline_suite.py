"""同一模型与进程池内比较串行、阶段重叠、原版和融合 CPU 解码。"""
import argparse
import hashlib
import json
from pathlib import Path
import statistics
import time

from benchmark_ginza_optimized import complete_snapshot
from ginza_bert_engine import BertEngine, load_gpu


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--reference", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--workers", type=int, default=6)
    parser.add_argument("--repeats", type=int, default=3)
    args = parser.parse_args()
    units = json.loads(args.input.read_text(encoding="utf-8"))
    reference = json.loads(args.reference.read_text(encoding="utf-8"))["outputs"]
    texts = [unit["text"] for unit in units]
    nlp = load_gpu()
    engine = BertEngine(nlp, args.workers)
    import torch
    tokens = sum(len(item["tokens"]) for item in reference)
    results = []
    try:
        for mode, overlap in (("original", False), ("original", True)):
            engine.implementation = mode
            engine.analyze(texts, overlap=overlap)
            runs, stages, exports, differences = [], [], [], []
            for _ in range(args.repeats):
                torch.cuda.synchronize()
                started = time.perf_counter()
                docs = engine.analyze(texts, overlap=overlap)
                torch.cuda.synchronize()
                runs.append(time.perf_counter() - started)
                stages.append(engine.metrics)
                started = time.perf_counter()
                outputs = [{"id": unit["id"], **complete_snapshot(doc)} for unit, doc in zip(units, docs)]
                encoded = json.dumps(outputs, ensure_ascii=False).encode("utf-8")
                exports.append(time.perf_counter() - started)
                differences.append([a["id"] for a, b in zip(outputs, reference) if a != b])
                assert len(outputs) == len(reference)
                del docs
            result = {"implementation": mode, "overlap": overlap, "workers": args.workers,
                      "runs_seconds": runs, "seconds": statistics.median(runs),
                      "tokens_per_second": tokens/statistics.median(runs), "stages": stages,
                      "export_seconds": exports, "output_bytes": len(encoded),
                      "export_tokens_per_second": tokens/statistics.median([a+b for a,b in zip(runs, exports)]),
                      "differences": differences}
            results.append(result)
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(json.dumps({"schema": "readux.ginza-pipeline-suite.v1", "tokens": tokens,
                                              "input_sha256": hashlib.sha256(json.dumps(units, ensure_ascii=False).encode("utf-8")).hexdigest(),
                                              "results": results}, ensure_ascii=False, indent=2)+"\n", encoding="utf-8")
            print(json.dumps({key:value for key,value in result.items() if key != "stages"}), flush=True)
            assert not any(differences)
    finally:
        engine.close()


if __name__ == "__main__":
    main()
