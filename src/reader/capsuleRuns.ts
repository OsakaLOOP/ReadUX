export interface CapsuleActiveRange {
  range: [number, number];
  tone: "blue" | "grammar";
  group?: string;
}

export function stemEndingHighlights(
  parts: { range: [number, number]; role?: "stem" | "grammar" }[],
  endingRange: [number, number],
): CapsuleActiveRange[] | null {
  const ordered = [...parts].sort((left, right) => left.range[0] - right.range[0]);
  const stem = ordered.filter((part) => part.role === "stem" && part.range[1] <= endingRange[0])
    .sort((left, right) => right.range[1] - left.range[1])[0];
  if (!stem) return null;
  let stemStart = stem.range[0];
  const stemEnd = stem.range[1];
  for (const part of [...ordered].reverse()) {
    if (part.range[1] === stemStart && part.role === "stem") stemStart = part.range[0];
  }
  const result: CapsuleActiveRange[] = [{ range: [stemStart, stemEnd], tone: "blue" }];
  let end = stemEnd;
  for (const part of ordered) {
    if (part.range[1] <= end) continue;
    if (part.range[0] !== end || part.role !== "grammar") break;
    result.push({ range: part.range, tone: "grammar" });
    end = part.range[1];
    if (end >= endingRange[1]) return result;
  }
  return null;
}

export function groupCapsuleRuns<T extends { key: string; text: string }>(
  runs: { key: string; reading?: string; parts: T[] }[],
  activeRanges: CapsuleActiveRange[] = [],
) {
  type Atom = { part: T; run: (typeof runs)[number]; active: boolean; activeGroup?: string };
  const atoms: Atom[] = runs.flatMap((run) => run.parts.map((part) => {
    const range = (part as T & { range?: [number, number] }).range;
    const active = range && activeRanges.find((candidate) => range[0] < candidate.range[1] && range[1] > candidate.range[0]);
    return { part, run, active: Boolean(active), activeGroup: active ? active.group ?? `${active.range[0]}:${active.range[1]}` : undefined };
  }));
  const result: { key: string; active: boolean; activeGroup?: string; runs: { key: string; reading?: string; parts: T[] }[] }[] = [];
  for (const atom of atoms) {
    let group = result[result.length - 1];
    if (!group || group.active !== atom.active || group.activeGroup !== atom.activeGroup) {
      group = { key: atom.part.key, active: atom.active, activeGroup: atom.activeGroup, runs: [] };
      result.push(group);
    }
    let run = group.runs[group.runs.length - 1];
    if (!run || run.key !== atom.run.key) {
      run = { key: atom.run.key, reading: atom.run.reading, parts: [] };
      group.runs.push(run);
    }
    run.parts.push(atom.part);
  }
  return result;
}
