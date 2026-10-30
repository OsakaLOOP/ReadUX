/** 检查图书分片、查询引用与平台文件限制。 */
import assert from 'node:assert/strict';
import { readFileSync, readdirSync, statSync, mkdirSync, writeFileSync } from 'node:fs';
import { gunzipSync } from 'node:zlib';
import { build } from 'esbuild';

const read = path => JSON.parse((path.endsWith('.gz') ? gunzipSync(readFileSync('dist/' + path)) : readFileSync('dist/' + path)).toString('utf8'));
const files = path => readdirSync(path, { withFileTypes: true }).flatMap(entry => entry.isDirectory() ? files(path + '/' + entry.name) : [path + '/' + entry.name]);
const allFiles = files('dist');
assert.ok(allFiles.length <= 20000);
for (const file of allFiles) assert.ok(statSync(file).size < 25000000, file);
const bundle = await build({ stdin: { contents: "export { projectReaderSentence } from './src/reader/capsuleProjection.ts'; export { compileReaderDocument } from './src/reader/document.ts';", resolveDir: process.cwd(), loader: 'ts' }, bundle: true, write: false, platform: 'node', format: 'esm' });
const api = await import('data:text/javascript;base64,' + Buffer.from(bundle.outputFiles[0].text).toString('base64'));
const summary = read('library/catalog.json').books[0];
const base = summary.manifest.slice(0, summary.manifest.lastIndexOf('/') + 1);
const manifest = read(summary.manifest), plan = read(base + manifest.plan), index = read(base + manifest.index);
const book = api.compileReaderDocument(read(base + manifest.book).markdown);
assert.ok(plan.prepared.mapping.source_text === book.analysisText, '生成正文应与正式阅读器一致');
const ids = new Set(), queries = new Set(), grammar = {}, units = new Map();
for (const file of new Set(Object.values(index.contents))) for (const [key, value] of Object.entries(read(base + file))) {
  ids.add(key);
  if (key.startsWith('g')) grammar[key] = value;
}
for (const file of new Set(Object.values(index.queries))) for (const [key, variants] of Object.entries(read(base + file))) {
  queries.add(key);
  for (const form of variants.default.result.forms) assert.ok(variants[form.form_id]);
  for (const saved of Object.values(variants)) for (const id of saved.entryKeys) assert.ok(ids.has(id), id);
}
for (const keys of Object.values(index.words)) for (const id of keys) assert.ok(ids.has(id), id);
function hydrate(value) {
  if (!value || typeof value !== 'object') return value;
  if ('$content' in value) { assert.ok(ids.has(value.$content)); return grammar[value.$content]; }
  return Array.isArray(value) ? value.map(hydrate) : Object.fromEntries(Object.entries(value).map(([key, item]) => [key, hydrate(item)]));
}
for (const file of new Set(Object.values(manifest.chunks))) for (const unit of hydrate(read(base + file))) {
  assert.equal(unit.stage, 'complete');
  assert.equal(unit.analysis_complete, true);
  assert.ok(unit.document.source.tokens.length);
  assert.ok(Array.isArray(unit.document.ruby_validations));
  for (const qid of Object.values(unit.queries)) assert.ok(queries.has(qid), qid);
  const projection = api.projectReaderSentence(unit.document, unit.lookup, unit.document.author_ruby);
  assert.equal(projection.flatMap(capsule => capsule.parts).map(part => part.text).join(''), unit.document.text);
  units.set(unit.unit_id, unit);
}
let end = 0;
for (const unit of plan.units) {
  assert.equal(unit.anchor.char_range[0], end);
  assert.equal(units.get(unit.id).document.text, Array.from(plan.prepared.text).slice(...unit.context_range).join(''));
  end = unit.anchor.char_range[1];
}
assert.equal(end, Array.from(plan.prepared.text).length);
assert.equal(units.size, plan.units.length);
const report = { characters: end, units: units.size, chunks: new Set(Object.values(manifest.chunks)).size, queries: queries.size, contents: ids.size, files: allFiles.length, bytes: allFiles.reduce((sum, file) => sum + statSync(file).size, 0) };
mkdirSync('.agents/analysis/static-library', { recursive: true });
writeFileSync('.agents/analysis/static-library/verification.json', JSON.stringify(report, null, 2) + '\n', 'utf8');
console.log(JSON.stringify(report));
