/** 测量指南和小说的静态资源；使用正式阅读投影核对精简数据。 */
import { readFileSync, writeFileSync, mkdirSync, readdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { build } from 'esbuild';
import { gzipSync, brotliCompressSync, constants } from 'node:zlib';
import assert from 'node:assert/strict';

const root = resolve(import.meta.dirname, '..');
const out = resolve(root, '.agents/analysis/static-storage');
mkdirSync(out, { recursive: true });
const bundle = await build({
  stdin: { contents: `export { compileReaderDocument } from './src/reader/document.ts';
    export { projectReaderSentence } from './src/reader/capsuleProjection.ts';
    export { readerChains, kanjiRubySegments } from './src/reader/lookupPresentation.ts';
    export { grammarMorphemeLookup, chainGrammarLookups } from './src/grammar/particleEntries.ts';`,
    resolveDir: root, loader: 'ts' },
  bundle: true, write: false, platform: 'node', format: 'esm',
});
const api = await import('data:text/javascript;base64,' + Buffer.from(bundle.outputFiles[0].text).toString('base64'));
const json = value => Buffer.from(JSON.stringify(value));
const sizes = value => {
  const bytes = Buffer.isBuffer(value) ? value : json(value);
  return { raw: bytes.length, gzip9: gzipSync(bytes, { level: 9 }).length,
    brotli9: brotliCompressSync(bytes, { params: { [constants.BROTLI_PARAM_QUALITY]: 9 } }).length };
};
if (process.argv.includes('--prepare')) {
  const books = JSON.parse(readFileSync(resolve(out, 'library.json'), 'utf8'));
  for (const book of books) {
    const markdown = readFileSync(book.path, 'utf8');
    const document = api.compileReaderDocument(markdown);
    book.text = document.analysisText;
    book.characters = Array.from(book.text).length;
    book.markdownSizes = sizes(Buffer.from(markdown));
    book.textSizes = sizes(Buffer.from(book.text));
    book.layoutSizes = sizes({ blocks: document.blocks, chapters: document.chapters, images: document.images });
  }
  writeFileSync(resolve(out, 'books.json'), JSON.stringify(books));
  process.exit(0);
}

const targetKeys = ['id', 'parent_outer_id', 'char_range', 'surface', 'morpheme_ids', 'lexical_core_ids',
  'source_formation_ids', 'lookup_forms', 'reading_evidence', 'decision', 'reason', 'matrix_request'];
const pick = (value, keys) => Object.fromEntries(keys.filter(k => k in value).map(k => [k, value[k]]));
function runtime(snapshot) {
  const d = snapshot.document;
  const group = snapshot.group ?? snapshot.lookup;
  return { document: pick(d, ['text', 'morphemes', 'morphology', 'formation', 'grammar', 'expression', 'author_ruby']),
    group: { ...pick(group, ['coverage_status', 'grammar_targets', 'excluded_ranges']),
      outer_targets: group.outer_targets.map(t => pick(t, targetKeys)),
      inner_targets: group.inner_targets.map(t => pick(t, targetKeys)) },
    ruby: snapshot.ruby ?? d.morphemes.filter(m => m.reading && /[一-龯々〆ヵヶ]/u.test(m.surface))
      .map(m => ({ char_range: m.char_range, base: m.surface, reading: m.reading })) };
}
let checked = 0;
function displayPayload(value) {
  const targets = [...value.group.outer_targets, ...value.group.inner_targets];
  const details = [], annotations = [], parts = [];
  const intern = (values, item) => {
    if (item == null) return -1;
    const key = JSON.stringify(item);
    let index = values.findIndex(v => JSON.stringify(v) === key);
    if (index < 0) { index = values.length; values.push(item); }
    return index;
  };
  const projection = api.projectReaderSentence(value.document, value.group, value.ruby.flatMap(api.kanjiRubySegments));
  const capsules = projection.map(c => {
    const indexes = new Map();
    for (const p of c.parts) {
      indexes.set(p, parts.length);
      parts.push([p.range[0], p.range[1], p.capsuleKey, p.target ? targets.indexOf(p.target) : -1,
        intern(details, p.detail), intern(annotations, p.annotation)]);
    }
    return [c.key, c.capsuleKey, c.parts.map(p => indexes.get(p)),
      c.runs.map(r => [r.key, r.reading ?? null, r.parts.map(p => indexes.get(p))])];
  });
  const chars = Array.from(value.document.text);
  const restoredParts = parts.map(p => ({ key: `${p[0]}:${p[1]}`, text: chars.slice(p[0], p[1]).join(''),
    range: p.slice(0, 2), capsuleKey: p[2], target: targets[p[3]] ?? null,
    detail: details[p[4]] ?? null, annotation: annotations[p[5]] }));
  const restored = capsules.map(c => ({ key: c[0], capsuleKey: c[1], parts: c[2].map(i => restoredParts[i]),
    runs: c[3].map(r => ({ key: r[0], reading: r[1] ?? undefined, parts: r[2].map(i => restoredParts[i]) })) }));
  assert.deepEqual(restored, projection);
  const grammarKeys = lookups => [...new Set(lookups.flatMap(l => l?.entries.map(e => e.entry_key) ?? []))];
  return { text: value.document.text, targets, outerCount: value.group.outer_targets.length,
    grammarTargets: value.group.grammar_targets, grammar: value.document.grammar, expression: value.document.expression,
    morphemes: value.document.morphemes, authorRuby: value.document.author_ruby,
    capsules, parts, details, annotations,
    chains: targets.map(t => api.readerChains(t, value.document)),
    grammarQueries: projection.flatMap(c => c.parts).map(p => p.target
      ? grammarKeys(api.chainGrammarLookups(p.target, value.document, { detailId: p.detail?.id, range: p.range })) : []),
    morphemeGrammarQueries: value.document.morphemes.map(m => grammarKeys([api.grammarMorphemeLookup(m, value.document)])) };
}
function verify(snapshot, compact) {
  const originalGroup = snapshot.group ?? snapshot.lookup;
  const ruby = compact.ruby.flatMap(api.kanjiRubySegments);
  // 目标原对象含查询诊断；比较显示结果时统一为相同的目标协议。
  const normalize = projection => JSON.parse(JSON.stringify(projection, (key, value) =>
    key === 'target' && value ? pick(value, targetKeys) : value));
  assert.deepEqual(normalize(api.projectReaderSentence(snapshot.document, originalGroup, ruby)),
    normalize(api.projectReaderSentence(compact.document, compact.group, ruby)));
  for (const target of [...compact.group.outer_targets, ...compact.group.inner_targets]) {
    assert.deepEqual(api.readerChains(target, snapshot.document), api.readerChains(target, compact.document));
    assert.deepEqual(api.chainGrammarLookups(target, snapshot.document), api.chainGrammarLookups(target, compact.document));
  }
  for (const morpheme of snapshot.document.morphemes) {
    assert.deepEqual(api.grammarMorphemeLookup(morpheme, snapshot.document), api.grammarMorphemeLookup(morpheme, compact.document));
  }
  checked++;
}
const report = { units: 'bytes', compression: 'gzip level 9 / Brotli quality 9', guides: [], novels: [], shared: [], frontend: [] };
const sharedDictionary = {};
for (const lesson of ['1-1', '2-1']) {
  const stem = `lesson-${lesson}`;
  const content = JSON.parse(readFileSync(resolve(root, 'src/grammar-guide', stem + '.json'), 'utf8'));
  const analysis = JSON.parse(readFileSync(resolve(root, 'src/grammar-guide', stem + '-analysis.json'), 'utf8'));
  const examples = [...content.examples, ...content.originalExamples];
  const compact = Object.fromEntries(Object.entries(analysis.examples).map(([id, snapshot]) => {
    const value = runtime(snapshot); verify(snapshot, value); return [id, value];
  }));
  writeFileSync(resolve(out, stem + '-runtime.json'), json(compact));
  const files = ['', '-analysis', '-dictionary'].map(suffix => {
    const name = stem + suffix + '.json';
    const bytes = readFileSync(resolve(root, 'src/grammar-guide', name));
    return { name, ...sizes(bytes), minified: sizes(JSON.parse(bytes)) };
  });
  const components = {};
  for (const field of ['document', 'group', 'lookups', 'particleRefs', 'providers', 'ruby'])
    components[field] = sizes(Object.fromEntries(Object.entries(analysis.examples).map(([id, s]) => [id, s[field]])));
  const documents = {};
  for (const field of Object.keys(Object.values(analysis.examples)[0].document))
    documents[field] = json(Object.values(analysis.examples).map(s => s.document[field])).length;
  const chunked = { raw: 0, gzip9: 0, brotli9: 0 };
  for (const value of Object.values(compact)) for (const [key, size] of Object.entries(sizes(value))) chunked[key] += size;
  const display = Object.fromEntries(Object.entries(compact).map(([id, value]) => [id, displayPayload(value)]));
  const entries = JSON.parse(readFileSync(resolve(root, 'src/grammar-guide', stem + '-dictionary.json'))).entries;
  Object.assign(sharedDictionary, entries);
  report.guides.push({ lesson, examples: examples.length,
    exampleCharacters: examples.reduce((n, e) => n + Array.from(e.ja).length, 0),
    sourceCharacters: Array.from(content.sourceMarkdown).length, files, components, documents,
    runtime: sizes(compact), display: sizes(display), runtimeOneExamplePerFile: chunked,
    dictionaryEntries: Object.keys(entries).length });
}
const dictionaryChunks = [];
let dictionaryChunk = {}, chunkBytes = 0;
for (const [key, value] of Object.entries(sharedDictionary)) {
  const bytes = json(value).length;
  if (chunkBytes && chunkBytes + bytes > 65536) {
    dictionaryChunks.push(dictionaryChunk); dictionaryChunk = {}; chunkBytes = 0;
  }
  dictionaryChunk[key] = value; chunkBytes += bytes;
}
if (chunkBytes) dictionaryChunks.push(dictionaryChunk);
report.sharedGuideDictionary = { entries: Object.keys(sharedDictionary).length, ...sizes(sharedDictionary),
  chunkCount: dictionaryChunks.length, chunkBrotli9: dictionaryChunks.reduce((n, chunk) => n + sizes(chunk).brotli9, 0),
  chunkMaxBrotli9: Math.max(...dictionaryChunks.map(chunk => sizes(chunk).brotli9)) };
const selected = JSON.parse(readFileSync(resolve(out, 'selected.json'), 'utf8'));
for (const book of selected) {
  const compact = [];
  const display = [];
  let originalBytes = 0;
  for (const path of book.snapshots) {
    const snapshot = JSON.parse(readFileSync(path, 'utf8'));
    const value = runtime(snapshot); verify(snapshot, value); compact.push(value);
    display.push(displayPayload(value));
    originalBytes += json(snapshot).length;
  }
  const chunks = {};
  for (const count of [1, 5, 10]) {
    const totals = { raw: 0, gzip9: 0, brotli9: 0 };
    for (let i = 0; i < compact.length; i += count)
      for (const [key, size] of Object.entries(sizes(compact.slice(i, i + count)))) totals[key] += size;
    chunks[count] = totals;
  }
  const name = book.id + '-runtime.json';
  writeFileSync(resolve(out, name), json(compact));
  const displayName = book.id + '-display.json';
  writeFileSync(resolve(out, displayName), json(display));
  const displayChunks = {};
  for (const count of [1, 5, 10]) {
    const totals = { raw: 0, gzip9: 0, brotli9: 0 };
    for (let i = 0; i < display.length; i += count)
      for (const [key, size] of Object.entries(sizes(display.slice(i, i + count)))) totals[key] += size;
    displayChunks[count] = totals;
  }
  report.novels.push({ ...book, snapshots: undefined, originalJsonBytes: originalBytes,
    runtimeFile: name, runtime: sizes(compact), chunks, displayFile: displayName, display: sizes(display), displayChunks });
}
for (const name of ['configs/grammar/particles.json', 'configs/grammar/auxiliaries.json',
  'configs/grammar/migrations/te_construction_ids.json', 'configs/unidic-nlp/morphology.json',
  'crates/readux-core/resources/grammar/compiled/grammar_catalog.json',
  'crates/readux-core/resources/grammar/compiled/grammar_explanations.json',
  'crates/readux-core/resources/grammar/compiled/grammar_search_index.json'])
  report.shared.push({ name, ...sizes(readFileSync(resolve(root, name))) });
for (const name of readdirSync(resolve(root, 'dist/assets')).filter(n => /\.(js|css)$/.test(n)))
  report.frontend.push({ name, ...sizes(readFileSync(resolve(root, 'dist/assets', name))) });
report.checkedSnapshots = checked;
writeFileSync(resolve(out, 'node-results.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ checkedSnapshots: checked, guides: report.guides.map(g => ({ lesson: g.lesson, runtime: g.runtime })),
  novels: report.novels.map(b => ({ title: b.title, characters: b.sampleCharacters, runtime: b.runtime })) }, null, 2));
