import type { DictionaryLookup } from "../types";
import type { MorphologyChain, MorphologyOperator, QueryOutput, UnifiedDocument } from "../types/nlp";

export interface ReaderChainSummary {
  id: string;
  role: string;
  surface: string;
  lemma: string;
  lookupForm: string;
  conjugation: string;
  formName: string;
  connections: string[];
  members: { surface: string; role: string }[];
  steps: { label: string; description: string; normalizedForm: string | null }[];
}

export interface ReaderMorphologyDetail {
  id: string;
  title: string;
  surface: string;
  description: string;
  normalizedForm: string | null;
  candidates: string[];
}

export interface ReaderMorphologyHit {
  range: [number, number];
  targetId: string;
  detail: ReaderMorphologyDetail;
}

export interface ReaderCapsuleRange { id: string; range: [number, number] }

export function readerCapsuleRanges(
  targets: ReadonlyArray<{ id: string; char_range: [number, number]; lexical_core_ids: string[]; source_formation_ids: string[] }>,
  document: UnifiedDocument,
): ReaderCapsuleRange[] {
  const chains = document.morphology.chains;
  const projected = targets.map((target) => {
    const formationChainIds = target.source_formation_ids.flatMap((id) =>
      document.formation.nodes.find((node) => node.id === id)?.word?.chain_ids ?? []);
    const memberChainIds = chains.filter((chain) => chain.core_morpheme_indices.some((index) => {
      const token = document.morphemes[index];
      return token && token.char_range[0] < target.char_range[1] && token.char_range[1] > target.char_range[0];
    })).map((chain) => chain.chain_id);
    const rootIds = [...target.lexical_core_ids, ...formationChainIds, ...memberChainIds];
    const roots = rootIds.flatMap((id) => {
      let chain = chains.find((item) => item.chain_id === id);
      if (!chain) return [];
      while (chain.parent_chain_id) {
        const parent = chains.find((item) => item.chain_id === chain!.parent_chain_id);
        if (!parent) break;
        chain = parent;
      }
      return [chain];
    });
    const included = new Set(roots.map((chain) => chain.chain_id));
    let start = target.char_range[0];
    let end = target.char_range[1];
    for (const chain of roots) {
      if (chain.char_range[0] <= start && chain.char_range[1] >= start) {
        start = Math.min(start, chain.char_range[0]);
        end = Math.max(end, chain.char_range[1]);
      }
    }
    const token = document.morphemes.find((item) => item.char_range[0] === start && item.surface === "ん"
      && item.lemma === "の" && item.pos[0] === "助詞");
    if (token) {
      const precedingCopula = chains.find((chain) => chain.role === "functional" && chain.char_range[1] === start
        && chain.lookup_form === "だ" && chain.surface_form.endsWith("た"));
      if (precedingCopula) start = precedingCopula.char_range[0];
    }
    for (let changed = true; changed;) {
      changed = false;
      for (const chain of chains) {
        if (!chain.parent_chain_id || !included.has(chain.parent_chain_id) || included.has(chain.chain_id)) continue;
        included.add(chain.chain_id);
        if (chain.char_range[0] <= end) {
          end = Math.max(end, chain.char_range[1]);
        }
        changed = true;
      }
    }
    return { id: target.id, range: [start, end] as [number, number] };
  });
  // 同一条活用链中的词典目标共用胶囊，查询目标身份继续保留。
  const result: ReaderCapsuleRange[] = [];
  for (const capsule of projected.sort((left, right) => left.range[0] - right.range[0] || right.range[1] - left.range[1])) {
    const previous = result[result.length - 1];
    if (previous && capsule.range[0] < previous.range[1]) previous.range[1] = Math.max(previous.range[1], capsule.range[1]);
    else result.push(capsule);
  }
  return result;
}

export function readerRubyAnnotations(
  annotations: ReadonlyArray<{ base: string; reading: string; char_range: [number, number] }>,
  validations: ReadonlyArray<{ base: string; ruby_reading: string; char_range: [number, number]; candidate_char_range: [number, number] }>,
) {
  const resolved = [...new Map(validations.map((validation) => {
    const annotation = {
      base: validation.base,
      reading: validation.ruby_reading,
      char_range: validation.char_range,
    };
    return [`${annotation.char_range[0]}:${annotation.char_range[1]}:${annotation.reading}`, annotation] as const;
  })).values()];
  const covered = validations.map((validation) => validation.candidate_char_range);
  const unresolved = annotations.filter((annotation) =>
    !covered.some((range) => range[0] <= annotation.char_range[0] && annotation.char_range[1] <= range[1]));
  return [...resolved, ...unresolved].sort((left, right) => left.char_range[0] - right.char_range[0]);
}

