export interface FindMatch {
  start: number;
  end: number;
}

export function findTextMatches(text: string, query: string): FindMatch[] {
  const characters = Array.from(text);
  const needle = Array.from(query);
  if (!needle.length || needle.length > characters.length) return [];
  const matches: FindMatch[] = [];
  for (let start = 0; start <= characters.length - needle.length; start++) {
    if (needle.every((character, offset) => characters[start + offset] === character)) {
      matches.push({ start, end: start + needle.length });
    }
  }
  return matches;
}
