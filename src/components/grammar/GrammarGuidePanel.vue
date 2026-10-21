<script setup lang="ts">
import { computed, ref, shallowRef, watch } from "vue";
import { Check, ChevronRight, Download, FileUp, Pencil, RefreshCw, Save } from "@lucide/vue";
import ReaderSurface from "../reader/ReaderSurface.vue";
import { markdownBlocks, renderMarkdownBlock } from "../../grammar-guide/markdown";
import basePackage from "../../grammar-guide/lesson-1-1.json";
import analysisUrl from "../../grammar-guide/lesson-1-1-analysis.json?url";
import dictionaryUrl from "../../grammar-guide/lesson-1-1-dictionary.json?url";
import type { AnnotatedToken } from "../../types";
import type { GuideLookupSnapshot } from "../../grammar-guide/types";
import type { DictEntry } from "../../types";
import GrammarGuideExample from "./GrammarGuideExample.vue";

type GuideExample = {
  id: string;
  group: string;
  ja: string;
  sourceRange?: [number, number];
  kana: string;
  zh: string;
  note: string;
  focus: string[];
  capsules?: { id: string; range: [number, number]; partIds: string[] }[];
  ruby?: { range: [number, number]; base: string; reading: string }[];
  parts?: { id: string; range: [number, number]; text: string; teachingClass: string; classLabel: string; term: string; focus: boolean; detailId: string }[];
  targets?: { id: string; range: [number, number]; detailId: string; defaultEntryId: string | null; entryIds: string[]; childTargetIds: string[] }[];
  dictionaryEntries?: {
    id: string; dictionary: string; headword: string; reading: string; body: string; status: string;
    variants?: { id: string; form: string; reading: string; body: string }[]; defaultVariantId?: string | null; relatedEntryIds?: string[];
  }[];
  morphology?: { form: string; label: string; note: string; steps?: { form: string; label: string; note: string }[] }[];
  dictionaryStatus?: "available" | "no-record";
  detail?: { id: string; label: string; sourceLabel: string; teachingClass: string; note: string; morphology: { form: string; label: string; note: string }[]; dictionaryEntryIds: string[] };
  details?: Record<string, { id: string; label: string; sourceLabel: string; teachingClass: string; note: string; dictionaryEntryIds: string[] }>;
  tokens?: AnnotatedToken[];
  lookups?: GuideLookupSnapshot[];
  source?: string;
};
type GuidePackage = {
  lessonId: string;
  title: string;
  version: number;
  status: "draft" | "reviewed";
  sourceMarkdown: string;
  placements: { group: string; title: string; exampleIds: string[]; layout: "list" | "contrast"; anchor: string }[];
  originalExamplesAnchor: string;
  originalExamples: GuideExample[];
  examples: GuideExample[];
  generatedFrom: Record<string, string>;
};
type GuideAnalysisPackage = {
  schema: string;
  examples: Record<string, { tokens: AnnotatedToken[]; lookups: GuideLookupSnapshot[] }>;
};
type GuideDictionaryPackage = { schema: string; entries: Record<string, DictEntry> };
const bundledPackage = basePackage as unknown as GuidePackage;

const props = defineProps<{ show: boolean }>();
const emit = defineEmits<{ close: [] }>();
const storageKey = "readux.grammarGuide.1.1";
const loadedStorageKey = "readux.grammarGuide.1.1.loaded";
const clone = <T,>(value: T): T => JSON.parse(JSON.stringify(value)) as T;
const loadedPackage = ref<GuidePackage>(clone(bundledPackage));
const draftPackage = ref<GuidePackage>(clone(bundledPackage));
const analysisPackage = shallowRef<GuideAnalysisPackage | null>(null);
const dictionaryPackage = shallowRef<GuideDictionaryPackage | null>(null);
let analysisLoad: Promise<void> | null = null;
const jsonText = ref("");
const editorOpen = ref(false);
const showChinese = ref(true);
const sourceHighlight = ref<{ blockIndex: number; range: [number, number] | null } | null>(null);
const error = ref("");
const blocks = computed(() => markdownBlocks(loadedPackage.value.sourceMarkdown));
const headings = computed(() => blocks.value.map((block, index) => ({ block, index })).filter(({ block }) => block.kind === "heading"));
const placementMap = computed(() => {
  const map = new Map<string, GuidePackage["placements"]>();
  for (const item of loadedPackage.value.placements) map.set(item.anchor, [...(map.get(item.anchor) ?? []), item]);
  return map;
});
const exampleMap = computed(() => new Map([...loadedPackage.value.examples, ...loadedPackage.value.originalExamples].map((item) => [item.id, item])));
const analysisMap = computed(() => new Map([...exampleMap.value.values()].map((example) => {
  const snapshot = analysisPackage.value?.examples[example.id];
  const tokens = snapshot?.tokens ?? example.tokens ?? [];
  const tokensMatchSource = tokens.map((token) => token.bunsetsu.surface).join("") === example.ja;
  const lookups = (snapshot?.lookups ?? example.lookups ?? []).map((lookup) => ({
    ...lookup,
    result: lookup.result ? { ...lookup.result, entries: (lookup.entryKeys ?? []).map((key) => dictionaryPackage.value?.entries[key]).filter((entry): entry is DictEntry => Boolean(entry)) } : null,
  }));
  return [example.id, { tokens: tokensMatchSource ? tokens : [], lookups }];
})));

