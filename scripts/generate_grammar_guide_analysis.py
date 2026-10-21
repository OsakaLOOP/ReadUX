"""使用现有 readux-nlp 服务生成语法指南的静态阅读器分析快照。"""
from __future__ import annotations

import json
import re
import subprocess
from copy import deepcopy
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKAGE = ROOT / "src" / "grammar-guide" / "lesson-1-1.json"
OUTPUT = ROOT / "src" / "grammar-guide" / "lesson-1-1-analysis.json"
DICTIONARY_OUTPUT = ROOT / "src" / "grammar-guide" / "lesson-1-1-dictionary.json"
SERVICE = ROOT / "target" / "debug" / "readux-nlp.exe"


def request(process: subprocess.Popen[str], payload: dict) -> dict:
    assert process.stdin is not None and process.stdout is not None
    process.stdin.write(json.dumps(payload, ensure_ascii=False) + "\n")
    process.stdin.flush()
    line = process.stdout.readline()
    if not line:
        raise RuntimeError("readux-nlp 服务未返回结果")
    value = json.loads(line)
    if value.get("error"):
        raise RuntimeError(value["error"])
    return value.get("result") or {}


def pos_of(morpheme: dict) -> dict[str, str]:
    values = [value or "*" for value in morpheme.get("pos", [])]
    values.extend(["*"] * (4 - len(values)))
    return {"major": values[0], "sub1": values[1], "sub2": values[2], "sub3": values[3]}


def teaching_class(pos: dict[str, str]) -> str:
    if pos["major"] == "助詞":
        return "particle"
    if pos["major"] in {"助動詞", "接尾辞"}:
        return "functional"
    if pos["major"] in {"名詞", "動詞", "形容詞", "副詞", "連体詞", "感動詞", "接続詞"}:
        return "content"
    return "other"


def teaching_label(pos: dict[str, str]) -> str:
    labels = {
        "格助詞": "格助词",
        "係助詞": "係助词",
        "副助詞": "副助词",
        "接続助詞": "接续助词",
        "終助詞": "终助词",
        "準体助詞": "准体助词",
        "並立助詞": "并列助词",
        "間投助詞": "间投助词",
        "助動詞": "助动词",
    }
    return labels.get(pos["sub1"], labels.get(pos["major"], pos["major"]))


def grammar_tag(example_id: str, morpheme: dict, occurrence: dict, index: int) -> dict:
    labels = occurrence.get("labels") or []
    kind = next((label for label in labels if label in {
        "morphology_feature", "functional_morpheme", "grammar_construction",
        "bunsetsu_function", "correlative_grammar",
    }), "functional_morpheme")
    surface = morpheme.get("surface", "")
    description = ""
    return {
        "pattern_id": occurrence.get("id", f"guide-{example_id}-{index}"),
        "name_ja": surface,
        "name_en": surface,
        "jlpt_level": None,
        "description": description,
        "morpheme_range": [0, 1],
        "char_range": occurrence.get("char_range", morpheme.get("char_range")),
        "occurrence_id": occurrence.get("id", f"guide-{example_id}-{index}"),
        "concept_id": occurrence.get("concept_id") or occurrence.get("id", f"guide-{example_id}-{index}"),
        "occurrence_kind": kind,
        "status": "accepted" if occurrence.get("status") == "accepted" else "pending",
        "show_badge": True,
        "display_ranges": [occurrence.get("char_range", morpheme.get("char_range"))],
        "selected_sense_id": None,
        "sense_candidates": [],
        "explanation": None,
        "teaching_class": "particle" if pos_of(morpheme)["major"] == "助詞" else "functional",
        "teaching_label": teaching_label(pos_of(morpheme)),
        "source_pos": pos_of(morpheme)["major"],
        "learning_note": None,
    }


