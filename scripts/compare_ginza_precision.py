"""比较 GiNZA 小模型与 BERT 的精度配置，保存来源输出及随机差异样本。"""
from __future__ import annotations
import argparse
import gc
import json
import os
from pathlib import Path
import random
import re
import time

from benchmark_ginza_cuda import MODEL, REVISION, measure, save

ROOT = Path(__file__).resolve().parents[1]


def samples():
    cases = [{"id":f"p{i+1:02}","text":text} for i,text in enumerate(
        (ROOT/"data/validation/refractor_source.txt").read_text(encoding="utf-8").splitlines())]
    source = Path(r"D:\Downloads\epub-exp\source\七日の喰い神 (ガガガ文庫) (カミツキレイニー)\output.md")
    text = source.read_text(encoding="utf-8").split("## 第一話",1)[1].split("## 第二話",1)[0]
    text = "\n".join(x for x in text.splitlines() if not x.startswith(("## ","![]("))).strip()
    text = re.sub(r"《[^》]*》","",text)
    text = "".join(line.strip() for line in text.splitlines())
    sentences = re.findall(r".+?[。！？][」』]?|.+$",text)
    units = []
    pending = ""
    for sentence in sentences:
        if pending and len(pending)+len(sentence)>447:
            units.append(pending)
            pending = ""
        pending += sentence
        if len(units)>=32: break
    cases += [{"id":f"novel-{i+1:02}","text":unit} for i,unit in enumerate(units[:32])]
    return cases


def snapshot(doc):
    import ginza
    return {
        "tokens":[{"range":[t.idx,t.idx+len(t.text)],"text":t.text,"lemma":t.lemma_,
            "norm":t.norm_,"pos":t.pos_,"tag":t.tag_,"morph":t.morph.to_dict(),
            "dep":t.dep_,"head":[t.head.idx,t.head.idx+len(t.head.text)],
            "entity":[t.ent_iob_,t.ent_type_],
            "bunsetsu":ginza.bunsetu_position_type(t)} for t in doc],
        "sentences":[[s.start_char,s.end_char] for s in doc.sents],
        "entities":[{"range":[s.start_char,s.end_char],"text":s.text,"label":s.label_} for s in doc.ents]}


def run(args):
    import torch
    torch.set_num_threads(4)
    torch.cuda.set_per_process_memory_fraction(.8)
    torchlib=Path(torch.__file__).parent/"lib"
    os.add_dll_directory(str(torchlib))
    os.environ["CUDA_PATH"]=str(torchlib.parent.parent/"nvidia"/"cuda_runtime")
    import spacy
    from thinc.api import set_gpu_allocator
    if args.variant=="small-cpu":
        spacy.require_cpu()
    else:
        spacy.require_gpu(0)
        set_gpu_allocator("pytorch")
    small=args.variant.startswith("small")
    started=time.perf_counter()
    nlp=spacy.load("ja_ginza" if small else "ja_ginza_bert_large")
    hf=None
    if not small:
        component=nlp.get_pipe("transformer")
        shim=component.model.layers[0].shims[0]
        hf=component.model.transformer
        if args.variant=="amp":
            shim._mixed_precision=True
        elif args.variant=="fp16":
            hf.half()
        elif args.variant in ("int8","nf4"):
            from transformers import AutoModel,BitsAndBytesConfig
            original=hf
            shim._model=None
            shim._hfmodel.transformer=None
            del original,hf
            gc.collect()
            torch.cuda.empty_cache()
            quant=BitsAndBytesConfig(load_in_8bit=True) if args.variant=="int8" else BitsAndBytesConfig(
                load_in_4bit=True,bnb_4bit_quant_type="nf4",bnb_4bit_compute_dtype=torch.float16,
                bnb_4bit_use_double_quant=True)
            hf=AutoModel.from_pretrained(MODEL,revision=REVISION,
                quantization_config=quant,device_map={"":0},torch_dtype=torch.float16,
                attn_implementation="sdpa")
            shim._model=hf
            shim._hfmodel.transformer=hf
        if args.variant in ("fp16","int8","nf4"):
            def output_float32(module,inputs,output):
                output["last_hidden_state"]=output.last_hidden_state.float()
                return output
            hf.register_forward_hook(output_float32)
    torch.cuda.synchronize()
    load_seconds=time.perf_counter()-started
    cases=samples()
    texts=[x["text"] for x in cases]
    # 同一批语料和同一批大小，预热之后统计稳定结果。
    with torch.inference_mode():
        list(nlp.pipe(texts,batch_size=args.batch))
        def operation():
            return list(nlp.pipe(texts,batch_size=args.batch))
        metrics=measure(torch,operation,args.repeats)
        docs=operation()
    outputs=[{"id":case["id"],"text":case["text"],**snapshot(doc)} for case,doc in zip(cases,docs)]
    count=sum(len(doc) for doc in docs)
    chars=sum(map(len,texts))
    model_mib=hf.get_memory_footprint()/2**20 if hf is not None else None
    output={"variant":args.variant,"model":nlp.meta["name"],"model_version":nlp.meta["version"],
        "load_seconds":load_seconds,"batch":args.batch,"cases":outputs,
        "characters":chars,"tokens":count,"metrics":metrics,"model_mib":model_mib,
        "characters_per_second":chars/metrics["seconds"],"tokens_per_second":count/metrics["seconds"]}
    save(args.output,output)
    print(json.dumps({k:v for k,v in output.items() if k not in ("cases","metrics")},ensure_ascii=False),flush=True)
    print(json.dumps({k:v for k,v in metrics.items() if k!="gpu_samples"},ensure_ascii=False),flush=True)


