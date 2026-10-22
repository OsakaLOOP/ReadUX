import resource from "../../configs/grammar/particles.json";
import auxiliaries from "../../configs/grammar/auxiliaries.json";
import type { DictEntry, DictionaryLookup, DictionaryLookupRequest, DictionarySense, PosTag } from "../types";
import type { MorphemeToken, UnifiedDocument } from "../types/nlp";
import { readerCapsuleRanges } from "../reader/lookupPresentation";

type ParticleResource = {
  entries: {
    id: string; headword: string; reading: string; pronunciation: string;
    variants: { surface: string; subtype: string; note: string }[];
    senses: { id: string; subtype: string; title: string; meaning: string; connection: string; examples: { ja: string; zh: string }[] }[];
  }[];
};
const data = resource as ParticleResource;
const auxiliaryData = auxiliaries as ParticleResource;
const fallbackPos: PosTag = { major: "助詞", sub1: "*", sub2: "*", sub3: "*" };

function makeSense(item: ParticleResource["entries"][number]["senses"][number]): DictionarySense {
  return {
    sense_id: item.id, marker: null, heading: item.title,
    glosses: [{ lang: "zh", html: item.meaning }], gloss_groups: [], definitions: [],
    tags: [],
    examples: item.examples.map((example) => ({ source: { lang: "ja", html: example.ja }, translation: { lang: "zh", html: example.zh }, note: null })),
    notes: [{ lang: "zh", html: `接续：${item.connection}` }], relations: [], children: [],
  };
}

function makeEntry(entry: ParticleResource["entries"][number], request: DictionaryLookupRequest, subtype: string): DictEntry {
  const key = `ReadUX内置语法词条\u001f${entry.id}`;
  const selected = entry.senses.filter((item) => item.subtype === subtype);
  return {
    entry_key: key, dict_name: "ReadUX 内置语法词条", headword: entry.headword, reading: entry.reading, is_preferred: true,
    definition_html: `<p>${request.word}（${entry.pronunciation}）</p>`, style_profile: "grammar", content_blocks: [],
    match_type: "exact_form", links: [], occurrence_id: key, source_record_index: 0, entry_kind: request.pos?.major === "助動詞" ? "grammar_auxiliary" : "grammar_particle",
    header: { display_form: entry.headword, canonical_form: entry.headword, reading: entry.reading, pronunciations: [],
      scoped_forms: entry.variants.map((item) => ({ form: item.surface, kind: item.subtype, reading: item.surface })),
      pos_tags: [{ kind: "pos", label: request.pos?.major ?? "助詞" }], usage_tags: [], origin: "readux-authored-grammar", short_note: null },
    senses: (selected.length ? selected : entry.senses).map(makeSense), sections: [],
    adapter_diagnostics: { coverage: "authored", warnings: [], omitted: [] },
    match_evidence: { kind: "builtin_particle", query_form: request.word, matched_form: request.word, requested_reading: request.reading ?? null,
      reading_match: "exact", pos_match: "exact", dictionary_local: true, penalties: [], score: 1 },
    raw_definition: null, content_loaded: true, has_definition: true,
  };
}

export function particleLookup(request: DictionaryLookupRequest, lemma = request.word): DictionaryLookup | null {
  if (!["助詞", "助動詞"].includes(request.pos?.major ?? "")) return null;
  const library = request.pos?.major === "助動詞" ? auxiliaryData : data;
  const entry = library.entries.find((entry) => entry.headword === lemma
    && request.pos?.major === "助動詞") ?? library.entries.find((entry) => entry.headword === request.word
    && entry.senses.some((sense) => sense.subtype === request.pos?.sub1))
    ?? library.entries.find((entry) => entry.variants.some((variant) => variant.surface === request.word && variant.subtype === request.pos?.sub1));
  if (!entry) return null;
  const subtype = request.pos?.major === "助動詞" ? "助動詞" : request.pos?.sub1 ?? "*";
  return {
    query: request.word, observed_form: request.observedForm ?? request.word, reading: request.reading ?? entry.reading,
    pos: { ...fallbackPos, ...request.pos, sub1: subtype }, mode: "builtin-grammar",
    forms: [{ form_id: `particle:${entry.id}`, display_form: entry.headword, normalized_form: entry.headword, readings: [entry.reading],
      evidence: ["builtin-grammar-entry"], score: 1, variants: [], dictionaries: [{ dictionary_name: "ReadUX 内置语法词条", available: true }] }],
    dictionary_names: ["ReadUX 内置语法词条"], entries: [makeEntry(entry, request, subtype)],
  };
}

export function grammarMorphemeLookup(morpheme: MorphemeToken): DictionaryLookup | null {
  if (morpheme.pos[0] === "動詞" && morpheme.pos[1] === "非自立可能" && morpheme.lemma === "居る") {
    return particleLookup({ word: "いる", observedForm: morpheme.surface, reading: morpheme.reading ?? undefined,
      pos: { major: "助動詞", sub1: "助動詞", sub2: "*", sub3: "*" },
    }, "いる");
  }
  return particleLookup({ word: morpheme.surface, observedForm: morpheme.surface, reading: morpheme.reading ?? undefined,
    pos: { major: morpheme.pos[0] ?? "*", sub1: morpheme.pos[1] ?? "*", sub2: morpheme.pos[2] ?? "*", sub3: morpheme.pos[3] ?? "*" },
  }, morpheme.lemma ?? morpheme.surface);
}

