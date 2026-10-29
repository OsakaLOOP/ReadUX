<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, shallowRef, watch } from "vue";
import { BookMarked, Info } from "@lucide/vue";
import type { DictionaryLookup } from "../../types";
import type { GuideReaderSnapshot } from "../../grammar-guide/types";
import { guideLinkedLookup, hydrateGuideLookup } from "../../grammar-guide/dictionary";
import { grammarMorphemeLookup } from "../../grammar/particleEntries";
import { componentForRange, projectReaderSentence, type LookupTarget } from "../../reader/capsuleProjection";
import type { CapsuleActiveRange } from "../../reader/capsuleRuns";
import { kanjiRubySegments, readerChains, readerTargetPresentation, type ReaderMorphologyDetail } from "../../reader/lookupPresentation";
import { snapshotRect, type RectSnapshot } from "../../explanation/geometry";
import { EXPLANATION_CLOSE_GRACE_MS } from "../../explanation/closeGrace";
import { useMorphologyLayout } from "../../explanation/morphologyLayout";
import { useTouchBrush } from "../../composables/useTouchBrush";
import ReaderCapsule from "../reader/ReaderCapsule.vue";
import ReaderMorphologySummary from "../reader/ReaderMorphologySummary.vue";
import ExplanationPopover from "../explanation/ExplanationPopover.vue";
import ParticleGrammarPopover from "./ParticleGrammarPopover.vue";

const props = defineProps<{
  id: string; ja: string; zh: string; note?: string; source?: string; snapshot?: GuideReaderSnapshot;
  paragraphId: number; showChinese: boolean; showRuby: boolean; lessonId: string; sheetHeight: number;
}>();
const morphologyLayout = useMorphologyLayout();
const root = ref<HTMLElement | null>(null);
const active = shallowRef<LookupTarget | null>(null);
const component = shallowRef<LookupTarget | null>(null);
const detail = shallowRef<ReaderMorphologyDetail | null>(null);
const anchor = ref<RectSnapshot | null>(null);
const wholeLookup = shallowRef<DictionaryLookup | null>(null);
const componentLookup = shallowRef<DictionaryLookup | null>(null);
const wholeHistory = ref<DictionaryLookup[]>([]);
const componentHistory = ref<DictionaryLookup[]>([]);
const wholeLoading = ref(false);
const componentLoading = ref(false);
const error = ref("");
const sourceElement = shallowRef<HTMLElement | null>(null);
const activeRange = shallowRef<[number, number] | null>(null);
const projection = computed(() => props.snapshot && props.snapshot.document.text === props.ja
  ? projectReaderSentence(props.snapshot.document, props.snapshot.group,
      props.showRuby ? props.snapshot.ruby.flatMap(kanjiRubySegments) : []) : []);
const chains = computed(() => active.value && props.snapshot ? readerChains(active.value, props.snapshot.document) : []);
const activeSourcePresentation = computed(() => active.value && props.snapshot
  ? readerTargetPresentation(active.value, props.snapshot.document)
  : null);
const componentSourcePresentation = computed(() => component.value && props.snapshot
  ? readerTargetPresentation(component.value, props.snapshot.document)
  : null);
const activeParticle = computed(() => active.value ? particleFor(active.value) : null);
const particleX = computed(() => Math.max(12, (anchor.value?.left ?? 12) + (anchor.value?.width ?? 0) / 2 - 195));
const particleY = computed(() => Math.min(window.innerHeight - 12, (anchor.value?.bottom ?? 0) + 8));
const hasCompanion = computed(() => Boolean(component.value || detail.value));
const whole = computed(() => hasCompanion.value ? wholeLookup.value : null);
const displayedComponent = computed(() => hasCompanion.value ? componentLookup.value : wholeLookup.value);
let generation = 0;
let closeTimer: ReturnType<typeof setTimeout> | undefined;
let hoverTimer: ReturnType<typeof setTimeout> | undefined;
const owner = Symbol("guide-source");
const touchBrush = useTouchBrush("[data-touch-brush]", (element, key) => {
  const [start, end] = key.split(":").map(Number);
  const part = projection.value.flatMap((capsule) => capsule.parts).find((item) => item.range[0] === start && item.range[1] === end);
  if (part?.target) void open(part, element);
});
watch([morphologyLayout.mode, () => props.sheetHeight], ([mode]) => {
  if (mode === "sheet" && active.value && sourceElement.value) void keepSourceAboveSheet(sourceElement.value);
});

