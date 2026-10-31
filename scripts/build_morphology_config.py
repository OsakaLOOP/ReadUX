"""按 v1 协议编译活用配置，源文件顺序与路径引用均显式声明。"""

import argparse
from copy import deepcopy
import json
from pathlib import Path

from check_morphology_connections import check, unique_object

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "configs/unidic-nlp/morphology/source"
OUTPUT = ROOT / "configs/unidic-nlp/morphology.json"
SCHEMA = "readux.morphology-config.v1"
OPERATOR_FILES = {
    "verb": {"verb_cycle", "verb_cycle+i_adjective_conversion"},
    "conversion": {"i_adjective_conversion", "nominalization"},
    "judgement": {"na_adjective_judgement"},
    "auxiliary": {"auxiliary_verb"},
    "classical": {"classical_conjugation"},
    "insertion": {"insertion"},
}
CONNECTION_TYPES = ("direct", "connector", "sequence", "predicate", "inflection", "insertion")
CATEGORIES = {"動詞", "形容詞", "形状詞", "名詞", "代名詞", "助動詞", "文語用言"}
FORMS = {"stem", "irrealis", "continuative", "terminal", "attributive",
         "conditional", "imperative", "volitional", "te", "other"}
TOKEN_FIELDS = {"pos", "sub", "detail", "ctype", "cform", "lemma", "base", "surface"}


def read_source(source, name):
    return json.loads((source / name).read_text(encoding="utf-8"), object_pairs_hook=unique_object)


def fields(value, required, optional, where):
    if not isinstance(value, dict):
        raise ValueError(f"{where} 应为对象")
    missing = set(required) - value.keys()
    unknown = value.keys() - set(required) - set(optional)
    if missing or unknown:
        raise ValueError(f"{where} 缺少字段 {sorted(missing)}，未知字段 {sorted(unknown)}")


def strings(value, where, allowed=None):
    if not isinstance(value, list) or any(not isinstance(item, str) for item in value):
        raise ValueError(f"{where} 应为字符串数组")
    if allowed is not None and not set(value).issubset(allowed):
        raise ValueError(f"{where} 使用未知值 {sorted(set(value) - allowed)}")


def pattern(value, where):
    fields(value, (), ("equals", "starts_with", "contains"), where)
    for mode, conditions in value.items():
        fields(conditions, (), TOKEN_FIELDS, f"{where}/{mode}")
        for key, alternatives in conditions.items():
            strings(alternatives, f"{where}/{mode}/{key}")


def ordered_objects(value, required, optional, where):
    if not isinstance(value, list):
        raise ValueError(f"{where} 应为有序数组")
    for index, item in enumerate(value):
        fields(item, required, optional, f"{where}/{index}")


