<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import type { AnnotatedToken, DictionaryLookup } from "../../types";
import { floatDebug } from "../../explanation/floatDebug";
import { explanationPanelWidth, morphologyPanelWidths, measureIntrinsicPanel, placeExplanationPanels, placeMorphologyPanels, type PanelPlacement, type PopoverPlacement, type RectSnapshot, type Size } from "../../explanation/geometry";
import TooltipPanel from "../TooltipPanel.vue";
import ReaderExplanationBubble from "../reader/ReaderExplanationBubble.vue";
import ReaderMorphologyEntry from "../reader/ReaderMorphologyEntry.vue";
import ReaderConnectionTree from "../reader/ReaderConnectionTree.vue";
import { morphologyLocalTransition } from "../../reader/morphologyBubble";
import type { ReaderMorphologyDetail } from "../../reader/lookupPresentation";
import type { ReaderTargetPresentation } from "../../reader/lookupPresentation";
import { useMorphologyLayout } from "../../explanation/morphologyLayout";

const props = defineProps<{
  show: boolean;
  anchor: RectSnapshot | null;
  componentAnchor: RectSnapshot | null;
  wholeToken: AnnotatedToken | null;
  wholeLookup: DictionaryLookup | null;
  wholeLoading: boolean;
  wholeCanGoBack: boolean;
  componentToken: AnnotatedToken | null;
  componentLookup: DictionaryLookup | null;
  componentLoading: boolean;
  componentCanGoBack: boolean;
  componentLabel: string;
  panelPrefix?: string;
  shortcutsEnabled?: boolean;
  presentation?: "reader" | "guide";
  wholeHeadword?: string;
  componentHeadword?: string;
  wholeSource?: ReaderTargetPresentation | null;
  componentSource?: ReaderTargetPresentation | null;
  summaryVisible?: boolean;
  detail?: ReaderMorphologyDetail | null;
  layoutTarget?: string;
}>();
const morphologyLayout = useMorphologyLayout();

const emit = defineEmits<{
  enter: [event: PointerEvent];
  leave: [event: PointerEvent];
  navigateWhole: [target: string];
  navigateComponent: [target: string];
  selectWholeForm: [formId: string];
  selectComponentForm: [formId: string];
  selectConnection: [id: string];
  backWhole: [];
  backComponent: [];
}>();

const placement = ref<PopoverPlacement>({
  component: { left: -10000, top: -10000, width: 420, height: 320, maxHeight: 620 },
});
const panelWidth = ref(420);
const connectionPlacement = ref<PanelPlacement | null>(null);
let observer: ResizeObserver | null = null;

function intrinsicPanelSize(panelId: string): Size | undefined {
  const panel = document.getElementById(panelId);
  if (!panel) return undefined;
  return measureIntrinsicPanel(panel);
}

const panelPrefix = computed(() => props.panelPrefix || "explanation");
const wholePanelId = computed(() => `${panelPrefix.value}-whole-panel`);
const componentPanelId = computed(() => `${panelPrefix.value}-component-panel`);
const connectionPanelId = computed(() => `${panelPrefix.value}-connection-panel`);
const enableShortcuts = computed(() => props.shortcutsEnabled ?? true);

