import type { UnifiedDocument } from "../types/nlp";
import { displayReading, readerCapsuleRanges, readerMorphologyHits, type ReaderMorphologyDetail } from "./lookupPresentation";

export interface LookupTarget {
  id: string; parent_outer_id: string | null; char_range: [number, number]; surface: string;
  morpheme_ids: string[]; lexical_core_ids: string[]; source_formation_ids: string[];
  lookup_forms: { kind: string; form: string; reading: string | null }[];
  reading_evidence: string[]; decision: string; reason: string;
}
export interface LookupGroup {
  outer_targets: LookupTarget[]; inner_targets: LookupTarget[];
  grammar_targets: { char_range: [number, number]; display_form: string; normalized_form: string }[];
  coverage_status: string;
}
export interface CapsulePart {
  key: string; text: string; range: [number, number]; capsuleKey: string | null;
  annotation?: { char_range: [number, number]; reading: string };
}

export function capsulesForParts<T extends CapsulePart>(parts: T[]) {
  const capsules: { key: string; capsuleKey: string | null; parts: T[]; runs: { key: string; reading?: string; parts: T[] }[] }[] = [];
  for (const part of parts) {
    const previous = capsules[capsules.length - 1];
    const last = previous?.parts[previous.parts.length - 1];
    if (previous && last && last.range[1] === part.range[0]
      && (last.capsuleKey === part.capsuleKey || part.annotation && last.annotation === part.annotation)) {
      previous.parts.push(part);
      previous.capsuleKey ??= part.capsuleKey;
    } else capsules.push({ key: part.key, capsuleKey: part.capsuleKey, parts: [part], runs: [] });
  }
  for (const capsule of capsules) {
    const runs: T[][] = [];
    for (const part of capsule.parts) {
      const previous = runs[runs.length - 1];
      if (previous && previous[0].annotation === part.annotation) previous.push(part);
      else runs.push([part]);
    }
    capsule.runs = runs.map((parts) => ({ key: parts[0].key, reading: parts[0].annotation ? displayReading(parts[0].annotation.reading) : undefined, parts }));
  }
  return capsules;
}

export function componentForRange(children: LookupTarget[], range: [number, number]) {
  return children.filter((target) => target.decision === "queryable"
    && target.char_range[0] <= range[0] && range[1] <= target.char_range[1])
    .sort((a, b) => (a.char_range[1] - a.char_range[0]) - (b.char_range[1] - b.char_range[0]))[0] ?? null;
}

export function projectReaderSentence(document: UnifiedDocument, group: LookupGroup, ruby: CapsulePart["annotation"][]) {
  const capsules = readerCapsuleRanges(group.outer_targets, document);
  const details = readerMorphologyHits([...group.outer_targets, ...group.inner_targets], document)
    .sort((left, right) => (left.range[1] - left.range[0]) - (right.range[1] - right.range[0]));
  const boundaries = new Set([0, Array.from(document.text).length]);
  for (const item of [...group.outer_targets, ...group.inner_targets]) item.char_range.forEach((offset) => boundaries.add(offset));
  for (const item of [...capsules, ...details]) item.range.forEach((offset) => boundaries.add(offset));
  for (const item of ruby) item?.char_range.forEach((offset) => boundaries.add(offset));
  const points = [...boundaries].sort((a, b) => a - b);
  const characters = Array.from(document.text);
  const parts = points.slice(0, -1).map((start, index) => {
    const end = points[index + 1];
    const detail = details.find((item) => item.range[0] <= start && end <= item.range[1]);
    const capsule = capsules.find((item) => item.range[0] <= start && end <= item.range[1]);
    const target = group.outer_targets.find((item) => item.id === detail?.targetId)
      ?? group.outer_targets.find((item) => item.char_range[0] <= start && end <= item.char_range[1]) ?? null;
    return {
      key: `${start}:${end}`, text: characters.slice(start, end).join(""), range: [start, end] as [number, number],
      capsuleKey: capsule?.id ?? null, target, detail: detail?.detail ?? null as ReaderMorphologyDetail | null,
      detailRole: detail?.role,
      detailRange: detail?.range,
      annotation: ruby.find((item) => item && item.char_range[0] <= start && end <= item.char_range[1]),
    };
  });
  return capsulesForParts(parts);
}
