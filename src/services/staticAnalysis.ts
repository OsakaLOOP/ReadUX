import { isTauri } from '@tauri-apps/api/core';
import type { DictEntry, DictionaryLookup } from '../types';
import type { DocumentPlan, DocumentUpdate, UnitUpdate } from '../reader/session';
import { loadStaticJson, staticBookManifest, type StaticBookManifest } from './staticLibrary';

export const isStaticReader = !isTauri() && (import.meta.env.PROD || import.meta.env.MODE === 'static');
interface StaticUnit extends UnitUpdate { queries: Record<string, string> }
interface LookupSnapshot { result?: Omit<DictionaryLookup, 'entries'>; selectedFormId?: string; entryKeys: string[] }
interface LookupIndex { queries: Record<string, string>; contents: Record<string, string>; words: Record<string, string[]> }
interface StaticSession {
  id: string; base: string; manifest: StaticBookManifest; plan: DocumentPlan;
  revision: number; chunks: Map<string, StaticUnit[]>;
  range: [number, number];
}
let sequence = 0;
const sessions = new Map<string, StaticSession>();

function index(session: StaticSession) {
  return loadStaticJson<LookupIndex>(session.base + session.manifest.index);
}

async function contents(session: StaticSession, keys: string[]): Promise<Record<string, unknown>> {
  if (!keys.length) return {};
  const lookup = await index(session);
  const files = [...new Set(keys.map(key => lookup.contents[key]))];
  const values = await Promise.all(files.map(file => loadStaticJson<Record<string, unknown>>(session.base + file)));
  return Object.assign({}, ...values);
}

async function hydrateGrammar(session: StaticSession, units: StaticUnit[]) {
  const keys = new Set<string>();
  function visit(value: unknown): void {
    if (!value || typeof value !== 'object') return;
    if ('$content' in value) keys.add(String(value.$content));
    else Object.values(value).forEach(visit);
  }
  visit(units);
  const shared = await contents(session, [...keys]);
  function restore(value: unknown): unknown {
    if (!value || typeof value !== 'object') return value;
    if ('$content' in value) return shared[String(value.$content)];
    if (Array.isArray(value)) return value.map(restore);
    return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, restore(item)]));
  }
  return restore(units) as StaticUnit[];
}

function snapshot(session: StaticSession): DocumentUpdate {
  const changes = [...session.chunks.values()].flat();
  const loaded = changes.length;
  return {
    schema: 'readux.document-update.v1', session_id: session.id, document_id: session.plan.id,
    text_version: session.plan.text_version, generation: 1, base_revision: null,
    revision: session.revision, snapshot: true, paused: false,
    progress: { total: session.plan.units.length, basic: loaded, complete: session.plan.units.length,
      failed: 0, pending: 0, analysis: { complete: 0, total: 0 }, cache: { complete: loaded, total: loaded } },
    changes,
  };
}

async function loadRange(session: StaticSession, range: [number, number]) {
  session.range = range;
  const visible = session.plan.units.filter(unit => unit.anchor.char_range[1] > range[0] && unit.anchor.char_range[0] < range[1]);
  const files = [...new Set(visible.map(unit => session.manifest.chunks[unit.id]))];
  for (const file of files) {
    let units = session.chunks.get(file);
    if (!units) units = await hydrateGrammar(session, await loadStaticJson<StaticUnit[]>(session.base + file));
    session.chunks.delete(file);
    session.chunks.set(file, units);
  }
  while (session.chunks.size > Math.max(4, files.length)) session.chunks.delete(session.chunks.keys().next().value!);
  session.revision++;
  return snapshot(session);
}

export async function staticNlpRequest(request: Record<string, unknown>): Promise<unknown> {
  const command = String(request.command);
  if (command === 'open_document') {
    const { base, manifest } = await staticBookManifest(String(request.document_id));
    const plan = await loadStaticJson<DocumentPlan>(base + manifest.plan);
    const session: StaticSession = { id: `static:${++sequence}`, base, manifest, plan, revision: 0, chunks: new Map(), range: [0, 1] };
    sessions.set(session.id, session);
    const start = Math.min(Number(request.initial_offset) || 0, plan.prepared.mapping.origins.length - 1);
    try {
      const update = await loadRange(session, [start, start + 1]);
      return { plan, update };
    } catch (error) { sessions.delete(session.id); throw error; }
  }
  if (command === 'close_document') {
    sessions.delete(String(request.session_id));
    return { closed: true };
  }
  const session = request.session_id ? sessions.get(String(request.session_id)) : [...sessions.values()].pop();
  if (!session) throw new Error('请先打开图书');
  switch (command) {
    case 'request_range': return loadRange(session, request.range as [number, number]);
    case 'sync_document':
    case 'continue_document':
    case 'poll_document': return snapshot(session);
    case 'retry_document': {
      const unit = session.plan.units.find(unit => unit.id === request.unit_id);
      if (unit) return loadRange(session, unit.anchor.char_range);
      return loadRange(session, session.range);
    }
    case 'query_lookup_document': {
      const unit = [...session.chunks.values()].flat().find(unit => unit.unit_id === request.unit_id);
      const qid = unit?.queries[String(request.target_id)];
      if (qid === undefined) throw new Error('该词条没有静态查询结果');
      const lookup = await index(session);
      const queries = await loadStaticJson<Record<string, Record<string, LookupSnapshot>>>(session.base + lookup.queries[qid]);
      const variants = queries[qid];
      const saved = variants[request.selected_form ? String(request.selected_form) : 'default'];
      if (!saved) throw new Error('该表记没有静态查询结果');
      const entries = await contents(session, saved.entryKeys);
      return { ...variants.default.result, ...(saved.result ?? { selected_form_id: saved.selectedFormId }),
        entries: saved.entryKeys.map(key => entries[key]) };
    }
    case 'search': {
      const word = String(request.word).trim();
      const lookup = await index(session);
      const keys = lookup.words[word] ?? [];
      const shared = await contents(session, keys);
      const entries = keys.map(key => shared[key] as DictEntry);
      return { analysis_id: null, token_id: null, dictionary_names: [...new Set(entries.map(entry => entry.dict_name))],
        groups: [{ form: { kind: 'search', form: word, reading: null }, entries, total: entries.length }], forms: [] };
    }
    default: throw new Error('此功能需要桌面版');
  }
}