def lookup_snapshot(result: dict, word: str, observed: str, reading: str, pos: dict) -> dict:
    entries = []
    for group in result.get("groups", []):
        entries.extend(group.get("entries", []))
    for entry in entries:
        evidence = entry.get("match_evidence") or {}
        entry.setdefault("matched_form", evidence.get("matched_form") or entry.get("headword", observed))
        entry.setdefault("readings", [entry.get("reading")] if entry.get("reading") else [])
    return {
        "query": word,
        "observed_form": observed,
        "reading": reading or None,
        "pos": pos,
        "mode": result.get("mode", "contextual"),
        "forms": result.get("forms", []),
        "dictionary_names": result.get("dictionary_names", []),
        "entries": entries,
        "timing": result.get("timing"),
    }


def group_bunsetsu(tokens: list[dict], analysis: dict) -> list[dict]:
    spans = analysis.get("bunsetsu", {}).get("nodes", [])
    if spans:
        groups = []
        for span in spans:
            group = [i for i, token in enumerate(tokens) if token["bunsetsu"]["char_range"][0] >= span["char_range"][0] and token["bunsetsu"]["char_range"][1] <= span["char_range"][1]]
            if group:
                groups.append(group)
        covered = {i for group in groups for i in group}
        groups.extend([[i] for i in range(len(tokens)) if i not in covered])
        groups.sort(key=lambda group: group[0])
    else:
        groups = []
        current = []
        has_function = False
        for index, token in enumerate(tokens):
            pos = token["bunsetsu"]["morphemes"][0]["pos"]["major"]
            if token["display_class"] == "punctuation":
                if current:
                    groups.append(current)
                    current = []
                    has_function = False
                groups.append([index])
                continue
            is_lexical = pos in {"名詞", "動詞", "形容詞", "副詞", "連体詞", "感動詞", "接続詞"}
            if current and has_function and is_lexical:
                groups.append(current)
                current = []
                has_function = False
            current.append(index)
            has_function = has_function or pos in {"助詞", "助動詞"}
        if current:
            groups.append(current)

    combined = []
    for indices in groups:
        if len(indices) == 1:
            combined.append(tokens[indices[0]])
            continue
        members = [tokens[index] for index in indices]
        merged = deepcopy(members[0])
        bunsetsu = merged["bunsetsu"]
        morphemes = []
        tags = []
        chains = []
        lexical_units = []
        formations = []
        expressions = []
        for local_index, token in enumerate(members):
            source = token["bunsetsu"]
            morphemes.extend(source["morphemes"])
            for tag in source["grammar_tags"]:
                tag["morpheme_range"] = [local_index, local_index + 1]
                tags.append(tag)
            for chain in source["morphology"]["chains"]:
                chain["anchor_morpheme"] = local_index
                chain["anchor_range"] = [local_index, local_index + 1]
                chain["morpheme_range"] = [local_index, local_index + 1]
                chain["morpheme_indices"] = [local_index]
                chain["core_morpheme_indices"] = [local_index]
                for operator in chain.get("operators", []):
                    operator["source_morpheme_range"] = [local_index, local_index + 1]
                chains.append(chain)
            for lexical in source["lexical_units"]:
                lexical["morpheme_range"] = [local_index, local_index + 1]
                lexical["head_morpheme"] = local_index
                lexical_units.append(lexical)
            formations.extend(source["word_formations"])
            expressions.extend(token["expressions"])
        bunsetsu["morphemes"] = morphemes
        bunsetsu["surface"] = "".join(m["surface"] for m in morphemes)
        bunsetsu["char_range"] = [morphemes[0]["char_range"][0], morphemes[-1]["char_range"][1]]
        head = next((unit for unit in reversed(lexical_units)), None)
        head_morpheme = morphemes[head["head_morpheme"]] if head else morphemes[0]
        bunsetsu["head_word"] = {"surface": head["surface"] if head else head_morpheme["surface"], "base_form": head["base_form"] if head else head_morpheme["base_form"], "reading": head["reading"] if head else head_morpheme["reading"], "pos": head["output_pos"] if head else head_morpheme["pos"]}
        bunsetsu["grammar_tags"] = tags
        bunsetsu["morphology"] = {"chains": chains}
        bunsetsu["word_formations"] = formations
        bunsetsu["lexical_units"] = lexical_units
        merged["expressions"] = expressions
        combined.append(merged)
    return combined