def validate(config):
    """核对运行字段、执行枚举与来源引用，正文检查复用既有入口。"""
    if type(config["max_auxiliary_depth"]) is not int or config["max_auxiliary_depth"] < 1:
        raise ValueError("max_auxiliary_depth 应为正整数")
    strings(config["intrinsic_features"], "intrinsic_features")
    support_owners = {}
    for kind, definition in config["operators"].items():
        fields(definition, ("label", "title", "concept", "prototype"),
               ("variants", "token_lemma", "normalized", "candidates", "support",
                "compound_candidate", "connector", "connector_alternatives", "display_suffix"), kind)
        strings([definition[key] for key in ("label", "title", "concept", "prototype", "normalized",
                                             "connector", "display_suffix") if key in definition], kind)
        for variant in definition.get("variants", []):
            fields(variant, ("token", "form"), ("concept",), f"{kind}/variants")
            pattern(variant["token"], f"{kind}/variants/token")
        for key in ("token_lemma", "compound_candidate"):
            if key in definition and type(definition[key]) is not bool:
                raise ValueError(f"{kind}/{key} 应为布尔值")
        for key in ("candidates", "connector_alternatives"):
            strings(definition.get(key, []), f"{kind}/{key}")
        if support := definition.get("support"):
            fields(support, ("group", "bases"), ("expanded",), f"{kind}/support")
            strings(support["bases"], f"{kind}/support/bases")
            if not support["bases"]:
                raise ValueError(f"{kind}/support/bases 应包含本体词元")
            if "expanded" in support:
                strings(support["expanded"], f"{kind}/support/expanded")
                if len(support["expanded"]) != 2:
                    raise ValueError(f"{kind}/support/expanded 应包含词形与读法")
            for base in support["bases"]:
                key = (support["group"], base)
                if key in support_owners:
                    raise ValueError(f"补助词元 {key} 同时归属 {support_owners[key]} 和 {kind}")
                support_owners[key] = kind
        metadata = config["rule_metadata"][kind]
        fields(metadata, ("stage", "executor", "rule_group", "input_label", "input_category_mode",
                          "input_categories", "output_category", "terminal_mode", "order_rank"),
               ("input_forms", "exclude_input_ctypes", "output_form", "preserve_state"), f"{kind}/execution")
        strings([metadata[key] for key in ("stage", "executor", "rule_group", "input_label",
                                          "input_category_mode", "output_category", "terminal_mode")],
                f"{kind}/execution")
        if metadata["input_category_mode"] not in ("any", "exact", "judgement_compatible"):
            raise ValueError(f"{kind} 的输入匹配模式无效")
        strings(metadata["input_categories"], f"{kind}/input_categories", CATEGORIES)
        strings(metadata.get("input_forms", []), f"{kind}/input_forms", FORMS)
        strings(metadata.get("exclude_input_ctypes", []), f"{kind}/exclude_input_ctypes")
        if metadata["output_category"] not in CATEGORIES | {"用言", "保持前项"}:
            raise ValueError(f"{kind} 的输出类别无效")
        if metadata.get("output_form") is not None and metadata["output_form"] not in FORMS:
            raise ValueError(f"{kind} 的输出活用形无效")
        if type(metadata.get("preserve_state", False)) is not bool:
            raise ValueError(f"{kind}/preserve_state 应为布尔值")
        if metadata["terminal_mode"] not in ("continue", "sentence_or_pause", "pause_or_continue", "preserve"):
            raise ValueError(f"{kind} 的停机模式无效")
        if type(metadata["order_rank"]) is not int:
            raise ValueError(f"{kind}/order_rank 应为整数")

    ordered_objects(config["features"], ("kind", "token"), ("exclude",), "features")
    ordered_objects(config["forms"], ("prefixes", "form"), (), "forms")
    ordered_objects(config["connections"], ("action", "when"), ("kinds", "support_group"), "connections")
    ordered_objects(config["sequences"], ("kind", "tokens", "forms"),
                    ("head", "categories", "operators", "terminal_only", "exclude_following", "phenomena"), "sequences")
    ordered_objects(config["predicate_links"], ("prefix", "next", "kind", "required_operators"), (), "predicate_links")
    ordered_objects(config["insertion_positions"], ("id",),
                    ("connector", "support_group", "categories", "forms", "bases"), "insertion_positions")
    ordered_objects(config["input_recoveries"],
                    ("id", "previous", "next", "following", "lemma", "reading", "surface_reading", "ctype"), (), "input_recoveries")
    for rule in config["features"]:
        if rule["kind"] not in config["operators"]:
            raise ValueError(f"特征引用未定义的 operator：{rule['kind']}")
        pattern(rule["token"], "features/token")
        for item in rule.get("exclude", []):
            pattern(item, "features/exclude")
    for rule in config["forms"]:
        strings(rule["prefixes"], "forms/prefixes")
        if rule["form"] not in FORMS:
            raise ValueError(f"活用形映射无效：{rule['form']}")
    for rule in config["connections"]:
        if rule["action"] not in ("core", "attach", "support", "stop"):
            raise ValueError(f"组链动作无效：{rule['action']}")
        ordered_objects(rule["when"], (), ("next", "last", "following", "at_end", "forms", "categories",
                                           "operators", "suffixes", "functional", "auxiliary_relation", "sahen"),
                        "connections/when")
        for condition in rule["when"]:
            for key in ("next", "last", "following"):
                if key in condition:
                    pattern(condition[key], f"connections/when/{key}")
            strings(condition.get("operators", []), "connections/when/operators", config["operators"].keys())
            strings(condition.get("forms", []), "connections/when/forms", FORMS)
            strings(condition.get("categories", []), "connections/when/categories", CATEGORIES)
    for rule in config["sequences"]:
        if rule["kind"] not in config["operators"]:
            raise ValueError(f"序列引用未定义的 operator：{rule['kind']}")
        for item in rule["tokens"]:
            pattern(item, "sequences/tokens")
        head = rule.get("head", 0)
        if type(head) is not int or not 0 <= head < len(rule["tokens"]):
            raise ValueError(f"{rule['kind']} 的序列核心越界")
        strings(rule["forms"], "sequences/forms", FORMS)
        strings(rule.get("categories", []), "sequences/categories", CATEGORIES)
        strings(rule.get("operators", []), "sequences/operators", config["operators"].keys())
        if "exclude_following" in rule:
            pattern(rule["exclude_following"], "sequences/exclude_following")
    for rule in config["predicate_links"]:
        strings([rule["kind"], rule["prefix"], *rule["required_operators"]],
                "predicate_links/operators", config["operators"].keys())
        pattern(rule["next"], "predicate_links/next")
    for item in config["insertion"]:
        pattern(item, "insertion")
    ordered_objects(config["classical_forms"], ("lemma", "lemma_suffix", "ctype", "class_name", "forms"),
                    ("aliases", "connection"), "classical_forms")
    ordered_objects(config["classical_auxiliaries"], ("lemma", "rule", "class_name", "forms", "connection", "inputs"),
                    ("aliases", "ctype"), "classical_auxiliaries")
    for entry in [*config["classical_forms"], *config["classical_auxiliaries"]]:
        for key in ("aliases", "ctype", "connection"):
            strings(entry.get(key, []), f"{entry['lemma']}/{key}")
        for form, values in entry["forms"].items():
            strings(values, f"{entry['lemma']}/forms/{form}")
        for condition in entry.get("inputs", []):
            fields(condition, ("categories", "forms"), ("ctype", "exclude_ctype"), "classical/inputs")
            strings(condition["categories"], "classical/categories", CATEGORIES)
            strings(condition["forms"], "classical/forms", FORMS)
            for key in ("ctype", "exclude_ctype"):
                strings(condition.get(key, []), f"classical/{key}")
    for rule in config["input_recoveries"]:
        for key in ("previous", "next"):
            pattern(rule[key], f"input_recoveries/{key}")
        for item in rule["following"]:
            pattern(item, "input_recoveries/following")
    for name, rule in config["connection_rules"].items():
        fields(rule, ("label", "operators", "paths"), ("scope", "source", "ctype_mode"), name)
        if rule.get("scope", "") not in ("", "inflection"):
            raise ValueError(f"{name} 的路径范围无效")
        if rule.get("ctype_mode", "prefix") not in ("prefix", "classical_prefix"):
            raise ValueError(f"{name} 的活用型匹配模式无效")
        for path in rule["paths"]:
            fields(path, ("id", "group", "label", "steps"),
                   ("categories", "forms", "ctypes", "exclude_ctypes", "cform_contains",
                    "exclude_cform_contains", "previous_operators", "exclude_previous_operators",
                    "exact_category", "evidence", "next", "predecessors", "connector", "connector_paths",
                    "following", "exclude_following"), f"{name}/{path['id']}")
            for key in ("next", "following", "exclude_following"):
                if key in path:
                    pattern(path[key], f"{name}/{path['id']}/{key}")
            for item in path.get("predecessors", []):
                pattern(item, f"{name}/{path['id']}/predecessors")
    fields(config["presentation"], ("sentence_compounds", "sentence_layers", "contraction_label",
                                   "observed_steps", "phenomena", "form_labels", "raw_form_labels",
                                   "families", "category_labels", "label_aliases"), (), "presentation")
    for key in ("families", "raw_form_labels"):
        ordered_objects(config["presentation"][key], ("prefix", "label"), (), f"presentation/{key}")
    ordered_objects(config["presentation"]["phenomena"], ("label", "cform_contains"), ("evidence",), "phenomena")
    for key in ("observed_steps", "sentence_compounds"):
        strings(config["presentation"][key], f"presentation/{key}", config["operators"].keys())
    for entry in config["entity_classes"].values():
        fields(entry, ("label", "executor"), ("summary",), "entity_classes")

    def tree(nodes):
        ordered_objects(nodes, ("id", "label"), ("summary", "input_examples", "output", "children"), "rule_tree")
        for node in nodes:
            tree(node.get("children", []))

    tree(config["rule_tree"])
    return check(config, report=False)