const formLabels: Record<string, string> = {
  stem: "词干", irrealis: "未然形", continuative: "连用形", terminal: "终止形",
  attributive: "连体形", conditional: "假定形", imperative: "命令形",
  volitional: "意向形", te: "て形",
};

export function displayReading(reading: string) {
  return Array.from(reading, (character) => {
    const code = character.codePointAt(0)!;
    return code >= 0x30a1 && code <= 0x30f6 ? String.fromCodePoint(code - 0x60) : character;
  }).join("");
}

function grammaticalLabel(value: string) {
  return ({ "丁寧": "礼貌", "受身等候选": "被动／可能等", "接续": "接续" } as Record<string, string>)[value] ?? value;
}

function conjugationLabel(type = "", form = "", category = "", stateForm = "") {
  const family = type.startsWith("五段") ? "五段活用" : type.startsWith("一段") ? "一段活用"
    : type.startsWith("サ行変格") ? "サ变活用" : type.startsWith("カ行変格") ? "カ变活用" : "";
  const formName = Object.entries({ "未然形": "未然形", "連用形": "连用形", "終止形": "终止形", "連体形": "连体形", "仮定形": "假定形", "已然形": "已然形", "命令形": "命令形", "意志推量形": "意向形" })
    .find(([prefix]) => form.startsWith(prefix))?.[1] ?? formLabels[stateForm] ?? "";
  return [family || (category === "形容詞" ? "形容词" : ""), formName].filter(Boolean).join(" · ");
}

function conjugationFormLabel(form = "", stateForm = "") {
  return Object.entries({ "未然形": "未然形", "連用形": "连用形", "終止形": "终止形", "連体形": "连体形", "仮定形": "假定形", "已然形": "已然形", "命令形": "命令形", "意志推量形": "意向形" })
    .find(([prefix]) => form.startsWith(prefix))?.[1] ?? formLabels[stateForm] ?? "活用";
}

function chainRoot(chains: ReadonlyArray<MorphologyChain>, chain: MorphologyChain) {
  let root = chain;
  while (root.parent_chain_id) {
    const parent = chains.find((item) => item.chain_id === root.parent_chain_id);
    if (!parent) break;
    root = parent;
  }
  return root;
}

function transitionDescription(input: string, output: string) {
  if (input === output) return `${input} → ${output}`;
  const before = Array.from(input);
  const after = Array.from(output);
  let prefixLength = 0;
  while (prefixLength < before.length && prefixLength < after.length && before[prefixLength] === after[prefixLength]) prefixLength += 1;
  // 接续 operator 会在原活用尾部追加假名，提示保留该尾假名及追加后的完整尾部。
  if (prefixLength === before.length && before.length > 0 && after.length > before.length) prefixLength -= 1;
  const localInput = before.slice(prefixLength).join("") || "∅";
  const localOutput = after.slice(prefixLength).join("") || "∅";
  return `${input} → ${output}（${localInput}→${localOutput}）`;
}

function localTransition(
  chain: MorphologyChain,
  operator: MorphologyOperator,
  document: UnifiedDocument,
  chains: ReadonlyArray<MorphologyChain>,
) {
  const root = chainRoot(chains, chain);
  const characters = Array.from(document.text ?? "");
  const inflection = ["conjugation", "initial_alternation", "final_alternation"].includes(operator.kind);
  const origin = inflection ? chain.char_range[0] : root.char_range[0];
  const output = characters.slice(origin, operator.char_range[1]).join("") || operator.output_state || chain.surface_form;
  const preceding = characters.slice(origin, operator.char_range[0]).join("");
  const input = (inflection ? operator.input_state : preceding || operator.input_state) || chain.lookup_form;
  return transitionDescription(input, output);
}

function visibleState(chain: MorphologyChain, targetEnd: number) {
  const visible = [...chain.operators].reverse().find((operator) => operator.char_range[1] <= targetEnd);
  if (visible) return visible.state_after ?? chain.final_state;
  const pending = chain.operators.find((operator) => operator.char_range[0] >= targetEnd
    || (operator.char_range[0] < targetEnd && targetEnd <= operator.char_range[1]));
  return pending?.state_before ?? chain.final_state;
}

