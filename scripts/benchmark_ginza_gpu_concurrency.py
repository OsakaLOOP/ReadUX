"""使用真实 GiNZA BERT 输入，对照单流合批与共享模型多 CUDA stream。"""
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import argparse
import json
import statistics
import time

from compare_ginza_precision import samples
from ginza_bert_engine import load_gpu


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--repeats", type=int, default=3)
    args = parser.parse_args()
    nlp = load_gpu("fp16")
    import torch
    hf = nlp.get_pipe("transformer").model.transformer
    hf.eval()
    captured = []
    def capture(module, positional, kwargs):
        captured.append({key: value.detach().clone() if torch.is_tensor(value) else value
                         for key, value in kwargs.items()})
    hook = hf.register_forward_pre_hook(capture, with_kwargs=True)
    texts = [case["text"] for case in samples()]
    with torch.inference_mode():
        list(nlp.get_pipe("transformer").pipe(nlp.tokenizer.pipe(texts), batch_size=16))
    hook.remove()
    # spaCy shim 结束预测时恢复训练模式，独立 PyTorch 实验显式进入 eval。
    hf.eval()
    # 展平真实分窗输入，统一 padding，所有配置执行同一组序列。
    keys = [key for key in captured[0] if torch.is_tensor(captured[0][key])]
    length = max(item["input_ids"].shape[1] for item in captured)
    tensors = {key: torch.cat([torch.nn.functional.pad(item[key], (0, length-item[key].shape[1]))
                              for item in captured]) for key in keys}
    sequences = len(tensors["input_ids"])
    wordpieces = int(tensors["attention_mask"].sum().item())
    base_options = {key: value for key, value in captured[0].items() if key not in keys}
    cases = []
    reference = None
    for batch, concurrency in ((8, 1), (16, 1), (32, 1), (64, 1), (8, 2), (8, 4), (16, 2)):
        streams = [torch.cuda.Stream() for _ in range(concurrency)]
        pool = ThreadPoolExecutor(concurrency) if concurrency > 1 else None
        batches = [{key: value[start:start+batch] for key, value in tensors.items()}
                   for start in range(0, sequences, batch)]
        def execute_lane(lane):
            with torch.inference_mode(), torch.cuda.stream(streams[lane]):
                return [(index, hf(**batches[index], **base_options).last_hidden_state)
                        for index in range(lane, len(batches), concurrency)]
        def operation():
            if pool:
                futures = [pool.submit(execute_lane, lane) for lane in range(concurrency)]
                indexed = [item for future in futures for item in future.result()]
            else:
                indexed = execute_lane(0)
            return [output for _, output in sorted(indexed, key=lambda item: item[0])]
        torch.cuda.synchronize()
        operation()
        torch.cuda.synchronize()
        torch.cuda.reset_peak_memory_stats()
        runs = []
        for _ in range(args.repeats):
            started = time.perf_counter()
            output = operation()
            torch.cuda.synchronize()
            runs.append(time.perf_counter()-started)
        joined = torch.cat(output)
        if reference is None:
            reference = joined.clone()
        max_absolute_difference = float((joined-reference).abs().max().item())
        active_difference = float((joined-reference).abs()[tensors["attention_mask"].bool()].max().item())
        seconds = statistics.median(runs)
        result = {"batch": batch, "streams": concurrency, "seconds": seconds, "runs_seconds": runs,
                  "wordpieces_per_second": wordpieces/seconds,
                  "peak_allocated_mib": torch.cuda.max_memory_allocated()/2**20,
                  "max_absolute_difference": max_absolute_difference,
                  "active_max_absolute_difference": active_difference}
        cases.append(result)
        print(json.dumps(result), flush=True)
        if pool:
            pool.shutdown()
        del output, joined
    result = {"schema": "readux.ginza-gpu-concurrency.v1", "gpu": torch.cuda.get_device_name(),
              "precision": "fp16", "attention": hf.config._attn_implementation,
              "sequences": sequences, "padded_length": length,
              "wordpieces": wordpieces, "cases": cases}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2)+"\n", encoding="utf-8")


if __name__ == "__main__":
    main()