function cancelClose() { clearTimeout(closeTimer); closeTimer = undefined; }
function close() {
  if (active.value) morphologyLayout.active.value = false;
  ++generation;
  clearTimeout(hoverTimer);
  cancelClose();
  active.value = null;
  component.value = null;
  detail.value = null;
  activeRange.value = null;
  wholeLookup.value = null;
  componentLookup.value = null;
  sourceElement.value = null;
}
function scheduleClose(event?: PointerEvent) {
  if (morphologyLayout.docked.value) return;
  const next = event?.relatedTarget;
  if (next instanceof Element && (root.value?.contains(next) || next.closest(`[data-guide-panels="${props.id}"]`))) return;
  clearTimeout(hoverTimer);
  if (!closeTimer) closeTimer = setTimeout(close, EXPLANATION_CLOSE_GRACE_MS);
}
function activate() {
  window.dispatchEvent(new CustomEvent("readux:guide-lookup", { detail: owner }));
}
function otherSource(event: Event) {
  if ((event as CustomEvent).detail !== owner) close();
}
function particleFor(target: LookupTarget): DictionaryLookup | null {
  const document = props.snapshot?.document;
  const morpheme = document?.morphemes.find((item) => ["助詞", "助動詞"].includes(item.pos[0] ?? "")
    && item.char_range[0] === target.char_range[0] && item.char_range[1] === target.char_range[1]);
  if (!morpheme) return null;
  return grammarMorphemeLookup(morpheme, document);
}
async function query(target: LookupTarget, selectedForm?: string) {
  const particle = particleFor(target);
  if (particle) return particle;
  const saved = props.snapshot?.lookups[target.id];
  const snapshot = saved?.[selectedForm ?? "default"];
  return snapshot ? hydrateGuideLookup(snapshot.result, snapshot.entryKeys, props.lessonId) : null;
}
type Part = ReturnType<typeof projectReaderSentence>[number]["parts"][number];
async function open(part: Part, source: HTMLElement) {
  clearTimeout(hoverTimer);
  cancelClose();
  if (!part.target || window.getSelection()?.isCollapsed === false) return;
  activate();
  morphologyLayout.active.value = true;
  morphologyLayout.contentKey.value = `${props.id}:${part.target.id}:${part.range.join(':')}`;
  sourceElement.value = source;
  if (morphologyLayout.mode.value === "sheet") void keepSourceAboveSheet(source);
  anchor.value = snapshotRect(source.getBoundingClientRect());
  const child = !part.detail ? componentForRange(
    props.snapshot?.group.inner_targets.filter((item) => item.parent_outer_id === part.target!.id) ?? [], part.range,
  ) : null;
  if (active.value?.id === part.target.id && component.value?.id === child?.id && detail.value?.id === part.detail?.id
    && activeRange.value?.[0] === part.range[0] && activeRange.value?.[1] === part.range[1]) return;
  const current = ++generation;
  active.value = part.target;
  component.value = child;
  detail.value = part.detail;
  activeRange.value = part.range;
  if (part.detailRole === "grammar") void scrollMorphologyToTop();
  wholeHistory.value = [];
  componentHistory.value = [];
  wholeLookup.value = null;
  componentLookup.value = null;
  wholeLoading.value = true;
  componentLoading.value = Boolean(child);
  error.value = "";
  try {
    const [outer, inner] = await Promise.all([query(part.target), child ? query(child) : Promise.resolve(null)]);
    if (current !== generation) return;
    wholeLookup.value = outer;
    componentLookup.value = inner;
  } catch (reason) {
    if (current === generation) error.value = reason instanceof Error ? reason.message : String(reason);
  } finally {
    if (current === generation) {
      wholeLoading.value = false;
      componentLoading.value = false;
      if (part.detailRole === "grammar") void scrollMorphologyToTop();
    }
  }
}
async function keepSourceAboveSheet(source: HTMLElement) {
  await nextTick();
  await nextTick();
  const reading = root.value?.closest<HTMLElement>(".guide-reading");
  const sheet = document.getElementById("grammar-guide-dock-pane");
  if (!reading || !sheet || !source.isConnected) return;
  let previousTop = Number.NaN;
  let stableFrames = 0;
  for (let frame = 0; frame < 12 && stableFrames < 2; frame++) {
    await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    const top = sheet.getBoundingClientRect().top;
    stableFrames = Math.abs(top - previousTop) < 0.5 ? stableFrames + 1 : 0;
    previousTop = top;
  }
  const area = reading.getBoundingClientRect();
  const boundary = Math.min(area.bottom, sheet.getBoundingClientRect().top - 32);
  const target = source.getBoundingClientRect();
  const offset = target.top < area.top ? target.top - area.top : target.bottom > boundary ? target.bottom - boundary : 0;
  console.log("sheet-source", target.bottom, boundary, offset, reading.scrollTop);
  if (offset) reading.scrollBy({ top: offset, behavior: "auto" });
}
function hover(part: Part, event: PointerEvent) {
  if (morphologyLayout.docked.value) return;
  cancelClose();
  clearTimeout(hoverTimer);
  if (event.buttons) return;
  const source = event.currentTarget as HTMLElement;
  hoverTimer = setTimeout(() => void open(part, source), 60);
}
function activatePart(part: Part, event: MouseEvent) {
  if (touchBrush.consumeClick()) return;
  if (part.target)
    void open(part, event.currentTarget as HTMLElement);
}
async function scrollMorphologyToTop() {
  if (!morphologyLayout.docked.value) return;
  await nextTick();
  await nextTick();
  const scroll = document.querySelector<HTMLElement>("#grammar-guide-dock-pane [data-explanation-scroll]");
  const entry = scroll?.querySelector<HTMLElement>(".reader-explanation--docked:not(.reader-explanation--contained)");
  if (scroll && entry) scroll.scrollTop += entry.getBoundingClientRect().top - scroll.getBoundingClientRect().top;
}
function selectConnectionPart(id: string) {
  const range = detail.value?.connection?.sentence.find((part) => part.id === id)?.range;
  if (!range) return;
  const parts = projection.value.flatMap((capsule) => capsule.parts).filter((item) => item.target
    && item.range[0] < range[1] && item.range[1] > range[0]);
  const part = parts.find((item) => item.detailRole === "grammar") ?? parts[0];
  if (!part) return;
  const source = root.value?.querySelector<HTMLElement>(`[data-char-start="${part.range[0]}"][data-char-end="${part.range[1]}"]`);
  if (source) void open(part, source);
}
function activeRangesForCapsule(capsule: ReturnType<typeof projectReaderSentence>[number]): CapsuleActiveRange[] {
  if (!active.value) return [];
  if (active.value.decision === "grammar") return capsule.parts.flatMap((part) =>
    part.range[0] < active.value!.char_range[1] && part.range[1] > active.value!.char_range[0]
      ? [{ range: part.range, tone: "grammar" as const, group: active.value!.id }]
      : []);
  if (detail.value) {
    const related = capsule.parts.filter((part) => part.target?.id === active.value!.id);
    const selected = related.filter((part) => part.detail?.id === detail.value!.id);
    if (!selected.length) return [];
    const clicked = selected.find((part) => activeRange.value
      && part.range[0] < activeRange.value[1] && part.range[1] > activeRange.value[0]) ?? selected[0];
    if (clicked.detail?.highlight) return clicked.detail.highlight.segments.map((segment) => ({ ...segment, group: clicked.detail!.highlight!.unit }));
    const detailRange = clicked.detailRange ?? clicked.range;
    return [{ range: detailRange, tone: clicked.detailRole === "stem" ? "blue" : "grammar", group: clicked.detail?.id }];
  }
  return capsule.parts.flatMap((part) => {
    if (part.range[0] >= active.value!.char_range[1] || part.range[1] <= active.value!.char_range[0]) return [];
    return [{ range: part.range, tone: part.detailRole === "grammar" || active.value!.decision === "grammar" ? "grammar" as const : "blue" as const, group: active.value!.id }];
  });
}
async function changeForm(form: string, inner: boolean) {
  const target = inner ? component.value : active.value;
  if (!target) return;
  const current = generation;
  const lookup = await query(target, form);
  if (current === generation) (inner ? componentLookup : wholeLookup).value = lookup;
}
async function navigate(target: string, inner: boolean) {
  const lookup = inner ? componentLookup : wholeLookup;
  const history = inner ? componentHistory : wholeHistory;
  const current = generation;
  const result = await guideLinkedLookup(target, props.lessonId);
  if (current !== generation || !result) return;
  if (lookup.value) history.value.push(lookup.value);
  lookup.value = result;
}
function back(inner: boolean) {
  const history = inner ? componentHistory : wholeHistory;
  const previous = history.value.pop();
  if (previous) (inner ? componentLookup : wholeLookup).value = previous;
}
function onViewportChange() {
  if (active.value && sourceElement.value?.isConnected) anchor.value = snapshotRect(sourceElement.value.getBoundingClientRect());
}
function outside(event: PointerEvent) {
  const target = event.target;
  if (target instanceof Element && target.closest(".morphology-layout-toggle")) return;
  if (target instanceof Element && target.closest("[data-target-id]")) return;
  if (target instanceof Element && target.closest(`[data-explanation-panel^="guide-${props.id}-"]`)) return;
  if (target instanceof Element && target.closest("[data-guide-layout-pane]")) {
    if (target.matches("[data-guide-layout-pane]")) close();
    return;
  }
  if (!(target instanceof Element) || target.closest(`[data-guide-panels="${props.id}"]`)) return;
  close();
}
function keydown(event: KeyboardEvent) {
  if (event.key !== "Escape" || !active.value) return;
  event.preventDefault();
  event.stopImmediatePropagation();
  const source = sourceElement.value;
  close();
  source?.focus({ preventScroll: true });
}
function monitorPointer(event: PointerEvent) {
  if (!active.value || morphologyLayout.docked.value) return;
  const inside = event.target instanceof Element && event.target.closest(`[data-explanation-panel^="guide-${props.id}-"]`);
  if (inside) cancelClose();
  else if (!(event.target instanceof Node && root.value?.contains(event.target))) scheduleClose();
}
onMounted(() => {
  window.addEventListener("pointermove", monitorPointer);
  window.addEventListener("readux:guide-lookup", otherSource);
  window.addEventListener("pointerdown", outside);
  window.addEventListener("keydown", keydown, true);
  window.addEventListener("resize", onViewportChange);
  window.addEventListener("scroll", onViewportChange, true);
});
onBeforeUnmount(() => {
  window.removeEventListener("pointermove", monitorPointer);
  close();
  window.removeEventListener("readux:guide-lookup", otherSource);
  window.removeEventListener("pointerdown", outside);
  window.removeEventListener("keydown", keydown, true);
  window.removeEventListener("resize", onViewportChange);
  window.removeEventListener("scroll", onViewportChange, true);
});
</script>

