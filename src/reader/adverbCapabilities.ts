import type { MorphologyChain, UnifiedDocument } from "../types/nlp";

/** 接续资格与 Rust、配置编译器共用位值；空值表示未审定。 */
export const adverbFlags = { none: 1, suru: 2, naDa: 4, toSuru: 8, tari: 16 } as const;

export function adverbCapabilities(chain: MorphologyChain): number | null {
  return chain.adverb_capabilities ?? null;
}

export function adverbInflecting(chain: MorphologyChain): boolean {
  return ((adverbCapabilities(chain) ?? 0) & 30) !== 0;
}

export function adverbCapabilityLabel(chain: MorphologyChain): string {
  const flags = adverbCapabilities(chain);
  if (flags == null) return "";
  if (flags === adverbFlags.none) return "不可活用";
  const labels = [[2, "サ变活用"], [4, "判断活用（だ・な・で・に）"], [8, "トス活用"], [16, "タリ活用"]] as const;
  return `可活用 · ${labels.filter(([flag]) => (flags & flag) !== 0).map(([, label]) => label).join("／")}`;
}

export function adverbStemLabel(chain: MorphologyChain): string {
  return adverbInflecting(chain) ? `词干 · ${adverbCapabilityLabel(chain)}` : "无活用";
}

export function adverbChainAt(document: UnifiedDocument, index: number): MorphologyChain | undefined {
  return document.morphology?.chains.find((chain) => chain.anchor_morpheme === index);
}
