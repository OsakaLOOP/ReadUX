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
  steps: { label: string; description: string; normalizedForm?: string | null }[];
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
  const chains = document.morphology?.chains ?? [];
  const projected = targets.map((target) => {
    const formationChainIds = (target.source_formation_ids ?? []).flatMap((id) =>
      document.formation?.nodes?.find((node) => node.id === id)?.word?.chain_ids ?? []);
    const memberChainIds = chains.filter((chain) => (chain.core_morpheme_indices ?? []).some((index) => {
      const token = document.morphemes?.[index];
      return token && token.char_range && token.char_range[0] < target.char_range[1] && token.char_range[1] > target.char_range[0];
    })).map((chain) => chain.chain_id);
    const rootIds = [...(target.lexical_core_ids ?? []), ...formationChainIds, ...memberChainIds];
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
    let start = target.char_range?.[0] ?? 0;
    let end = target.char_range?.[1] ?? 0;
    for (const chain of roots) {
      const chainRange = chain.char_range ?? chain.anchor_range;
      if (chainRange) {
        start = Math.min(start, chainRange[0]);
        end = Math.max(end, chainRange[1]);
      }
    }
    for (let changed = true; changed;) {
      changed = false;
      for (const chain of chains) {
        if (!chain.parent_chain_id || !included.has(chain.parent_chain_id) || included.has(chain.chain_id)) continue;
        included.add(chain.chain_id);
        const chainRange = chain.char_range ?? chain.anchor_range;
        if (chainRange) {
          start = Math.min(start, chainRange[0]);
          end = Math.max(end, chainRange[1]);
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
  if (!input || !output || input === output) return `${input || ""} → ${output || ""}`;
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

export function grammaticalConceptTitle(operator: MorphologyOperator): string {
  switch (operator.kind) {
    case "negative":
      return "否定形";
    case "passive_potential":
      return "可能、被动等";
    case "causative":
      return "使役";
    case "past":
      return "过去";
    case "politeness_masu":
    case "politeness_desu":
      return "敬体";
    case "desire":
      return "愿望";
    case "volitional":
      return "意向形";
    case "obligation":
      return "当为";
    case "negative_volitional":
      return "否定意向";
    case "desire_outward":
      return "外向愿望";
    case "sou_modality":
      return "样态 / 传闻";
    case "you_modality":
    case "mitai_modality":
    case "rashii_modality":
      return "推量";
    case "copula":
    case "copula_aru":
      return "断定";
    case "ease":
      return "容易";
    case "difficulty":
      return "困难";
    case "nominalization":
      return "名词化";
    case "ba_connection":
    case "tara_condition":
      return "条件";
    case "nagara_connection":
      return "伴随（ながら）";
    case "concessive_connection":
      return "逆接";
    case "te_connection":
    case "te_form":
    case "de_form":
      return "て形";
    case "prohibitive":
      return "禁止";
    case "enumerative":
      return "列举";
    case "tari_suru":
      return "列举·举例";
    case "excessive":
      return "过度";
    case "inceptive":
      return "动作开始";
    case "continuative_aspect":
      return "动作持续";
    case "terminative":
      return "动作完了";
    case "iterative_aspect":
      return "反复动作";
    case "inchoative":
      return "开始发生";
    case "corrective":
      return "重新进行";
    case "incomplete":
      return "动作未完";
    case "completion":
      return "彻底完成";
    case "reluctance":
      return "难以做出";
    case "failure":
      return "失败 / 错过";
    case "missed_opportunity":
      return "错失机会";
    default:
      return grammaticalLabel(operator.label) || "活用";
  }
}

export function operatorPrototypeForm(
  operator: MorphologyOperator,
  morphemes?: ReadonlyArray<{ surface: string; lemma?: string | null; pos?: (string | null)[] }>,
): string {
  if (operator.normalized_form) return operator.normalized_form;
  const token = morphemes && operator.source_morpheme_range
    ? morphemes.slice(operator.source_morpheme_range[0], operator.source_morpheme_range[1])[0]
    : undefined;
  const tokenLemma = token?.lemma || "";
  const tokenSurface = token?.surface || "";

  switch (operator.kind) {
    case "negative":
      if (tokenLemma === "ぬ" || tokenSurface === "ぬ") return "ぬ";
      if (tokenLemma === "ず" || tokenSurface === "ず") return "ず";
      return "ない";
    case "past":
      return "た";
    case "politeness_masu":
      return "ます";
    case "politeness_desu":
      return "です";
    case "desire":
      return "たい";
    case "causative":
      if (tokenLemma.includes("させる") || tokenSurface.startsWith("させ")) return "させる";
      if (tokenLemma.includes("しめる") || tokenSurface.startsWith("しめ")) return "しめる";
      return "せる";
    case "passive_potential":
      if (tokenLemma.includes("られる") || tokenSurface.startsWith("られ")) return "られる";
      return "れる";
    case "volitional":
      if (tokenSurface.startsWith("よう") || tokenLemma.includes("よう")) return "よう";
      return "う";
    case "obligation":
      if (tokenLemma.includes("べし") || tokenSurface.includes("べし")) return "べし";
      return "べきだ";
    case "negative_volitional":
      return "まい";
    case "desire_outward":
      return "たがる";
    case "sou_modality":
      return "そうだ";
    case "you_modality":
      return "ようだ";
    case "mitai_modality":
      return "みたいだ";
    case "rashii_modality":
      return "らしい";
    case "copula":
      return "だ";
    case "copula_aru":
      return "である";
    case "ease":
      return "やすい";
    case "difficulty":
      if (tokenSurface.includes("づらい") || tokenLemma.includes("づらい")) return "づらい";
      return "にくい";
    case "nominalization":
      return tokenLemma === "の" ? "の" : "さ";
    case "ba_connection":
      return "ば";
    case "tara_condition":
      return "たら";
    case "nagara_connection":
      return "ながら";
    case "concessive_connection":
      return tokenSurface === "ども" ? "ども" : "ど";
    case "te_connection":
    case "te_form":
      return tokenSurface === "で" ? "で" : "て";
    case "de_form":
      return "で";
    case "prohibitive":
      return "な";
    case "enumerative":
      return "たり";
    case "tari_suru":
      return "たりする";
    case "te_iru":
    case "contracted_te_iru":
      return "ている";
    case "te_aru":
      return "てある";
    case "te_shimau":
    case "contracted_te_shimau":
      return "てしまう";
    case "te_oku":
    case "contracted_te_oku":
      return "ておく";
    case "te_iku":
      return "ていく";
    case "te_kuru":
      return "てくる";
    case "te_miru":
      return "てみる";
    case "te_kudasaru":
      return "てくださる";
    case "te_morau":
      return "てもらう";
    case "te_ageru":
      return "てあげる";
    case "te_kureru":
      return "てくれる";
    case "te_yaru":
      return "てやる";
    case "te_hoshii":
      return "てほしい";
    case "te_goran":
      return "てごらん";
    case "te_miseru":
      return "てみせる";
    case "te_tamaranai":
      return "てたまらない";
    case "te_naranai":
      return "てならない";
    case "te_yamanai":
      return "てやまない";
    case "excessive":
      return "すぎる";
    case "inceptive":
      return tokenLemma || "始める";
    case "continuative_aspect":
      return tokenLemma || "続ける";
    case "terminative":
      return tokenLemma || "終わる";
    case "iterative_aspect":
      return tokenLemma || "まくる";
    case "inchoative":
      return tokenLemma || "出す";
    case "corrective":
      return tokenLemma || "直す";
    case "incomplete":
      return tokenLemma || "掛ける";
    case "completion":
      return tokenLemma || "切る";
    case "reluctance":
      return tokenLemma || "兼ねる";
    case "failure":
      return tokenLemma || "損なう";
    case "missed_opportunity":
      return tokenLemma || "そびれる";
    default:
      if (tokenLemma && (token?.pos?.[0] === "動詞" || token?.pos?.[0] === "助動詞")) {
        return tokenLemma;
      }
      return grammaticalLabel(operator.label) || "活用";
  }
}

// 兼容已有导出的别名
export const operatorPrototypeTitle = operatorPrototypeForm;

function combinePrototype(prefix: string, prototype: string): string {
  if (!prefix) return prototype;
  if (prototype.startsWith("て") && (prefix.endsWith("て") || prefix.endsWith("で"))) {
    const connector = prefix.endsWith("で") ? "で" : "て";
    return prefix.slice(0, -1) + connector + prototype.slice(1);
  }
  if (prototype.startsWith("たり") && (prefix.endsWith("たり") || prefix.endsWith("だり"))) {
    const connector = prefix.endsWith("だり") ? "だり" : "たり";
    return prefix.slice(0, -2) + connector + prototype.slice(2);
  }
  return `${prefix}${prototype}`;
}

export interface DerivedStepInfo {
  operatorId: string;
  kind: string;
  conceptTitle: string;
  prototypeForm: string;
  surface: string;
  input: string;
  output: string;
  description: string;
  normalizedForm: string | null;
}

function collectChainFamily(chains: ReadonlyArray<MorphologyChain>, targetChain: MorphologyChain) {
  const root = chainRoot(chains, targetChain);
  const family = [root];
  const seen = new Set([root.chain_id]);
  for (let i = 0; i < family.length; i++) {
    const parent = family[i];
    for (const item of chains) {
      if (item.parent_chain_id === parent.chain_id && !seen.has(item.chain_id)) {
        seen.add(item.chain_id);
        family.push(item);
      }
    }
  }
  return { root, family };
}

function buildChainStepMap(
  root: MorphologyChain,
  familyChains: ReadonlyArray<MorphologyChain>,
  document: UnifiedDocument,
): Map<string, DerivedStepInfo> {
  const characters = Array.from(document.text ?? "");
  const result = new Map<string, DerivedStepInfo>();

  const nonInflectionOperators = familyChains
    .flatMap((c) => (c.operators ?? []).map((op) => ({ op, chain: c })))
    .filter(({ op }) => !["conjugation", "initial_alternation", "final_alternation"].includes(op.kind))
    .sort((a, b) => {
      const aStart = a.op.char_range ? a.op.char_range[0] : 0;
      const bStart = b.op.char_range ? b.op.char_range[0] : 0;
      return aStart - bStart;
    });

  const rootStart = root.char_range ? root.char_range[0] : 0;
  const rootLemma = root.lookup_form || root.lemma_form || root.dictionary_form || root.surface_form || "";

  // 检查核心词本身是否存在独立的屈折步骤（如 未然形: 読む -> 読ま）
  const synthetic = syntheticConjugationStep(root, root.char_range?.[1] ?? Number.POSITIVE_INFINITY, document, root.operators ?? []);
  const hasCoreInflection = Boolean(synthetic) || (root.operators ?? []).some((op) => op.kind === "conjugation");

  let coreBase = rootLemma;
  if (nonInflectionOperators.length > 0) {
    const firstOp = nonInflectionOperators[0].op;
    if (firstOp.char_range && root.char_range) {
      coreBase = characters.slice(rootStart, firstOp.char_range[0]).join("");
    }
  } else if (root.char_range) {
    coreBase = characters.slice(rootStart, root.char_range[1]).join("") || root.surface_form || rootLemma;
  }
  if (!coreBase) coreBase = root.surface_form || rootLemma;

  // 若核心词未单独作为屈折步骤展示（例如サ变力説して），首个变形的起点承接辞书原形
  const initialBase = hasCoreInflection ? coreBase : (rootLemma || coreBase);
  let currentPrototype = initialBase;

  for (let i = 0; i < nonInflectionOperators.length; i++) {
    const { op, chain } = nonInflectionOperators[i];
    const conceptTitle = grammaticalConceptTitle(op);
    const prototypeForm = operatorPrototypeForm(op, document.morphemes);
    const opSurface = op.char_range
      ? characters.slice(op.char_range[0], op.char_range[1]).join("")
      : (op.output_state || chain.surface_form);

    const precedingSlice = op.char_range
      ? characters.slice(rootStart, op.char_range[0]).join("")
      : "";
    const currentSlice = op.char_range
      ? characters.slice(rootStart, op.char_range[1]).join("")
      : "";

    // 起点：第 0 个算子从 initialBase 出发；后续算子从上一个的原型开始
    const input = i === 0 ? initialBase : currentPrototype;

    // 当前算子的现实原型形式（以该算子的现实原型结束，如 読まない、読み始める、読んでおく）
    const stepPrototype = combinePrototype(precedingSlice || coreBase, prototypeForm);

    // 终点：后续还有算子时以原型结束；末尾算子输出当前实际完整呈现形式
    const isLast = i === nonInflectionOperators.length - 1;
    let output = isLast ? (currentSlice || stepPrototype) : stepPrototype;
    if (!output) output = currentSlice || op.output_state || chain.surface_form;

    const desc = op.char_range ? transitionDescription(input, output) : (op.description || "");

    result.set(op.operator_id, {
      operatorId: op.operator_id,
      kind: op.kind,
      conceptTitle,
      prototypeForm,
      surface: opSurface,
      input,
      output,
      description: desc,
      normalizedForm: op.normalized_form ?? null,
    });

    currentPrototype = isLast ? output : stepPrototype;
  }

  return result;
}

function resolveOperatorStep(
  chain: MorphologyChain,
  operator: MorphologyOperator,
  document: UnifiedDocument,
  chains: ReadonlyArray<MorphologyChain>,
): DerivedStepInfo {
  if (["conjugation", "initial_alternation", "final_alternation"].includes(operator.kind)) {
    const characters = Array.from(document.text ?? "");
    const charRange = operator.char_range || chain.char_range;
    const output = charRange
      ? characters.slice(charRange[0], charRange[1]).join("")
      : (operator.output_state || chain.surface_form);
    const input = operator.input_state || chain.lookup_form || chain.lemma_form || chain.dictionary_form || chain.surface_form || "";
    const label = conjugationLabel(
      operator.state_after?.conjugation_type || chain.final_state?.conjugation_type,
      operator.state_after?.conjugation_form || chain.final_state?.conjugation_form,
      operator.state_after?.category || chain.final_state?.category,
      operator.state_after?.form || chain.final_state?.form,
    ) || conjugationFormLabel(
      operator.state_after?.conjugation_form || chain.final_state?.conjugation_form,
      operator.state_after?.form || chain.final_state?.form,
    );
    return {
      operatorId: operator.operator_id,
      kind: operator.kind,
      conceptTitle: operator.evidence?.includes("unidic:potential") ? `可能形 · ${conjugationFormLabel(operator.state_after?.conjugation_form, operator.state_after?.form)}` : label,
      prototypeForm: output,
      surface: output,
      input,
      output,
      description: transitionDescription(input, output),
      normalizedForm: operator.normalized_form ?? null,
    };
  }

  const { root, family } = collectChainFamily(chains, chain);
  const stepMap = buildChainStepMap(root, family, document);
  const step = stepMap.get(operator.operator_id);
  if (step) return step;

  const characters = Array.from(document.text ?? "");
  const rootRange = root.char_range ?? chain.char_range;
  const origin = rootRange ? rootRange[0] : 0;
  const output = operator.char_range
    ? characters.slice(origin, operator.char_range[1]).join("")
    : (operator.output_state || chain.surface_form);
  const preceding = operator.char_range
    ? characters.slice(origin, operator.char_range[0]).join("")
    : "";
  const input = preceding || operator.input_state || chain.lookup_form;
  const conceptTitle = grammaticalConceptTitle(operator);
  const prototypeForm = operatorPrototypeForm(operator, document.morphemes);
  const surface = operator.char_range
    ? characters.slice(operator.char_range[0], operator.char_range[1]).join("")
    : (operator.output_state || chain.surface_form);
  return {
    operatorId: operator.operator_id,
    kind: operator.kind,
    conceptTitle,
    prototypeForm,
    surface,
    input,
    output: output || operator.output_state || chain.surface_form,
    description: operator.char_range ? transitionDescription(input, output) : (operator.description || ""),
    normalizedForm: operator.normalized_form ?? null,
  };
}

function visibleState(chain: MorphologyChain, targetEnd: number) {
  const visible = [...(chain.operators ?? [])].reverse().find((operator) =>
    operator.char_range ? operator.char_range[1] <= targetEnd : true);
  if (visible) return visible.state_after ?? chain.final_state;
  const pending = (chain.operators ?? []).find((operator) =>
    operator.char_range
      ? (operator.char_range[0] >= targetEnd || (operator.char_range[0] < targetEnd && targetEnd <= operator.char_range[1]))
      : false);
  return pending?.state_before ?? chain.final_state;
}

function syntheticConjugationStep(
  chain: MorphologyChain,
  targetEnd: number,
  document: UnifiedDocument,
  visibleOperators: ReadonlyArray<MorphologyOperator>,
) {
  if (visibleOperators.some((operator) => ["conjugation", "initial_alternation", "final_alternation"].includes(operator.kind))) return null;
  const next = (chain.operators ?? []).find((operator) => operator.char_range ? operator.char_range[0] >= targetEnd : false)
    ?? visibleOperators[0];
  const state = next?.state_before;
  if (!next || !state?.conjugation_form || !next.char_range || !chain.char_range) return null;
  const characters = Array.from(document.text ?? "");
  const outputEnd = Math.min(targetEnd, next.char_range[0], chain.char_range[1]);
  if (chain.char_range[0] >= outputEnd) return null;
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
  const chainIds = new Set(target.lexical_core_ids ?? []);
  for (const node of document.formation?.nodes ?? []) {
    if ((target.source_formation_ids ?? []).includes(node.id)) {
      for (const id of node.word?.chain_ids ?? []) chainIds.add(id);
    }
  }
  let chains = (document.morphology?.chains ?? []).filter((chain) => chainIds.has(chain.chain_id));
  if (!chains.length) {
    const members = new Set(target.morpheme_ids ?? []);
    chains = (document.morphology?.chains ?? []).filter((chain) =>
      chain.role === "lexical" && (chain.core_morpheme_indices ?? []).some((index) => members.has(document.morphemes?.[index]?.id)));
  }
  for (const chain of chains) chainIds.add(chain.chain_id);
  for (let index = 0; index < chains.length; index++) {
    chains.push(...(document.morphology?.chains ?? []).filter((chain) => chain.parent_chain_id === chains[index].chain_id && !chainIds.has(chain.chain_id)));
    for (const chain of chains) chainIds.add(chain.chain_id);
  }
  const targetEnd = target.char_range?.[1] ?? Number.POSITIVE_INFINITY;
  chains = chains.filter((chain) =>
    chain.char_range ? (chain.char_range[0] < targetEnd && chain.char_range[1] > (target.char_range?.[0] ?? 0)) : true);
  const visibleOperators = (chain: MorphologyChain) =>
    (chain.operators ?? []).filter((operator) => operator.char_range ? operator.char_range[1] <= targetEnd : true);

  return chains.map((chain) => ({
    id: chain.chain_id,
    role: chain.role === "functional" ? "补助成分" : "词汇核心",
    surface: (() => {
      if (!chain.char_range) return chain.surface_form;
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
    connections: (chain.connection_forms ?? []).filter((value) => value && value !== "*"),
    members: (chain.morpheme_indices ?? []).filter((index) => (document.morphemes?.[index]?.char_range?.[1] ?? 0) <= targetEnd).flatMap((index) => {
      const morpheme = document.morphemes?.[index];
      return morpheme ? [{ surface: morpheme.surface, role: (chain.core_morpheme_indices ?? []).includes(index) ? "核心" : morpheme.pos?.[0] || "连接" }] : [];
    }),
    steps: (() => {
      const operators = visibleOperators(chain);
      const synthetic = syntheticConjugationStep(chain, targetEnd, document, operators);
      return [
        ...(synthetic ? [synthetic] : []),
        ...operators.map((operator) => {
          const step = resolveOperatorStep(chain, operator, document, document.morphology?.chains ?? []);
          return {
            label: step.conceptTitle,
            description: step.description,
            normalizedForm: step.normalizedForm ?? undefined,
          };
        }),
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
  const chains = new Map((document.morphology?.chains ?? []).map((chain) => [chain.chain_id, chain]));
  const characters = Array.from(document.text ?? "");
  function rootChain(chainId: string) {
    let root = chains.get(chainId);
    if (!root) return undefined;
    while (root.parent_chain_id && chains.has(root.parent_chain_id)) root = chains.get(root.parent_chain_id)!;
    return root;
  }
  function targetForChain(chainId: string) {
    const root = rootChain(chainId);
    if (!root) return undefined;
    const target = targets.find((item) => (item.lexical_core_ids ?? []).includes(root.chain_id))
      ?? targets.find((item) => (item.source_formation_ids ?? []).some((id) => source.formation?.nodes?.find((node) => node.id === id)?.word?.chain_ids.includes(root.chain_id)))
      ?? targets.find((item) => root.anchor_range && item.char_range && item.char_range[0] <= root.anchor_range[0] && root.anchor_range[1] <= item.char_range[1]);
    // 正文悬浮入口关联外层目标；最小内部对象仍由阅读器在当前位置选择。
    return target && targets.find((item) => item.id === (target as { parent_outer_id?: string | null }).parent_outer_id) || target;
  }
  const occurrences = (document.morphology?.occurrences ?? []).flatMap((occurrence) => {
    const target = targetForChain(occurrence.chain_id);
    if (!target) return [];
    const operators = (occurrence.operator_ids ?? []).flatMap((id) =>
      (document.morphology?.chains ?? []).flatMap((chain) => (chain.operators ?? []).filter((operator) => operator.operator_id === id)));
    const operator = operators.find((item) => item.kind === occurrence.kind) ?? operators[operators.length - 1];
    if (!operator) return [];
    const owner = (document.morphology?.chains ?? []).find((chain) =>
      (chain.operators ?? []).some((item) => item.operator_id === operator.operator_id));
    const step = owner
      ? resolveOperatorStep(owner, operator, document, document.morphology?.chains ?? [])
      : null;
    const detail: ReaderMorphologyDetail = {
      id: occurrence.id,
      title: step ? step.conceptTitle : grammaticalConceptTitle(operator),
      surface: step ? step.prototypeForm : operatorPrototypeForm(operator, document.morphemes),
      description: step ? step.description : (operator.description || ""),
      normalizedForm: operator.normalized_form ?? null,
      candidates: occurrence.candidates ?? [],
    };
    return (occurrence.hit_ranges ?? []).map((range) => ({ range, targetId: target.id, detail }));
  });
  const conjugations = (document.morphology?.chains ?? []).flatMap((chain) => {
    const target = targetForChain(chain.chain_id);
    const chainRange = chain.char_range ?? chain.anchor_range;
    if (!target || !chainRange || chain.surface_form === (chain.lemma_form || chain.dictionary_form)) return [];
    const synthetic = syntheticConjugationStep(chain, chainRange[1], document, chain.operators ?? []);
    const firstOperator = (chain.operators ?? [])[0];
    const syntheticRange: [number, number] = [chainRange[0], firstOperator?.char_range ? firstOperator.char_range[0] : chainRange[0]];
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
    const operatorHits = (chain.operators ?? []).filter((operator) => operator.kind === "conjugation" && operator.char_range && operator.char_range[0] < operator.char_range[1])
      .map((operator) => {
        const step = resolveOperatorStep(chain, operator, document, document.morphology?.chains ?? []);
        return {
          range: operator.char_range,
          targetId: target.id,
          detail: {
            id: operator.operator_id,
            title: step.conceptTitle,
            surface: step.prototypeForm || characters.slice(operator.char_range[0], operator.char_range[1]).join(""),
            description: step.description,
            normalizedForm: null,
            candidates: [],
          },
        };
      });
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
