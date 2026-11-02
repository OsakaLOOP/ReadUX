"""单次扫描 UniDic 副词词条，并合并、筛查接续资格来源。"""

import argparse
from collections import defaultdict
import csv
import json
from pathlib import Path

SCHEMA = "readux.adverb-capabilities.v1"
FLAGS = {"none": 1, "suru": 2, "na_da": 4, "to_suru": 8, "tari": 16}

# 首批人工合成的正式映射。词元身份仍以 UniDic catalog 为准核对。
SYNTHESIZED = {
    "none": ["26503", "11140", "36084", "24363", "19286", "20750", "4983", "258", "19134", "6912", "4531", "35816", "15670", "20169", "22656"],
    "suru": ["15478", "29839", "19244", "38854", "35264", "8089", "8678", "10237", "27193", "33232", "14255", "38855", "12803", "15522", "17558"],
    "na_da": ["200", "1031", "1081", "1614", "2617", "6200", "6232", "6701", "6913", "7228", "8055", "8079", "8152", "8163", "9957", "10126"],
    "tari": ["23303", "27141", "52653", "137328"],
}


def read(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def write(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")


def flag_bits(flags):
    if not isinstance(flags, list) or not flags or len(flags) != len(set(flags)):
        raise ValueError("资格应为非空、无重复的标志数组")
    if set(flags) - FLAGS.keys() or "none" in flags and len(flags) != 1:
        raise ValueError(f"资格无效或 none 与正向资格冲突：{flags}")
    return sum(FLAGS[flag] for flag in flags)


def candidate_flags(positions):
    result = set()
    for pos in positions:
        if pos[:2] == ("形状詞", "タリ"):
            result.update(("tari", "to_suru"))
        elif pos[0] == "形状詞" or pos[0] == "名詞" and "形状詞" in pos[2]:
            result.add("na_da")
        if pos[0] == "名詞" and "サ変" in pos[2]:
            result.add("suru")
    return sorted(result)


def export_lexicon(path):
    """按词元聚合，完整 CSV 仅顺序读取一次。"""
    records = {}
    with Path(path).open(encoding="utf-8", newline="") as stream:
        for row in csv.reader(stream):
            if len(row) != 33:
                raise ValueError("UniDic lex.csv 应含 4 个词典字段和 29 个词法字段")
            if row[4] not in {"副詞", "形状詞", "名詞"}:
                continue
            key = row[32]
            entry = records.setdefault(key, {"lemma_id": key, "lemma": row[11], "reading": row[10],
                                              "spellings": set(), "pos": set()})
            if (entry["lemma"], entry["reading"]) != (row[11], row[10]):
                raise ValueError(f"同词元 ID 的词元或读法不一致：{key}")
            entry["spellings"].add(row[0])
            entry["pos"].add(tuple(row[4:8]))
    peers = defaultdict(set)
    for key, entry in records.items():
        for spelling in entry["spellings"]:
            peers[(spelling, entry["reading"])].add(key)
    entries = []
    for key, entry in sorted(records.items(), key=lambda item: int(item[0])):
        if not any(pos[0] == "副詞" for pos in entry["pos"]):
            continue
        related = set().union(*(peers[(form, entry["reading"])] for form in entry["spellings"])) - {key}
        related_ids = sorted(related, key=int)
        entries.append({**entry, "spellings": sorted(entry["spellings"]),
                        "pos": [list(pos) for pos in sorted(entry["pos"])],
                        "candidate_flags": candidate_flags(entry["pos"]),
                        "related_lemma_ids": related_ids,
                        "related_lexemes": [{"lemma_id": peer, "lemma": records[peer]["lemma"],
                            "reading": records[peer]["reading"],
                            "pos": [list(pos) for pos in sorted(records[peer]["pos"])],
                            "candidate_flags": candidate_flags(records[peer]["pos"])} for peer in related_ids]})
    return {"schema": "readux.adverb-lexicon.v1", "unidic": "2025.12", "entries": entries}


def merge_sources(catalog, bundles):
    """候选与冲突保留待审，一致的审定结果可以发布。"""
    lexemes = {entry["lemma_id"]: entry for entry in catalog["entries"]}
    proposals = defaultdict(list)
    seen_sources = set()
    for bundle in bundles:
        if bundle.get("schema") != SCHEMA:
            raise ValueError("副词来源协议无效")
        for source in bundle["sources"]:
            name = source["id"]
            if not name or name in seen_sources:
                raise ValueError(f"来源名称为空或重复：{name}")
            seen_sources.add(name)
            seen = set()
            for entry in source["entries"]:
                key = entry["lemma_id"]
                if not isinstance(key, str) or key not in lexemes or key in seen:
                    raise ValueError(f"来源 {name} 的词元 ID 未登记或重复：{key}")
                seen.add(key)
                known = lexemes[key]
                if (entry["lemma"], entry["reading"]) != (known["lemma"], known["reading"]):
                    raise ValueError(f"来源 {name} 的词元身份不符：{key}")
                flag_bits(entry["flags"])
                if entry["status"] not in {"candidate", "reviewed"} or not entry.get("reason"):
                    raise ValueError(f"来源 {name} 缺少审定状态或理由：{key}")
                if entry["status"] == "reviewed" and not entry.get("examples"):
                    raise ValueError(f"审定记录缺少诊断例句：{key}")
                proposals[key].append({**entry, "source": name})
    entries, reviewed = [], []
    for key, lexeme in lexemes.items():
        rows = proposals[key]
        approved = [row for row in rows if row["status"] == "reviewed"]
        masks = {flag_bits(row["flags"]) for row in approved}
        status = "conflict" if len(masks) > 1 else "reviewed" if masks else "unreviewed"
        entries.append({"lemma_id": key, "lemma": lexeme["lemma"], "reading": lexeme["reading"],
                        "status": status, "proposals": rows, "candidate_flags": lexeme["candidate_flags"]})
        if status == "reviewed":
            reviewed.append({k: approved[0][k] for k in ("lemma_id", "lemma", "reading", "flags", "status", "reason", "examples")})
    return {"schema": "readux.adverb-review.v1", "entries": entries,
            "mapping": {"schema": SCHEMA, "sources": [{"id": "merged-review", "entries": reviewed}]}}


def compile_reviewed(bundle):
    """日常构建读取小型审定表，独立于完整 UniDic。"""
    if bundle.get("schema") != SCHEMA:
        raise ValueError("副词映射协议无效")
    identities = {}
    for source in bundle["sources"]:
        for entry in source["entries"]:
            key = entry["lemma_id"]
            if not isinstance(key, str) or not key.isdecimal():
                raise ValueError("副词词元 ID 应为十进制字符串")
            if entry["status"] != "reviewed" or not entry.get("reason") or not entry.get("examples"):
                raise ValueError(f"运行映射缺少审定证据：{key}")
            item = {"lemma": entry["lemma"], "reading": entry["reading"], "flags": flag_bits(entry["flags"])}
            if not item["lemma"] or not item["reading"]:
                raise ValueError(f"词元身份为空：{key}")
            if key in identities and identities[key] != item:
                raise ValueError(f"审定来源冲突：{key}")
            identities[key] = item
    return dict(sorted(identities.items(), key=lambda item: int(item[0])))


def synthesize_reviewed(catalog):
    """依据已审定例词合成首批正式映射，并严格核对 UniDic 身份。"""
    lexemes = {entry["lemma_id"]: entry for entry in catalog["entries"]}
    used = set()
    entries = []
    for flag, ids in SYNTHESIZED.items():
        for key in ids:
            if key in used or key not in lexemes:
                raise ValueError(f"首批副词映射 ID 无效或重复：{key}")
            used.add(key)
            known = lexemes[key]
            entries.append({
                "lemma_id": key,
                "lemma": known["lemma"],
                "reading": known["reading"],
                "flags": ["tari", "to_suru"] if flag == "tari" else [flag],
                "status": "reviewed",
                "reason": f"UniDic 副词词元的人工诊断分类：{flag}",
                "examples": [{"text": f"{known['lemma']}的典型{flag}接续", "source": "synthesized-diagnostic"}],
            })
    if len(entries) != 50:
        raise ValueError(f"首批正式映射应为 50 条：{len(entries)}")
    return {"schema": SCHEMA, "sources": [{"id": "synthesized-diagnostic-50", "entries": entries}]}


def fill_unidic_multi(catalog):
    """填充 UniDic 同词元资格与同形同音多词元关联来源。"""
    entries = []
    for item in catalog["entries"]:
        if not item["candidate_flags"] and not item["related_lemma_ids"]:
            continue
        entries.append({
            "lemma_id": item["lemma_id"],
            "lemma": item["lemma"],
            "reading": item["reading"],
            "same_lemma": {"candidate_flags": item["candidate_flags"], "pos": item["pos"]},
            "homophone_lexemes": item["related_lexemes"],
        })
    return {"schema": "readux.adverb-unidic-multi.v1", "unidic": catalog["unidic"], "entries": entries}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    export = commands.add_parser("export", help="单次扫描并导出副词词条表")
    export.add_argument("--lexicon", type=Path, required=True)
    export.add_argument("--output", type=Path, required=True)
    merge = commands.add_parser("merge", help="合并来源并筛查冲突与待审条目")
    merge.add_argument("--catalog", type=Path, required=True)
    merge.add_argument("--input", type=Path, action="append", default=[])
    merge.add_argument("--output", type=Path, required=True)
    merge.add_argument("--mapping-output", type=Path, help="单独输出可编译的审定映射")
    synthesize = commands.add_parser("synthesize", help="生成首批 50 条正式映射")
    synthesize.add_argument("--catalog", type=Path, required=True)
    synthesize.add_argument("--output", type=Path, required=True)
    fill = commands.add_parser("fill-unidic", help="填充 UniDic 多词条来源")
    fill.add_argument("--catalog", type=Path, required=True)
    fill.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.command == "export":
        result = export_lexicon(args.lexicon)
    elif args.command == "merge":
        result = merge_sources(read(args.catalog), [read(path) for path in args.input])
    elif args.command == "synthesize":
        result = synthesize_reviewed(read(args.catalog))
    else:
        result = fill_unidic_multi(read(args.catalog))
    write(args.output, result)
    if args.command == "merge" and args.mapping_output:
        write(args.mapping_output, result["mapping"])
    count = len(result["entries"]) if "entries" in result else sum(len(source["entries"]) for source in result["sources"])
    print(f"已生成 {args.output}，共 {count} 个词元")


if __name__ == "__main__":
    main()
