"""批次队列重放、worker 容量模型和 CUDA kernel trace 归因。"""
from collections import defaultdict
import argparse
import heapq
import json
from pathlib import Path
import statistics


def union_seconds(intervals):
    end, total = float("-inf"), 0.
    for start, stop in sorted(intervals):
        total += max(0., stop - max(start, end))
        end = max(end, stop)
    return total


def summarize_trace(path):
    events = json.loads(path.read_text(encoding="utf-8"))["traceEvents"]
    timed = [e for e in events if e.get("ph") == "X" and "dur" in e]
    kernels = [e for e in timed if e.get("cat") == "kernel"]
    copies = [e for e in timed if e.get("cat") in ("gpu_memcpy", "gpu_memset")]
    phases = [e for e in timed if e.get("name") in ("ginza.bert", "ginza.heads")]
    jobs = [e for e in timed if e.get("name") == "ginza.job"]
    launches = {e.get("args", {}).get("correlation"): e for e in timed
                if e.get("cat") in ("cuda_runtime", "cuda_driver")
                and "Launch" in e.get("name", "")}
    def interval(e):
        return e["ts"] / 1e6, (e["ts"] + e["dur"]) / 1e6
    active = union_seconds(map(interval, kernels))
    device_busy = union_seconds(map(interval, kernels + copies))
    by_phase = defaultdict(list)
    for kernel in kernels:
        launch = launches.get(kernel.get("args", {}).get("correlation"))
        phase = next((e["name"] for e in phases if launch and
                      e["ts"] <= launch["ts"] <= e["ts"] + e["dur"]), "unattributed")
        by_phase[phase].append(interval(kernel))
    result = {"kernel_count": len(kernels), "kernel_active_seconds": active,
              "device_busy_seconds": device_busy,
              "copy_exclusive_seconds": device_busy - active,
              "phase_kernel_seconds": {name: union_seconds(items) for name, items in by_phase.items()},
              "copy_sum_seconds": sum(e["dur"] for e in copies) / 1e6}
    if jobs and active:
        elapsed = jobs[0]["dur"] / 1e6
        exposed = max(0., elapsed - device_busy)
        result.update(job_seconds=elapsed, exposed_host_seconds=exposed,
                      exposed_host_over_gpu_active=exposed / active,
                      cpu_50_percent_pass=exposed / active <= .5)
    return result


def replay(stage, workers, depth=2, gpu_scale=1., cpu_scale=1., single_cpu_seconds=None):
    """FIFO CPU 池与固定槽容量的离散事件模型；输入使用逐批服务成本。"""
    lanes = [0.] * workers
    completed, timeline, producer = [], [], 0.
    tasks = stage["decode_tasks"]
    total_tokens = sum(batch["tokens"] for batch in stage["batches"])
    for index, batch in enumerate(stage["batches"]):
        prepare = batch["prepare_finished"] - batch["prepare_started"]
        gpu = batch.get("gpu_event_seconds", 0.)
        prepare_target = gpu / gpu_scale + max(0., prepare - gpu) / cpu_scale
        submit = batch.get("submit_finished", batch["prepare_finished"]) - batch.get("d2h_ready", batch["prepare_finished"])
        submit = max(0., submit)
        if "submit_finished" not in batch:
            submit = stage.get("submit_seconds", 0.) / len(stage["batches"])
        start = max(producer, completed[index - depth] if index >= depth else 0.)
        ready = start + prepare_target + submit / cpu_scale
        batch_tasks = [task for task in tasks if task["batch_index"] == index]
        stops = []
        for task in batch_tasks:
            lane = heapq.heappop(lanes)
            service = ((single_cpu_seconds * task["tokens"] / total_tokens) if single_cpu_seconds is not None
                       else task["task_finished"] - task["task_started"]) / cpu_scale
            stop = max(lane, ready) + service
            heapq.heappush(lanes, stop)
            stops.append(stop)
        completed.append(max(stops, default=ready))
        producer = ready
        timeline.append({"batch": index, "start": start, "ready": ready, "complete": completed[-1]})
    wall = max(completed, default=0.) + stage["tokenizer_seconds"] / cpu_scale + stage["reconstruct_seconds"] / cpu_scale
    return {"seconds": wall, "tokens_per_second": total_tokens / wall if wall else 0., "batches": timeline}


def queue_summary(stage, workers):
    batches, tasks = stage["batches"], stage["decode_tasks"]
    if not batches:
        return {}
    start = batches[0]["prepare_started"]
    stop = max(task["task_finished"] for task in tasks)
    service = sum(task["task_finished"] - task["task_started"] for task in tasks)
    per_worker = defaultdict(list)
    for task in tasks:
        per_worker[task.get("worker_pid", "unrecorded")].append((task["task_started"], task["task_finished"]))
    imbalance = []
    for index in range(len(batches)):
        values = [task["tokens"] for task in tasks if task["batch_index"] == index]
        imbalance.append(max(values) / statistics.mean(values))
    queued = [max(0., task["task_started"] - task["task_submitted"]) for task in tasks if "task_submitted" in task]
    return {"pool_busy_fraction": service / (workers * (stop - start)),
            "worker_busy_seconds": {str(pid): union_seconds(items) for pid, items in per_worker.items()},
            "batch_max_over_mean_tokens": imbalance,
            "queue_wait_median_seconds": statistics.median(queued) if queued else None,
            "queue_wait_max_seconds": max(queued) if queued else None,
            "first_batch_to_first_decode_seconds": min(t["task_started"] for t in tasks) - start,
            "tail_seconds": max(0., stop - batches[-1]["prepare_finished"]),
            "slot_backpressure_seconds": sum(max(0., b["prepare_started"] - a["prepare_finished"])
                                              for a, b in zip(batches[1:], batches[2:]))}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--single-cpu", type=Path)
    parser.add_argument("--trace", type=Path)
    args = parser.parse_args()
    data = json.loads(args.input.read_text(encoding="utf-8"))
    single = json.loads(args.single_cpu.read_text(encoding="utf-8")) if args.single_cpu else None
    c1 = statistics.median(stage["cpu_decode_seconds"] for stage in single["stages"]) if single else None
    results = data.get("results", [data])
    report = []
    for result in results:
        workers = result.get("workers", result.get("options", {}).get("workers", 6))
        depth = 2 if result.get("overlap", not result.get("options", {}).get("serial", False)) else 1
        runs = []
        for seconds, stage in zip(result["runs_seconds"], result["stages"]):
            estimate = replay(stage, workers, depth)
            runs.append({"observed_seconds": seconds, "replay": estimate,
                         "relative_residual": (estimate["seconds"] - seconds) / seconds,
                         "queue": queue_summary(stage, workers)})
        target = []
        if c1 is not None:
            stage = result["stages"][len(result["stages"]) // 2]
            for count in (4, 6, 8, 10):
                scenario = replay(stage, count, 2, 2.2, 1.5, c1)
                scenario.update(workers=count, gpu_hardware_scale=2.2, cpu_hardware_scale=1.5,
                                cpu_parallel_efficiency_assumed=1., extrapolation=True)
                target.append(scenario)
        report.append({"implementation": result.get("implementation", result.get("options", {}).get("mode")),
                       "runs": runs, "target_scenarios": target})
    output = {"schema": "readux.ginza-queue-model.v1", "single_cpu_seconds": c1, "results": report}
    if args.trace:
        output["trace"] = summarize_trace(args.trace)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"runs": len(report), "single_cpu_seconds": c1, "trace": output.get("trace")}, ensure_ascii=False))


if __name__ == "__main__":
    main()