function syntheticConjugationStep(
  chain: MorphologyChain,
  targetEnd: number,
  document: UnifiedDocument,
  visibleOperators: ReadonlyArray<MorphologyOperator>,
) {
  if (visibleOperators.some((operator) => ["conjugation", "initial_alternation", "final_alternation"].includes(operator.kind))) return null;
  const next = chain.operators.find((operator) => operator.char_range[0] >= targetEnd)
    ?? visibleOperators[0];
  const state = next?.state_before;
  if (!next || !state?.conjugation_form) return null;
  const characters = Array.from(document.text ?? "");
  const outputEnd = Math.min(targetEnd, next.char_range[0], chain.char_range[1]);
  const output = characters.slice(chain.char_range[0], outputEnd).join("");
  if (!output || output === chain.lookup_form) return null;
  return {
    label: conjugationFormLabel(state.conjugation_form, state.form),
    description: transitionDescription(chain.lookup_form, output),
    normalizedForm: null,
  };
}

export function readerChains(
  target: { lexical_core_ids: string[]; source_formation_ids: string[]; morpheme_ids: string[]; char_range?: [number, number] },
  document: UnifiedDocument | null | undefined,
): ReaderChainSummary[] {
  if (!document) return [];
  const chainIds = new Set(target.lexical_core_ids);
  for (const node of document.formation.nodes) {
    if (target.source_formation_ids.includes(node.id)) {
      for (const id of node.word?.chain_ids ?? []) chainIds.add(id);
    }
  }
  let chains = document.morphology.chains.filter((chain) => chainIds.has(chain.chain_id));
  if (!chains.length) {
    const members = new Set(target.morpheme_ids);
    chains = document.morphology.chains.filter((chain) =>
      chain.role === "lexical" && chain.core_morpheme_indices.some((index) => members.has(document.morphemes[index]?.id)));
  }
  for (const chain of chains) chainIds.add(chain.chain_id);
  for (let index = 0; index < chains.length; index++) {
    chains.push(...document.morphology.chains.filter((chain) => chain.parent_chain_id === chains[index].chain_id && !chainIds.has(chain.chain_id)));
    for (const chain of chains) chainIds.add(chain.chain_id);
  }
  const targetEnd = target.char_range?.[1] ?? Number.POSITIVE_INFINITY;
  chains = chains.filter((chain) => chain.char_range[0] < targetEnd && chain.char_range[1] > (target.char_range?.[0] ?? 0));
  const visibleOperators = (chain: MorphologyChain) => chain.operators.filter((operator) => operator.char_range[1] <= targetEnd);
  return chains.map((chain) => ({
    id: chain.chain_id,
    role: chain.role === "functional" ? "补助成分" : "词汇核心",
    surface: (() => {
      const end = Math.min(targetEnd, chain.char_range[1]);
      const visible = Array.from(document.text ?? "").slice(chain.char_range[0], end).join("");
      return visible || chain.surface_form;
    })(),
    lemma: chain.lemma_form || chain.dictionary_form || chain.base_lexeme,
    lookupForm: chain.lookup_form,
    conjugation: (() => {
      const state = visibleState(chain, targetEnd);
      return conjugationLabel(state?.conjugation_type, state?.conjugation_form, state?.category, state?.form);
    })(),
    formName: [...new Set(visibleOperators(chain).filter((operator) => !["conjugation", "initial_alternation", "final_alternation"].includes(operator.kind))
      .map((operator) => grammaticalLabel(operator.label)).filter(Boolean))].join(" · "),
    connections: chain.connection_forms.filter((value) => value && value !== "*"),
    members: chain.morpheme_indices.filter((index) => (document.morphemes[index]?.char_range[1] ?? 0) <= targetEnd).flatMap((index) => {
      const morpheme = document.morphemes[index];
      return morpheme ? [{ surface: morpheme.surface, role: chain.core_morpheme_indices.includes(index) ? "核心" : morpheme.pos[0] || "连接" }] : [];
    }),
    steps: (() => {
      const operators = visibleOperators(chain);
      const synthetic = syntheticConjugationStep(chain, targetEnd, document, operators);
      return [
        ...(synthetic ? [synthetic] : []),
        ...operators.map((operator) => ({
          label: operator.kind === "conjugation"
            ? conjugationFormLabel(operator.state_after?.conjugation_form, operator.state_after?.form)
            : grammaticalLabel(operator.label),
          description: localTransition(chain, operator, document, document.morphology.chains),
          normalizedForm: operator.normalized_form,
        })),
      ];
    })(),
  }));
}

