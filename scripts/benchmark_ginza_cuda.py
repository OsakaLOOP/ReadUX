"""在本机 CUDA 上测量 GiNZA BERT large 的显存、吞吐与批上限。"""
from __future__ import annotations
import argparse
import gc
import importlib.metadata as metadata
import json
import os
from pathlib import Path
import statistics
import subprocess
import threading
import time

MODEL = "megagonlabs/transformers-ud-japanese-bert-large-ginza"
REVISION = "7683285d102ba2098cf5f2641388cd69c7406241"


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def gpu_status():
    text = subprocess.check_output([
        "nvidia-smi", "--query-gpu=name,memory.total,memory.used,power.draw,temperature.gpu,utilization.gpu,clocks.sm",
        "--format=csv,noheader,nounits"], encoding="utf-8").strip()
    return text


def measure(torch, operation, repeats=2):
    gc.collect()
    torch.cuda.empty_cache()
    torch.cuda.reset_peak_memory_stats()
    baseline = torch.cuda.memory_allocated()
    times = []
    samples = []
    stop = threading.Event()
    def sample():
        while not stop.is_set():
            samples.append(gpu_status())
            stop.wait(0.5)
    worker = threading.Thread(target=sample, daemon=True)
    worker.start()
    try:
        for _ in range(repeats):
            torch.cuda.synchronize()
            start = time.perf_counter()
            value = operation()
            torch.cuda.synchronize()
            times.append(time.perf_counter() - start)
            del value
    finally:
        stop.set()
        worker.join()
    return {
        "seconds": statistics.median(times), "runs_seconds": times,
        "baseline_mib": baseline / 2**20,
        "peak_allocated_mib": torch.cuda.max_memory_allocated() / 2**20,
        "peak_reserved_mib": torch.cuda.max_memory_reserved() / 2**20,
        "gpu_status": gpu_status(),
        "gpu_samples": samples,
    }


def raw(args, torch):
    from transformers import AutoModel
    dtype = getattr(torch, args.precision)
    started = time.perf_counter()
    model = AutoModel.from_pretrained(MODEL, revision=REVISION,
        torch_dtype=dtype, attn_implementation="eager").eval().cuda()
    torch.cuda.synchronize()
    parameters = sum(p.numel() for p in model.parameters())
    result = {"model": MODEL, "revision": REVISION, "precision": args.precision,
        "memory_fraction": args.memory_fraction,
        "attention": "eager", "output_hidden_states": True,
        "parameters": parameters, "parameter_mib": sum(p.numel()*p.element_size() for p in model.parameters())/2**20,
        "load_seconds": time.perf_counter()-started,
        "loaded_allocated_mib": torch.cuda.memory_allocated()/2**20, "cases": []}
    save(args.output, result)
    with torch.inference_mode():
        model(input_ids=torch.full((1, 128),100,device="cuda",dtype=torch.long))
    torch.cuda.synchronize()
    for length in args.lengths:
        def attempt(batch, repeats=1):
            try:
                ids = torch.full((batch, length), 100, device="cuda", dtype=torch.long)
                def operation():
                    with torch.inference_mode():
                        output = model(input_ids=ids, attention_mask=torch.ones_like(ids),
                            output_hidden_states=True, output_attentions=False)
                        if not torch.isfinite(output.last_hidden_state).all().item():
                            raise ValueError("模型输出含非有限值")
                        return output
                item = measure(torch, operation, repeats)
                item.update(length=length, batch=batch, status="ok",
                            tokens_per_second=batch*length/item["seconds"])
                del ids
            except torch.OutOfMemoryError as error:
                item = {"length": length, "batch": batch, "status": "oom", "error": str(error)}
                gc.collect()
                torch.cuda.empty_cache()
            result["cases"].append(item)
            save(args.output, result)
            print(json.dumps({k:v for k,v in item.items() if k!="gpu_samples"}, ensure_ascii=False), flush=True)
            return item["status"] == "ok"
        if args.batches:
            for batch in args.batches:
                if not attempt(batch,args.repeats): break
            continue
        good, bad, batch = 0, None, 1
        while batch <= args.max_batch:
            if attempt(batch):
                good = batch
                batch *= 2
            else:
                bad = batch
                break
        if bad is not None:
            while bad-good > 1:
                mid = (bad+good)//2
                if attempt(mid): good = mid
                else: bad = mid
        if good: attempt(good, 3)
        result.setdefault("limits", []).append({"length": length, "max_success_batch": good,
            "first_failed_batch": bad, "search_cap": args.max_batch})
        save(args.output, result)


