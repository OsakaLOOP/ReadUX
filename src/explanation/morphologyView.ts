import type { AnnotatedToken, DictionaryTag, MorphologyChain, MorphologyOperator, PosTag } from "../types";

const POS_LABELS: Record<string, { group: string; label: string }> = {
  "名詞": { group: "体言", label: "名詞" },
  "代名詞": { group: "体言", label: "代名詞" },
  "数詞": { group: "体言", label: "数詞" },
  "動詞": { group: "用言", label: "動詞" },
  "形容詞": { group: "用言", label: "形容詞" },
  "形状詞": { group: "用言", label: "形容動詞" },
  "形容動詞": { group: "用言", label: "形容動詞" },
  "形動": { group: "用言", label: "形容動詞" },
  "副詞": { group: "独立語", label: "副詞" },
  "連体詞": { group: "独立語", label: "連体詞" },
  "接続詞": { group: "独立語", label: "接続詞" },
  "感動詞": { group: "独立語", label: "感動詞" },
  "助詞": { group: "付属語", label: "助詞" },
  "助動詞": { group: "付属語", label: "助動詞" },
  "接頭辞": { group: "接辞", label: "接頭辞" },
  "接尾辞": { group: "接辞", label: "接尾辞" },
  "記号": { group: "その他", label: "記号" },
  "補助記号": { group: "その他", label: "補助記号" },
  "空白": { group: "その他", label: "空白" },
  "その他": { group: "その他", label: "その他" },
};

const POS_SUBLABELS: Record<string, Record<string, string>> = {
  "格助詞": { label: "格助詞" },
  "接続助詞": { label: "接続助詞" },
  "副助詞": { label: "副助詞" },
  "係助詞": { label: "係助詞" },
  "終助詞": { label: "終助詞" },
  "準体助詞": { label: "準体助詞" },
  "並立助詞": { label: "並立助詞" },
  "間投助詞": { label: "間投助詞" },
  "ナイ": { label: "ない" },
  "タ": { label: "た" },
  "ダ": { label: "だ" },
  "マス": { label: "ます" },
  "タイ": { label: "たい" },
  "セル": { label: "せる" },
  "サセル": { label: "させる" },
  "レル": { label: "れる" },
  "ラレル": { label: "られる" },
  "ウ": { label: "う・よう" },
  "デス": { label: "です" },
  "ベシ": { label: "べし" },
  "ナリ": { label: "なり" },
  "リ": { label: "り" },
};

function normalizeConjugationType(value: string) {
  const type = value.trim().replace(/^[^/]*\//, "");
  if (!type) return "";
  if (/サ行変格|サ変/.test(type)) return "サ変";
  if (/カ行変格|カ変/.test(type)) return "カ変";
  if (/五段/.test(type)) return "五段";
  if (/四段/.test(type)) return "四段";
  if (/下一段/.test(type)) return "下一段";
  if (/上一段/.test(type)) return "上一段";
  if (/上二段/.test(type)) return "上二段";
  if (/下二段/.test(type)) return "下二段";
  if (/形容動詞|ナ形/.test(type)) return "ナ形";
  if (/形容詞|イ形/.test(type)) return "イ形";
  const auxiliary = type.replace(/^助動詞[-・]/, "");
  if (POS_SUBLABELS[auxiliary]) return POS_SUBLABELS[auxiliary].label;
  return type.replace(/-/g, "·");
}

function tagParts(label: string) {
  const normalized = label.trim().replace(/^文語\s*/, "");
  const verb = normalized.match(/^動(?:詞)?\s*(.+)$/);
  if (verb) return { major: "動詞", detail: normalizeConjugationType(verb[1]) };
  const major = Object.keys(POS_LABELS).find((candidate) => normalized === candidate || normalized.startsWith(`${candidate}-`));
  if (major) {
    const subtype = normalized.slice(major.length).replace(/^-/, "");
    return { major, detail: POS_SUBLABELS[subtype]?.label ?? "" };
  }
  if (/^[サカ]変$/.test(normalized)) return { major: "動詞", detail: normalizeConjugationType(normalized) };
  return null;
}

export function posDisplayLabel(pos: PosTag | null | undefined, conjugationType = "", dictionaryTags: readonly (DictionaryTag | string)[] = []) {
  const sourceMajor = pos?.major && POS_LABELS[pos.major] ? pos.major : "";
  const parsed = dictionaryTags
    .map((tag) => typeof tag === "string" ? tag : tag.label)
    .map(tagParts)
    .find((item) => item && (!sourceMajor || item.major === sourceMajor));
  const major = sourceMajor || parsed?.major || "その他";
  const displayMajor = major === "名詞" && pos?.sub1 === "数詞" ? "数詞" : major;
  const base = POS_LABELS[displayMajor] ?? POS_LABELS["その他"];
  let detail = ["動詞", "形容詞", "形状詞", "形容動詞", "形動", "助動詞"].includes(major)
    ? normalizeConjugationType(conjugationType)
    : "";
  if (!detail && ["形状詞", "形容動詞", "形動"].includes(major)) detail = "ナ形";
  if (!detail && pos?.sub1) detail = POS_SUBLABELS[pos.sub1]?.label ?? "";
  if (!detail) detail = parsed?.detail ?? "";
  return [base.group, base.label, detail].filter(Boolean).join(" · ");
}

export function morphologyLemma(chain: MorphologyChain) {
  return chain.lemma_form || chain.dictionary_form || chain.lookup_form || chain.surface_form;
}

export function primaryMorphologyChain(token: AnnotatedToken) {
  const head = token.bunsetsu.head_word;
  const chains = token.bunsetsu.morphology.chains;
  const lexical = chains.filter((chain) => chain.role === "lexical");
  return lexical.find((chain) => chain.surface_form === head.surface)
    ?? lexical.find((chain) => chain.lookup_form === head.base_form)
    ?? lexical.find((chain) => morphologyLemma(chain) === head.base_form)
    ?? chains.find((chain) => chain.lookup_form === head.base_form)
    ?? null;
}

export function morphologyPosLabel(chain: MorphologyChain | null, pos: PosTag) {
  const lemma = chain ? morphologyLemma(chain) : "";
  const inferredType = chain && lemma.endsWith("する") && chain.lookup_form !== lemma
    ? "サ行変格"
    : chain && lemma.endsWith("な") && chain.dictionary_form.endsWith("だ")
      ? "形容動詞"
      : "";
  return posDisplayLabel(pos, inferredType);
}

export function morphologySteps(chain: MorphologyChain | null): MorphologyOperator[] {
  const seen = new Set<string>();
  return (chain?.operators ?? []).filter((operator) => {
    const key = operator.concept_id || operator.output_state;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

export function readingForMorphologyLemma(chain: MorphologyChain | null, reading: string | null | undefined) {
  if (!chain || !reading) return reading ?? null;
  const lemma = morphologyLemma(chain);
  if (!lemma.startsWith(chain.lookup_form)) return reading;
  const suffix = lemma.slice(chain.lookup_form.length);
  if (!suffix) return reading;
  const usesKatakana = /[ァ-ヶ]/u.test(reading);
  const normalizedSuffix = usesKatakana
    ? Array.from(suffix).map((character) => (
        character >= "ぁ" && character <= "ゖ"
          ? String.fromCharCode(character.charCodeAt(0) + 0x60)
          : character
      )).join("")
    : suffix;
  return `${reading}${normalizedSuffix}`;
}
