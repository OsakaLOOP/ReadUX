import type { DictionaryLookup, DictionaryLookupRequest } from "../types";
import type { UnifiedDocument } from "../types/nlp";
import type { LookupGroup } from "../reader/capsuleProjection";

export interface GuideReaderSnapshot {
  document: UnifiedDocument;
  group: LookupGroup;
  ruby: { char_range: [number, number]; base: string; reading: string }[];
  lookups: Record<string, Record<string, { result: DictionaryLookup; entryKeys: string[] }>>;
  particleRefs: Record<string, string>;
}

export interface GuideLookupSnapshot {
  request: DictionaryLookupRequest;
  result: DictionaryLookup | null;
  entryKeys?: string[];
  dictionaryId?: string;
}
