"""选取 dist 图书的三段正文，与当前正式分析服务逐项比较。"""

import argparse
import gzip
import json
from pathlib import Path
import subprocess

from generate_grammar_guide_analysis import request

ROOT = Path(__file__).resolve().parents[1]
INTRINSIC = {"conjugation", "initial_alternation", "final_alternation"}


def read_json(path):
    data = path.read_bytes()
    return json.loads(gzip.decompress(data) if path.suffix == ".gz" else data)


def paragraph(document, anchor):
    text = document["text"]
    candidates = []
    offset = 0
    for part in text.split("\n\n"):
        end = offset + len(part)
        if 40 <= len(part) <= 240 and anchor[0] <= offset and end <= anchor[1]:
            count = sum(offset <= op["char_range"][0] < end
                        for chain in document["morphology"]["chains"]
                        for op in chain["operators"] if op["kind"] not in INTRINSIC)
            candidates.append((count, offset, end))
        offset = end + 2
    if not candidates:
        raise ValueError("所选单元的锚点内没有完整正文段落")
    _, start, end = max(candidates)
    return [start, end]


def within(item, span):
    start, end = item["char_range"]
    return start < span[1] and span[0] < end


def project(document, lookup, span):
    tokens = [{key: token.get(key) for key in ("surface", "pos", "lemma", "reading", "char_range", "query_forms")}
              for token in document["morphemes"] if within(token, span)]
    by_id = {chain["chain_id"]: chain for chain in document["morphology"]["chains"]}
    chains = []
    for chain in by_id.values():
        if not within(chain, span):
            continue
        result = {key: chain.get(key) for key in (
            "anchor_morpheme", "char_range", "morpheme_range", "surface_form", "dictionary_form",
            "lemma_form", "lookup_form", "display_form", "role", "final_state", "status", "terminal")}
        parent = by_id.get(chain.get("parent_chain_id"))
        result["parent_range"] = parent["char_range"] if parent else None
        result["operators"] = []
        for op in chain["operators"]:
            if not within(op, span):
                continue
            item = {key: op.get(key) for key in (
                "kind", "source_morpheme_range", "char_range", "concept_id", "normalized_form",
                "state_before", "state_after", "stage", "executor", "rule_group", "input_category",
                "output_category", "terminal_mode", "order_rank")}
            connection = op.get("connection")
            item["connection"] = {key: connection.get(key) for key in (
                "rule", "path", "matched_paths", "predecessor_range", "predecessor_surface",
                "dependency_type", "context_range", "context_surface", "diagnostics")} if connection else None
            result["operators"].append(item)
        chains.append(result)
    targets = {}
    for group in ("outer_targets", "inner_targets"):
        targets[group] = [{key: target.get(key) for key in (
            "char_range", "surface", "lookup_forms", "decision", "reason")}
            for target in lookup[group] if within(target, span)]
    return {"tokens": tokens, "chains": chains, "lookup": targets}


def differences(before, after, path=""):
    if type(before) is not type(after):
        return [{"path": path, "before": before, "after": after}]
    if isinstance(before, dict):
        result = []
        for key in sorted(before.keys() | after.keys()):
            if key not in before or key not in after:
                result.append({"path": path + "/" + key, "before": before.get(key), "after": after.get(key)})
            else:
                result.extend(differences(before[key], after[key], path + "/" + key))
        return result
    if isinstance(before, list):
        if len(before) != len(after):
            return [{"path": path, "before": before, "after": after}]
        return [item for index, (left, right) in enumerate(zip(before, after))
                for item in differences(left, right, path + f"/{index}")]
    return [] if before == after else [{"path": path, "before": before, "after": after}]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dist", type=Path, default=ROOT / "dist")
    parser.add_argument("--service", type=Path, default=ROOT / "target/debug/readux-nlp.exe")
    parser.add_argument("--output", type=Path, default=ROOT / ".agents/analysis/morphology-dist-comparison.json")
    args = parser.parse_args()
    book = read_json(args.dist / "library/catalog.json")["books"][0]
    manifest_path = args.dist / book["manifest"]
    manifest = read_json(manifest_path)
    base = manifest_path.parent
    plan = read_json(base / manifest["plan"])
    plans = {unit["id"]: unit for unit in plan["units"]}
    files = sorted(set(manifest["chunks"].values()))
    samples = []
    for file in (files[0], files[len(files) // 2], files[-1]):
        unit = read_json(base / file)[0]
        unit_plan = plans[unit["unit_id"]]
        context_start = unit_plan["context_range"][0]
        anchor = [position - context_start for position in unit_plan["anchor"]["char_range"]]
        span = paragraph(unit["document"], anchor)
        samples.append((file, unit, span, context_start))
    report = {"title": book["title"], "manifest": book["manifest"],
              "version": manifest["version"], "samples": []}
    process = subprocess.Popen([str(args.service.resolve()), "stdio"], cwd=ROOT,
                               stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               text=True, encoding="utf-8", bufsize=1,
                               creationflags=subprocess.CREATE_NO_WINDOW if hasattr(subprocess, "CREATE_NO_WINDOW") else 0)
    try:
        for number, (file, saved, span, context_start) in enumerate(samples, 1):
            old = saved["document"]
            print(f"比较第 {number} 段：{saved['unit_id']}，上下文 {len(old['text'])} 字符", flush=True)
            basic = request(process, "analyze_routed", text=old["text"], policy="auto")
            enriched = request(process, "enrich", analysis_id=basic["id"])
            if not enriched["providers"] or any(provider["status"] != "ready" for provider in enriched["providers"]):
                raise RuntimeError(f"完整分析来源未就绪：{enriched['providers']}")
            live = enriched["document"]
            if live["text"] != old["text"]:
                raise ValueError("静态与当前分析的正文不一致")
            group = request(process, "lookup_targets", analysis_id=live["id"], range=[0, len(live["text"])])
            before = project(old, saved["lookup"], span)
            after = project(live, group, span)
            diff = differences(before, after)
            item = {"unit_id": saved["unit_id"], "chunk": file, "local_range": span,
                    "book_range": [position + context_start for position in span],
                    "text": old["text"][span[0]:span[1]],
                    "routing_equal": old.get("routing") == live.get("routing"),
                    "counts": {name: {"before": len(before[name]), "after": len(after[name])}
                               for name in ("tokens", "chains")},
                    "differences": diff, "before": before, "after": after}
            report["samples"].append(item)
            print(f"第 {number} 段：{len(before['tokens'])} 个 token，{len(before['chains'])} 条链，{len(diff)} 项差异", flush=True)
    finally:
        process.stdin.close()
        try:
            process.wait(timeout=15)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(f"比较报告：{args.output}", flush=True)


if __name__ == "__main__":
    main()
