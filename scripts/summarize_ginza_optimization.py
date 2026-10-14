"""汇总本轮测量及批次时间线，供后续阶段模型分析。"""
from pathlib import Path
import argparse
import json
import statistics


def median_stages(stages):
    return {key: statistics.median(stage[key] for stage in stages)
            for key, value in stages[0].items() if isinstance(value, (int, float))}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--folder", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    results = []
    for path in sorted(args.folder.glob("*.json")):
        data = json.loads(path.read_text(encoding="utf-8"))
        if not isinstance(data, dict) or data.get("schema") != "readux.ginza-optimization.v1":
            continue
        summary = {key: value for key, value in data.items() if key not in ("outputs", "stages")}
        summary["name"] = path.stem
        if data["stages"]:
            summary["stage_medians"] = median_stages(data["stages"])
            if "gpu_compute_seconds" in summary["stage_medians"]:
                stage = summary["stage_medians"]
                summary["cpu_wait_over_gpu_interval"] = stage["cpu_wait_seconds"]/stage["gpu_compute_seconds"]
        results.append(summary)
    suite_path = args.folder / "pipeline-suite.json"
    suite = json.loads(suite_path.read_text(encoding="utf-8")) if suite_path.exists() else None
    result = {"schema": "readux.ginza-optimization-summary.v1", "runs": results,
              "pipeline_suite": suite,
              "gpu_concurrency": json.loads((args.folder/"gpu-concurrency.json").read_text(encoding="utf-8"))}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2)+"\n", encoding="utf-8")
    for item in results:
        print(item["name"], round(item["tokens_per_second"]), item["differences"])


if __name__ == "__main__":
    main()