function selectedMorpheme(morphemes: MorphemeToken[], selectedRange: [number, number]) {
  const overlapping = morphemes.filter((item) => selectedRange[0] < item.char_range[1]
    && selectedRange[1] > item.char_range[0]);
  return overlapping.find((item) => item.char_range[0] === selectedRange[0]
    && item.char_range[1] === selectedRange[1])
    ?? overlapping.find((item) => selectedRange[0] >= item.char_range[0]
      && selectedRange[1] <= item.char_range[1])
    ?? overlapping[overlapping.length - 1];
}

function teIruLookups(document: UnifiedDocument, selectedRange: [number, number]): DictionaryLookup[] | null {
  const morphemes = document.morphemes;
  for (let index = 0; index < morphemes.length; index += 1) {
    const iru = morphemes[index];
    if (iru.pos[0] !== "動詞" || iru.pos[1] !== "非自立可能" || iru.lemma !== "居る") continue;
    const te = morphemes[index - 1];
    if (!te || !["て", "で"].includes(te.surface) || te.pos[0] !== "助詞" || te.pos[1] !== "接続助詞"
      || te.char_range[1] !== iru.char_range[0]) continue;
    const selectsChain = selectedRange[0] < iru.char_range[1] && selectedRange[1] > te.char_range[0];
    if (!selectsChain) continue;
    const iruIndex = morphemes.indexOf(iru);
    const chain = document.morphology.chains.find((item) => item.core_morpheme_indices.includes(iruIndex));
    const hasAspectLink = chain?.operators.some((operator) => operator.kind === "te_iru");
    if (!hasAspectLink) continue;
    const surface = `${te.surface}${iru.surface}`;
    const combined = particleLookup({ word: surface, observedForm: surface, reading: surface,
      pos: { major: "助動詞", sub1: "助動詞", sub2: "*", sub3: "*" },
    }, "ている");
    const selected = selectedMorpheme([te, iru], selectedRange);
    const component = selected === te
      ? grammarMorphemeLookup(te)
      : grammarMorphemeLookup(iru);
    return combined ? [combined, ...(component ? [component] : [])] : null;
  }
  return null;
}

function nodeLookups(document: UnifiedDocument, selectedRange: [number, number]): DictionaryLookup[] | null {
  const morphemes = document.morphemes;
  for (let index = 0; index < morphemes.length - 1; index += 1) {
    const no = morphemes[index];
    const de = morphemes[index + 1];
    if (no.surface !== "の" || de.surface !== "で" || no.char_range[1] !== de.char_range[0]) continue;
    if (selectedRange[0] >= de.char_range[1] || selectedRange[1] <= no.char_range[0]) continue;
    const combined = particleLookup({ word: "ので", observedForm: "ので", reading: "ので",
      pos: { major: "助詞", sub1: "接続助詞", sub2: "*", sub3: "*" },
    });
    const selected = selectedMorpheme([no, de], selectedRange);
    const component = selected === no
      ? particleLookup({ word: "の", observedForm: no.surface, reading: no.reading ?? "の",
        pos: { major: "助詞", sub1: "準体助詞", sub2: "*", sub3: "*" },
      })
      : particleLookup({ word: "で", observedForm: de.surface, reading: de.reading ?? "で",
        pos: { major: "助詞", sub1: "接続助詞", sub2: "*", sub3: "*" },
      });
    return combined ? [combined, ...(component ? [component] : [])] : null;
  }
  return null;
}

/** 复合助词链先显示整体词条，再显示当前选中的组成项。 */
export function chainGrammarLookups(
  target: { id: string; char_range: [number, number]; lexical_core_ids: string[]; source_formation_ids: string[] },
  document: UnifiedDocument,
  selection?: { detailId?: string; range?: [number, number] },
): DictionaryLookup[] {
  const detailRange = selection?.detailId
    ? document.morphology.occurrences.find((item) => item.id === selection.detailId)?.hit_ranges[0]
      ?? document.morphology.chains.flatMap((chain) => chain.operators)
        .find((operator) => operator.operator_id === selection.detailId)?.char_range
    : undefined;
  const selectedRange = detailRange ?? selection?.range ?? target.char_range;
  const teIru = teIruLookups(document, selectedRange);
  if (teIru) return teIru;
  const node = nodeLookups(document, selectedRange);
  if (node) return node;

  const selected = document.morphemes.find((item) => selectedRange[0] <= item.char_range[0]
    && item.char_range[1] <= selectedRange[1]);
  const lookup = selected ? grammarMorphemeLookup(selected) : null;
  if (lookup) return [lookup];

  const capsuleRange = readerCapsuleRanges([target], document)[0]?.range ?? target.char_range;
  const related = document.morphemes.filter((item) => capsuleRange[0] <= item.char_range[0]
    && item.char_range[1] <= capsuleRange[1] && selectedRange[0] <= item.char_range[0]
    && item.char_range[1] <= selectedRange[1]).map(grammarMorphemeLookup)
    .find((item): item is DictionaryLookup => Boolean(item));
  return related ? [related] : [];
}