def pipeline(args, torch):
    torch_lib = Path(torch.__file__).parent / "lib"
    os.add_dll_directory(str(torch_lib))
    os.environ["CUDA_PATH"] = str(Path(torch.__file__).parent.parent / "nvidia" / "cuda_runtime")
    import spacy
    import cupy
    from thinc.api import set_gpu_allocator
    set_gpu_allocator("pytorch")
    spacy.require_gpu(0)
    started = time.perf_counter()
    nlp = spacy.load("ja_ginza_bert_large")
    if args.disable_ner:
        nlp.disable_pipe("ner")
    transformer = nlp.get_pipe("transformer")
    if args.precision == "float16":
        transformer.model.layers[0].shims[0]._mixed_precision = True
    hf = transformer.model.transformer
    text = args.text.read_text(encoding="utf-8")
    if "## 第一話" in text:
        text = text.split("## 第一話",1)[1].split("## 第二話",1)[0]
    text = "\n".join(x for x in text.splitlines() if not x.startswith(("## ", "![]("))).strip()
    units = [text[i:i+args.unit_chars] for i in range(0,min(len(text),args.unit_chars*args.units),args.unit_chars)]
    result = {"model": "ja_ginza_bert_large", "precision": args.precision,
        "precision_mode": "amp" if args.precision=="float16" else "float32",
        "memory_fraction": args.memory_fraction,
        "load_seconds": time.perf_counter()-started,
        "pipeline": nlp.pipe_names, "config": nlp.config["components"]["transformer"],
        "transformer_class": type(hf).__name__,
        "parameter_mib": sum(p.numel()*p.element_size() for p in hf.parameters())/2**20,
        "unit_characters": args.unit_chars, "units": len(units),
        "versions": {x:metadata.version(x) for x in ["torch","spacy","ginza","ja-ginza-bert-large","transformers","spacy-transformers"]},
        "cases": []}
    print(json.dumps({k:v for k,v in result.items() if k not in ("config","cases")},ensure_ascii=False),flush=True)
    save(args.output,result)
    shapes = []
    def capture(module, positional, kwargs):
        ids = kwargs.get("input_ids")
        if ids is not None:
            shapes.append(list(ids.shape))
    hf.register_forward_pre_hook(capture,with_kwargs=True)
    if args.profile:
        batch = (args.batches or [8])[0]
        with torch.inference_mode():
            list(nlp.pipe(units,batch_size=batch))
            stages = []
            for repeat in range(args.repeats):
                shapes.clear()
                torch.cuda.synchronize()
                start = time.perf_counter()
                docs = list(nlp.tokenizer.pipe(units))
                stages.append({"repeat":repeat,"stage":"tokenizer","seconds":time.perf_counter()-start})
                for name,component in nlp.pipeline:
                    torch.cuda.synchronize()
                    start = time.perf_counter()
                    docs = list(component.pipe(docs,batch_size=batch)) if hasattr(component,"pipe") else [component(doc) for doc in docs]
                    torch.cuda.synchronize()
                    stages.append({"repeat":repeat,"stage":name,"seconds":time.perf_counter()-start})
                del docs
                gc.collect()
        result["profile"] = stages
        result["transformer_shapes"] = list(shapes)
        save(args.output,result)
        print(json.dumps({"profile":stages,"transformer_shapes":shapes},ensure_ascii=False),flush=True)
        return
    # 完整管线计入 token 化、Transformer、结构解析和 Doc 生成。
    for batch in (args.batches or [1,2,4,8,16,32,64]):
        seen = []
        def operation():
            seen.clear()
            shapes.clear()
            for doc in nlp.pipe(units, batch_size=batch):
                seen.append(len(doc))
            return list(seen)
        try:
            with torch.inference_mode():
                warmup_start = time.perf_counter()
                operation()
                torch.cuda.synchronize()
                warmup_seconds = time.perf_counter()-warmup_start
                item = measure(torch,operation,args.repeats)
            item.update(batch=batch,status="ok",warmup_seconds=warmup_seconds,characters=sum(map(len,units)),
                tokens=sum(seen),characters_per_second=sum(map(len,units))/item["seconds"],
                tokens_per_second=sum(seen)/item["seconds"],
                transformer_shapes=list(shapes),
                cupy_pool_mib=cupy.get_default_memory_pool().total_bytes()/2**20)
        except (torch.OutOfMemoryError,cupy.cuda.memory.OutOfMemoryError) as error:
            item = {"batch":batch,"status":"oom","error":str(error)}
            gc.collect()
            torch.cuda.empty_cache()
            cupy.get_default_memory_pool().free_all_blocks()
        result["cases"].append(item)
        save(args.output,result)
        print(json.dumps({k:v for k,v in item.items() if k!="gpu_samples"},ensure_ascii=False),flush=True)
        if item["status"]=="oom": break


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode",choices=["raw","pipeline"],required=True)
    parser.add_argument("--precision",choices=["float32","float16"],default="float32")
    parser.add_argument("--lengths",type=int,nargs="+",default=[128,256,512])
    parser.add_argument("--max-batch",type=int,default=128)
    parser.add_argument("--batches",type=int,nargs="+")
    parser.add_argument("--repeats",type=int,default=2)
    parser.add_argument("--units",type=int,default=16)
    parser.add_argument("--profile",action="store_true")
    parser.add_argument("--disable-ner",action="store_true")
    parser.add_argument("--memory-fraction",type=float,default=0.80)
    parser.add_argument("--unit-chars",type=int,default=447)
    parser.add_argument("--text",type=Path)
    parser.add_argument("--output",type=Path,required=True)
    args=parser.parse_args()
    import torch
    torch.set_num_threads(4)
    torch.cuda.set_per_process_memory_fraction(args.memory_fraction)
    print(json.dumps({"gpu":gpu_status(),"torch":torch.__version__,"cuda":torch.version.cuda,
        "memory_fraction":args.memory_fraction,"mode":args.mode},ensure_ascii=False),flush=True)
    if args.mode=="raw": raw(args,torch)
    else: pipeline(args,torch)


if __name__=="__main__":
    main()
