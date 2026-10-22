import dictionaryUrl from "./lesson-1-1-dictionary.json?url";
import type { DictEntry, DictionaryLookup } from "../types";

type GuideDictionaryPackage = { schema: string; entries: Record<string, DictEntry> };
let dictionaryLoad: Promise<GuideDictionaryPackage> | null = null;

async function loadPackage() {
  dictionaryLoad ??= fetch(dictionaryUrl).then(async (response) => {
    if (!response.ok) throw new Error(`词典快照加载失败：${response.status}`);
    const value = await response.json() as GuideDictionaryPackage;
    if (value.schema !== "readux.grammar-guide-dictionary.v1") throw new Error("语法指南词典快照版本无效");
    return value;
  }).catch((error: unknown) => { dictionaryLoad = null; throw error; });
  return dictionaryLoad;
}

export async function hydrateGuideLookup(lookup: DictionaryLookup, entryKeys: string[]) {
  const dictionary = await loadPackage();
  return {
    ...lookup,
    entries: entryKeys.map((key) => dictionary.entries[key]).filter((entry): entry is DictEntry => Boolean(entry)),
  };
}

export async function guideLinkedLookup(target: string): Promise<DictionaryLookup | null> {
  const dictionary = await loadPackage();
  const entries = Object.values(dictionary.entries).filter((entry) => entry.entry_key === target || entry.headword === target);
  if (!entries.length) return null;
  return { query: entries[0].headword, reading: entries[0].reading, mode: "static-link", forms: [], dictionary_names: [...new Set(entries.map((entry) => entry.dict_name))], entries };
}
