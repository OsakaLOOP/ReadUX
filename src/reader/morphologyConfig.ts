import configuration from "../../configs/unidic-nlp/morphology.json" with { type: "json" };

type TokenField = "pos" | "sub" | "detail" | "ctype" | "cform" | "lemma" | "base" | "surface";
interface TokenPattern {
  equals?: Partial<Record<TokenField, string[]>>;
  starts_with?: Partial<Record<TokenField, string[]>>;
  contains?: Partial<Record<TokenField, string[]>>;
}
interface OperatorDefinition {
  label: string;
  title: string;
  prototype: string;
  normalized?: string;
  token_lemma?: boolean;
  support?: { group: string; expanded?: string[] };
  connector?: string;
  variants?: { token: TokenPattern; form: string }[];
}
export interface RuleTreeNode {
  id: string; label: string; summary?: string; input_examples?: string[]; output?: string; children?: RuleTreeNode[];
}
interface Token {
  surface: string;
  lemma?: string | null;
  pos?: (string | null)[];
}

export const morphologyPresentation = configuration.presentation;
export const operatorDefinitions: Record<string, OperatorDefinition> = configuration.operators;
export const morphologyEntityClasses = configuration.entity_classes ?? {};
export const morphologyRuleTree: RuleTreeNode[] = configuration.rule_tree ?? [];
export const morphologyConnectionRules = configuration.connection_rules;

export function morphologyTags(kind: string, cform = "", evidence: string[] = []): string[] {
  const labels = operatorDefinitions[kind]?.support?.group === "contracted" ? [morphologyPresentation.contraction_label] : [];
  for (const rule of morphologyPresentation.phenomena) {
    const marked = rule.evidence !== undefined && evidence.includes(rule.evidence);
    const observed = rule.cform_contains.some((value) => cform.includes(value));
    if (marked || observed) labels.push(rule.label);
  }
  return [...new Set(labels)];
}

export function configuredPrototype(kind: string, token?: Token): string {
  const entry = operatorDefinitions[kind];
  if (!entry) return "";
  if (entry.normalized) return entry.normalized;
  if (token) {
    const values: Partial<Record<TokenField, string | null>> = {
      surface: token.surface, lemma: token.lemma, pos: token.pos?.[0], sub: token.pos?.[1], detail: token.pos?.[2],
    };
    const matches = (pattern: TokenPattern) =>
      Object.entries(pattern).every(([mode, fields]) => Object.entries(fields as Partial<Record<TokenField, string[]>>)
        .every(([field, alternatives]) => {
          const value = values[field as TokenField] ?? "";
          return alternatives?.some((alternative) => mode === "equals" ? value === alternative
            : mode === "starts_with" ? value.startsWith(alternative) : value.includes(alternative));
        }));
    const variant = entry.variants?.find((variant) => matches(variant.token));
    if (variant) return variant.form;
    if (entry.token_lemma && token.lemma) return token.lemma;
  }
  return entry.prototype;
}
