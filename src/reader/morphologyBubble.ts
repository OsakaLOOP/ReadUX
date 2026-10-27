import explanations from "../../crates/readux-core/resources/grammar/compiled/grammar_explanations.json" with { type: "json" };
import auxiliaries from "../../configs/grammar/auxiliaries.json" with { type: "json" };
import { configuredPrototype, morphologyConnectionRules, morphologyPresentation, morphologyTags, operatorDefinitions } from "./morphologyConfig.ts";
import type { MorphologyChain, MorphologyOperator, MorphologyState, UnifiedDocument } from "../types/nlp";

export interface MorphologyEntryContent {
  headword: string;
  identity: string;
  conjugation: string;
  usage: string;
  transition: string;
  summary: string;
  examples: { ja: string; zh: string }[];
}
export interface ConnectionPathView {
  id: string;
  label: string;
  condition: string;
  matched: boolean;
  actual: string;
}
export interface ConnectionGroupView { id: string; label: string; paths: ConnectionPathView[] }
export interface SentenceChainPart {
  id: string;
  range?: [number, number];
  surface: string;
  caption: string;
  label?: string;
  form: string;
  breakBefore: boolean;
  relation: string;
  inserted: boolean;
  selected: boolean;
  unit: string;
}
export interface ConnectionDiagram {
  id: string;
  target: string | null;
  status: string;
  diagnostics: string[];
  branches: { id: string; label: string; matched: boolean; groups: ConnectionGroupView[] }[];
  sentence: SentenceChainPart[];
}

const explanationByConcept = new Map(explanations.explanations.map((item) => [item.concept_id, item]));
const posLabels: Record<string, string> = { 動詞: "动词", 形容詞: "イ形容词", 形状詞: "ナ形容词", 名詞: "名词", 代名詞: "代词", 助詞: "助词", 助動詞: "助动词" };

function formLabel(state: MorphologyState | undefined, kind = "", evidence: string[] = [], rawForm = "") {
  const observed = rawForm || state?.conjugation_form || "";
  const forms: [string, string][] = [
    ["未然形", "未然形"], ["連用形", "连用形"], ["終止形", "终止形"],
    ["連体形", "连体形"], ["仮定形", "假定形"], ["已然形", "已然形"],
    ["命令形", "命令形"], ["意志推量形", "未然形"], ["意向形", "未然形"],
  ];
  const form = forms.find(([prefix]) => observed.startsWith(prefix))?.[1]
    || ({ irrealis: "未然形", continuative: "连用形", terminal: "终止形", attributive: "连体形",
      conditional: "假定形", imperative: "命令形", volitional: "未然形" } as Record<string, string>)[state?.form ?? ""]
    || "无活用";
  const phenomena = morphologyTags(kind, observed, evidence).filter((tag) => tag !== "插入语");
  return [form, ...phenomena].join(" · ");
}

/** 每个元素采用自身词形的活用信息，复合接续规则分别读取各成员。 */
function elementForm(document: UnifiedDocument, index: number, state?: MorphologyState, kind = "", evidence: string[] = []) {
  const token = document.morphemes[index];
  if (!token) return formLabel(state, kind, evidence);
  const source = document.source.tokens.find((item) => item.index === token.source_index);
  const rawForm = source?.fields.find((field) => field.name === "cForm")?.value || "";
  const inflecting = ["動詞", "形容詞", "助動詞"].includes(token.pos?.[0] ?? "")
    || token.pos?.[0] === "接尾辞" && ["動詞的", "形容詞的"].includes(token.pos?.[1] ?? "");
  if (!inflecting) return "无活用";
  return formLabel(state, kind, evidence, rawForm);
}

/** 复用推导步骤提供的局部变化，保留最后一组箭头。 */
export function morphologyLocalTransition(description: string, lexical = false) {
  const start = description.lastIndexOf("（");
  const local = start >= 0 && description.endsWith("）")
    ? lexical ? description.slice(0, start) : description.slice(start + 1, -1) : description;
  return local.includes("→") ? local.split("→").map((part) => part.trim()).join(" → ") : "";
}

