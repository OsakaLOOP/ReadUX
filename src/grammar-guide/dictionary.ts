import dictionaryUrl from "./lesson-1-1-dictionary.json?url";
import dictionary21Url from "./lesson-2-1-dictionary.json?url";
import type { DictEntry, DictionaryLookup } from "../types";

type GuideDictionaryPackage = { schema: string; entries: Record<string, DictEntry> };
const dictionaryLoads = new Map<string, Promise<GuideDictionaryPackage>>();

async function loadPackage(lessonId: string) {
  const url = lessonId === "grammar-2.1" ? dictionary21Url : dictionaryUrl;
  let pending = dictionaryLoads.get(lessonId);
  if (!pending) {
    pending = fetch(url).then(async (response) => {
    if (!response.ok) throw new Error("词典快照加载失败：" + response.status);
    const value = await response.json() as GuideDictionaryPackage;
    if (value.schema !== "readux.grammar-guide-dictionary.v1") throw new Error("语法指南词典快照版本无效");
    return value;
    }).catch((error: unknown) => { dictionaryLoads.delete(lessonId); throw error; });
    dictionaryLoads.set(lessonId, pending);
  }
  return pending;
}

export async function hydrateGuideLookup(lookup: DictionaryLookup, entryKeys: string[], lessonId: string) {
  const dictionary = await loadPackage(lessonId);
  return {
    ...lookup,
    entries: entryKeys.map((key) => dictionary.entries[key]).filter((entry): entry is DictEntry => Boolean(entry)),
  };
}

export async function guideLinkedLookup(target: string, lessonId: string): Promise<DictionaryLookup | null> {
  const dictionary = await loadPackage(lessonId);
  const entries = Object.values(dictionary.entries).filter((entry) => entry.entry_key === target || entry.headword === target);
  if (!entries.length) return null;
  return { query: entries[0].headword, reading: entries[0].reading, mode: "static-link", forms: [], dictionary_names: [...new Set(entries.map((entry) => entry.dict_name))], entries };
}
