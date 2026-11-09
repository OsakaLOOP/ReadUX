import configuration from "../../configs/unidic-nlp/morphology.json" with { type: "json" };
import type { MorphologyOperator } from "../types/nlp";

if (configuration.schema !== "readux.morphology-config.v1") {
  throw new Error(`活用配置版本不匹配：${configuration.schema}`);
}

interface OperatorDefinition {
  label: string;
  title: string;
  prototype: string;
  normalized?: string;
  support?: { group: string; bases: string[]; expanded?: string[] };
  connector?: string;
  connector_alternatives?: string[];
}
export interface RuleTreeNode {
  id: string; label: string; summary?: string; input_examples?: string[]; output?: string; children?: RuleTreeNode[];
}
export interface ConnectionPathDefinition {
  id: string; group: string; label: string; steps: string[];
  capabilities?: number;
  connector?: string; connector_paths?: string[];
}
export interface ConnectionRuleDefinition {
  label: string; operators: string[]; paths: ConnectionPathDefinition[];
}
export const morphologyPresentation = configuration.presentation;
export const operatorDefinitions: Record<string, OperatorDefinition> = configuration.operators;
export const morphologyEntityClasses = configuration.entity_classes;
export const morphologyRuleTree: RuleTreeNode[] = configuration.rule_tree;
export const morphologyConnectionRules: Record<string, ConnectionRuleDefinition> = configuration.connection_rules;
export const morphologyConnectionTypes: Record<string, string[]> = configuration.connection_types;

const ruleByOperator = new Map(Object.values(morphologyConnectionRules)
  .flatMap((rule) => rule.operators.map((kind) => [kind, rule] as const)));
const typeByOperator = new Map(Object.entries(morphologyConnectionTypes)
  .flatMap(([type, kinds]) => kinds.map((kind) => [kind, type] as const)));

export function morphologyRuleForOperator(kind: string) {
  return ruleByOperator.get(kind);
}

export function morphologyConnectionType(kind: string): string {
  return typeByOperator.get(kind) ?? "direct";
}

export function isIntrinsicFeature(kind: string): boolean {
  return configuration.intrinsic_features.includes(kind);
}

export function morphologyTags(kind: string, cform = "", evidence: string[] = []): string[] {
  const labels = operatorDefinitions[kind]?.support?.group === "contracted" ? [morphologyPresentation.contraction_label] : [];
  for (const rule of morphologyPresentation.phenomena) {
    const marked = rule.evidence !== undefined && evidence.includes(rule.evidence);
    const observed = rule.cform_contains.some((value) => cform.includes(value));
    if (marked || observed) labels.push(rule.label);
  }
  return [...new Set(labels)];
}

export function configuredPrototype(kind: string): string {
  const entry = operatorDefinitions[kind];
  if (!entry) return "";
  return entry.normalized || entry.prototype;
}

/** 本句原型由分析阶段根据完整 UniDic 字段确定。 */
export function morphologyPrototype(operator: Pick<MorphologyOperator, "kind" | "normalized_form">): string {
  return operator.normalized_form || configuredPrototype(operator.kind);
}

export function morphologyUsage(operator: Pick<MorphologyOperator, "kind" | "concept_id" | "evidence" | "label">): string {
  if (operator.kind === "nominalization") {
    if (operator.concept_id === "grammar.nominalization.no") return "准体名词化";
    if (operator.concept_id === "grammar.functional.sa") return "性质名词化";
  }
  if (operator.kind === "sou_modality") {
    if (operator.evidence?.includes("judgement_usage:appearance")) return "样态";
    if (operator.evidence?.includes("judgement_usage:hearsay")) return "传闻";
  }
  return operatorDefinitions[operator.kind]?.title || operator.label || "活用";
}

export function morphologyFormLabel(rawForm = "", stateForm = "", fallback = "无活用"): string {
  return morphologyPresentation.raw_form_labels.find(({ prefix }) => rawForm.trim().startsWith(prefix))?.label
    || (rawForm.startsWith("意向形") ? "意向形" : "")
    || (morphologyPresentation.form_labels as Record<string, string>)[stateForm]
    || fallback;
}

export function morphologyCategoryLabel(category: string): string {
  return ({ 動詞: "动词", 形容詞: "イ形容词", 形状詞: "ナ形容词", 名詞: "名词", 代名詞: "代词",
    副詞: "副词", 助詞: "助词", 助動詞: "助动词", 文語用言: "文言用言" } as Record<string, string>)[category] || category;
}
