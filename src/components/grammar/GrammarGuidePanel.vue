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
  ruby: { range: [number, number]; base: string; reading: string }[];
  parts: { range: [number, number]; text: string; teachingClass: string; term: string; focus: boolean; detailId: string }[];
  targets: { id: string; range: [number, number]; detailId: string; entryIds: string[]; childTargetIds: string[] }[];
  dictionaryEntries: { id: string; dictionary: string; headword: string; reading: string; body: string; status: string }[];
  morphology: { form: string; label: string; note: string }[];
};
type GuidePackage = {
  lessonId: string;
  title: string;
  version: number;
  status: "draft" | "reviewed";
  sourceMarkdown: string;
  placements: { afterBlock: number; group: string; exampleIds: string[]; layout: "list" | "contrast"; anchor: string }[];
  originalExamples: { id: string; group: string; source: string; ja: string; kana: string; note: string }[];
  examples: GuideExample[];
  generatedFrom: Record<string, string>;
};
const bundledPackage = basePackage as unknown as GuidePackage;

const props = defineProps<{ show: boolean }>();
const emit = defineEmits<{ close: [] }>();
const storageKey = "readux.grammarGuide.1.1";
const clone = <T,>(value: T): T => JSON.parse(JSON.stringify(value)) as T;
const loadedPackage = ref<GuidePackage>(clone(bundledPackage));
const draftPackage = ref<GuidePackage>(clone(bundledPackage));
const jsonText = ref("");
const editorOpen = ref(false);
const showChinese = ref(true);
const showKana = ref(false);
const selectedExampleId = ref("");
const error = ref("");
const blocks = computed(() => markdownBlocks(loadedPackage.value.sourceMarkdown));
const headings = computed(() => blocks.value.map((block, index) => ({ block, index })).filter(({ block }) => block.kind === "heading"));
const placementMap = computed(() => new Map(loadedPackage.value.placements.map((item) => [item.afterBlock, item])));
const exampleMap = computed(() => new Map(loadedPackage.value.examples.map((item) => [item.id, item])));
const selectedExample = computed(() => exampleMap.value.get(selectedExampleId.value) ?? null);
const selectedOriginal = computed(() => loadedPackage.value.originalExamples.find((item) => item.id === selectedExampleId.value) ?? null);

watch(() => props.show, (show) => {
  if (!show) return;
  try {
    const saved = localStorage.getItem(storageKey);
    if (saved) draftPackage.value = JSON.parse(saved) as GuidePackage;
  } catch { /* 页面继续使用内置课节 */ }
}, { immediate: true });

