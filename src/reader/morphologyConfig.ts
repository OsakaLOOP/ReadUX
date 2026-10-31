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
