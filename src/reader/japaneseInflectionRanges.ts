export function volitionalStemAndEnding(surface: string, range: [number, number]) {
  const ending = surface.endsWith("よう") ? "よう" : surface.endsWith("う") ? "う" : "";
  if (!ending) return null;
  const length = Array.from(ending).length;
  const split = range[1] - length;
  if (split <= range[0]) return null;
  return {
    stem: [range[0], split] as [number, number],
    ending: [split, range[1]] as [number, number],
  };
}

/** 依据 UniDic 活用型，将句中词元拆成词干和紧随词干的首个活用假名。 */
export function inflectionStemAndEnding(
  surface: string,
  range: [number, number],
  conjugationType = "",
  conjugationForm = "",
) {
  const volitional = volitionalStemAndEnding(surface, range);
  if (volitional) return volitional;
  const characters = Array.from(surface);
  if (characters.length < 2) return null;
  const adjectiveEnding = conjugationType === "形容詞"
    ? ({ "未然形-一般": 1, "未然形-補助": 1, "連用形-一般": 1, "連用形-促音便": 2,
        "終止形-一般": 1, "連体形-一般": 1, "仮定形-一般": 2, "已然形-一般": 2,
        "命令形-一般": 2 } as Record<string, number>)[conjugationForm] ?? 0
    : 0;
  const endingLength = adjectiveEnding || (/(五段|一段|下一段|上一段|変格)/u.test(conjugationType)
    && /[\p{Script=Hiragana}\p{Script=Katakana}ー]$/u.test(characters[characters.length - 1]) ? 1 : 0);
  if (!endingLength || characters.length <= endingLength) return null;
  return {
    stem: [range[0], range[1] - endingLength] as [number, number],
    ending: [range[1] - endingLength, range[1]] as [number, number],
  };
}