watch(() => props.show, (show) => {
  if (!show) return;
  if (!analysisPackage.value && !analysisLoad) {
    analysisLoad = Promise.all([
      fetch(analysisUrl).then((response) => response.json() as Promise<GuideAnalysisPackage>),
      fetch(dictionaryUrl).then((response) => response.json() as Promise<GuideDictionaryPackage>),
    ]).then(([analysis, dictionary]) => {
      if (analysis.schema !== "readux.grammar-guide-analysis.v1" || dictionary.schema !== "readux.grammar-guide-dictionary.v1") throw new Error("语法指南分析快照版本无效");
      analysisPackage.value = analysis;
      dictionaryPackage.value = dictionary;
    }).catch((reason: unknown) => {
      analysisLoad = null;
      setNotice(`例句分析加载失败：${reason instanceof Error ? reason.message : String(reason)}`);
    });
    void analysisLoad;
  }
  try {
    const savedDraft = localStorage.getItem(storageKey);
    const savedLoaded = localStorage.getItem(loadedStorageKey);
    if (savedDraft) { const value = JSON.parse(savedDraft) as GuidePackage; validatePackage(value); draftPackage.value = value; }
    if (savedLoaded) { const value = JSON.parse(savedLoaded) as GuidePackage; validatePackage(value); loadedPackage.value = value; }
  } catch { /* 页面继续使用内置课节 */ }
}, { immediate: true });