function place() {
  if (morphologyLayout.docked.value) return;
  if (!props.show || !props.anchor || !props.componentAnchor) {
    floatDebug.record("layout", "explanation-popover", "place-skipped", "render-input-missing", {
      show: props.show,
      hasAnchor: Boolean(props.anchor),
      hasComponentAnchor: Boolean(props.componentAnchor),
    });
    return;
  }
  const componentSize = intrinsicPanelSize(componentPanelId.value);
  const wholeSize = intrinsicPanelSize(wholePanelId.value);
  const connectionSize = intrinsicPanelSize(connectionPanelId.value);
  const entrySize = props.detail?.entryVisible === false ? null : componentSize;
  if (props.detail?.connection && connectionSize && entrySize !== undefined) {
    const stacked = placeMorphologyPanels(props.anchor, entrySize, connectionSize,
      wholeSize ?? { width: panelWidth.value, height: 320 }, { width: window.innerWidth, height: window.innerHeight });
    placement.value = { component: stacked.entry, whole: stacked.dictionary };
    connectionPlacement.value = stacked.connection;
    return;
  }
  if (!componentSize) {
    floatDebug.record("layout", "explanation-popover", "place-skipped", "component-panel-unmounted");
    return;
  }
  connectionPlacement.value = null;
  placement.value = placeExplanationPanels(
    props.anchor,
    props.componentAnchor,
    componentSize,
    { width: window.innerWidth, height: window.innerHeight },
    wholeSize,
  );
  floatDebug.snapshot("layout", {
    viewport: { width: window.innerWidth, height: window.innerHeight },
    panelWidth: panelWidth.value,
    componentIntrinsic: sizeSnapshot(componentSize),
    wholeIntrinsic: sizeSnapshot(wholeSize),
    component: placementSnapshot(placement.value.component),
    whole: placementSnapshot(placement.value.whole),
  });
  floatDebug.record("layout", "explanation-popover", "placed", wholeSize ? "dual-panel" : "single-panel", {
    panelWidth: panelWidth.value,
    component: placementSnapshot(placement.value.component),
    whole: placementSnapshot(placement.value.whole),
  });
  void nextTick().then(capturePanelBoxes);
}

async function connectAndPlace() {
  if (!props.show) {
    floatDebug.record("layout", "explanation-popover", "connect-skipped", "render-gate-closed");
    return;
  }
  const hasWhole = Boolean(props.wholeLookup || props.wholeLoading);
  panelWidth.value = props.detail?.connection ? morphologyPanelWidths(window.innerWidth).morphology : explanationPanelWidth(window.innerWidth, hasWhole);
  await nextTick();
  observer?.disconnect();
  observer = new ResizeObserver(place);
  const contents = [
    document.querySelector(`[data-explanation-content="${wholePanelId.value}"]`),
    document.querySelector(`[data-explanation-content="${componentPanelId.value}"]`),
    document.getElementById(componentPanelId.value),
    document.getElementById(connectionPanelId.value),
  ].filter((content): content is Element => Boolean(content));
  contents.forEach((content) => observer?.observe(content));
  floatDebug.record("layout", "explanation-popover", "observer-connected", `${contents.length} content nodes`);
  place();
}

function sizeSnapshot(size: Size | undefined) {
  return size ? { width: Math.round(size.width), height: Math.round(size.height) } : null;
}

function placementSnapshot(value: PopoverPlacement["component"] | undefined) {
  if (!value) return null;
  return {
    left: Math.round(value.left),
    top: Math.round(value.top),
    width: Math.round(value.width),
    height: Math.round(value.height),
    maxHeight: Math.round(value.maxHeight),
  };
}

function capturePanelBoxes() {
  const component = document.getElementById(componentPanelId.value);
  const whole = document.getElementById(wholePanelId.value);
  floatDebug.snapshot("panelBoxes", {
    component: elementRectSnapshot(component),
    whole: elementRectSnapshot(whole),
    plannedComponent: placementSnapshot(placement.value.component),
    plannedWhole: placementSnapshot(placement.value.whole),
  });
}

function elementRectSnapshot(element: HTMLElement | null) {
  if (!element) return null;
  const rect = element.getBoundingClientRect();
  return {
    id: element.id,
    left: Math.round(rect.left),
    top: Math.round(rect.top),
    right: Math.round(rect.right),
    bottom: Math.round(rect.bottom),
    width: Math.round(rect.width),
    height: Math.round(rect.height),
    scrollHeight: Math.round(element.scrollHeight),
  };
}

watch(
  () => [props.show, props.anchor, props.componentAnchor, props.wholeLookup, props.wholeLoading, props.componentLookup, props.componentLoading, props.detail, morphologyLayout.mode.value],
  connectAndPlace,
  { flush: "post" },
);

onMounted(() => window.addEventListener("resize", connectAndPlace));
onBeforeUnmount(() => {
  observer?.disconnect();
  window.removeEventListener("resize", connectAndPlace);
});
</script>

