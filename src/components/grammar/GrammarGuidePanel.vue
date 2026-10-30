<script setup lang="ts">
import { computed, ref, shallowRef, watch } from "vue";
import { Check, ChevronRight, Download, FileUp, MessageSquareMore, Pencil, RefreshCw, Save } from "@lucide/vue";
import ReaderSurface from "../reader/ReaderSurface.vue";
import { markdownBlocks, renderMarkdownBlock } from "../../grammar-guide/markdown";
import grammar11Package from "../../grammar-guide/lesson-1-1.json";
import grammar21Package from "../../grammar-guide/lesson-2-1.json";
import morphologyPackage from "../../grammar-guide/lesson-stage11.json";
import grammar11AnalysisUrl from "../../grammar-guide/lesson-1-1-analysis.json?url";
import grammar21AnalysisUrl from "../../grammar-guide/lesson-2-1-analysis.json?url";
import morphologyAnalysisUrl from "../../grammar-guide/lesson-stage11-analysis.json?url";
import type { AnnotatedToken } from "../../types";
import type { GuideLookupSnapshot, GuideReaderSnapshot } from "../../grammar-guide/types";
import { loadGuideJson } from "../../grammar-guide/loadJson";
import GrammarGuideExample from "./GrammarGuideExample.vue";
import MorphologyLayoutToggle from "../explanation/MorphologyLayoutToggle.vue";
import { provideMorphologyLayout } from "../../explanation/morphologyLayout";
import { useExplanationSheet } from "../../composables/useExplanationSheet";

type GuideExample = {
  id: string;
  group: string;
  ja: string;
  sourceJa?: string;
  sourceRange?: [number, number];
  kana: string;
  zh: string;
  note: string;
  source?: string;
  focus: string[];
  capsules?: { id: string; range: [number, number]; partIds: string[] }[];
  ruby?: { range: [number, number]; base: string; reading: string }[];
  rubySource?: "author";
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
};
type GuidePackage = {
  lessonId: string;
  title: string;
  version: number;
  status: "draft" | "reviewed";
  sourceMarkdown: string;
  placements: {
    group: string;
    title: string;
    exampleIds: string[];
    layout: "list" | "contrast";
    anchor: string;
    items?: ({ type: "example"; id: string } | { type: "explanation"; id: string; text: string })[];
  }[];
  originalExamplesAnchor: string;
  originalExamples: GuideExample[];
  examples: GuideExample[];
  generatedFrom: Record<string, string>;
};
type GuideAnalysisPackage = {
  schema: string;
  examples: Record<string, GuideReaderSnapshot>;
};
const guideLessons = [
  { id: "grammar-1.1", number: "1.1", label: "1.1 句法入门", package: grammar11Package as unknown as GuidePackage, analysisUrl: grammar11AnalysisUrl, exampleCount: 43, originalExampleCount: 0 },
  { id: "grammar-2.1", number: "2.1", label: "2.1 认识实词、虚词、助词", package: grammar21Package as unknown as GuidePackage, analysisUrl: grammar21AnalysisUrl, exampleCount: 70, originalExampleCount: 19 },
  { id: "grammar-stage11", number: "测试", label: "活用与接续 · 气泡测试", package: morphologyPackage as unknown as GuidePackage, analysisUrl: morphologyAnalysisUrl, exampleCount: morphologyPackage.examples.length, originalExampleCount: 0 },
] as const;

const props = defineProps<{ show: boolean }>();
const morphologyLayout = provideMorphologyLayout();
const panelExpanded = ref(false);
const { element: sheetElement, height: sheetHeight, dragging: sheetDragging,
  startDrag: startSheetDrag, moveDrag: moveSheetDrag, endDrag: finishSheetDrag, handleKey: sheetKeydown } = useExplanationSheet(morphologyLayout);