function setNotice(message: string) { error.value = message; window.setTimeout(() => { if (error.value === message) error.value = ""; }, 4000); }
function showChineseFor(_group: string) { return showChinese.value; }
function highlightExample(anchor: string, exampleId: string) {
  const blockIndex = blocks.value.findIndex((block) => block.source === anchor);
  const range = exampleMap.value.get(exampleId)?.sourceRange ?? null;
  sourceHighlight.value = blockIndex < 0 ? null : { blockIndex, range };
}
function clearExampleHighlight() { sourceHighlight.value = null; }
function applySourceRangeHighlight() {
  const css = CSS as typeof CSS & { highlights?: Map<string, Highlight> };
  css.highlights?.delete("grammar-guide-source-range");
  const active = sourceHighlight.value;
  if (!active?.range) return;
  const article = document.getElementById(`guide-block-${active.blockIndex}`);
  if (!article) return;
  const walker = document.createTreeWalker(article, NodeFilter.SHOW_TEXT);
  const nodes: Text[] = [];
  let textLength = 0;
  while (walker.nextNode()) {
    const node = walker.currentNode as Text;
    nodes.push(node);
    textLength += Array.from(node.data).length;
  }
  const [start, end] = active.range;
  if (start < 0 || start >= end || end > textLength) return;
  const pointAt = (offset: number, isEnd: boolean) => {
    let remaining = offset;
    for (const node of nodes) {
      const size = Array.from(node.data).length;
      if (remaining <= size && (!isEnd || remaining < size || node === nodes[nodes.length - 1])) {
        return { node, offset: Array.from(node.data).slice(0, remaining).join("").length };
      }
      remaining -= size;
    }
    return null;
  };
  const first = pointAt(start, false);
  const last = pointAt(end, true);
  if (!first || !last || typeof Highlight === "undefined" || !css.highlights) return;
  const range = document.createRange();
  range.setStart(first.node, first.offset);
  range.setEnd(last.node, last.offset);
  css.highlights.set("grammar-guide-source-range", new Highlight(range));
}
watch(sourceHighlight, () => requestAnimationFrame(applySourceRangeHighlight), { flush: "post" });
function makeDraft() {
  const seed = editorOpen.value ? parseEditor() : draftPackage.value;
  draftPackage.value = { ...clone(seed ?? loadedPackage.value), status: "draft", version: loadedPackage.value.version + 1 };
  jsonText.value = JSON.stringify(draftPackage.value, null, 2);
  editorOpen.value = true;
}
function openEditor() {
  jsonText.value = JSON.stringify(draftPackage.value, null, 2);
  editorOpen.value = true;
}
function rangeIsValid(range: unknown, length: number): range is [number, number] {
  return Array.isArray(range) && range.length === 2 && Number.isInteger(range[0]) && Number.isInteger(range[1]) && range[0] >= 0 && range[0] < range[1] && range[1] <= length;
}
function validatePackage(value: GuidePackage) {
  if (value.lessonId !== "grammar-1.1" || !Array.isArray(value.examples) || value.examples.length !== 70 || !Array.isArray(value.placements) || !Array.isArray(value.originalExamples) || value.originalExamples.length !== 19)
    throw new Error("课节编号、70 条补充例句、19 个原稿例句或数据结构不正确。");
  if (value.sourceMarkdown !== bundledPackage.sourceMarkdown) throw new Error("原稿正文为只读内容，不能随内容包修改。");
  const sourceBlocks = markdownBlocks(value.sourceMarkdown).map((block) => block.source);
  if (value.placements.some((item) => !item.anchor || sourceBlocks.filter((source) => source === item.anchor).length !== 1)) throw new Error("例句插入锚点必须唯一对应一个原稿段落。");
  if (!sourceBlocks.includes(value.originalExamplesAnchor)) throw new Error("原稿例句锚点未指向现有对照表。");
  const allExamples = [...value.examples, ...value.originalExamples];
  const exampleIds = allExamples.map((item) => item.id);
  if (new Set(exampleIds).size !== exampleIds.length) throw new Error("例句 ID 存在重复。");
  const placedIds = value.placements.flatMap((item) => item.exampleIds);
  if (new Set(placedIds).size !== placedIds.length || value.examples.some((item) => !placedIds.includes(item.id))) throw new Error("补充例句必须且只能插入一次。");
  for (const example of allExamples) {
    const chars = Array.from(example.ja);
    if (!example.ja) throw new Error(`${example.id} 缺少日文。`);
    if (!example.parts || !example.targets || !example.capsules || !example.ruby || !example.details || !example.detail || !example.dictionaryEntries) continue;
    const detailIds = new Set([example.detail.id, ...Object.keys(example.details)]);
    const targets = new Map(example.targets.map((target) => [target.id, target]));
    if (targets.size !== example.targets.length) throw new Error(`${example.id} 的交互目标 ID 重复。`);
    const entryIds = new Set(example.dictionaryEntries.map((entry) => entry.id));
    let coveredEnd = 0;
    for (const part of example.parts) {
      if (!rangeIsValid(part.range, chars.length) || chars.slice(part.range[0], part.range[1]).join("") !== part.text) throw new Error(`${example.id} 的成分范围与日文不一致。`);
      if (part.range[0] !== coveredEnd) throw new Error(`${example.id} 的成分范围存在空缺或重叠。`);
      coveredEnd = part.range[1];
      if (!detailIds.has(part.detailId)) throw new Error(`${example.id} 的成分 ${part.id} 缺少详情记录。`);
    }
    if (coveredEnd !== chars.length) throw new Error(`${example.id} 的成分标注没有覆盖整句。`);
    for (const target of example.targets) {
      if (!rangeIsValid(target.range, chars.length) || !detailIds.has(target.detailId)) throw new Error(`${example.id} 的交互目标范围或详情引用无效。`);
      if (target.childTargetIds.some((id) => !targets.has(id)) || target.entryIds.some((id) => !entryIds.has(id)) || (target.defaultEntryId && !entryIds.has(target.defaultEntryId))) throw new Error(`${example.id} 的目标关系或词典引用无效。`);
    }
    for (const entry of example.dictionaryEntries) {
      const variants = new Set((entry.variants ?? []).map((variant) => variant.id));
      if ((entry.defaultVariantId && !variants.has(entry.defaultVariantId)) || (entry.relatedEntryIds ?? []).some((id) => !entryIds.has(id))) throw new Error(`${example.id} 的表记或关联词典引用无效。`);
    }
    if (example.capsules.some((capsule) => !rangeIsValid(capsule.range, chars.length) || capsule.partIds.some((id) => !example.parts?.some((part) => part.id === id)))) throw new Error(`${example.id} 的胶囊范围或成分引用无效。`);
    if (example.ruby.some((ruby) => !rangeIsValid(ruby.range, chars.length) || chars.slice(ruby.range[0], ruby.range[1]).join("") !== ruby.base || !ruby.reading)) throw new Error(`${example.id} 的注音范围或读音无效。`);
  }
  const bundledOriginals = new Map(bundledPackage.originalExamples.map((item) => [item.id, item]));
  if (value.originalExamples.some((item) => bundledOriginals.get(item.id)?.ja !== item.ja || bundledOriginals.get(item.id)?.source !== item.source)) throw new Error("原稿例句的日文和表格归属必须保持原样。");
  for (const placement of value.placements) if (placement.exampleIds.some((id) => !value.examples.some((example) => example.id === id))) throw new Error(`${placement.group} 引用了不存在的例句。`);
}
function parseEditor(): GuidePackage | null {
  try {
    const value = JSON.parse(jsonText.value) as GuidePackage;
    validatePackage(value);
    return value;
  } catch (reason) {
    setNotice(reason instanceof Error ? reason.message : String(reason));
    return null;
  }
}
function saveDraft() {
  const value = parseEditor();
  if (!value) return;
  value.status = "draft";
  draftPackage.value = value;
  try { localStorage.setItem(storageKey, JSON.stringify(value)); }
  catch (reason) { setNotice(`保存失败：${String(reason)}`); return; }
  jsonText.value = JSON.stringify(value, null, 2);
  setNotice("草稿已保存。");
}
function reviewDraft() {
  const value = parseEditor();
  if (!value) return;
  value.status = "reviewed";
  draftPackage.value = value;
  try { localStorage.setItem(storageKey, JSON.stringify(value)); }
  catch (reason) { setNotice(`保存失败：${String(reason)}`); return; }
  jsonText.value = JSON.stringify(value, null, 2);
  setNotice("内容已标记为已审阅。");
}
function loadDraft() {
  const value = parseEditor();
  if (!value) return;
  loadedPackage.value = clone(value);
  draftPackage.value = clone(value);
  try { localStorage.setItem(loadedStorageKey, JSON.stringify(value)); }
  catch (reason) { setNotice(`已加载，但写入加载版本失败：${String(reason)}`); }
  editorOpen.value = false;
  setNotice(`已加载 v${value.version}。`);
}
function downloadDraft() {
  const value = parseEditor() ?? draftPackage.value;
  const link = document.createElement("a");
  link.href = URL.createObjectURL(new Blob([JSON.stringify(value, null, 2)], { type: "application/json" }));
  link.download = "grammar-guide-1.1.json";
  link.click();
  URL.revokeObjectURL(link.href);
}
function importPackage(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;
  void file.text().then((text) => {
    jsonText.value = text;
    editorOpen.value = true;
    const value = parseEditor();
    if (value) draftPackage.value = value;
  }).catch((reason: unknown) => setNotice(String(reason)));
  (event.target as HTMLInputElement).value = "";
}
function scrollToHeading(index: number) {
  document.getElementById(`guide-block-${index}`)?.scrollIntoView({ behavior: "smooth", block: "start" });
}
function htmlForBlock(blockIndex: number) { return renderMarkdownBlock(blocks.value[blockIndex]); }
function paragraphId(example: GuideExample) {
  return [...example.id].reduce((value, character) => ((value * 31 + character.charCodeAt(0)) >>> 0), 0) % 2_000_000_000;
}
</script>

