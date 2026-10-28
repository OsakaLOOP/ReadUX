"""使用正式分析、GiNZA 和词典查询目标生成指南静态快照。"""
from __future__ import annotations

import json
import argparse
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PARTICLES = ROOT / "configs/grammar/particles.json"
SERVICE = ROOT / "target/debug/readux-nlp.exe"


def request(process, command, **payload):
    process.stdin.write(json.dumps({"command": command, **payload}, ensure_ascii=False) + "\n")
    process.stdin.flush()
    line = process.stdout.readline()
    if not line:
        raise RuntimeError("readux-nlp 服务未返回结果")
    response = json.loads(line)
    if response.get("error"):
        raise RuntimeError(response["error"])
    return response["result"]


def main():
    parser = argparse.ArgumentParser(description="生成课节分析和词典静态资源。")
    parser.add_argument("--lesson", choices=("1.1", "2.1", "stage11"), default="1.1")
    parser.add_argument("--service", type=Path, default=SERVICE, help="正式分析服务的可执行文件路径")
    parser.add_argument("--group", help="只更新指定例句组的快照")
    args = parser.parse_args()
    lesson = args.lesson
    stem = f"lesson-{lesson.replace('.', '-')}.json"
    package_path = ROOT / "src/grammar-guide" / stem
    output_path = package_path.with_name(package_path.stem + "-analysis.json")
    dictionary_path = package_path.with_name(package_path.stem + "-dictionary.json")
    package = json.loads(package_path.read_text(encoding="utf-8"))
    particles = json.loads(PARTICLES.read_text(encoding="utf-8"))["entries"]
    particle_ids = {entry["headword"]: entry["id"] for entry in particles}
    examples = package["examples"] + package["originalExamples"]
    if args.group:
        examples = [example for example in examples if example["group"] == args.group]
        if not examples:
            raise ValueError(f"例句组不存在：{args.group}")
    connection_cases = {
        case["text"]: case for case in json.loads((ROOT / "data/validation/morphology_stage11_cases.json").read_text(encoding="utf-8"))
    } if lesson == "stage11" else {}
    output, dictionary = {}, {}
    if args.group:
        if output_path.exists():
            output = json.loads(output_path.read_text(encoding="utf-8"))["examples"]
        if dictionary_path.exists():
            dictionary = json.loads(dictionary_path.read_text(encoding="utf-8"))["entries"]
    process = subprocess.Popen(
        [str(args.service.resolve()), "stdio"], cwd=ROOT, stdin=subprocess.PIPE,
        stdout=subprocess.PIPE, text=True, encoding="utf-8", bufsize=1,
    )
    try:
        for example in examples:
            base = request(process, "analyze", text=example["ja"], register="cwj")
            enriched = request(process, "enrich", analysis_id=base["id"])
            ginza = next((p for p in enriched["providers"] if p.get("id") == "ginza" or p.get("provider") == "ginza"), None)
            # 正常配置的外部来源必须成功，缺失时停止生成并保留已保存版本。
            if not enriched["providers"] or any(p["status"] != "ready" for p in enriched["providers"]):
                raise RuntimeError(f'{example["id"]}：GiNZA 来源未就绪：{enriched["providers"]}')
            document = enriched["document"]
            if case := connection_cases.get(example["ja"]):
                operators = [op for chain in document["morphology"]["chains"] for op in chain["operators"] if op["kind"] == case["kind"]]
                if len(operators) != 1 or operators[0].get("connection", {}).get("matched_paths") != [case["path"]]:
                    raise RuntimeError(f'{example["id"]}：接续分支与验收清单不一致')
            if document["text"] != example["ja"]:
                raise RuntimeError(f'{example["id"]}：分析文字与例句不一致')
            group = request(process, "lookup_targets", analysis_id=document["id"], range=[0, len(document["text"])])
            lookups, particle_refs = {}, {}
            for target in group["outer_targets"] + group["inner_targets"]:
                members = [m for m in document["morphemes"] if m["id"] in target["morpheme_ids"]]
                particle = next((m for m in members if m["pos"][0] == "助詞" and m["char_range"] == target["char_range"]), None)
                if particle:
                    surface = particle["surface"]
                    canonical = "て" if surface == "で" and particle["pos"][1] == "接続助詞" else surface
                    if canonical in particle_ids:
                        particle_refs[target["id"]] = particle_ids[canonical]
                        continue
                if target["decision"] == "grammar":
                    continue
                result = request(process, "query_target", analysis_id=document["id"], target_id=target["id"])
                snapshots = {}
                for form in [None] + [f["form_id"] for f in result.get("forms", []) if f["form_id"] != result.get("selected_form_id")]:
                    lookup = result if form is None else request(process, "query_target", analysis_id=document["id"], target_id=target["id"], selected_form=form)
                    keys = []
                    for entry in lookup["entries"]:
                        # 查询条目正文使用正式服务，保留同一词条中的各个分条。
                        details = request(process, "query_entry", entry_key=entry["entry_key"]) if not entry.get("content_loaded") else [entry]
                        detail = next((d for d in details if d.get("occurrence_id") == entry.get("occurrence_id")), entry) if isinstance(details, list) else entry
                        saved = {**entry, **detail}
                        key = saved.get("occurrence_id") or saved["entry_key"]
                        dictionary[key] = saved
                        keys.append(key)
                    lookup["entries"] = []
                    snapshots[form or "default"] = {"result": lookup, "entryKeys": keys}
                lookups[target["id"]] = snapshots
            ruby = [
                {"char_range": m["char_range"], "base": m["surface"], "reading": m["reading"]}
                for m in document["morphemes"] if m.get("reading") and re.search(r"[一-龯々〆ヵヶ]", m["surface"])
            ]
            output[example["id"]] = {
                "document": document, "group": group, "ruby": ruby,
                "lookups": lookups, "particleRefs": particle_refs,
                "providers": enriched["providers"],
            }
            print(f'{example["id"]}：{len(group["outer_targets"])} 个外层目标，{len(group["inner_targets"])} 个内部目标', flush=True)
    finally:
        process.kill()
        process.wait()
    output_path.write_text(json.dumps({"schema": "readux.grammar-guide-analysis.v2", "source": package_path.name, "examples": output}, ensure_ascii=False, separators=(",", ":")) + "\n", encoding="utf-8")
    dictionary_path.write_text(json.dumps({"schema": "readux.grammar-guide-dictionary.v1", "entries": dictionary}, ensure_ascii=False, separators=(",", ":")) + "\n", encoding="utf-8")
    print(f"完成 {len(output)} 条例句、{len(dictionary)} 个词典分条")


if __name__ == "__main__":
    main()