def build_example(example: dict, analysis: dict, searches: dict[str, dict]) -> dict:
    morphemes = analysis.get("morphemes", [])
    chains = analysis.get("morphology", {}).get("chains", [])
    occurrences = analysis.get("grammar", {}).get("occurrences", [])
    occurrence_by_index = {
        index: occurrence
        for occurrence in occurrences
        for index in occurrence.get("morpheme_indices", [])
    }
    tokens = []
    parts = []
    details = {}
    ruby = []
    lookups = []
    for index, morpheme in enumerate(morphemes):
        char_range = morpheme["char_range"]
        surface = morpheme.get("surface", "")
        pos = pos_of(morpheme)
        teaching = teaching_class(pos)
        detail_id = f"{example['id']}-detail-{index}"
        part_id = f"{example['id']}-part-{index}"
        label = pos["major"] if pos["major"] != "*" else teaching
        parts.append({"id": part_id, "range": char_range, "text": surface, "teachingClass": teaching, "classLabel": label, "term": label, "focus": False, "detailId": detail_id})
        details[detail_id] = {"id": detail_id, "label": surface, "sourceLabel": "UniDic 分词", "teachingClass": teaching, "note": label, "dictionaryEntryIds": []}
        if re.search(r"[一-龯々〆ヵヶ]", surface) and morpheme.get("reading"):
            ruby.append({"range": char_range, "base": surface, "reading": morpheme["reading"]})
        grammar = []
        occurrence = occurrence_by_index.get(index)
        if occurrence:
            grammar = [grammar_tag(example["id"], morpheme, occurrence, index)]
        elif teaching in {"particle", "functional"}:
            grammar = [grammar_tag(example["id"], morpheme, {"id": f"guide-{example['id']}-{index}", "labels": ["functional_morpheme", f"pos1:{pos['major']}", f"pos2:{pos['sub1']}"], "char_range": char_range, "status": "candidate"}, index)]
        chain = next((item for item in chains if index in item.get("morpheme_indices", [])), None)
        lexical = []
        if teaching == "content":
            lexical.append({"surface": surface, "base_form": morpheme.get("lemma") or surface, "reading": morpheme.get("reading", ""), "output_pos": pos, "morpheme_range": [0, 1], "char_range": char_range, "head_morpheme": 0, "lexical_shape": "unidic", "dictionary_refs": [], "reading_candidates": [morpheme.get("reading", "")], "confidence": 1, "evidence": ["provider:unidic"]})
        bunsetsu = {"morphemes": [{"surface": surface, "pos": pos, "base_form": morpheme.get("lemma") or surface, "reading": morpheme.get("reading", ""), "conjugation_type": (chain or {}).get("final_state", {}).get("conjugation_type", "*"), "conjugation_form": (chain or {}).get("final_state", {}).get("conjugation_form", "*"), "char_range": char_range}], "surface": surface, "head_word": {"surface": surface, "base_form": morpheme.get("lemma") or surface, "reading": morpheme.get("reading", ""), "pos": pos}, "grammar_tags": grammar, "morphology": {"chains": [chain] if chain else []}, "word_formations": [], "lexical_units": lexical, "char_range": char_range, "function": None}
        tokens.append({"bunsetsu": bunsetsu, "novelty_score": 0, "is_selected": False, "is_known": True, "inference_reason": "guide-static-analysis", "expressions": [], "display_class": "punctuation" if re.fullmatch(r"[。！？、，．,.!?]", surface) else "content"})
        query = (chain or {}).get("lookup_form") or morpheme.get("lemma") or surface
        search = searches.get(query)
        if search:
            lookups.append({"request": {"word": query, "observedForm": surface, "reading": morpheme.get("reading", ""), "pos": pos}, "result": lookup_snapshot(search, query, surface, morpheme.get("reading", ""), pos)})
    example["parts"] = parts
    example["ruby"] = ruby
    example["tokens"] = group_bunsetsu(tokens, analysis)
    example["lookups"] = lookups
    example["capsules"] = [{"id": f"{example['id']}-whole", "range": [0, len(example["ja"])], "partIds": [part["id"] for part in parts]}]
    example["targets"] = [{"id": part["id"], "range": part["range"], "detailId": part["detailId"], "defaultEntryId": None, "entryIds": [], "childTargetIds": []} for part in parts]
    example["details"] = details
    example["detail"] = {"id": f"{example['id']}-detail", "label": "静态分析例句", "sourceLabel": "UniDic / 语法资源", "teachingClass": "整句", "note": "", "morphology": [], "dictionaryEntryIds": []}
    example["dictionaryStatus"] = "available" if lookups else "no-record"
    example["morphology"] = [{"form": chain.get("surface_form", ""), "label": ", ".join(operator.get("label", "") for operator in chain.get("operators", [])), "note": ""} for chain in chains]
    return example


