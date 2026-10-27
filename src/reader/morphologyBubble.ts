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
const formLabels: Record<string, string> = { stem: "词干", irrealis: "未然形", continuative: "连用形", terminal: "终止形", attributive: "连体形", conditional: "假定形", imperative: "命令形", volitional: "意向形", te: "て形" };
const posLabels: Record<string, string> = { 動詞: "动词", 形容詞: "イ形容词", 形状詞: "ナ形容词", 名詞: "名词", 代名詞: "代词", 助詞: "助词", 助動詞: "助动词" };

function formLabel(state: MorphologyState | undefined, kind = "", evidence: string[] = []) {
  return [formLabels[state?.form ?? ""], ...morphologyTags(kind, state?.conjugation_form, evidence)].filter(Boolean).join(" · ");
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
  const ctype = operator.state_after?.conjugation_type ?? "";
  const family = Object.entries(morphologyPresentation.families).find(([prefix]) => ctype.startsWith(prefix))?.[1];
  return {
    headword: headword || token?.surface || operator.normalized_form || "",
    identity: support ? "补助动词" : operator.executor === "insertion" ? "插入成分"
      : operator.executor === "classical_conjugation" ? "文言助动词"
      : particle ? "接续助词" : "助动词",
    conjugation: particle || operator.executor === "insertion" ? "无活用" : family || (ctype ? "特殊活用" : ""),
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

/** 节点覆盖原文的互斥分段；多语素规则保持一个接续单元。 */
export function morphologySentenceParts(operator: MorphologyOperator, owner: MorphologyChain | undefined, document: UnifiedDocument): SentenceChainPart[] {
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
          form: formLabel(state), breakBefore: false, relation: "", inserted: false,
          selected: false, unit: `${chain.chain_id}:core` });
      } else {
        const definition = operatorDefinitions[op.kind];
        const first = index === op.source_morpheme_range[0];
        const inserted = op.executor === "insertion";
        const support = definition?.support;
        const before = category(op.state_before, "動詞");
        const after = category(op.state_after, before);
        const converted = before !== after;
        const prior = parts[parts.length - 1];
        const connectorId = op.connection?.connector_operator_id;
        const jointSupport = support && connectorId && parts.some((part) => part.unit === connectorId)
          && parts.slice(parts.findIndex((part) => part.unit === connectorId) + 1).every((part) => part.inserted);
        const particle = token.pos?.[0] === "助詞";
        const breakBefore = Boolean(first && prior && !inserted && (support ? !jointSupport : particle || converted));
        const entry = morphologyEntry(op, document);
        const multi = op.source_morpheme_range[1] - op.source_morpheme_range[0] > 1;
        const caption = multi ? token.query_forms?.find((form) => form.kind === "base")?.form || token.surface : entry.headword;
        const label = inserted ? "插入语" : !first && multi ? "" : support ? "补助" : op.kind === "negative" ? "否定形" : entry.usage;
        parts.push({ id: `${op.operator_id}:${index}`, surface: token.surface, caption, label,
          form: inserted || particle ? "无活用" : formLabel(op.state_after, op.kind, op.evidence), breakBefore,
          relation: breakBefore ? support ? "补助接续" : converted ? `${posLabels[before] ?? "用言"} → ${posLabels[after] ?? "用言"}` : "助词接续" : "",
          inserted, selected: op.operator_id === operator.operator_id, unit: op.operator_id });
      }
      index++;
    }
  }
  return parts;
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