def compile_config(source=SOURCE):
    base = read_source(source, "base.json")
    fields(base, ("max_auxiliary_depth", "intrinsic_features", "forms", "input_recoveries"), (), "base")
    entries = {}
    for name, executors in OPERATOR_FILES.items():
        for kind, entry in read_source(source, f"operators/{name}.json").items():
            fields(entry, ("definition", "execution", "connection_type", "connection_rule"), (), kind)
            if kind in entries:
                raise ValueError(f"operator 重复：{kind}")
            if entry["execution"]["executor"] not in executors:
                raise ValueError(f"{kind} 的执行器与 operators/{name}.json 归属不符")
            if entry["connection_type"] not in CONNECTION_TYPES:
                raise ValueError(f"{kind} 的接续分类无效")
            entries[kind] = entry
    entries = dict(sorted(entries.items()))
    owners = {}
    members = {}
    for kind, entry in entries.items():
        if rule := entry["connection_rule"]:
            owners[kind] = rule
            members.setdefault(rule, []).append(kind)

    auxiliaries = read_source(source, "classical/auxiliaries.json")
    inputs = {}
    runtime_auxiliaries = deepcopy(auxiliaries)
    for entry, runtime in zip(auxiliaries, runtime_auxiliaries):
        if entry["rule"] in inputs:
            raise ValueError(f"文言助动词规则重复：{entry['rule']}")
        by_id = {}
        for item, output in zip(entry["inputs"], runtime["inputs"]):
            fields(item, ("id", "categories", "forms"), ("ctype", "exclude_ctype"), f"{entry['rule']}/inputs")
            if item["id"] in by_id:
                raise ValueError(f"{entry['rule']} 的输入 ID 重复：{item['id']}")
            strings(item["categories"], "classical/categories", CATEGORIES)
            strings(item["forms"], "classical/forms", FORMS)
            by_id[item["id"]] = item
            del output["id"]
        inputs[entry["rule"]] = by_id

    groups = read_source(source, "paths/explicit.json")
    for name, rule in groups.items():
        fields(rule, ("label", "paths"), ("scope", "ctype_mode"), name)
    classical = read_source(source, "paths/classical.json")
    predicates = read_source(source, "paths/predicates.json")
    if set(groups) & set(classical) or set(groups) & set(predicates) or set(classical) & set(predicates):
        raise ValueError("接续规则 ID 在路径源文件中重复")
    for name, spec in classical.items():
        fields(spec, ("label", "auxiliary", "paths"), (), name)
        auxiliary = spec["auxiliary"]
        if auxiliary not in inputs or owners.get(auxiliary) != name:
            raise ValueError(f"{name} 的文言助动词归属无效：{auxiliary}")
        paths = []
        for layout in spec["paths"]:
            fields(layout, ("id", "input", "categories", "group", "label", "steps"), (), f"{name}/paths")
            condition = inputs[auxiliary][layout["input"]]
            strings(layout["categories"], f"{name}/{layout['id']}/categories", CATEGORIES)
            if not set(layout["categories"]).issubset(condition["categories"]):
                raise ValueError(f"{name}/{layout['id']} 的类别不属于所引用输入")
            path = {key: layout[key] for key in ("id", "group", "label", "steps")}
            path.update(categories=layout["categories"], forms=condition["forms"], exact_category=True)
            for original, target in (("ctype", "ctypes"), ("exclude_ctype", "exclude_ctypes")):
                if original in condition:
                    path[target] = condition[original]
            paths.append(path)
        groups[name] = {"label": spec["label"], "source": "classical_auxiliaries",
                        "ctype_mode": "classical_prefix", "paths": paths}

    links = read_source(source, "rules/predicate_links.json")
    links_by_kind = {}
    for link in links:
        if link["kind"] in links_by_kind:
            raise ValueError(f"谓语构式重复：{link['kind']}")
        links_by_kind[link["kind"]] = link

    def predicate_group(name, visiting=()):
        if name in visiting:
            raise ValueError(f"谓语路径引用循环：{' → '.join((*visiting, name))}")
        if name in groups:
            return groups[name]
        spec = predicates[name]
        fields(spec, ("label", "predicate"), (), name)
        kind = spec["predicate"]
        if owners.get(kind) != name:
            raise ValueError(f"{name} 的谓语构式归属无效")
        prefix = owners[links_by_kind[kind]["prefix"]]
        inherited = predicate_group(prefix, (*visiting, name))
        groups[name] = {"label": spec["label"], "source": "predicate_links",
                        "paths": deepcopy(inherited["paths"])}
        return groups[name]

    for name in predicates:
        predicate_group(name)
    if set(members) != set(groups):
        raise ValueError(f"规则定义与归属不一致：{sorted(set(members) ^ set(groups))}")
    for name, rule in groups.items():
        rule["operators"] = members[name]

    insertion = read_source(source, "rules/insertion.json")
    fields(insertion, ("insertion", "insertion_positions"), (), "insertion")
    presentation = read_source(source, "presentation.json")
    fields(presentation, ("entity_classes", "rule_tree", "presentation"), (), "presentation source")
    config = {
        "schema": SCHEMA,
        "max_auxiliary_depth": base["max_auxiliary_depth"],
        "intrinsic_features": base["intrinsic_features"],
        "operators": {kind: value["definition"] for kind, value in entries.items()},
        "rule_metadata": {kind: value["execution"] for kind, value in entries.items()},
        "connection_types": {name: [kind for kind, value in entries.items() if value["connection_type"] == name]
                             for name in CONNECTION_TYPES},
        "connection_rules": groups,
        "features": read_source(source, "rules/features.json"),
        "connections": read_source(source, "rules/connections.json"),
        "sequences": read_source(source, "rules/sequences.json"),
        "predicate_links": links,
        "forms": base["forms"],
        "classical_forms": read_source(source, "classical/forms.json"),
        "classical_auxiliaries": runtime_auxiliaries,
        "insertion": insertion["insertion"],
        "insertion_positions": insertion["insertion_positions"],
        "input_recoveries": base["input_recoveries"],
        "entity_classes": presentation["entity_classes"],
        "rule_tree": presentation["rule_tree"],
        "presentation": presentation["presentation"],
    }
    validate(config)
    return config