export function readerMorphologyHits(
  targets: { id: string; char_range: [number, number]; lexical_core_ids: string[]; source_formation_ids: string[] }[],
  document: UnifiedDocument | null | undefined,
): ReaderMorphologyHit[] {
  if (!document) return [];
  const source = document;
  const chains = new Map(document.morphology.chains.map((chain) => [chain.chain_id, chain]));
  const characters = Array.from(document.text);
  function rootChain(chainId: string) {
    let root = chains.get(chainId);
    if (!root) return undefined;
    while (root.parent_chain_id && chains.has(root.parent_chain_id)) root = chains.get(root.parent_chain_id)!;
    return root;
  }
  function targetForChain(chainId: string) {
    const root = rootChain(chainId);
    if (!root) return undefined;
    const target = targets.find((item) => item.lexical_core_ids.includes(root.chain_id))
      ?? targets.find((item) => item.source_formation_ids.some((id) => source.formation.nodes.find((node) => node.id === id)?.word?.chain_ids.includes(root.chain_id)))
      ?? targets.find((item) => item.char_range[0] <= root.anchor_range[0] && root.anchor_range[1] <= item.char_range[1]);
    // 正文悬浮入口关联外层目标；最小内部对象仍由阅读器在当前位置选择。
    return target && targets.find((item) => item.id === (target as { parent_outer_id?: string | null }).parent_outer_id) || target;
  }
  const occurrences = document.morphology.occurrences.flatMap((occurrence) => {
    const target = targetForChain(occurrence.chain_id);
    if (!target) return [];
    const operators = occurrence.operator_ids.flatMap((id) => document.morphology.chains.flatMap((chain) => chain.operators.filter((operator) => operator.operator_id === id)));
    const operator = operators.find((item) => item.kind === occurrence.kind) ?? operators[operators.length - 1];
    if (!operator) return [];
    const owner = document.morphology.chains.find((chain) => chain.operators.some((item) => item.operator_id === operator.operator_id));
    const detail: ReaderMorphologyDetail = {
      id: occurrence.id,
      title: operator.kind === "conjugation"
        ? conjugationFormLabel(operator.state_after?.conjugation_form, operator.state_after?.form)
        : grammaticalLabel(operator.label) || "活用",
      surface: characters.slice(occurrence.char_range[0], occurrence.char_range[1]).join(""),
      description: owner ? localTransition(owner, operator, document, document.morphology.chains) : operator.description || "",
      normalizedForm: operator.normalized_form,
      candidates: occurrence.candidates,
    };
    return occurrence.hit_ranges.map((range) => ({ range, targetId: target.id, detail }));
  });
  const conjugations = document.morphology.chains.flatMap((chain) => {
    const target = targetForChain(chain.chain_id);
    if (!target || chain.surface_form === (chain.lemma_form || chain.dictionary_form)) return [];
    const synthetic = syntheticConjugationStep(chain, chain.char_range[1], document, chain.operators);
    const firstOperator = chain.operators[0];
    const syntheticRange: [number, number] = [chain.char_range[0], firstOperator?.char_range[0] ?? chain.char_range[0]];
    const syntheticHits: ReaderMorphologyHit[] = synthetic && syntheticRange[0] < syntheticRange[1] ? [{
      range: syntheticRange,
      targetId: target.id,
      detail: {
        id: `${chain.chain_id}:core-conjugation`,
        title: synthetic.label,
        surface: characters.slice(...syntheticRange).join(""),
        description: synthetic.description,
        normalizedForm: null,
        candidates: [],
      },
    }] : [];
    const operatorHits = chain.operators.filter((operator) => operator.kind === "conjugation" && operator.char_range[0] < operator.char_range[1])
      .map((operator) => ({
        range: operator.char_range,
        targetId: target.id,
        detail: {
          id: operator.operator_id,
          title: conjugationFormLabel(operator.state_after?.conjugation_form || chain.final_state.conjugation_form,
            operator.state_after?.form || chain.final_state.form),
          surface: characters.slice(operator.char_range[0], operator.char_range[1]).join(""),
          description: localTransition(chain, operator, document, document.morphology.chains),
          normalizedForm: null,
          candidates: [],
        },
      }));
    return [...syntheticHits, ...operatorHits];
  });
  return [...occurrences, ...conjugations];
}

export function dictionaryLookupFromSearch(output: QueryOutput, word: string, formId = output.selected_form_id): DictionaryLookup {
  const group = output.groups.find((item) => item.form_id === formId) ?? output.groups[0];
  return {
    query: word,
    observed_form: word,
    reading: output.reading ?? null,
    mode: output.mode || "search",
    forms: output.forms,
    selected_form_id: formId ?? output.forms[0]?.form_id ?? null,
    dictionary_names: output.dictionary_names,
    entries: group?.entries ?? [],
  };
}