export function morphologyEntry(operator: MorphologyOperator, document: UnifiedDocument, transition = ""): MorphologyEntryContent {
  const definition = operatorDefinitions[operator.kind];
  const token = document.morphemes?.[operator.source_morpheme_range?.[0]];
  const prototype = configuredPrototype(operator.kind, token);
  const support = definition?.support;
  const headword = support?.expanded?.[0] ?? (support?.group === "te" || support?.group === "contracted"
    ? prototype.replace(/^(?:て|で)/u, "") : prototype);
  const explanation = explanationByConcept.get(operator.concept_id);
  const builtin = auxiliaries.entries.find((entry) => entry.headword === headword);
  const pairs = (explanation?.body_blocks ?? []).filter((block) => block.kind === "example_pair")
    .map((block) => ({ ja: block.label ?? "", zh: block.text }));
  const examples = pairs.length ? pairs : builtin?.senses[0]?.examples
    ?? (explanation?.examples ?? []).map((ja) => ({ ja, zh: "" }));
  const particle = token?.pos?.[0] === "助詞";
  return {
    headword: headword || token?.surface || operator.normalized_form || "",
    identity: support ? "补助动词" : operator.executor === "insertion" ? "插入成分"
      : operator.executor === "classical_conjugation" ? "文言助动词"
      : particle ? "接续助词" : "助动词",
    conjugation: elementForm(document, operator.source_morpheme_range[1] - 1, operator.state_after, operator.kind, operator.evidence),
    usage: definition?.title || operator.label,
    transition: morphologyLocalTransition(transition),
    summary: operator.kind === "polite_negative" ? builtin?.senses[0]?.meaning || "礼貌地否定动作或状态。"
      : explanation?.function_summary || builtin?.senses[0]?.meaning || definition?.title || operator.description,
    examples: examples.slice(0, 2),
  };
}

/** 按实际父子链收集整个谓词家族。 */
function chainFamily(owner: MorphologyChain | undefined, chains: MorphologyChain[]) {
  let root = owner;
  const visited = new Set<string>();
  while (root?.parent_chain_id && !visited.has(root.chain_id)) {
    visited.add(root.chain_id);
    root = chains.find((chain) => chain.chain_id === root!.parent_chain_id) ?? root;
  }
  const family = root ? [root] : [];
  for (let index = 0; index < family.length; index++) {
    family.push(...chains.filter((chain) => chain.parent_chain_id === family[index].chain_id
      && !family.some((included) => included.chain_id === chain.chain_id)));
  }
  return family.sort((a, b) => (a.char_range?.[0] ?? 0) - (b.char_range?.[0] ?? 0));
}

function category(state: MorphologyState | undefined, fallback: string) {
  return state?.category === "文語用言" ? fallback : state?.category ?? fallback;
}

/** 判断词尾承接前项谓语身份；词尾自身的词性单独用于词条说明。 */
const predicateEndings = new Set([
  "copula", "copula_de", "copula_past", "copula_aru", "politeness_desu",
  "darou_modality", "deshou_modality", "nara_condition", "attributive_na", "adverbial_ni",
]);

function predecessorCategory(operator: MorphologyOperator, chain: MorphologyChain, document: UnifiedDocument) {
  if (!predicateEndings.has(operator.kind)) return category(operator.state_before, "動詞");
  const position = chain.operators.findIndex((item) => item.operator_id === operator.operator_id);
  const previous = chain.operators.slice(0, Math.max(0, position)).reverse()
    .find((item) => !predicateEndings.has(item.kind) && item.executor !== "insertion");
  return previous?.state_after?.category
    || document.morphemes[chain.anchor_morpheme]?.pos?.[0]
    || category(operator.state_before, "名詞");
}