<template>
  <ReaderSurface :show="show" variant="fullscreen" title="日文语法指南 · 1.1" description="认识实词、虚词、助词" label="日文语法指南" @close="emit('close')" @back="emit('close')">
    <template #actions>
      <div class="guide-actions">
        <label class="guide-toggle"><input v-model="showChinese" type="checkbox"><span>中文</span></label>
        <button type="button" title="从内置课节生成草稿" aria-label="生成草稿" @click="makeDraft"><RefreshCw :size="16" /></button>
        <button type="button" title="编辑 JSON" aria-label="编辑 JSON" @click="openEditor"><Pencil :size="16" /></button>
      </div>
    </template>
    <div class="guide-layout">
      <nav class="guide-toc" aria-label="本课目录">
        <span class="guide-toc__label">本课内容</span>
        <button v-for="item in headings" :key="item.index" type="button" @click="scrollToHeading(item.index)">
          <span>{{ item.block.source.replace(/^#+\s*/, "") }}</span><ChevronRight :size="14" />
        </button>
        <div class="guide-progress">{{ loadedPackage.examples.length }} 条补充例句 · v{{ loadedPackage.version }}</div>
      </nav>
      <main class="guide-reading" aria-label="语法讲解正文">
        <template v-for="(block, index) in blocks" :key="index">
          <article :id="`guide-block-${index}`" class="guide-source" :class="[`guide-source--${block.kind}`, { 'guide-source--highlighted': sourceHighlight?.blockIndex === index && !sourceHighlight.range }]" v-html="htmlForBlock(index)" />
          <section v-for="placement in placementMap.get(block.source) ?? []" :key="placement.group" class="guide-examples" :class="`guide-examples--${placement.layout}`" :aria-label="placement.title">
            <div class="guide-example-list">
              <div v-for="id in placement.exampleIds" :key="id" class="guide-example-row" @pointerenter="highlightExample(placement.anchor, id)" @pointerleave="clearExampleHighlight">
                <GrammarGuideExample
                  v-if="exampleMap.get(id)"
                  :id="id"
                  :ja="exampleMap.get(id)!.ja"
                  :zh="exampleMap.get(id)!.zh"
                  :tokens="analysisMap.get(id)?.tokens ?? []"
                  :lookups="analysisMap.get(id)?.lookups ?? []"
                  :paragraph-id="paragraphId(exampleMap.get(id)!)"
                  :show-chinese="showChineseFor(placement.group)"
                />
              </div>
            </div>
          </section>
        </template>
      </main>
    </div>
    <ReaderSurface :show="editorOpen" variant="modal" title="语法指南内容包" description="编辑草稿 JSON" @close="editorOpen = false">
      <div class="guide-editor">
        <div class="guide-editor__tools">
          <label class="guide-file-button"><FileUp :size="15" />打开 JSON<input type="file" accept="application/json,.json" @change="importPackage"></label>
          <button type="button" title="导出 JSON" @click="downloadDraft"><Download :size="15" />导出</button>
        </div>
        <textarea v-model="jsonText" spellcheck="false" aria-label="语法指南内容包 JSON" />
        <div v-if="error" class="guide-editor__notice" role="status">{{ error }}</div>
        <footer>
          <button type="button" @click="saveDraft"><Save :size="15" />保存</button>
          <button type="button" @click="reviewDraft"><Check :size="15" />标记已审阅</button>
          <button class="primary" type="button" @click="loadDraft">加载到前台</button>
        </footer>
      </div>
    </ReaderSurface>
  </ReaderSurface>
</template>

<style scoped>
.guide-actions,.guide-toggle,.guide-editor__tools,.guide-editor footer{display:flex;align-items:center;gap:10px}.guide-actions{padding-right:22px}.guide-actions button,.guide-editor button{display:flex;align-items:center;justify-content:center;gap:6px;min-width:34px;height:34px;padding:0 9px;border:1px solid var(--border-color);border-radius:6px;background:var(--bg-primary);color:var(--text-secondary);cursor:pointer}.guide-toggle{font-size:12px;color:var(--text-secondary);cursor:pointer}.guide-toggle input{accent-color:var(--accent-color)}
.guide-layout{display:grid;grid-template-columns:220px minmax(0,1fr);width:100%;height:100%;min-height:0;background:var(--bg-primary);color:var(--text-primary)}.guide-toc{display:flex;flex-direction:column;gap:3px;overflow:auto;padding:22px 12px;border-right:1px solid var(--border-color);background:var(--bg-secondary)}.guide-toc__label{padding:0 10px 10px;color:var(--text-muted);font-size:11px;font-weight:700}.guide-toc button{display:flex;align-items:center;justify-content:space-between;gap:8px;padding:9px 10px;border:0;border-radius:5px;background:transparent;color:var(--text-secondary);text-align:left;cursor:pointer}.guide-toc button:hover{background:var(--bg-primary);color:var(--text-primary)}.guide-toc button span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.guide-progress{margin-top:auto;padding:12px 10px;color:var(--text-muted);font-size:11px}
.guide-reading{overflow:auto;padding:34px clamp(22px,5vw,76px) 70px;scroll-behavior:smooth}.guide-source{max-width:820px;margin:0 auto 18px;font:15px/1.9 var(--font-ui);overflow-wrap:anywhere}.guide-source :deep(h1),.guide-source :deep(h2),.guide-source :deep(h3){margin:1.5em 0 .6em;line-height:1.35}.guide-source :deep(h1){font-size:24px}.guide-source :deep(h2){font-size:20px}.guide-source :deep(h3){font-size:17px}.guide-source :deep(p){margin:0 0 1em}.guide-source :deep(strong){font-weight:750;color:var(--text-primary)}.guide-source :deep(a){color:var(--accent-color)}.guide-source :deep(code){padding:1px 4px;border-radius:3px;background:var(--bg-secondary)}.guide-source :deep(blockquote){margin:12px 0;padding:8px 14px;border-left:3px solid var(--accent-color);background:var(--bg-secondary);color:var(--text-secondary)}.guide-note-inline{color:#9b6417}.guide-table-wrap{overflow:auto;margin:18px 0 22px}.guide-source :deep(table){width:100%;border-collapse:collapse;font-size:13px}.guide-source :deep(td){padding:9px 11px;border:1px solid var(--border-color);vertical-align:top}.guide-source :deep(tr:first-child){background:var(--bg-secondary);font-weight:650}.guide-source :deep(ul){padding-left:24px}
.guide-examples{max-width:820px;margin:20px auto 34px;border-top:1px solid var(--border-color)}.guide-example-list{display:grid;gap:0}
.guide-editor{display:flex;height:100%;min-height:0;flex-direction:column;gap:10px;padding:14px}.guide-editor__tools{justify-content:space-between}.guide-file-button{display:flex;align-items:center;gap:6px;cursor:pointer}.guide-file-button input{display:none}.guide-editor textarea{width:100%;min-height:0;flex:1;resize:none;padding:12px;border:1px solid var(--border-color);border-radius:5px;background:var(--bg-primary);color:var(--text-primary);font:12px/1.6 ui-monospace,Consolas,monospace}.guide-editor footer{justify-content:flex-end}.guide-editor button.primary{background:var(--accent-color);color:white}.guide-editor__notice{color:#9b6417;font-size:12px}
@media(max-width:1000px){.guide-layout{grid-template-columns:180px minmax(0,1fr)}}@media(max-width:680px){.guide-layout{grid-template-columns:minmax(0,1fr)}.guide-toc{display:none}.guide-reading{padding:24px 16px 50px}.guide-actions{gap:5px;padding-right:12px}.guide-toggle{gap:4px;font-size:11px}.guide-editor footer{flex-wrap:wrap}.guide-editor footer button{flex:1}}
.guide-example-row{padding:7px 0}.guide-example-row + .guide-example-row{border-top:1px solid color-mix(in srgb,var(--border-color) 55%,transparent)}
.guide-source :deep(.guide-note-inline){color:#9b6417}
.guide-source--highlighted{background:color-mix(in srgb,var(--accent-color) 10%,transparent);box-shadow:0 0 0 3px color-mix(in srgb,var(--accent-color) 18%,transparent);animation:guide-source-highlight-in 260ms ease-out}@keyframes guide-source-highlight-in{0%{background:color-mix(in srgb,var(--accent-color) 24%,transparent);box-shadow:0 0 0 7px color-mix(in srgb,var(--accent-color) 30%,transparent)}100%{background:color-mix(in srgb,var(--accent-color) 10%,transparent);box-shadow:0 0 0 3px color-mix(in srgb,var(--accent-color) 18%,transparent)}}@media(prefers-reduced-motion:reduce){.guide-source--highlighted{animation:none;outline:1px solid var(--accent-color);outline-offset:3px}}
::highlight(grammar-guide-source-range){background:color-mix(in srgb,var(--accent-color) 20%,transparent);text-decoration:underline 2px color-mix(in srgb,var(--accent-color) 70%,transparent);text-underline-offset:3px}
</style>