<template>
  <article ref="root" class="guide-example" :data-example-id="id" @pointerleave="scheduleClose">
    <BookMarked class="guide-example__bookmark" :size="17" aria-hidden="true" />
    <div class="guide-example__text">
      <span class="guide-example__index">{{ id }}</span>
      <span class="guide-example__scroll">
        <template v-if="projection.length">
          <ReaderCapsule v-for="capsule in projection" :key="capsule.key" :runs="capsule.runs" :active-ranges="activeRangesForCapsule(capsule)" :grouped="Boolean(capsule.capsuleKey)">
            <template #default="{ part }">
              <span class="reader-capsule__part" :class="{ 'reader-capsule__part--lookup': part.target }"
                :tabindex="part.target ? 0 : undefined" :role="part.target ? 'button' : undefined"
                :data-touch-brush="part.target ? part.range.join(':') : undefined"
                @pointerdown="touchBrush.start"
                @keydown.enter.stop.prevent="part.target && open(part, $event.currentTarget as HTMLElement)"
                @keydown.space.stop.prevent="part.target && open(part, $event.currentTarget as HTMLElement)"
                :data-char-start="part.range[0]" :data-char-end="part.range[1]" :data-target-id="part.target?.id" :data-detail-role="part.detailRole"
                @pointerenter="part.target ? hover(part, $event) : scheduleClose()"
                @pointerleave="scheduleClose" @click.stop="activatePart(part, $event)">{{ part.text }}</span>
            </template>
          </ReaderCapsule>
        </template>
        <span v-else>{{ ja }}</span>
        <span v-if="source" class="guide-example__source">{{ source }}</span>
      </span>
    </div>
    <span v-if="note" class="guide-example__note">
      <button type="button" aria-label="查看例句讲解"><Info :size="15" aria-hidden="true" /></button>
      <span class="guide-example__tooltip" role="tooltip">{{ note }}</span>
    </span>
    <p v-if="showChinese && zh" class="guide-example__translation">{{ zh }}</p>
    <p v-if="error" class="guide-example__error" :class="{ 'guide-example__error--translated': showChinese && zh }">{{ error }}</p>
  </article>
  <div :data-guide-panels="id">
    <ExplanationPopover v-if="!activeParticle" :show="Boolean(active)" :anchor="anchor" :component-anchor="anchor"
      :whole-token="null" :component-token="null"
      :whole-headword="active?.surface" :component-headword="component?.surface || active?.surface"
      :whole-source="wholeHistory.length === 0 ? activeSourcePresentation : null"
      :component-source="(hasCompanion ? componentHistory.length === 0 : wholeHistory.length === 0) ? (componentSourcePresentation || activeSourcePresentation) : null"
      :whole-lookup="whole" :component-lookup="displayedComponent"
      :whole-loading="hasCompanion && wholeLoading" :component-loading="hasCompanion ? componentLoading : wholeLoading"
      :whole-can-go-back="wholeHistory.length > 0"
      :component-can-go-back="(hasCompanion ? componentHistory : wholeHistory).length > 0"
      :component-label="component ? '组分' : ''" :detail="detail" :summary-visible="chains.length > 0"
      :panel-prefix="`guide-${id}`" :shortcuts-enabled="false" presentation="guide"
      layout-target="#grammar-guide-dock-pane-content"
      @enter="cancelClose" @leave="scheduleClose" @select-connection="selectConnectionPart"
      @navigate-whole="navigate($event, false)" @navigate-component="navigate($event, Boolean(component))"
      @select-whole-form="changeForm($event, false)" @select-component-form="changeForm($event, Boolean(component))"
      @back-whole="back(false)" @back-component="back(Boolean(component))">
      <template #summary><ReaderMorphologySummary :chains="chains" /></template>
    </ExplanationPopover>
    <Teleport v-if="activeParticle" defer :disabled="!morphologyLayout.docked.value" :to="morphologyLayout.docked.value ? '#grammar-guide-dock-pane-content' : 'body'">
      <ParticleGrammarPopover :show="Boolean(activeParticle)" :x="particleX" :y="particleY" :lookup="activeParticle"
        :docked="morphologyLayout.docked.value" :panel-id="`guide-${id}-particle-panel`" @enter="cancelClose" @leave="scheduleClose" />
    </Teleport>
  </div>