def canonical(value):
    """对象键固定排序，所有有序数组保留原顺序。"""
    if isinstance(value, dict):
        return {key: canonical(value[key]) for key in sorted(value)}
    if isinstance(value, list):
        return [canonical(item) for item in value]
    return value


def render_config(config):
    ordered = {key: canonical(value) for key, value in config.items()}
    return (json.dumps(ordered, ensure_ascii=False, indent=2) + "\n").encode("utf-8")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="比较源文件编译结果与已提交的运行配置")
    args = parser.parse_args()
    try:
        config = compile_config()
        content = render_config(config)
        if args.check:
            if OUTPUT.read_bytes() != content:
                raise ValueError("运行配置与源文件不一致，请执行 npm run morphology:build")
            result = "编译与引用检查通过"
        elif OUTPUT.is_file() and OUTPUT.read_bytes() == content:
            result = "配置内容相同"
        else:
            OUTPUT.write_bytes(content)
            result = "配置已生成"
        print(f"{result}：{len(config['operators'])} 个 operator，"
              f"{len(config['connection_rules'])} 组规则，"
              f"{sum(len(rule['paths']) for rule in config['connection_rules'].values())} 条路径")
    except (ValueError, KeyError, TypeError, OSError) as error:
        parser.exit(1, f"活用配置编译失败：{error}\n")


if __name__ == "__main__":
    main()