watch([morphologyLayout.active, morphologyLayout.mode], ([active, mode]) => {
  if (active && mode !== "modal") panelExpanded.value = true;
  else if (mode === "modal") panelExpanded.value = false;
});
const emit = defineEmits<{ close: [] }>();
const requestedLesson = new URLSearchParams(window.location.search).get("lesson");
const activeLessonId = ref<string>(guideLessons.find((lesson) => lesson.id === requestedLesson)?.id ?? guideLessons[0].id);
const activeLesson = computed(() => guideLessons.find((lesson) => lesson.id === activeLessonId.value) ?? guideLessons[0]);
const storageKeyFor = (lessonId: string) => "readux.grammarGuide.v2." + lessonId.replace("grammar-", "");
const loadedStorageKeyFor = (lessonId: string) => storageKeyFor(lessonId) + ".loaded";
const clone = <T,>(value: T): T => JSON.parse(JSON.stringify(value)) as T;
const loadedPackage = ref<GuidePackage>(clone(guideLessons[0].package));
const draftPackage = ref<GuidePackage>(clone(guideLessons[0].package));
const analysisPackage = shallowRef<GuideAnalysisPackage | null>(null);
let analysisLoadGeneration = 0;
const jsonText = ref("");
const editorOpen = ref(false);
const showChinese = ref(true);
const showRuby = ref(true);
const sourceHighlight = ref<{ blockIndex: number; range: [number, number] | null } | null>(null);
const error = ref("");
function finishGuidePaneClose() {
  if (!morphologyLayout.active.value || morphologyLayout.mode.value === "modal")
    panelExpanded.value = false;
}
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
  return [example.id, snapshot?.document.text === example.ja ? snapshot : undefined];
})));

function activateLesson(lessonId: string) {
  const lesson = guideLessons.find((item) => item.id === lessonId);
  if (!lesson) return;
  const generation = ++analysisLoadGeneration;
  loadedPackage.value = clone(lesson.package);
  draftPackage.value = clone(lesson.package);
  analysisPackage.value = null;
  sourceHighlight.value = null;
  try {
    const storageKey = storageKeyFor(lessonId);
    const loadedStorageKey = loadedStorageKeyFor(lessonId);
    let savedDraft = localStorage.getItem(storageKey);
    let savedLoaded = localStorage.getItem(loadedStorageKey);
    if (lessonId === "grammar-2.1" && !savedDraft && !savedLoaded) {
      const oldDraft = localStorage.getItem("readux.grammarGuide.1.1");
      const oldLoaded = localStorage.getItem("readux.grammarGuide.1.1.loaded");
      if (oldDraft) {
        const migrated = migrateLegacyPackage(JSON.parse(oldDraft));
        validatePackage(migrated, lessonId);
        localStorage.setItem(storageKey, JSON.stringify(migrated));
        savedDraft = JSON.stringify(migrated);
      }
      if (oldLoaded) {
        const migrated = migrateLegacyPackage(JSON.parse(oldLoaded));
        validatePackage(migrated, lessonId);
        localStorage.setItem(loadedStorageKey, JSON.stringify(migrated));
        savedLoaded = JSON.stringify(migrated);
      }
    }
    if (savedDraft) { const value = JSON.parse(savedDraft) as GuidePackage; validatePackage(value, lessonId); draftPackage.value = value; }
    if (savedLoaded) { const value = JSON.parse(savedLoaded) as GuidePackage; validatePackage(value, lessonId); loadedPackage.value = value; }
  } catch { /* 页面继续使用内置课节 */ }
  void loadGuideJson<GuideAnalysisPackage>(lesson.analysisUrl).then((analysis) => {
    if (generation !== analysisLoadGeneration) return;
    if (analysis.schema !== "readux.grammar-guide-analysis.v2") throw new Error("语法指南分析快照版本无效");
    analysisPackage.value = analysis;
  }).catch((reason: unknown) => {
    if (generation === analysisLoadGeneration) setNotice("例句分析加载失败：" + (reason instanceof Error ? reason.message : String(reason)));
  });
}
watch([() => props.show, activeLessonId], ([show, lessonId]) => {
  if (show) activateLesson(lessonId);
}, { immediate: true });

