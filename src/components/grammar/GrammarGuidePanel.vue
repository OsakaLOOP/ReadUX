<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Check, ChevronRight, Download, FileUp, Pencil, RefreshCw, Save, X } from "@lucide/vue";
import ReaderSurface from "../reader/ReaderSurface.vue";
import { markdownBlocks, renderMarkdownBlock } from "../../grammar-guide/markdown";
import basePackage from "../../grammar-guide/lesson-1-1.json";

type GuideExample = {
  id: string;
  group: string;
  ja: string;
  kana: string;
  zh: string;
  note: string;
  focus: string[];
  capsules: { id: string; range: [number, number]; partIds: string[] }[];
  ruby: { range: [number, number]; base: string; reading: string }[];
  parts: { id: string; range: [number, number]; text: string; teachingClass: string; classLabel: string; term: string; focus: boolean; detailId: string }[];
  targets: { id: string; range: [number, number]; detailId: string; defaultEntryId: string | null; entryIds: string[]; childTargetIds: string[] }[];
  dictionaryEntries: {
    id: string; dictionary: string; headword: string; reading: string; body: string; status: string;
    variants?: { id: string; form: string; reading: string; body: string }[]; defaultVariantId?: string | null; relatedEntryIds?: string[];
  }[];
  morphology: { form: string; label: string; note: string; steps?: { form: string; label: string; note: string }[] }[];
  dictionaryStatus: "available" | "no-record";
  detail: { id: string; label: string; sourceLabel: string; teachingClass: string; note: string; morphology: { form: string; label: string; note: string }[]; dictionaryEntryIds: string[] };
  details: Record<string, { id: string; label: string; sourceLabel: string; teachingClass: string; note: string; dictionaryEntryIds: string[] }>;
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
const bundledPackage = basePackage as unknown as GuidePackage;

const props = defineProps<{ show: boolean }>();
const emit = defineEmits<{ close: [] }>();
const storageKey = "readux.grammarGuide.1.1";
const loadedStorageKey = "readux.grammarGuide.1.1.loaded";
const clone = <T,>(value: T): T => JSON.parse(JSON.stringify(value)) as T;
const loadedPackage = ref<GuidePackage>(clone(bundledPackage));
const draftPackage = ref<GuidePackage>(clone(bundledPackage));
const jsonText = ref("");
const editorOpen = ref(false);
const showChinese = ref(true);
const showKana = ref(false);
const groupSettings = ref<Record<string, { chinese?: boolean; kana?: boolean }>>({});
const selectedExampleId = ref("");
const selectedPartId = ref("");
const highlightedBlockIndex = ref(-1);
const error = ref("");
const blocks = computed(() => markdownBlocks(loadedPackage.value.sourceMarkdown));
const headings = computed(() => blocks.value.map((block, index) => ({ block, index })).filter(({ block }) => block.kind === "heading"));
const placementMap = computed(() => {
  const map = new Map<string, GuidePackage["placements"]>();
  for (const item of loadedPackage.value.placements) map.set(item.anchor, [...(map.get(item.anchor) ?? []), item]);
  return map;
});
const exampleMap = computed(() => new Map([...loadedPackage.value.examples, ...loadedPackage.value.originalExamples].map((item) => [item.id, item])));
const selectedExample = computed(() => exampleMap.value.get(selectedExampleId.value) ?? null);
const selectedPart = computed(() => selectedExample.value?.parts.find((part) => part.id === selectedPartId.value) ?? null);
const selectedDictionaryEntryId = ref("");
const selectedVariantId = ref("");
const selectedDictionaryEntry = computed(() => selectedExample.value?.dictionaryEntries.find((entry) => entry.id === selectedDictionaryEntryId.value) ?? selectedExample.value?.dictionaryEntries[0] ?? null);
const selectedDictionaryVariant = computed(() => selectedDictionaryEntry.value?.variants?.find((item) => item.id === selectedVariantId.value) ?? selectedDictionaryEntry.value?.variants?.find((item) => item.id === selectedDictionaryEntry.value?.defaultVariantId) ?? null);

watch(() => props.show, (show) => {
  if (!show) return;
  try {
    const savedDraft = localStorage.getItem(storageKey);
    const savedLoaded = localStorage.getItem(loadedStorageKey);
    if (savedDraft) { const value = JSON.parse(savedDraft) as GuidePackage; validatePackage(value); draftPackage.value = value; }
    if (savedLoaded) { const value = JSON.parse(savedLoaded) as GuidePackage; validatePackage(value); loadedPackage.value = value; }
  } catch { /* 页面继续使用内置课节 */ }
}, { immediate: true });

function setNotice(message: string) { error.value = message; window.setTimeout(() => { if (error.value === message) error.value = ""; }, 4000); }
function showChineseFor(group: string) { return showChinese.value && groupSettings.value[group]?.chinese !== false; }
function showKanaFor(group: string) { return showKana.value && groupSettings.value[group]?.kana !== false; }
function updateGroupSetting(group: string, key: "chinese" | "kana", value: boolean) {
  groupSettings.value = { ...groupSettings.value, [group]: { ...groupSettings.value[group], [key]: value } };
}
function selectTarget(exampleId: string, partId = "") {
  const selected = selectedExampleId.value === exampleId && selectedPartId.value === partId;
  selectedExampleId.value = selected ? "" : exampleId;
  selectedPartId.value = selected ? "" : partId;
}
function kanaSegments(example: GuideExample) {
  const chars = Array.from(example.ja);
  const annotations = [...example.ruby].sort((left, right) => left.range[0] - right.range[0]);
  const segments: { text: string; reading?: string }[] = [];
  let cursor = 0;
  for (const item of annotations) {
    if (item.range[0] < cursor || item.range[1] > chars.length || chars.slice(item.range[0], item.range[1]).join("") !== item.base) continue;
    if (item.range[0] > cursor) segments.push({ text: chars.slice(cursor, item.range[0]).join("") });
    segments.push({ text: item.base, reading: item.reading });
    cursor = item.range[1];
  }
  if (cursor < chars.length) segments.push({ text: chars.slice(cursor).join("") });
  return segments;
}
function hasCompleteRuby(example: GuideExample) {
  let end = 0;
  for (const item of [...example.ruby].sort((left, right) => left.range[0] - right.range[0])) {
    if (item.range[0] !== end) return false;
    end = item.range[1];
  }
  return end === Array.from(example.ja).length;
}
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
  if (value.lessonId !== "grammar-1.1" || !Array.isArray(value.examples) || value.examples.length !== 59 || !Array.isArray(value.placements) || !Array.isArray(value.originalExamples) || value.originalExamples.length !== 19)
    throw new Error("课节编号、59 条补充例句、19 个原稿例句或数据结构不正确。");
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
    if (!example.ja || !example.kana || !example.note || !Array.isArray(example.parts) || !example.parts.length || !Array.isArray(example.targets) || !Array.isArray(example.capsules)) throw new Error(`${example.id} 缺少日文、读音、观察说明或交互标注。`);
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
    if (example.capsules.some((capsule) => !rangeIsValid(capsule.range, chars.length) || capsule.partIds.some((id) => !example.parts.some((part) => part.id === id)))) throw new Error(`${example.id} 的胶囊范围或成分引用无效。`);
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
  groupSettings.value = {};
  editorOpen.value = false;
  selectedExampleId.value = "";
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
function scrollToSource(anchor: string) {
  const index = blocks.value.findIndex((block) => block.source === anchor);
  if (index < 0) return;
  highlightedBlockIndex.value = index;
  scrollToHeading(index);
  window.setTimeout(() => { if (highlightedBlockIndex.value === index) highlightedBlockIndex.value = -1; }, 1800);
}
function htmlForBlock(blockIndex: number) { return renderMarkdownBlock(blocks.value[blockIndex]); }
function focusLabel(example: GuideExample) { return example.focus.join("、") || "整句"; }
</script>

<template>
  <ReaderSurface :show="show" variant="fullscreen" title="日文语法指南 · 1.1" description="认识实词、虚词、助词" label="日文语法指南" @close="emit('close')" @back="emit('close')">
    <template #actions>
      <div class="guide-actions">
        <label class="guide-toggle"><input v-model="showChinese" type="checkbox"><span>中文</span></label>
        <label class="guide-toggle"><input v-model="showKana" type="checkbox"><span>假名</span></label>
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
        <div class="guide-legend" aria-label="成分分类"><span><i class="is-content"></i>实词</span><span><i class="is-functional"></i>虚词</span><span><i class="is-particle"></i>助词</span><span><i class="is-other"></i>其他</span></div>
        <template v-for="(block, index) in blocks" :key="index">
          <article :id="`guide-block-${index}`" class="guide-source" :class="[`guide-source--${block.kind}`, { 'guide-source--highlighted': highlightedBlockIndex === index }]" v-html="htmlForBlock(index)" />
          <section v-for="placement in placementMap.get(block.source) ?? []" :key="placement.group" class="guide-examples" :class="`guide-examples--${placement.layout}`" :aria-label="placement.title">
            <div class="guide-examples__heading"><span>{{ placement.title }}</span><label class="guide-group-toggle"><input type="checkbox" :checked="showChineseFor(placement.group)" @change="updateGroupSetting(placement.group, 'chinese', ($event.target as HTMLInputElement).checked)"><span>中文</span></label><label class="guide-group-toggle"><input type="checkbox" :checked="showKanaFor(placement.group)" @change="updateGroupSetting(placement.group, 'kana', ($event.target as HTMLInputElement).checked)"><span>假名</span></label><small>{{ placement.exampleIds.length }} 例</small></div>
            <div class="guide-example-list">
              <article v-for="id in placement.exampleIds" :key="id" class="guide-example" :class="{ 'guide-example--selected': selectedExampleId === id }">
                <div class="guide-example__capsule" :title="exampleMap.get(id)?.note">
                  <button v-for="part in exampleMap.get(id)?.parts" :key="part.id" type="button" class="guide-example__part" :class="[`is-${part.teachingClass}`, { 'is-focused': part.focus }]" :title="`${part.classLabel} · ${part.term}`" @click="selectTarget(id, part.id)">{{ part.text }}</button>
                  <button class="guide-example__whole" type="button" :title="`${id} 整句详情`" @click="selectTarget(id)">{{ id }}</button>
                </div>
                <div v-if="showKanaFor(placement.group) && exampleMap.get(id)?.ruby.length" class="guide-example__reading"><ruby v-for="(segment, partIndex) in kanaSegments(exampleMap.get(id)!)" :key="`${id}-${partIndex}`"><rb>{{ segment.text }}</rb><rt v-if="segment.reading">{{ segment.reading }}</rt></ruby></div>
                <span v-if="showKanaFor(placement.group) && !hasCompleteRuby(exampleMap.get(id)!)" class="guide-example__kana">{{ exampleMap.get(id)?.kana }}</span>
                <p v-if="showChineseFor(placement.group) && exampleMap.get(id)?.zh" class="guide-example__translation">{{ exampleMap.get(id)?.zh }}</p>
                <p class="guide-example__note">{{ exampleMap.get(id)?.note }}</p>
                <div class="guide-example__meta"><span>{{ focusLabel(exampleMap.get(id)!) }}</span><span>{{ exampleMap.get(id)?.parts.length || 0 }} 个标注成分</span></div>
                <button type="button" class="guide-example__source-link" @click="scrollToSource(placement.anchor)">对应知识点</button>
              </article>
            </div>
          </section>
          <section v-if="block.source === loadedPackage.originalExamplesAnchor" class="guide-originals">
            <div class="guide-examples__heading"><span>原稿表格例句</span><label class="guide-group-toggle"><input type="checkbox" :checked="showChineseFor('originalExamples')" @change="updateGroupSetting('originalExamples', 'chinese', ($event.target as HTMLInputElement).checked)"><span>中文</span></label><label class="guide-group-toggle"><input type="checkbox" :checked="showKanaFor('originalExamples')" @change="updateGroupSetting('originalExamples', 'kana', ($event.target as HTMLInputElement).checked)"><span>假名</span></label><small>{{ loadedPackage.originalExamples.length }} 例</small></div>
          <div class="guide-original-list">
            <div v-for="item in loadedPackage.originalExamples" :key="item.id" class="guide-original-item" :class="{ 'is-selected': selectedExampleId === item.id }">
              <button type="button" class="guide-original-item__sentence" @click="selectTarget(item.id)">{{ item.ja }}</button><small>{{ item.source }}</small>
              <small v-if="showKanaFor('originalExamples')">{{ item.kana }}</small>
              <span v-if="showChineseFor('originalExamples')" class="guide-original-item__translation">{{ item.zh }}</span>
              <p class="guide-example__note">{{ item.note }}</p>
              <div class="guide-original-item__parts"><button v-for="part in item.parts" :key="part.id" type="button" :class="`is-${part.teachingClass}`" :title="part.classLabel" @click="selectTarget(item.id, part.id)">{{ part.text }}</button></div>
              <button type="button" class="guide-example__source-link" @click="scrollToSource(loadedPackage.originalExamplesAnchor)">对应原文知识点</button>
            </div>
          </div>
        </section>
        </template>
      </main>
      <aside class="guide-detail" aria-label="例句详情" :class="{ 'guide-detail--open': selectedExample }">
        <template v-if="selectedExample">
          <div class="guide-detail__heading"><span>{{ selectedExample.id }}</span><button type="button" title="关闭详情" aria-label="关闭详情" @click="selectedExampleId = ''"><X :size="16" /></button></div>
          <h2>{{ selectedExample.ja }}</h2>
          <p v-if="showChinese && selectedExample.zh">{{ selectedExample.zh }}</p>
          <section v-if="selectedExample.source"><h3>原稿来源</h3><p>{{ selectedExample.source }}</p></section>
          <section><h3>{{ selectedPart ? selectedPart.classLabel : '观察要点' }}</h3><p>{{ selectedPart ? `${selectedPart.text} · ${selectedPart.term}` : selectedExample.note }}</p></section>
          <section><h3>重点范围</h3><p>{{ focusLabel(selectedExample) }}</p></section>
          <section>
            <h3>词典快照</h3>
            <label v-if="selectedExample.dictionaryEntries.length > 1" class="guide-detail__select">词典<select v-model="selectedDictionaryEntryId"><option v-for="entry in selectedExample.dictionaryEntries" :key="entry.id" :value="entry.id">{{ entry.dictionary }}</option></select></label>
            <template v-if="selectedDictionaryEntry">
              <label v-if="(selectedDictionaryEntry.variants?.length ?? 0) > 1" class="guide-detail__select">表记<select v-model="selectedVariantId"><option v-for="variant in selectedDictionaryEntry.variants" :key="variant.id" :value="variant.id">{{ variant.form }}{{ variant.reading ? `（${variant.reading}）` : '' }}</option></select></label>
              <article><strong>{{ selectedDictionaryVariant?.form || selectedDictionaryEntry.headword }} <small>{{ selectedDictionaryVariant?.reading || selectedDictionaryEntry.reading }}</small></strong><p>{{ selectedDictionaryVariant?.body || selectedDictionaryEntry.body }}</p><small>{{ selectedDictionaryEntry.dictionary }} · {{ selectedDictionaryEntry.status }}</small></article>
              <div v-if="selectedDictionaryEntry.relatedEntryIds?.length" class="guide-related-entry"><span>相关条目</span><button v-for="id in selectedDictionaryEntry.relatedEntryIds" :key="id" type="button" @click="selectedDictionaryEntryId = id; selectedVariantId = ''">{{ selectedExample.dictionaryEntries.find((entry) => entry.id === id)?.headword || id }}</button></div>
            </template>
            <p v-else class="guide-empty">{{ selectedExample.dictionaryStatus === 'no-record' ? '已保存：当前生成结果无词典记录' : '尚未保存词典条目' }}</p>
          </section>
          <section><h3>活用记录</h3><p v-if="selectedExample.morphology.length === 0" class="guide-empty">尚未保存活用步骤</p><ol v-else><li v-for="step in selectedExample.morphology" :key="step.form"><strong>{{ step.form }} · {{ step.label }}</strong><p>{{ step.note }}</p></li></ol></section>
          <section><h3>教学标记</h3><button v-for="part in selectedExample.parts" :key="part.id" type="button" class="guide-part" :class="`is-${part.teachingClass}`" :aria-pressed="selectedPartId === part.id" @click="selectedPartId = selectedPartId === part.id ? '' : part.id"><span>{{ part.text }}</span><small>{{ part.classLabel }}</small></button></section>
        </template>
        <div v-else class="guide-detail__empty"><span>例句详情</span><p>选择例句可查看固定内容包保存的观察要点。</p></div>
        <div class="guide-review-state" :class="`is-${loadedPackage.status}`"><Check v-if="loadedPackage.status === 'reviewed'" :size="14" /><span>{{ loadedPackage.status === 'reviewed' ? '已审阅' : '草稿' }} · {{ draftPackage.version }}</span></div>
      </aside>
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
.guide-layout{display:grid;grid-template-columns:220px minmax(0,1fr) 290px;width:100%;height:100%;min-height:0;background:var(--bg-primary);color:var(--text-primary)}.guide-toc{display:flex;flex-direction:column;gap:3px;overflow:auto;padding:22px 12px;border-right:1px solid var(--border-color);background:var(--bg-secondary)}.guide-toc__label{padding:0 10px 10px;color:var(--text-muted);font-size:11px;font-weight:700}.guide-toc button{display:flex;align-items:center;justify-content:space-between;gap:8px;padding:9px 10px;border:0;border-radius:5px;background:transparent;color:var(--text-secondary);text-align:left;cursor:pointer}.guide-toc button:hover{background:var(--bg-primary);color:var(--text-primary)}.guide-toc button span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.guide-progress{margin-top:auto;padding:12px 10px;color:var(--text-muted);font-size:11px}
.guide-reading{overflow:auto;padding:34px clamp(22px,5vw,76px) 70px;scroll-behavior:smooth}.guide-source{max-width:820px;margin:0 auto 18px;font:15px/1.9 var(--font-ui);overflow-wrap:anywhere}.guide-source :deep(h1),.guide-source :deep(h2),.guide-source :deep(h3){margin:1.5em 0 .6em;line-height:1.35}.guide-source :deep(h1){font-size:24px}.guide-source :deep(h2){font-size:20px}.guide-source :deep(h3){font-size:17px}.guide-source :deep(p){margin:0 0 1em}.guide-source :deep(strong){font-weight:750;color:var(--text-primary)}.guide-source :deep(a){color:var(--accent-color)}.guide-source :deep(code){padding:1px 4px;border-radius:3px;background:var(--bg-secondary)}.guide-source :deep(blockquote){margin:12px 0;padding:8px 14px;border-left:3px solid var(--accent-color);background:var(--bg-secondary);color:var(--text-secondary)}.guide-note-inline{color:#9b6417}.guide-table-wrap{overflow:auto;margin:18px 0 22px}.guide-source :deep(table){width:100%;border-collapse:collapse;font-size:13px}.guide-source :deep(td){padding:9px 11px;border:1px solid var(--border-color);vertical-align:top}.guide-source :deep(tr:first-child){background:var(--bg-secondary);font-weight:650}.guide-source :deep(ul){padding-left:24px}
.guide-examples,.guide-originals{max-width:820px;margin:20px auto 34px;border-top:1px solid var(--border-color)}.guide-examples__heading{display:flex;justify-content:space-between;align-items:center;padding:9px 0;color:var(--text-muted);font-size:11px;font-weight:700}.guide-examples__heading small{font-weight:500}.guide-example-list{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,290px),1fr));gap:9px}.guide-example{min-width:0;padding:12px 14px;border:1px solid var(--border-color);border-radius:6px;background:var(--bg-primary)}.guide-example--selected{border-color:var(--accent-color);box-shadow:inset 3px 0 var(--accent-color)}.guide-example__sentence{display:flex;align-items:baseline;gap:10px;max-width:100%;padding:0;border:0;background:none;color:var(--text-primary);font:20px/1.7 var(--font-ja);text-align:left;cursor:pointer;overflow-wrap:anywhere}.guide-example__sentence small{flex:0 0 auto;color:var(--text-muted);font:10px var(--font-ui)}.guide-example__translation{margin:3px 0;color:var(--text-secondary);font-size:13px}.guide-example__note{margin:7px 0;color:var(--text-muted);font-size:12px;line-height:1.6}.guide-example__meta{display:flex;flex-wrap:wrap;gap:6px 12px;color:var(--text-muted);font-size:10px}.guide-original-list{display:flex;flex-wrap:wrap;gap:7px}.guide-original-list button{display:grid;gap:3px;max-width:100%;padding:7px 9px;border:1px solid var(--border-color);border-radius:5px;background:var(--bg-primary);color:var(--text-primary);text-align:left;cursor:pointer}.guide-original-list button.is-selected{border-color:var(--accent-color)}.guide-original-list small{color:var(--text-muted);font-size:10px}
.guide-detail{position:relative;overflow:auto;padding:20px 18px 52px;border-left:1px solid var(--border-color);background:var(--bg-secondary)}.guide-detail__heading{display:flex;justify-content:space-between;align-items:center;color:var(--text-muted);font-size:11px}.guide-detail__heading button{display:grid;width:30px;height:30px;place-items:center;border:0;background:transparent;color:var(--text-secondary);cursor:pointer}.guide-detail h2{font:20px/1.6 var(--font-ja);overflow-wrap:anywhere}.guide-detail>p,.guide-detail section p{color:var(--text-secondary);font-size:12px;line-height:1.65}.guide-detail section{margin-top:22px}.guide-detail h3{margin:0 0 8px;font-size:12px}.guide-detail article{padding:8px 0;border-top:1px solid var(--border-color)}.guide-empty{color:var(--text-muted)!important}.guide-part{display:flex;width:100%;justify-content:space-between;gap:8px;padding:6px 0;border:0;border-top:1px solid var(--border-color);background:transparent;color:var(--text-primary);font-size:12px;text-align:left;cursor:pointer}.guide-part small{color:var(--text-muted)}.guide-part.is-content{box-shadow:inset 3px 0 #318273;padding-left:7px}.guide-part.is-functional{box-shadow:inset 3px 0 #b16b37;padding-left:7px}.guide-part.is-particle{box-shadow:inset 3px 0 #397bb0;padding-left:7px}.guide-part.is-other{box-shadow:inset 3px 0 #9b841f;padding-left:7px}.guide-part[aria-pressed="true"]{background:var(--bg-primary)}.guide-detail__empty{padding-top:50px;color:var(--text-muted);font-size:13px}.guide-detail__empty p{font-size:12px;line-height:1.6}.guide-review-state{position:absolute;right:16px;bottom:15px;display:flex;align-items:center;gap:5px;color:var(--text-muted);font-size:11px}.guide-review-state.is-reviewed{color:#277460}
.guide-editor{display:flex;height:100%;min-height:0;flex-direction:column;gap:10px;padding:14px}.guide-editor__tools{justify-content:space-between}.guide-file-button{display:flex;align-items:center;gap:6px;cursor:pointer}.guide-file-button input{display:none}.guide-editor textarea{width:100%;min-height:0;flex:1;resize:none;padding:12px;border:1px solid var(--border-color);border-radius:5px;background:var(--bg-primary);color:var(--text-primary);font:12px/1.6 ui-monospace,Consolas,monospace}.guide-editor footer{justify-content:flex-end}.guide-editor button.primary{background:var(--accent-color);color:white}.guide-editor__notice{color:#9b6417;font-size:12px}
@media(max-width:1000px){.guide-layout{grid-template-columns:180px minmax(0,1fr)}.guide-detail{display:none}}@media(max-width:680px){.guide-layout{grid-template-columns:minmax(0,1fr)}.guide-toc{display:none}.guide-reading{padding:24px 16px 50px}.guide-actions{gap:5px;padding-right:12px}.guide-toggle{gap:4px;font-size:11px}.guide-example__sentence{font-size:18px}.guide-editor footer{flex-wrap:wrap}.guide-editor footer button{flex:1}}
.guide-examples__heading{flex-wrap:wrap;justify-content:flex-start;gap:10px}.guide-examples__heading>span{margin-right:auto}.guide-examples__heading small{margin-left:auto}.guide-group-toggle{display:flex;align-items:center;gap:3px;font-size:10px;font-weight:500;cursor:pointer}.guide-group-toggle input{accent-color:var(--accent-color)}.guide-example__kana{display:block;margin-top:2px;color:var(--text-muted);font:12px/1.5 var(--font-ja);overflow-wrap:anywhere}
.guide-part{width:100%;border:0;border-top:1px solid var(--border-color);background:transparent;color:var(--text-primary);text-align:left;cursor:pointer}.guide-part.is-content{box-shadow:inset 3px 0 #318273;padding-left:7px}.guide-part.is-functional{box-shadow:inset 3px 0 #b16b37;padding-left:7px}.guide-part.is-particle{box-shadow:inset 3px 0 #397bb0;padding-left:7px}.guide-part.is-other{box-shadow:inset 3px 0 #9b841f;padding-left:7px}.guide-part[aria-pressed="true"]{background:var(--bg-primary)}
@media(max-width:1000px){.guide-detail--open{position:fixed;z-index:5;top:58px;right:0;bottom:0;display:block;width:min(360px,94vw);border-left:1px solid var(--border-color);box-shadow:-14px 0 36px #0002}}
.guide-legend{display:flex;flex-wrap:wrap;gap:7px 14px;max-width:820px;margin:0 auto 18px;color:var(--text-muted);font-size:10px}.guide-legend span{display:flex;align-items:center;gap:5px}.guide-legend i{width:9px;height:9px;border-radius:2px}.guide-legend .is-content{background:#318273}.guide-legend .is-functional{background:#b16b37}.guide-legend .is-particle{background:#397bb0}.guide-legend .is-other{background:#9b841f}
.guide-example__capsule{display:flex;flex-wrap:wrap;align-items:baseline;gap:0;width:fit-content;max-width:100%;padding:4px 7px;border:1px solid var(--border-color);border-radius:5px;background:var(--bg-primary)}.guide-example__part{max-width:100%;padding:0;border:0;border-radius:2px;background:transparent;font:20px/1.7 var(--font-ja);overflow-wrap:anywhere;cursor:pointer}.guide-example__part.is-content{color:#276e60}.guide-example__part.is-functional{color:#9c5829}.guide-example__part.is-particle{color:#326b98}.guide-example__part.is-other{color:#826e1d}.guide-example__part:hover{background:var(--accent-light);box-shadow:inset 0 -2px var(--accent-color)}.guide-example__whole{margin-left:auto;padding:1px 3px;border:0;border-radius:3px;background:var(--bg-secondary);color:var(--text-muted);font:9px var(--font-ui);cursor:pointer}.guide-example__reading{display:flex;flex-wrap:wrap;margin-top:2px;color:var(--text-secondary);font:12px/1.5 var(--font-ja)}.guide-example__reading ruby{ruby-position:over}
.guide-original-item{display:grid;gap:4px;max-width:100%;padding:8px;border:1px solid var(--border-color);border-radius:5px;background:var(--bg-primary)}.guide-original-item.is-selected{border-color:var(--accent-color)}.guide-original-item__sentence{justify-self:start;padding:0;border:0;background:transparent;color:var(--text-primary);font:14px/1.6 var(--font-ja);text-align:left;cursor:pointer}.guide-original-item>small{color:var(--text-muted);font-size:10px}.guide-original-item__parts{display:flex;flex-wrap:wrap;gap:2px}.guide-original-item__parts button{padding:1px 3px;border:0;border-radius:2px;background:transparent;font-size:11px;cursor:pointer}.guide-original-item__parts .is-content{color:#276e60}.guide-original-item__parts .is-functional{color:#9c5829}.guide-original-item__parts .is-particle{color:#326b98}.guide-original-item__parts .is-other{color:#826e1d}
.guide-detail__select{display:grid;gap:4px;margin:8px 0;color:var(--text-muted);font-size:11px}.guide-detail__select select{min-width:0;height:30px;padding:0 8px;border:1px solid var(--border-color);border-radius:4px;background:var(--bg-primary);color:var(--text-primary)}.guide-related-entry{display:flex;flex-wrap:wrap;align-items:center;gap:6px;color:var(--text-muted);font-size:11px}.guide-related-entry button{padding:4px 6px;border:1px solid var(--border-color);border-radius:4px;background:var(--bg-primary);color:var(--text-primary);cursor:pointer}.guide-example__part.is-focused{border-bottom:1px solid currentColor}.guide-original-item__translation{color:var(--text-secondary);font-size:11px;line-height:1.5}.guide-source :deep(.guide-note-inline){color:#9b6417}
.guide-examples--contrast .guide-example-list{grid-template-columns:repeat(2,minmax(0,1fr))}@media(max-width:720px){.guide-examples--contrast .guide-example-list{grid-template-columns:minmax(0,1fr)}}
.guide-source--highlighted{outline:2px solid var(--accent-color);outline-offset:5px;transition:outline-color 1.8s ease}.guide-example__source-link{margin-top:6px;padding:0;border:0;background:transparent;color:var(--accent-color);font-size:11px;text-align:left;text-decoration:underline;text-underline-offset:2px;cursor:pointer}
</style>