<template>
  <Teleport v-if="show" defer :disabled="!morphologyLayout.docked.value" :to="morphologyLayout.docked.value && layoutTarget ? layoutTarget : 'body'">
  <TooltipPanel
    v-if="wholeLookup || wholeLoading"
    :panel-id="wholePanelId"
    :show="show"
    :x="placement.whole?.left ?? -10000"
    :y="placement.whole?.top ?? -10000"
    :width="placement.whole?.width ?? panelWidth"
    :max-height="placement.whole?.maxHeight ?? 620"
    :token="wholeToken"
    :headword="wholeHeadword"
    :source-presentation="wholeSource"
    :summary-visible="summaryVisible"
    :lookup="wholeLookup"
    :loading="wholeLoading"
    :can-go-back="wholeCanGoBack"
    :shortcuts-enabled="false"
    :presentation="presentation"
    :docked="morphologyLayout.docked.value"
    @enter="emit('enter', $event)"
    @leave="emit('leave', $event)"
    @navigate="emit('navigateWhole', $event)"
    @select-form="emit('selectWholeForm', $event)"
    @back="emit('backWhole')"
  >
    <template #summary><slot name="summary" /></template>
  </TooltipPanel>
  <ReaderExplanationBubble
    v-if="detail && detail.entryVisible !== false"
    :id="componentPanelId"
    :data-explanation-panel="componentPanelId"
    :show="show"
    :x="placement.component.left"
    :y="placement.component.top"
    :width="placement.component.width"
    :max-height="placement.component.maxHeight"
    :docked="morphologyLayout.docked.value"
    :title="detail.entry?.usage || detail.title"
    label="活用词条"
    :surface="detail.entry?.headword || detail.surface"
    :identity="detail.entry?.identity"
    :conjugation="detail.entry?.conjugation || detail.conjugation"
    :transition="detail.entry?.transition || morphologyLocalTransition(detail.description, true)"
    @enter="emit('enter', $event)"
    @leave="emit('leave', $event)"
  >
    <ReaderMorphologyEntry :key="detail.id" :detail="detail" />
  </ReaderExplanationBubble>
  <TooltipPanel
    v-else-if="!detail"
    :panel-id="componentPanelId"
    :show="show"
    :x="placement.component.left"
    :y="placement.component.top"
    :width="panelWidth"
    :max-height="placement.component.maxHeight"
    :token="componentToken"
    :headword="componentHeadword"
    :source-presentation="componentSource"
    :summary-visible="!wholeLookup && summaryVisible"
    :lookup="componentLookup"
    :loading="componentLoading"
    :kind-label="componentLabel"
    :can-go-back="componentCanGoBack"
    :shortcuts-enabled="enableShortcuts"
    :presentation="presentation"
    :docked="morphologyLayout.docked.value"
    @enter="emit('enter', $event)"
    @leave="emit('leave', $event)"
    @navigate="emit('navigateComponent', $event)"
    @select-form="emit('selectComponentForm', $event)"
    @back="emit('backComponent')"
  >
    <template #summary><slot name="summary" /></template>
  </TooltipPanel>
  <ReaderExplanationBubble v-if="detail?.connection"
    :id="connectionPanelId" :data-explanation-panel="connectionPanelId" :show="show"
    :x="connectionPlacement?.left ?? -10000" :y="connectionPlacement?.top ?? -10000"
    :width="connectionPlacement?.width ?? panelWidth" :max-height="connectionPlacement?.maxHeight ?? 300" :docked="morphologyLayout.docked.value"
    title="" surface="" label="接续方式与接续规则"
    contained v-slot="{ available }"
    @enter="emit('enter', $event)" @leave="emit('leave', $event)">
    <ReaderConnectionTree :key="detail.connection.sentence[0]?.id ?? detail.connection.id" :diagram="detail.connection" :max-height="morphologyLayout.docked.value ? Number.POSITIVE_INFINITY : available" :scroll-mode="morphologyLayout.docked.value ? 'flow' : 'split'" @resize="place" @select="emit('selectConnection', $event)" />
  </ReaderExplanationBubble>
  </Teleport>
</template>