function setNotice(message: string) { error.value = message; window.setTimeout(() => { if (error.value === message) error.value = ""; }, 4000); }
function showChineseFor(_group: string) { return showChinese.value; }
function migrateLegacyPackage(value: unknown): GuidePackage {
  const rewrite = (item: unknown): unknown => {
    if (typeof item === "string") return item.split("1.1").join("2.1");
    if (Array.isArray(item)) return item.map(rewrite);
    if (item && typeof item === "object") return Object.fromEntries(Object.entries(item).map(([key, entry]) => [key.split("1.1").join("2.1"), rewrite(entry)]));
    return item;
  };
  return rewrite(value) as GuidePackage;
}
function highlightExample(anchor: string, exampleId: string) {
  const blockIndex = blocks.value.findIndex((block) => block.source === anchor);
  const range = exampleMap.value.get(exampleId)?.sourceRange ?? null;
  sourceHighlight.value = blockIndex < 0 ? null : { blockIndex, range };
}
function clearExampleHighlight() { sourceHighlight.value = null; }
function placementItems(placement: GuidePackage["placements"][number]) {
  return placement.items ?? placement.exampleIds.map((id) => ({ type: "example" as const, id }));
}
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
function validatePackage(value: GuidePackage, lessonId = activeLessonId.value) {
  const lesson = guideLessons.find((item) => item.id === lessonId);
  if (!lesson || value.lessonId !== lessonId || !Array.isArray(value.examples) || value.examples.length !== lesson.exampleCount || !Array.isArray(value.placements) || !Array.isArray(value.originalExamples) || value.originalExamples.length !== lesson.originalExampleCount)
    throw new Error("课节编号、例句数量或数据结构不正确。");
  if (value.sourceMarkdown !== lesson.package.sourceMarkdown) throw new Error("原稿正文为只读内容，不能随内容包修改。");
  const sourceBlocks = markdownBlocks(value.sourceMarkdown).map((block) => block.source);
  if (value.placements.some((item) => !item.anchor || sourceBlocks.filter((source) => source === item.anchor).length !== 1)) throw new Error("例句插入锚点必须唯一对应一个原稿段落。");
  if (lesson.originalExampleCount && !sourceBlocks.includes(value.originalExamplesAnchor)) throw new Error("原稿例句锚点未指向现有对照表。");
  const allExamples = [...value.examples, ...value.originalExamples];
  const exampleIds = allExamples.map((item) => item.id);
  if (new Set(exampleIds).size !== exampleIds.length) throw new Error("例句 ID 存在重复。");
  const placedIds = value.placements.flatMap((item) => item.exampleIds);
  if (new Set(placedIds).size !== placedIds.length || value.examples.some((item) => !placedIds.includes(item.id))) throw new Error("补充例句必须且只能插入一次。");
  for (const placement of value.placements) {
    if (!placement.items) continue;
    const itemExampleIds = placement.items.filter((item) => item.type === "example").map((item) => item.id);
    if (itemExampleIds.join("\0") !== placement.exampleIds.join("\0")
      || placement.items.some((item) => item.type === "explanation" && !item.text.trim()))
      throw new Error(`${placement.group} 的卡片项顺序或讲解内容无效。`);
  }
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
  const bundledOriginals = new Map(lesson.package.originalExamples.map((item) => [item.id, item]));
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
  try { localStorage.setItem(storageKeyFor(activeLessonId.value), JSON.stringify(value)); }
  catch (reason) { setNotice(`保存失败：${String(reason)}`); return; }
  jsonText.value = JSON.stringify(value, null, 2);
  setNotice("草稿已保存。");
}
function reviewDraft() {
  const value = parseEditor();
  if (!value) return;
  value.status = "reviewed";
  draftPackage.value = value;
  try { localStorage.setItem(storageKeyFor(activeLessonId.value), JSON.stringify(value)); }
  catch (reason) { setNotice(`保存失败：${String(reason)}`); return; }
  jsonText.value = JSON.stringify(value, null, 2);
  setNotice("内容已标记为已审阅。");
}
function loadDraft() {
  const value = parseEditor();
  if (!value) return;
  loadedPackage.value = clone(value);
  draftPackage.value = clone(value);
  try { localStorage.setItem(loadedStorageKeyFor(activeLessonId.value), JSON.stringify(value)); }
  catch (reason) { setNotice(`已加载，但写入加载版本失败：${String(reason)}`); }
  editorOpen.value = false;
  setNotice(`已加载 v${value.version}。`);
}
function downloadDraft() {
  const value = parseEditor() ?? draftPackage.value;
  const link = document.createElement("a");
  link.href = URL.createObjectURL(new Blob([JSON.stringify(value, null, 2)], { type: "application/json" }));
  link.download = "grammar-guide-" + activeLesson.value.number.replace(".", "-") + ".json";
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
function preventCapsuleTextSelection(event: Event) {
  const target = event.target instanceof Element ? event.target : event.target instanceof Node ? event.target.parentElement : null;
  if (target?.closest(".reader-capsule")) event.preventDefault();
}
</script>

<template>
  <ReaderSurface class="grammar-guide-surface" :show="show" variant="fullscreen" :title="'日文语法指南 · ' + activeLesson.number" :description="activeLesson.package.title" label="日文语法指南" @close="emit('close')" @back="emit('close')">
    <template #actions>
      <div class="guide-actions">
        <label class="guide-lesson-select"><span>课节</span><select v-model="activeLessonId" aria-label="选择语法指南课节"><option v-for="lesson in guideLessons" :key="lesson.id" :value="lesson.id">{{ lesson.label }}</option></select></label>
        <label class="guide-toggle"><input v-model="showChinese" type="checkbox"><span>中文</span></label>
        <label class="guide-toggle"><input v-model="showRuby" type="checkbox"><span>假名</span></label>
        <button type="button" title="从内置课节生成草稿" aria-label="生成草稿" @click="makeDraft"><RefreshCw :size="16" /></button>
        <button type="button" title="编辑 JSON" aria-label="编辑 JSON" @click="openEditor"><Pencil :size="16" /></button>
        <MorphologyLayoutToggle
          :docked="morphologyLayout.docked.value"
          :narrow="morphologyLayout.mode.value === 'sheet'"
          @toggle="morphologyLayout.toggle"
        />
      </div>
    </template>
    <div class="guide-layout" :class="{
      'guide-layout--docked': panelExpanded && morphologyLayout.mode.value === 'panel',
      'guide-layout--sheet': panelExpanded && morphologyLayout.mode.value === 'sheet',
    }">
      <nav class="guide-toc" aria-label="本课目录">
        <span class="guide-toc__label">{{ activeLesson.label }}</span>
        <button v-for="item in headings" :key="item.index" type="button" @click="scrollToHeading(item.index)">
          <span>{{ item.block.source.replace(/^#+\s*/, "") }}</span><ChevronRight :size="14" />
        </button>
        <div class="guide-progress">{{ loadedPackage.examples.length }} 条例句 · v{{ loadedPackage.version }}</div>
      </nav>
      <div v-if="morphologyLayout.active.value && morphologyLayout.mode.value === 'sheet'" class="guide-sheet-guard" :style="{ bottom: `${sheetHeight}px` }" aria-hidden="true" @pointerdown.stop />
      <Transition name="guide-pane" @after-leave="finishGuidePaneClose">
      <aside ref="sheetElement" v-show="morphologyLayout.docked.value && morphologyLayout.active.value" id="grammar-guide-dock-pane"
        class="guide-dock-pane" :class="{ 'guide-dock-pane--sheet': panelExpanded && morphologyLayout.mode.value === 'sheet', 'guide-dock-pane--dragging': sheetDragging }"
        data-guide-layout-pane :style="panelExpanded && morphologyLayout.mode.value === 'sheet' ? { height: `${sheetHeight}px` } : undefined"
        aria-label="词典、活用与接续">
        <button v-if="morphologyLayout.mode.value === 'sheet'" type="button" class="guide-dock-pane__handle"
          aria-label="展开或收起词典、活用与接续面板"
          @pointerdown="startSheetDrag" @pointermove="moveSheetDrag" @pointerup="finishSheetDrag" @pointercancel="finishSheetDrag" @keydown="sheetKeydown"><span /></button>
        <div class="guide-dock-pane__scroll" data-explanation-scroll>
          <div id="grammar-guide-dock-pane-content" class="guide-dock-pane__content" data-explanation-flow />
        </div>
      </aside>
      </Transition>
      <main class="guide-reading" aria-label="语法讲解正文" @selectstart="preventCapsuleTextSelection">
        <template v-for="(block, index) in blocks" :key="index">
          <article :id="`guide-block-${index}`" class="guide-source" :class="[`guide-source--${block.kind}`, { 'guide-source--highlighted': sourceHighlight?.blockIndex === index && !sourceHighlight.range }]" v-html="htmlForBlock(index)" />
          <section v-for="placement in placementMap.get(block.source) ?? []" :key="placement.group" class="guide-examples" :class="`guide-examples--${placement.layout}`" :aria-label="placement.title">
            <div class="guide-example-list">
              <div v-for="item in placementItems(placement)" :key="`${item.type}:${item.id}`" class="guide-example-row" :class="{ 'guide-example-row--explanation': item.type === 'explanation' }" @pointerenter="item.type === 'example' && highlightExample(placement.anchor, item.id)" @pointerleave="item.type === 'example' && clearExampleHighlight()">
                <template v-if="item.type === 'example'">
                  <template v-if="exampleMap.get(item.id)">
                  <GrammarGuideExample
                    :id="item.id"
                    :ja="exampleMap.get(item.id)!.ja"
                    :zh="exampleMap.get(item.id)!.zh"
                    :note="exampleMap.get(item.id)!.note"
                    :source="exampleMap.get(item.id)!.source"
                    :snapshot="analysisMap.get(item.id)"
                    :paragraph-id="paragraphId(exampleMap.get(item.id)!)"
                    :show-chinese="showChineseFor(placement.group)"
                    :show-ruby="showRuby"
                    :lesson-id="activeLessonId"
                    :sheet-height="sheetHeight"
                  />
                  </template>
                </template>
                <template v-else>
                  <MessageSquareMore class="guide-item-icon guide-item-icon--explanation" :size="17" aria-hidden="true" />
                  <p class="guide-explanation__body">{{ item.text }}</p>
                </template>
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
.guide-actions,.guide-toggle,.guide-lesson-select,.guide-editor__tools,.guide-editor footer{display:flex;align-items:center;gap:10px}.guide-actions{padding-right:22px}.guide-actions button,.guide-editor button{display:flex;align-items:center;justify-content:center;gap:6px;min-width:34px;height:34px;padding:0 9px;border:1px solid var(--border-color);border-radius:6px;background:var(--bg-primary);color:var(--text-secondary);cursor:pointer}.guide-toggle,.guide-lesson-select{font-size:12px;color:var(--text-secondary);cursor:pointer}.guide-toggle input{accent-color:var(--accent-color)}.guide-lesson-select select{max-width:180px;height:30px;padding:0 7px;border:1px solid var(--border-color);border-radius:5px;background:var(--bg-primary);color:var(--text-primary);font:inherit}
.guide-layout{--guide-toc-width:220px;--guide-dock-width:clamp(340px,32vw,400px);display:grid;grid-template-columns:var(--guide-toc-width) minmax(0,1fr);width:100%;height:100%;min-height:0;background:var(--bg-primary);color:var(--text-primary);transition:grid-template-columns 220ms cubic-bezier(.2,0,0,1)}.guide-toc{display:flex;grid-column:1;grid-row:1;flex-direction:column;gap:3px;overflow:auto;padding:22px 12px;border-right:1px solid var(--border-color);background:var(--bg-secondary)}.guide-toc__label{padding:0 10px 10px;color:var(--text-muted);font-size:11px;font-weight:700}.guide-toc button{display:flex;align-items:center;justify-content:space-between;gap:8px;padding:9px 10px;border:0;border-radius:5px;background:transparent;color:var(--text-secondary);text-align:left;cursor:pointer}.guide-toc button:hover{background:var(--bg-primary);color:var(--text-primary)}.guide-toc button span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.guide-progress{margin-top:auto;padding:12px 10px;color:var(--text-muted);font-size:11px}
.guide-reading{overflow:auto;padding:34px clamp(22px,5vw,76px) 70px;scroll-behavior:smooth}.guide-source{max-width:820px;margin:0 auto 18px;font:15px/1.9 var(--font-ui);overflow-wrap:anywhere}.guide-source :deep(h1),.guide-source :deep(h2),.guide-source :deep(h3){margin:1.5em 0 .6em;line-height:1.35}.guide-source :deep(h1){font-size:24px}.guide-source :deep(h2){font-size:20px}.guide-source :deep(h3){font-size:17px}.guide-source :deep(p){margin:0 0 1em}.guide-source :deep(strong){font-weight:750;color:var(--text-primary)}.guide-source :deep(a){color:var(--accent-color)}.guide-source :deep(code){padding:1px 4px;border-radius:3px;background:var(--bg-secondary)}.guide-source :deep(blockquote){margin:12px 0;padding:8px 14px;border-left:3px solid var(--accent-color);background:var(--bg-secondary);color:var(--text-secondary)}.guide-note-inline{color:#9b6417}.guide-table-wrap{overflow:auto;margin:18px 0 22px}.guide-source :deep(table){width:100%;border-collapse:collapse;font-size:13px}.guide-source :deep(td){padding:9px 11px;border:1px solid var(--border-color);vertical-align:top}.guide-source :deep(tr:first-child){background:var(--bg-secondary);font-weight:650}.guide-source :deep(ul){padding-left:24px}
.guide-layout--docked{grid-template-columns:var(--guide-toc-width) minmax(0,1fr) var(--guide-dock-width)}
.guide-layout--docked .guide-reading{grid-column:2;grid-row:1}
.guide-dock-pane{grid-column:3;grid-row:1;box-sizing:border-box;display:flex;min-width:0;min-height:0;flex-direction:column;overflow:hidden;border-left:1px solid var(--border-color);background:#fff}
.guide-dock-pane--sheet{position:fixed;z-index:1005;inset:auto 12px 0;display:flex;width:auto;min-height:0;flex-direction:column;overflow:hidden;border:1px solid var(--border-color);border-radius:16px 16px 0 0;background:#fff}
.guide-dock-pane__handle{position:relative;z-index:1012;box-sizing:border-box;display:grid;width:100%;height:22px;min-height:22px;flex:0 0 22px;place-items:center;padding:0;border:0;background:#fff;touch-action:none;cursor:grab}.guide-dock-pane__handle span{width:36px;height:4px;border-radius:2px;background:var(--text-muted)}
.guide-dock-pane__scroll{flex:1 1 auto;min-height:0;min-width:0;overflow-x:hidden;overflow-y:auto;overscroll-behavior:contain;scrollbar-width:thin}
.guide-dock-pane__handle:focus-visible{outline:2px solid var(--accent-color);outline-offset:-3px}
.guide-sheet-guard{position:fixed;z-index:1004;left:12px;right:12px;height:32px}
.guide-actions{flex-wrap:wrap;justify-content:flex-end;row-gap:8px}.guide-toggle,.guide-lesson-select{white-space:nowrap;flex-shrink:0}.guide-lesson-select select{max-width:160px}.guide-actions :deep(.morphology-layout-toggle){margin-left:auto}
.guide-dock-pane__content{display:flex;width:var(--guide-dock-width);min-width:var(--guide-dock-width);flex-direction:column;gap:0}
.guide-dock-pane--sheet .guide-dock-pane__content{width:100%;min-width:0}
.guide-dock-pane__content{flex:0 0 auto;width:calc(var(--guide-dock-width) - 1px);min-width:calc(var(--guide-dock-width) - 1px)}
.guide-examples{max-width:820px;margin:20px auto 34px;border-top:1px solid var(--border-color)}.guide-example-list{display:grid;gap:0}
.guide-editor{display:flex;height:100%;min-height:0;flex-direction:column;gap:10px;padding:14px}.guide-editor__tools{justify-content:space-between}.guide-file-button{display:flex;align-items:center;gap:6px;cursor:pointer}.guide-file-button input{display:none}.guide-editor textarea{width:100%;min-height:0;flex:1;resize:none;padding:12px;border:1px solid var(--border-color);border-radius:5px;background:var(--bg-primary);color:var(--text-primary);font:12px/1.6 ui-monospace,Consolas,monospace}.guide-editor footer{justify-content:flex-end}.guide-editor button.primary{background:var(--accent-color);color:white}.guide-editor__notice{color:#9b6417;font-size:12px}
@media(max-width:1000px){.guide-layout{grid-template-columns:180px minmax(0,1fr)}}@media(max-width:680px){.guide-layout{grid-template-columns:minmax(0,1fr)}.guide-toc{display:none}.guide-reading{padding:24px 16px 50px}.guide-actions{gap:5px;padding-right:12px}.guide-toggle{gap:4px;font-size:11px}.guide-editor footer{flex-wrap:wrap}.guide-editor footer button{flex:1}}
.guide-example-row{display:grid;grid-template-columns:minmax(0,1fr);align-items:start;padding:10px 0}.guide-example-row + .guide-example-row{border-top:1px solid color-mix(in srgb,var(--border-color) 55%,transparent)}.guide-example-row--explanation{grid-template-columns:24px minmax(0,1fr);align-items:baseline;gap:8px}.guide-item-icon{align-self:baseline;justify-self:center;color:var(--accent-color)}.guide-explanation__body{margin:0;padding:8px 12px;border-left:2px solid color-mix(in srgb,var(--accent-color) 42%,var(--border-color));border-radius:0 4px 4px 0;background:color-mix(in srgb,var(--accent-color) 4%,var(--bg-primary));color:var(--text-secondary);font:13px/1.75 var(--font-ui)}
.guide-source :deep(.guide-note-inline){color:#9b6417}
.guide-source--highlighted{background:color-mix(in srgb,var(--accent-color) 10%,transparent);box-shadow:0 0 0 3px color-mix(in srgb,var(--accent-color) 18%,transparent);animation:guide-source-highlight-in 260ms ease-out}@keyframes guide-source-highlight-in{0%{background:color-mix(in srgb,var(--accent-color) 24%,transparent);box-shadow:0 0 0 7px color-mix(in srgb,var(--accent-color) 30%,transparent)}100%{background:color-mix(in srgb,var(--accent-color) 10%,transparent);box-shadow:0 0 0 3px color-mix(in srgb,var(--accent-color) 18%,transparent)}}@media(prefers-reduced-motion:reduce){.guide-source--highlighted{animation:none;outline:1px solid var(--accent-color);outline-offset:3px}}
::highlight(grammar-guide-source-range){background:color-mix(in srgb,var(--accent-color) 20%,transparent);text-decoration:underline 2px color-mix(in srgb,var(--accent-color) 70%,transparent);text-underline-offset:3px}
.guide-pane-enter-active,.guide-pane-leave-active{transition:opacity 180ms ease,transform 180ms ease}.guide-pane-enter-from,.guide-pane-leave-to{opacity:0}.guide-layout--docked .guide-pane-enter-from,.guide-layout--docked .guide-pane-leave-to{transform:translateX(14px)}.guide-dock-pane--sheet.guide-pane-enter-from,.guide-dock-pane--sheet.guide-pane-leave-to{transform:translateY(14px)}
@media(prefers-reduced-motion:reduce){.guide-layout{transition:none}.guide-pane-enter-active,.guide-pane-leave-active{transition:none}}
@media(min-width:961px){
  .guide-layout{grid-template-columns:var(--guide-toc-width) minmax(0,1fr) 0}
  .guide-layout--docked{grid-template-columns:var(--guide-toc-width) minmax(0,1fr) var(--guide-dock-width)}
  .guide-reading{grid-column:2;grid-row:1;min-width:0}
  .guide-source,.guide-examples{width:min(820px,calc(100vw - var(--guide-toc-width) - var(--guide-dock-width) - clamp(22px,5vw,76px) - clamp(22px,5vw,76px) - 20px));min-width:min(820px,calc(100vw - var(--guide-toc-width) - var(--guide-dock-width) - clamp(22px,5vw,76px) - clamp(22px,5vw,76px) - 20px));margin-inline:auto}
}
@media(min-width:961px) and (max-width:1000px){.guide-layout{--guide-toc-width:180px}}
@media(max-width:600px){
  .grammar-guide-surface :deep(.app-header){flex-wrap:wrap;gap:8px}
  .grammar-guide-surface :deep(.app-header__actions){flex:1 0 100%}
  .grammar-guide-surface :deep(.app-header__page-identity){max-width:none}
  .guide-actions{width:100%;padding:0;gap:6px;justify-content:space-between}
  .guide-lesson-select select{max-width:122px}
  .guide-lesson-select>span{display:none}
}
</style>