function connectionFunction(operator: MorphologyOperator, before: string, after: string, surface: string) {
  const kind = operator.kind;
  if (predicateEndings.has(kind)) {
    if (kind === "adverbial_ni") return "连用修饰";
    if (kind === "attributive_na") return "连体修饰";
    if (kind === "nara_condition") return "条件接续";
    if (kind === "darou_modality" || kind === "deshou_modality") return kind === "deshou_modality" ? "推量（敬体）" : "推量（简体）";
    const polite = kind === "politeness_desu" || surface.startsWith("でし");
    const identity = ["名詞", "代名詞"].includes(before) ? "名词谓语"
      : before === "形状詞" ? "形容动词" : "谓语";
    const form = kind === "copula_de" ? "连用" : polite ? "敬体" : "简体";
    const past = kind === "copula_past" ? "·过去" : "";
    return `${identity}（${form}${past}）· 判断`;
  }
  if (operator.executor === "nominalization" || kind === "degree") return "名词化";
  if (operator.executor === "classical_conjugation") return operatorDefinitions[kind]?.title || "文言活用";
  const modality: Record<string, string> = {
    sou_modality: ["terminal", "attributive"].includes(operator.state_before?.form ?? "") ? "传闻" : "样态",
    you_modality: "比况／推量", mitai_modality: "比况／推量", rashii_modality: "推量",
    obligation: "当为判断", hazu_modality: "预期判断", kamoshirenai: "可能性推量",
  };
  if (modality[kind]) return modality[kind];
  if (before !== after && ["i_adjective_conversion", "verb_cycle+i_adjective_conversion"].includes(operator.executor)) {
    return `${posLabels[before] || before} → ${posLabels[after] || after}`;
  }
  return operatorDefinitions[kind]?.title || operator.label || "接续";
}

/** 节点覆盖原文的互斥分段；多语素规则保持一个接续单元。 */
export function morphologySentenceParts(operator: MorphologyOperator | null, owner: MorphologyChain | undefined, document: UnifiedDocument): SentenceChainPart[] {
  const parts: SentenceChainPart[] = [];
  const family = chainFamily(owner, document.morphology?.chains ?? []);
  for (const chain of family) {
    if (!chain.morpheme_range) continue;
    const operators = (chain.operators ?? []).filter((op) => operatorDefinitions[op.kind]
      && !morphologyPresentation.inflections.includes(op.kind) && op.char_range);
    // 复合规则拥有完整范围，同一范围仅显示一个语法身份。
    const owners: MorphologyOperator[] = [];
    for (const op of [...operators].sort((a, b) => (b.char_range[1] - b.char_range[0]) - (a.char_range[1] - a.char_range[0]))) {
      if (!owners.some((other) => op.char_range[0] < other.char_range[1] && other.char_range[0] < op.char_range[1])) owners.push(op);
    }
    let index = chain.morpheme_range[0];
    while (index < chain.morpheme_range[1]) {
      const token = document.morphemes?.[index];
      if (!token) { index++; continue; }
      const op = owners.find((item) => item.source_morpheme_range[0] <= index && index < item.source_morpheme_range[1]);
      if (!op) {
        const start = index;
        while (index + 1 < chain.morpheme_range[1] && !owners.some((item) => item.source_morpheme_range[0] <= index + 1 && index + 1 < item.source_morpheme_range[1])) index++;
        const surface = (document.morphemes ?? []).slice(start, index + 1).map((item) => item.surface).join("");
        const next = owners.find((item) => item.source_morpheme_range[0] === index + 1);
        const state = next?.state_before ?? chain.final_state;
        const caption = chain.lookup_form || chain.dictionary_form || token.lemma || "";
        parts.push({ id: `${chain.chain_id}:core:${start}`, surface, caption,
          range: token.char_range && document.morphemes[index].char_range ? [token.char_range[0], document.morphemes[index].char_range[1]] : undefined,
          form: elementForm(document, index, state), breakBefore: false, relation: "", inserted: false,
          selected: false, unit: `${chain.chain_id}:core` });
      } else {
        const definition = operatorDefinitions[op.kind];
        const first = index === op.source_morpheme_range[0];
        const inserted = op.executor === "insertion";
        const support = definition?.support;
        const before = predecessorCategory(op, chain, document);
        const after = predicateEndings.has(op.kind) ? before : category(op.state_after, before);
        const converted = before !== after;
        const prior = parts[parts.length - 1];
        const connectorId = op.connection?.connector_operator_id;
        const jointSupport = support && connectorId && parts.some((part) => part.unit === connectorId)
          && parts.slice(parts.findIndex((part) => part.unit === connectorId) + 1).every((part) => part.inserted);
        const particle = token.pos?.[0] === "助詞";
        const breakBefore = Boolean(first && prior && !inserted && (support ? !jointSupport : particle || converted || predicateEndings.has(op.kind)));
        const entry = morphologyEntry(op, document);
        const multi = op.source_morpheme_range[1] - op.source_morpheme_range[0] > 1;
        const caption = multi ? token.query_forms?.find((form) => form.kind === "base")?.form || token.surface : entry.headword;
        const label = inserted ? "插入语" : !first && multi ? "" : support ? "补助" : op.kind === "negative" ? "否定形" : entry.usage;
        parts.push({ id: `${op.operator_id}:${index}`, surface: token.surface, caption, label,
          range: token.char_range,
          form: elementForm(document, index, op.state_after, op.kind, op.evidence), breakBefore,
          relation: breakBefore ? support ? "补助接续" : connectionFunction(op, before, after,
            document.morphemes.slice(...op.source_morpheme_range).map((item) => item.surface).join("")) : "",
          inserted, selected: op.operator_id === operator?.operator_id, unit: op.operator_id });
      }
      index++;
    }
  }
  return parts;
}

