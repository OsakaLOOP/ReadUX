import type { AnnotatedToken, Morpheme, MorphologyChain, PosTag } from "../types";
import {
  morphologyLemma,
  readingForMorphologyLemma,
} from "../explanation/morphologyView";

export interface MorphemeLookupTarget {
  chain: MorphologyChain | null;
  surface: string;
  lemma: string;
  query: string;
  reading: string;
  lookupReading: string;
  pos: PosTag;
  charRange: [number, number];
}

export function morphemeLookupTarget(token: AnnotatedToken, morpheme: Morpheme): MorphemeLookupTarget {
  const chain = token.bunsetsu.morphology.chains.find((candidate) =>
    candidate.source_ranges.some(([start, end]) => start <= morpheme.char_range[0] && morpheme.char_range[1] <= end),
  ) ?? null;
  if (!chain) {
    const lemma = dictionaryLemma(morpheme);
    return {
      chain: null,
      surface: morpheme.surface,
      lemma,
      query: lemma,
      reading: morpheme.reading,
      lookupReading: morpheme.reading,
      pos: morpheme.pos,
      charRange: morpheme.char_range,
    };
  }
  return {
    chain,
    surface: chain.surface_form,
    lemma: morphologyLemma(chain),
    query: chain.lookup_form || chain.dictionary_form,
    reading: readingForMorphologyLemma(chain, morpheme.reading) ?? morpheme.reading,
    lookupReading: token.bunsetsu.morphemes[chain.anchor_morpheme]?.reading ?? morpheme.reading,
    pos: token.bunsetsu.morphemes[chain.anchor_morpheme]?.pos ?? morpheme.pos,
    charRange: chain.char_range,
  };
}

export function dictionaryLemma(morpheme: Morpheme): string {
  if (
    morpheme.pos.major === "助詞"
    || morpheme.pos.major === "助動詞"
    || morpheme.pos.major === "動詞" && morpheme.pos.sub1 === "接尾"
  ) {
    return morpheme.surface;
  }
  return morpheme.base_form && morpheme.base_form !== "*" ? morpheme.base_form : morpheme.surface;
}

export function dictionaryTargetForToken(token: AnnotatedToken) {
  const lexical = token.bunsetsu.lexical_units[0];
  if (lexical) return { word: lexical.base_form, reading: lexical.reading };

  const formation = token.bunsetsu.word_formations[0];
  const morpheme = formation && token.bunsetsu.morphemes[formation.head_morpheme];
  if (morpheme) return { word: dictionaryLemma(morpheme), reading: morpheme.reading };

  return {
    word: dictionaryLemma({
      surface: token.bunsetsu.head_word.surface,
      base_form: token.bunsetsu.head_word.base_form,
      reading: token.bunsetsu.head_word.reading,
      pos: token.bunsetsu.head_word.pos,
      conjugation_type: "*",
      conjugation_form: "*",
      char_range: token.bunsetsu.char_range,
    }),
    reading: token.bunsetsu.head_word.reading,
  };
}