def compare(args):
    folder=args.output
    runs={p.stem:json.loads(p.read_text(encoding="utf-8")) for p in folder.glob("*.json")
          if p.stem in ("small-cpu","small-cuda","fp32","amp","fp16","int8","nf4")}
    pairs=[("small-cpu","small-cuda"),("small-cpu","fp32"),("small-cpu","fp16")]+[("fp32",v) for v in ("amp","fp16","int8","nf4") if v in runs]
    result={}
    for left,right in pairs:
        if left not in runs or right not in runs: continue
        differences=[]
        counts={}
        common=0
        sentence_changes=0
        entity_changes=0
        for a,b in zip(runs[left]["cases"],runs[right]["cases"]):
            sentence_changes += int(a["sentences"]!=b["sentences"])
            entity_changes += int(a["entities"]!=b["entities"])
            la={tuple(t["range"]):t for t in a["tokens"]}
            lb={tuple(t["range"]):t for t in b["tokens"]}
            for anchor in sorted(la.keys()|lb.keys()):
                ta,tb=la.get(anchor),lb.get(anchor)
                if ta and tb:
                    common+=1
                    fields=[key for key in ta if key not in ("range","text") and ta[key]!=tb[key]]
                else: fields=["boundary"]
                if not fields: continue
                for key in fields: counts[key]=counts.get(key,0)+1
                start,end=anchor
                differences.append({"case_id":a["id"],"anchor":list(anchor),"surface":a["text"][start:end],
                    "fields":fields,"context":a["text"][max(0,start-65):min(len(a["text"]),end+65)],
                    "left":ta,"right":tb,
                    "left_neighbors":[t for t in a["tokens"] if t["range"][0]<end+20 and t["range"][1]>start-20],
                    "right_neighbors":[t for t in b["tokens"] if t["range"][0]<end+20 and t["range"][1]>start-20]})
        key=left+"-vs-"+right
        eligible=[d for d in differences if d["surface"].strip()
                  and (d["left"] or d["right"])["pos"] not in ("PUNCT","SPACE")]
        seed=20261014 if right=="fp16" and left=="small-cpu" else 20261013
        random_samples=random.Random(seed).sample(eligible,min(10,len(eligible)))
        result[key]={"common_tokens":common,"different_positions":len(differences),
            "sentence_changed_units":sentence_changes,"entity_changed_units":entity_changes,
            "field_counts":counts,"eligible_sample_positions":len(eligible),
            "random_seed":seed,"samples":random_samples}
        save(folder/(key+"-all-diffs.json"),differences)
    save(folder/"comparison.json",result)
    print(json.dumps({k:{x:y for x,y in v.items() if x!="samples"} for k,v in result.items()},ensure_ascii=False,indent=2))


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--variant",choices=["small-cpu","small-cuda","fp32","amp","fp16","int8","nf4"])
    p.add_argument("--compare",action="store_true")
    p.add_argument("--batch",type=int,default=16)
    p.add_argument("--repeats",type=int,default=2)
    p.add_argument("--output",type=Path,required=True)
    args=p.parse_args()
    if args.compare:compare(args)
    else:run(args)


if __name__=="__main__":
    main()
