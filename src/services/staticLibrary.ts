import type { LibraryBook, LibraryBookSummary } from '../reader/library';
import type { SavedSelection } from '../types/reader';

export interface StaticBookManifest {
  schema: string; id: string; version: string;
  book: string; plan: string; index: string; chunks: Record<string, string>;
}
interface CatalogBook extends LibraryBookSummary { manifest: string }
const packages = new Map<string, Promise<unknown>>();

export function staticUrl(path: string) {
  return new URL(path, new URL(import.meta.env.BASE_URL, location.href)).href;
}

export function loadStaticJson<T>(path: string): Promise<T> {
  const url = staticUrl(path);
  let pending = packages.get(url);
  if (pending) {
    packages.delete(url);
    packages.set(url, pending);
  } else {
    pending = fetch(url).then(async response => {
      if (!response.ok) throw new Error(`图书资源加载失败：${response.status}（${path}）`);
      const bytes = new Uint8Array(await response.arrayBuffer());
      const body = bytes[0] === 0x1f && bytes[1] === 0x8b
        ? new Response(new Blob([bytes]).stream().pipeThrough(new DecompressionStream('gzip')))
        : new Response(bytes);
      return body.json();
    }).catch(error => { packages.delete(url); throw error; });
    packages.set(url, pending);
    if (packages.size > 8) packages.delete(packages.keys().next().value!);
  }
  return pending as Promise<T>;
}

let catalog: Promise<{ schema: string; books: CatalogBook[] }> | undefined;
export async function staticCatalog() {
  catalog ??= loadStaticJson<{ schema: string; books: CatalogBook[] }>('library/catalog.json')
    .catch(error => { catalog = undefined; throw error; });
  return catalog;
}

export async function staticBookManifest(id: string) {
  const book = (await staticCatalog()).books.find(book => book.id === id);
  if (!book) throw new Error('书架中没有这本书');
  const base = book.manifest.slice(0, book.manifest.lastIndexOf('/') + 1);
  return { base, manifest: await loadStaticJson<StaticBookManifest>(book.manifest) };
}

interface BrowserState {
  books: Record<string, Partial<LibraryBookSummary>>;
  selections: SavedSelection[];
  words: Record<string, { known: boolean; exposures: number }>;
}
const storageKey = 'readux.static-reader.v1';
let browserState: BrowserState;
function localState() {
  if (!browserState) {
    const saved = localStorage.getItem(storageKey);
    browserState = saved ? JSON.parse(saved) : { books: {}, selections: [], words: {} };
  }
  return browserState;
}
function persist() { localStorage.setItem(storageKey, JSON.stringify(localState())); }
async function summary(id: string) {
  const book = (await staticCatalog()).books.find(book => book.id === id);
  if (!book) throw new Error('书架中没有这本书');
  return { ...book, ...localState().books[id] };
}

export async function staticReaderRequest(command: string, payload: Record<string, unknown>): Promise<unknown> {
  const state = localState();
  const id = String(payload.id ?? '');
  const matchesSelection = (item: SavedSelection) => item.book_id === payload.book && item.text_version === payload.version;
  switch (command) {
    case 'reader_library': return { books: await Promise.all((await staticCatalog()).books.map(book => summary(book.id))), path: '' };
    case 'reader_book_summary': return summary(id);
    case 'reader_open_book': {
      const { base, manifest } = await staticBookManifest(id);
      const book = await loadStaticJson<LibraryBook>(base + manifest.book);
      state.books[id] = { ...state.books[id], lastOpenedAt: new Date().toISOString() };
      persist();
      return { ...book, ...await summary(id), resources: book.resources.map(resource => ({ ...resource, path: staticUrl(resource.path) })) };
    }
    case 'reader_close': return null;
    case 'reader_progress':
    case 'reader_reset':
    case 'reader_organize': {
      const book = await summary(id);
      const changes = command === 'reader_organize'
        ? { accentColor: payload.color as string | null, tags: payload.tags as string[] }
        : { progressOffset: command === 'reader_reset' ? 0 : Number(payload.offset),
          progressPercent: command === 'reader_reset' ? 0 : Math.min(100, Number(payload.offset) / Number(payload.total) * 100),
          currentChapter: command === 'reader_reset' ? null : payload.chapter as string | null };
      state.books[id] = { ...state.books[id], ...changes };
      persist();
      return { ...book, ...changes };
    }
    case 'reader_selections': return state.selections.filter(matchesSelection);
    case 'reader_save_selection': {
      const selection = payload.selection as SavedSelection;
      state.selections = state.selections.filter(item => !(item.book_id === selection.book_id && item.text_version === selection.text_version && item.start === selection.start && item.end === selection.end));
      state.selections.push(selection);
      persist();
      return null;
    }
    case 'reader_clear_selections':
    case 'reader_delete_selection':
      state.selections = state.selections.filter(item => !matchesSelection(item) || (command === 'reader_delete_selection' && (item.start !== payload.start || item.end !== payload.end)));
      persist();
      return null;
    case 'reader_word':
    case 'reader_expose':
    case 'reader_mark': {
      const key = JSON.stringify([payload.base, payload.reading]);
      const word = state.words[key] ?? { known: false, exposures: 0 };
      if (command === 'reader_expose') word.exposures++;
      if (command === 'reader_mark') word.known = Boolean(payload.known);
      state.words[key] = word;
      if (command !== 'reader_word') persist();
      return word;
    }
    default: throw new Error('此功能需要桌面版');
  }
}