function setNotice(message: string) { error.value = message; window.setTimeout(() => { if (error.value === message) error.value = ""; }, 4000); }
function makeDraft() {
  draftPackage.value = { ...clone(bundledPackage), status: "draft", version: loadedPackage.value.version + 1 };
  jsonText.value = JSON.stringify(draftPackage.value, null, 2);
  editorOpen.value = true;
}
function openEditor() {
  jsonText.value = JSON.stringify(draftPackage.value, null, 2);
  editorOpen.value = true;
}
function parseEditor(): GuidePackage | null {
  try {
    const value = JSON.parse(jsonText.value) as GuidePackage;
    if (value.lessonId !== "grammar-1.1" || !Array.isArray(value.examples) || !Array.isArray(value.placements)) throw new Error("课节编号或数据结构不正确。");
    const ids = new Set(value.examples.map((item) => item.id));
    const missing = value.placements.flatMap((item) => item.exampleIds).find((id) => !ids.has(id));
    if (missing) throw new Error(`插入位置引用了不存在的例句 ${missing}。`);
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
  localStorage.setItem(storageKey, JSON.stringify(value));
  jsonText.value = JSON.stringify(value, null, 2);
  setNotice("草稿已保存。");
}
function reviewDraft() {
  const value = parseEditor();
  if (!value) return;
  value.status = "reviewed";
  draftPackage.value = value;
  localStorage.setItem(storageKey, JSON.stringify(value));
  jsonText.value = JSON.stringify(value, null, 2);
  setNotice("内容已标记为已审阅。");
}
function loadDraft() {
  const value = parseEditor();
  if (!value) return;
  loadedPackage.value = clone(value);
  draftPackage.value = clone(value);
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
        <template v-for="(block, index) in blocks" :key="index">
          <article :id="`guide-block-${index}`" class="guide-source" :class="`guide-source--${block.kind}`" v-html="htmlForBlock(index)" />
          <section v-if="placementMap.has(index)" class="guide-examples" :class="`guide-examples--${placementMap.get(index)?.layout}`" :aria-label="placementMap.get(index)?.group">
            <div class="guide-examples__heading"><span>{{ placementMap.get(index)?.group }}</span><small>{{ placementMap.get(index)?.exampleIds.length }} 例</small></div>
            <div class="guide-example-list">
              <article v-for="id in placementMap.get(index)?.exampleIds" :key="id" class="guide-example" :class="{ 'guide-example--selected': selectedExampleId === id }">
                <button class="guide-example__sentence" type="button" @click="selectedExampleId = selectedExampleId === id ? '' : id">
                  <ruby v-if="showKana && exampleMap.get(id)?.ruby?.length" v-for="ruby in exampleMap.get(id)?.ruby" :key="`${id}-${ruby.range[0]}`"><rb>{{ ruby.base }}</rb><rt>{{ ruby.reading }}</rt></ruby>
                  <span v-else>{{ showKana ? exampleMap.get(id)?.kana || exampleMap.get(id)?.ja : exampleMap.get(id)?.ja }}</span>
                  <small>{{ id }}</small>
                </button>
                <p v-if="showChinese" class="guide-example__translation">{{ exampleMap.get(id)?.zh }}</p>
                <p class="guide-example__note">{{ exampleMap.get(id)?.note }}</p>
                <div class="guide-example__meta"><span>{{ focusLabel(exampleMap.get(id)!) }}</span><span>{{ exampleMap.get(id)?.parts.length || 0 }} 个标注成分</span></div>
              </article>
            </div>
          </section>
        </template>
        <section v-if="loadedPackage.originalExamples.length" class="guide-originals">
          <div class="guide-examples__heading"><span>原稿例句胶囊</span><small>{{ loadedPackage.originalExamples.length }} 例</small></div>
          <div class="guide-original-list">
            <button v-for="item in loadedPackage.originalExamples" :key="item.id" type="button" @click="selectedExampleId = selectedExampleId === item.id ? '' : item.id" :class="{ 'is-selected': selectedExampleId === item.id }">
              <span>{{ item.ja }}</span><small>{{ item.source }}</small>
            </button>
          </div>
        </section>
      </main>
      <aside class="guide-detail" aria-label="例句详情">
        <template v-if="selectedExample || selectedOriginal">
          <div class="guide-detail__heading"><span>{{ selectedExample?.id || selectedOriginal?.id }}</span><button type="button" title="关闭详情" aria-label="关闭详情" @click="selectedExampleId = ''"><X :size="16" /></button></div>
          <h2>{{ selectedExample?.ja || selectedOriginal?.ja }}</h2>
          <p v-if="showChinese && selectedExample">{{ selectedExample.zh }}</p>
          <section v-if="selectedOriginal"><h3>原稿来源</h3><p>{{ selectedOriginal.source }} · {{ selectedOriginal.note }}</p></section>
          <template v-if="selectedExample">
          <section><h3>观察要点</h3><p>{{ selectedExample.note }}</p></section>
          <section><h3>重点范围</h3><p>{{ focusLabel(selectedExample) }}</p></section>
          <section><h3>词典快照</h3><p v-if="selectedExample.dictionaryEntries.length === 0" class="guide-empty">尚未保存词典条目</p><article v-for="entry in selectedExample.dictionaryEntries" :key="entry.id"><strong>{{ entry.headword }} <small>{{ entry.reading }}</small></strong><p>{{ entry.body }}</p><small>{{ entry.dictionary }} · {{ entry.status }}</small></article></section>
          <section><h3>活用记录</h3><p v-if="selectedExample.morphology.length === 0" class="guide-empty">尚未保存活用步骤</p><ol v-else><li v-for="step in selectedExample.morphology" :key="step.form"><strong>{{ step.form }} · {{ step.label }}</strong><p>{{ step.note }}</p></li></ol></section>
          <section><h3>教学标记</h3><p v-if="selectedExample.parts.length === 0" class="guide-empty">整体范围可选择；成分标注可在 JSON 审阅器中编辑。</p><div v-for="part in selectedExample.parts" :key="part.range.join('-')" class="guide-part"><span>{{ part.text }}</span><small>{{ part.term }} · {{ part.teachingClass }}</small></div></section>
          </template>
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
.guide-detail{position:relative;overflow:auto;padding:20px 18px 52px;border-left:1px solid var(--border-color);background:var(--bg-secondary)}.guide-detail__heading{display:flex;justify-content:space-between;align-items:center;color:var(--text-muted);font-size:11px}.guide-detail__heading button{display:grid;width:30px;height:30px;place-items:center;border:0;background:transparent;color:var(--text-secondary);cursor:pointer}.guide-detail h2{font:20px/1.6 var(--font-ja);overflow-wrap:anywhere}.guide-detail>p,.guide-detail section p{color:var(--text-secondary);font-size:12px;line-height:1.65}.guide-detail section{margin-top:22px}.guide-detail h3{margin:0 0 8px;font-size:12px}.guide-detail article{padding:8px 0;border-top:1px solid var(--border-color)}.guide-empty{color:var(--text-muted)!important}.guide-part{display:flex;justify-content:space-between;gap:8px;padding:6px 0;border-top:1px solid var(--border-color);font-size:12px}.guide-part small{color:var(--text-muted)}.guide-detail__empty{padding-top:50px;color:var(--text-muted);font-size:13px}.guide-detail__empty p{font-size:12px;line-height:1.6}.guide-review-state{position:absolute;right:16px;bottom:15px;display:flex;align-items:center;gap:5px;color:var(--text-muted);font-size:11px}.guide-review-state.is-reviewed{color:#277460}
.guide-editor{display:flex;height:100%;min-height:0;flex-direction:column;gap:10px;padding:14px}.guide-editor__tools{justify-content:space-between}.guide-file-button{display:flex;align-items:center;gap:6px;cursor:pointer}.guide-file-button input{display:none}.guide-editor textarea{width:100%;min-height:0;flex:1;resize:none;padding:12px;border:1px solid var(--border-color);border-radius:5px;background:var(--bg-primary);color:var(--text-primary);font:12px/1.6 ui-monospace,Consolas,monospace}.guide-editor footer{justify-content:flex-end}.guide-editor button.primary{background:var(--accent-color);color:white}.guide-editor__notice{color:#9b6417;font-size:12px}
@media(max-width:1000px){.guide-layout{grid-template-columns:180px minmax(0,1fr)}.guide-detail{display:none}}@media(max-width:680px){.guide-layout{grid-template-columns:minmax(0,1fr)}.guide-toc{display:none}.guide-reading{padding:24px 16px 50px}.guide-actions{gap:5px;padding-right:12px}.guide-toggle{gap:4px;font-size:11px}.guide-example__sentence{font-size:18px}.guide-editor footer{flex-wrap:wrap}.guide-editor footer button{flex:1}}
</style>