def main() -> None:
    if not SERVICE.exists():
        raise SystemExit(f"缺少分析服务：{SERVICE}")
    package = json.loads(PACKAGE.read_text(encoding="utf-8"))
    examples = package["examples"] + package["originalExamples"]
    process = subprocess.Popen([str(SERVICE), "stdio"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, encoding="utf-8", bufsize=1)
    try:
        analyses = {example["id"]: request(process, {"command": "analyze", "text": example["ja"], "register": "cwj"}) for example in examples}
        words = sorted({(chain.get("lookup_form") or morpheme.get("lemma") or morpheme.get("surface")) for result in analyses.values() for chain in result.get("morphology", {}).get("chains", []) for morpheme in result.get("morphemes", []) if chain.get("morpheme_indices") and morpheme.get("surface")})
        searches = {word: request(process, {"command": "search", "word": word}) for word in words if word}
        for search in searches.values():
            for group in search.get("groups", []):
                detailed_entries = []
                for entry in group.get("entries", []):
                    detailed = request(process, {"command": "query_entry", "entry_key": entry.get("entry_key", "")})
                    detailed_entries.append(detailed[0] if isinstance(detailed, list) and detailed else entry)
                group["entries"] = detailed_entries
    finally:
        process.kill()
    output = {}
    for example in examples:
        generated = build_example(example, analyses[example["id"]], searches)
        output[example["id"]] = {
            "ruby": generated.get("ruby", []),
            "tokens": generated["tokens"],
            "lookups": generated["lookups"],
        }
    dictionary_entries = {}
    dictionary_refs = {}
    for value in output.values():
        for lookup in value["lookups"]:
            result = lookup.get("result") or {}
            keys = []
            dictionary_id = dictionary_refs.get(lookup["request"]["word"])
            if dictionary_id is None:
                dictionary_id = f"d{len(dictionary_refs) + 1:04d}"
                dictionary_refs[lookup["request"]["word"]] = dictionary_id
            entries_for_query = {}
            for entry in result.get("entries", []):
                key = entry.get("entry_key")
                if key:
                    entries_for_query[key] = entry
                    dictionary_entries[key] = entry
                    keys.append(key)
            lookup["entryKeys"] = keys
            lookup["dictionaryId"] = dictionary_id
            result["entries"] = []
    OUTPUT.write_text(json.dumps({"schema": "readux.grammar-guide-analysis.v1", "source": str(PACKAGE.name), "examples": output, "dictionaryRefs": dictionary_refs}, ensure_ascii=False, separators=(",", ":")) + "\n", encoding="utf-8")
    DICTIONARY_OUTPUT.write_text(json.dumps({"schema": "readux.grammar-guide-dictionary.v1", "entries": dictionary_entries}, ensure_ascii=False, separators=(",", ":")) + "\n", encoding="utf-8")
    if not dictionary_entries:
        raise RuntimeError("没有生成任何词典记录")
    print(f"生成 {len(output)} 条例句分析、{len(searches)} 个词典快照：{OUTPUT}")


if __name__ == "__main__":
    main()
