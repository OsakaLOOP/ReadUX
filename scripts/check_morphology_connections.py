"""检查活用词条的分类、接续引用和显示内容完整性。"""

import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"JSON 字段重复：{key}")
        result[key] = value
    return result


def read(path):
    return json.loads((ROOT / path).read_text(encoding="utf-8"), object_pairs_hook=unique_object)


def check(config=None, *, report=True):
    if config is None:
        config = read("configs/unidic-nlp/morphology.json")
    operators = config["operators"]
    rules = config["connection_rules"]
    explanations = {entry["concept_id"]: entry for entry in read(
        "crates/readux-core/resources/grammar/compiled/grammar_explanations.json"
    )["explanations"]}
    builtins = {entry["headword"]: entry for entry in read("configs/grammar/auxiliaries.json")["entries"]}
    classified = Counter(kind for kinds in config["connection_types"].values() for kind in kinds)
    if set(classified) != set(operators) or any(count != 1 for count in classified.values()):
        raise ValueError("条目分类存在遗漏、重复或未定义的成员")
    if set(config["rule_metadata"]) != set(operators):
        raise ValueError("条目与执行元数据的成员不一致")
    owners = {}
    valid_categories = {"動詞", "形容詞", "形状詞", "名詞", "代名詞", "助動詞", "文語用言"}
    valid_forms = {"stem", "irrealis", "continuative", "terminal", "attributive",
                   "conditional", "imperative", "volitional", "te", "other"}
    for name, rule in rules.items():
        if not rule["label"] or not rule["paths"] or not rule["operators"]:
            raise ValueError(f"{name} 缺少规则内容")
        ids = [path["id"] for path in rule["paths"]]
        if len(ids) != len(set(ids)):
            raise ValueError(f"{name} 存在重复路径")
        for kind in rule["operators"]:
            if kind not in operators or kind in owners:
                raise ValueError(f"{kind} 的接续规则归属重复或未定义")
            owners[kind] = name
        for path in rule["paths"]:
            if any(not path.get(field) for field in ("id", "group", "label", "steps")):
                raise ValueError(f"{name} 的路径字段不完整")
            if not set(path.get("categories", [])).issubset(valid_categories):
                raise ValueError(f"{name}/{path['id']} 使用了未定义的词类")
            if not set(path.get("forms", [])).issubset(valid_forms):
                raise ValueError(f"{name}/{path['id']} 使用了未定义的活用形")
            for field in ("previous_operators", "exclude_previous_operators"):
                if not set(path.get(field, [])).issubset(operators):
                    raise ValueError(f"{name}/{path['id']} 的前项引用无效")
            if connector := path.get("connector"):
                shared = next((entry for entry in rules.values() if connector in entry["operators"]), None)
                if not shared or not path.get("connector_paths") or not set(path["connector_paths"]).issubset(
                    entry["id"] for entry in shared["paths"]
                ):
                    raise ValueError(f"{name}/{path['id']} 的连接器路径引用无效")
        if rule.get("source") == "classical_auxiliaries":
            entry = next(item for item in config["classical_auxiliaries"] if item["rule"] in rule["operators"])
            expected = set()
            for condition in entry["inputs"]:
                for category in condition["categories"]:
                    expected.add((category, tuple(condition["forms"]), tuple(condition.get("ctype", [])),
                                  tuple(condition.get("exclude_ctype", []))))
            actual = {(category, tuple(path["forms"]), tuple(path.get("ctypes", [])),
                       tuple(path.get("exclude_ctypes", [])))
                      for path in rule["paths"] for category in path["categories"]}
            if actual != expected:
                raise ValueError(f"{name} 的路径与文言输入声明不一致")
        if rule.get("source") == "predicate_links":
            link = next(item for item in config["predicate_links"] if item["kind"] in rule["operators"])
            prefix = next(item for item in rules.values() if link["prefix"] in item["operators"])
            if rule["paths"] != prefix["paths"]:
                raise ValueError(f"{name} 的路径与构式前项声明不一致")
    def resolved_rules(kind, visiting=()):
        if kind in visiting:
            raise ValueError(f"{kind} 存在循环连接器引用")
        entry = operators[kind]
        resolved = [owners[kind]] if kind in owners else []
        connectors = ([entry["connector"]] if entry.get("connector") else []) + entry.get("connector_alternatives", [])
        if kind in owners:
            connectors += [path["connector"] for path in rules[owners[kind]]["paths"] if path.get("connector")]
        for connector in connectors:
            if connector not in operators:
                raise ValueError(f"{kind} 引用了未定义的连接器")
            resolved.extend(resolved_rules(connector, (*visiting, kind)))
        if not resolved:
            raise ValueError(f"{kind} 缺少直接路径或连接器引用")
        return resolved
    for kind, entry in operators.items():
        resolved_rules(kind)
        if not entry.get("title") or not entry.get("concept"):
            raise ValueError(f"{kind} 缺少词条名称或概念")
        support = entry.get("support", {})
        headword = support.get("expanded", [None])[0]
        if not headword:
            prototype = entry["prototype"]
            headword = prototype[1:] if support.get("group") in ("te", "contracted") else (
                support["bases"][0] if support.get("group") in ("enumerative", "volitional", "nominal_change") else prototype)
        concepts = [entry["concept"], *(variant["concept"] for variant in entry.get("variants", []) if variant.get("concept"))]
        for concept in concepts:
            explanation = explanations.get(concept)
            if not explanation or not explanation.get("function_summary"):
                raise ValueError(f"{kind} 缺少有效正文：{concept}")
            examples = [block for block in explanation.get("body_blocks", []) if block["kind"] == "example_pair"]
            fallback = builtins.get(headword, {}).get("senses", [{}])[0].get("examples", [])
            if not (examples or fallback or explanation.get("examples")):
                raise ValueError(f"{kind} 缺少例句：{concept}")
    for case in read("data/validation/morphology_stage11_cases.json"):
        if case["rule"] not in resolved_rules(case["kind"]):
            raise ValueError(f"{case['kind']} 的既有规则引用失效")
        if case["path"] not in {path["id"] for path in rules[case["rule"]]["paths"]}:
            raise ValueError(f"{case['kind']} 的既有路径引用失效")
    summary = {"词条": len(operators), "直接规则": len(owners),
                      "连接器引用条目": len(operators) - len(owners), "规则组": len(rules),
                      "路径": sum(len(rule["paths"]) for rule in rules.values()),
                      "分类": {name: len(kinds) for name, kinds in config["connection_types"].items()},
                      "结果": "完整性检查通过"}
    if report:
        print(json.dumps(summary, ensure_ascii=False, indent=2))
    return summary


if __name__ == "__main__":
    check()