</template>

<style scoped>
.guide-example { display: grid; grid-template-columns: 24px minmax(0,1fr); grid-template-rows: auto auto; align-items: baseline; column-gap: 8px; min-width: 0; margin: 0; }
.guide-example__bookmark { grid-column: 1; grid-row: 1; align-self: baseline; justify-self: center; color: var(--accent-color); }
.guide-example__text { display: flex; align-items: baseline; gap: 8px; grid-column: 2; grid-row: 1; min-width: 0; font: 21px/1.9 var(--font-ja); overflow-wrap: anywhere; }
.guide-example__index { flex: none; color: var(--accent-color); font: 700 11px/1 var(--font-ui); letter-spacing: .02em; }
.guide-example__scroll { box-sizing: border-box; min-width: 0; margin-inline: -8px; padding-inline: 8px; overflow-x: auto; overflow-y: hidden; overscroll-behavior-inline: contain; scrollbar-width: thin; white-space: nowrap; }
.guide-example__source { display: block; margin-top: 1px; color: var(--text-secondary); font: 12px/1.5 var(--font-ui); white-space: normal; }
.reader-capsule__part--lookup { cursor: pointer; touch-action: pan-y; }
.guide-example__translation { grid-column: 2; grid-row: 2; margin: 2px 0 0; color: var(--text-secondary); font: 13px/1.6 var(--font-ui); }
.guide-example__note { position: relative; grid-column: 1; grid-row: 2; align-self: baseline; justify-self: center; line-height: 1; }
.guide-example__note button { display: inline-grid; width: 17px; height: 17px; place-items: center; padding: 0; border: 0; border-radius: 4px; background: transparent; color: var(--accent-color); cursor: help; vertical-align: baseline; }
.guide-example__note button:hover,.guide-example__note button:focus-visible { background: var(--bg-secondary); }
.guide-example__tooltip { position: absolute; left: 0; bottom: calc(100% + 5px); z-index: 5; display: none; width: min(300px,calc(100vw - 48px)); padding: 9px 11px; border: 1px solid var(--border-color); border-radius: 5px; background: var(--bg-primary); box-shadow: 0 4px 14px color-mix(in srgb,#000 14%,transparent); color: var(--text-primary); font: 12px/1.65 var(--font-ui); white-space: normal; }
.guide-example__note:hover .guide-example__tooltip,.guide-example__note:focus-within .guide-example__tooltip { display: block; }
.guide-example__error { grid-column: 2; color: var(--text-secondary); font: 12px/1.5 var(--font-ui); }
.guide-example__error--translated { grid-row: 3; }
</style>