/** 词干查询沿用完整父子链，仅高亮其原文范围并显示句中关系。 */
export function morphologyStemDiagram(owner: MorphologyChain, document: UnifiedDocument, range: [number, number]): ConnectionDiagram | null {
  const sentence = morphologySentenceParts(null, owner, document);
  if (!sentence.length) return null;
  for (const part of sentence) {
    part.selected = Boolean(part.range && part.range[0] < range[1] && range[0] < part.range[1]);
  }
  return { id: `${owner.chain_id}:stem:${range[0]}:${range[1]}`,
    target: null, status: owner.status, diagnostics: [], branches: [], sentence };
}

export function morphologyConnectionDiagram(
  operator: MorphologyOperator, owner: MorphologyChain | undefined, document: UnifiedDocument, status: string,
): ConnectionDiagram | null {
  const connection = operator.connection;
  type Path = { id: string; group: string; label: string; steps: string[]; next?: { equals?: { surface?: string[] } } };
  const rules: Record<string, { label: string; paths: Path[] }> = morphologyConnectionRules;
  const rule = connection && rules[connection.rule];
  const sentence = morphologySentenceParts(operator, owner, document);
  if (!rule && !sentence.length) return null;
  const connector = connection?.connector_operator_id
    ? document.morphology.chains.flatMap((chain) => chain.operators).find((op) => op.operator_id === connection.connector_operator_id) : operator;
  const separateTe = connection?.rule === "te";
  const branches = (separateTe ? ["て", "で"] : [""]).map((label) => {
    const groups: ConnectionGroupView[] = [];
    const paths = (rule?.paths ?? []).filter((path) => !separateTe || path.next?.equals?.surface?.includes(label));
    for (const path of paths) {
      let group = groups.find((item) => item.label === path.group);
      if (!group) { group = { id: `${label}:${path.group}`, label: path.group, paths: [] }; groups.push(group); }
      const matched = connection?.matched_paths.includes(path.id) ?? false;
      const steps = path.steps.filter((step) => !separateTe || step !== label);
      // 词条自身的组成由目标节点说明，前接卡片仅保留输入条件。
      const conditions = connection?.rule === "zuni" ? steps.filter((step) => !["ず", "に", "に（接续用法）"].includes(step)) : steps;
      group.paths.push({ id: path.id, label: path.label.replace(" · 分词恢复", ""), condition: conditions.join(" · "), matched,
        actual: matched ? [connection?.predecessor_surface, ...morphologyTags("", connector?.state_before?.conjugation_form)].filter(Boolean).join(" · ") : "" });
    }
    return { id: label || connection?.rule || "", label, groups, matched: groups.some((group) => group.paths.some((path) => path.matched)) };
  }).filter((branch) => branch.groups.length);
  return { id: operator.operator_id, target: separateTe && !operatorDefinitions[operator.kind]?.support ? null : morphologyEntry(operator, document).headword,
    status, diagnostics: connection?.diagnostics ?? [], branches, sentence };
}
