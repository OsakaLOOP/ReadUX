import type { DictionaryLookup, DictionaryLookupRequest } from "../types";

export interface GuideLookupSnapshot {
  request: DictionaryLookupRequest;
  result: DictionaryLookup | null;
}
