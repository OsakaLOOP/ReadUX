<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, shallowRef } from "vue";
import { BookMarked, Info } from "@lucide/vue";
import type { DictionaryLookup } from "../../types";
import type { GuideReaderSnapshot } from "../../grammar-guide/types";
import { guideLinkedLookup, hydrateGuideLookup } from "../../grammar-guide/dictionary";
import { chainGrammarLookups, grammarMorphemeLookup } from "../../grammar/particleEntries";
import { componentForRange, projectReaderSentence, type LookupTarget } from "../../reader/capsuleProjection";
import { kanjiRubySegments, readerChains, type ReaderMorphologyDetail } from "../../reader/lookupPresentation";
import { snapshotRect, type RectSnapshot } from "../../explanation/geometry";
import { EXPLANATION_CLOSE_GRACE_MS } from "../../explanation/closeGrace";
import ReaderCapsule from "../reader/ReaderCapsule.vue";
import ReaderMorphologySummary from "../reader/ReaderMorphologySummary.vue";
import ExplanationPopover from "../explanation/ExplanationPopover.vue";
import ParticleGrammarPopover from "./ParticleGrammarPopover.vue";

const props = defineProps<{
  id: string; ja: string; zh: string; note?: string; snapshot?: GuideReaderSnapshot;
  paragraphId: number; showChinese: boolean; showRuby: boolean;
}>();
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
const chainParticles = computed(() => {
  if (!active.value || !props.snapshot || !detail.value) return [];
  return chainGrammarLookups(active.value, props.snapshot.document, { detailId: detail.value.id, range: activeRange.value ?? undefined });
});
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

function cancelClose() { clearTimeout(closeTimer); closeTimer = undefined; }
function close() {
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
  return snapshot ? hydrateGuideLookup(snapshot.result, snapshot.entryKeys) : null;
}
type Part = ReturnType<typeof projectReaderSentence>[number]["parts"][number];
async function open(part: Part, source: HTMLElement) {
  clearTimeout(hoverTimer);
  cancelClose();
  if (!part.target || window.getSelection()?.isCollapsed === false) return;
  activate();
  sourceElement.value = source;
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
    if (current === generation) { wholeLoading.value = false; componentLoading.value = false; }
  }
}
function hover(part: Part, event: PointerEvent) {
  cancelClose();
  clearTimeout(hoverTimer);
  if (event.buttons) return;
  const source = event.currentTarget as HTMLElement;
  hoverTimer = setTimeout(() => void open(part, source), 60);
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
  const result = await guideLinkedLookup(target);
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
  if (!(target instanceof Element) || root.value?.contains(target) || target.closest(`[data-guide-panels="${props.id}"]`)) return;
  close();
}
function keydown(event: KeyboardEvent) { if (event.key === "Escape") close(); }
onMounted(() => {
  window.addEventListener("readux:guide-lookup", otherSource);
  window.addEventListener("pointerdown", outside);
  window.addEventListener("keydown", keydown);
  window.addEventListener("resize", onViewportChange);
  window.addEventListener("scroll", onViewportChange, true);
});
onBeforeUnmount(() => {
  close();
  window.removeEventListener("readux:guide-lookup", otherSource);
  window.removeEventListener("pointerdown", outside);
  window.removeEventListener("keydown", keydown);
  window.removeEventListener("resize", onViewportChange);
  window.removeEventListener("scroll", onViewportChange, true);
});
</script>

<template>
  <article ref="root" class="guide-example" :data-example-id="id" @pointerleave="scheduleClose">
    <BookMarked class="guide-example__bookmark" :size="17" aria-hidden="true" />
    <div class="guide-example__text">
      <template v-if="projection.length">
        <ReaderCapsule v-for="capsule in projection" :key="capsule.key" :runs="capsule.runs" :grouped="Boolean(capsule.capsuleKey)">
          <template #default="{ part }">
            <span class="reader-capsule__part" :class="{ 'reader-capsule__part--lookup': part.target }"
              :data-char-start="part.range[0]" :data-char-end="part.range[1]" :data-target-id="part.target?.id"
              @pointerenter="part.target ? hover(part, $event) : scheduleClose()"
              @pointerleave="scheduleClose" @click.stop>{{ part.text }}</span>
          </template>
        </ReaderCapsule>
      </template>
      <span v-else>{{ ja }}</span>
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
      :whole-lookup="whole" :component-lookup="displayedComponent"
      :whole-loading="hasCompanion && wholeLoading" :component-loading="hasCompanion ? componentLoading : wholeLoading"
      :whole-can-go-back="wholeHistory.length > 0"
      :component-can-go-back="(hasCompanion ? componentHistory : wholeHistory).length > 0"
      :component-label="component ? '组分' : ''" :detail="detail" :summary-visible="chains.length > 0" :grammar-lookups="chainParticles"
      :panel-prefix="`guide-${id}`" :shortcuts-enabled="false" presentation="guide"
      @enter="cancelClose" @leave="scheduleClose"
      @navigate-whole="navigate($event, false)" @navigate-component="navigate($event, Boolean(component))"
      @select-whole-form="changeForm($event, false)" @select-component-form="changeForm($event, Boolean(component))"
      @back-whole="back(false)" @back-component="back(Boolean(component))">
      <template #summary><ReaderMorphologySummary :chains="chains" /></template>
    </ExplanationPopover>
    <ParticleGrammarPopover :show="Boolean(activeParticle)" :x="particleX" :y="particleY" :lookup="activeParticle"
      @enter="cancelClose" @leave="scheduleClose" />
  </div>
</template>

<style scoped>
.guide-example { display: grid; grid-template-columns: 24px minmax(0,1fr); grid-template-rows: auto auto; align-items: baseline; column-gap: 8px; min-width: 0; margin: 0; }
.guide-example__bookmark { grid-column: 1; grid-row: 1; align-self: baseline; justify-self: center; color: var(--accent-color); }
.guide-example__text { grid-column: 2; grid-row: 1; min-width: 0; font: 21px/1.9 var(--font-ja); overflow-wrap: anywhere; }
.reader-capsule__part--lookup { cursor: pointer; }
.guide-example__translation { grid-column: 2; grid-row: 2; margin: 2px 0 0; color: var(--text-secondary); font: 13px/1.6 var(--font-ui); }
.guide-example__note { position: relative; grid-column: 1; grid-row: 2; align-self: baseline; justify-self: center; line-height: 1; }
.guide-example__note button { display: inline-grid; width: 17px; height: 17px; place-items: center; padding: 0; border: 0; border-radius: 4px; background: transparent; color: var(--accent-color); cursor: help; vertical-align: baseline; }
.guide-example__note button:hover,.guide-example__note button:focus-visible { background: var(--bg-secondary); }
.guide-example__tooltip { position: absolute; left: 0; bottom: calc(100% + 5px); z-index: 5; display: none; width: min(300px,calc(100vw - 48px)); padding: 9px 11px; border: 1px solid var(--border-color); border-radius: 5px; background: var(--bg-primary); box-shadow: 0 4px 14px color-mix(in srgb,#000 14%,transparent); color: var(--text-primary); font: 12px/1.65 var(--font-ui); white-space: normal; }
.guide-example__note:hover .guide-example__tooltip,.guide-example__note:focus-within .guide-example__tooltip { display: block; }
.guide-example__error { grid-column: 2; color: var(--text-secondary); font: 12px/1.5 var(--font-ui); }
.guide-example__error--translated { grid-row: 3; }
</style>
