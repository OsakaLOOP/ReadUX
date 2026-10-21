<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from "vue";
import type {
  AnnotatedToken,
  DictionaryLookupRequest,
  GrammarDictionaryTarget,
} from "../../types";
import { useExplanationInteraction } from "../../composables/useExplanationInteraction";
import { useExplanationSession } from "../../composables/useExplanationSession";
import type { RectSnapshot } from "../../explanation/geometry";
import type { GuideLookupSnapshot } from "../../grammar-guide/types";
import AnalyzedTextItem from "../reader/AnalyzedTextItem.vue";
import ExplanationPopover from "../explanation/ExplanationPopover.vue";
import GrammarPopover from "../explanation/GrammarPopover.vue";

const props = defineProps<{
  id: string;
  ja: string;
  zh: string;
  tokens: AnnotatedToken[];
  lookups: GuideLookupSnapshot[];
  paragraphId: number;
  showChinese: boolean;
}>();

const itemRef = ref<InstanceType<typeof AnalyzedTextItem> | null>(null);
const rootElement = computed(() => itemRef.value?.rootElement ?? null);
const lookupWord = async (request: DictionaryLookupRequest) => {
  const snapshot = props.lookups.find(({ request: saved }) =>
    saved.word === request.word
    && (saved.observedForm ?? "") === (request.observedForm ?? "")
    && (saved.reading ?? "") === (request.reading ?? "")
    && (saved.pos?.major ?? "") === (request.pos?.major ?? ""),
  );
  return snapshot?.result ?? null;
};

const explanation = useExplanationSession(lookupWord);
const interaction = useExplanationInteraction({
  findToken(paragraphId, tokenIndex) {
    return paragraphId === props.paragraphId ? props.tokens[tokenIndex] : null;
  },
  session: explanation,
});

function handlePointerOver(event: PointerEvent) {
  interaction.handleParagraphPointerOver(event);
}

function handlePointerOut(event: PointerEvent) {
  interaction.handleParagraphPointerOut(event);
}

function panelAnchor(value: RectSnapshot | null): RectSnapshot | null {
  return value;
}

function openGrammarDictionary(target: GrammarDictionaryTarget) {
  const root = rootElement.value;
  if (!root) return;
  for (let tokenIndex = 0; tokenIndex < props.tokens.length; tokenIndex++) {
    const token = props.tokens[tokenIndex];
    const morphemeIndex = token.bunsetsu.morphemes.findIndex((morpheme) =>
      morpheme.char_range[0] === target.char_range[0]
      && morpheme.char_range[1] === target.char_range[1],
    );
    if (morphemeIndex < 0) continue;
    const capsule = root.querySelector<HTMLElement>(`[data-token-index="${tokenIndex}"]`);
    const morpheme = capsule?.querySelector<HTMLElement>(`[data-morpheme-index="${morphemeIndex}"]`);
    if (!capsule || !morpheme) return;
    explanation.focusMorpheme(
      { paragraphId: props.paragraphId, tokenIndex, morphemeIndex },
      token,
      capsule,
      morpheme,
    );
    return;
  }
}

onBeforeUnmount(() => explanation.closeAll("guide-example-unmounted"));
</script>

<template>
  <AnalyzedTextItem
    ref="itemRef"
    container="article"
    class="guide-example"
    :data-example-id="id"
    :tokens="tokens"
    :paragraph-id="paragraphId"
    content-class="guide-example__text"
    @pointerover="handlePointerOver"
    @pointerout="handlePointerOut"
  >
    <template #empty>
      <span class="guide-example__fallback">{{ ja }}</span>
    </template>
    <template #after>
      <p v-if="showChinese && zh" class="guide-example__translation">{{ zh }}</p>
    </template>
  </AnalyzedTextItem>

  <ExplanationPopover
    :show="explanation.renderGate.value.dictionary"
    :anchor="panelAnchor(explanation.anchorRect.value)"
    :component-anchor="explanation.hasWholePanel.value ? explanation.anchorRect.value : explanation.componentAnchorRect.value"
    :whole-token="explanation.wholeToken.value"
    :whole-lookup="explanation.wholeLookup.value"
    :whole-loading="explanation.wholeLoading.value"
    :whole-can-go-back="explanation.wholeHistory.value.length > 0"
    :component-token="explanation.componentToken.value"
    :component-lookup="explanation.componentLookup.value"
    :component-loading="explanation.componentLoading.value"
    :component-can-go-back="explanation.componentHistory.value.length > 0"
    :component-label="explanation.componentLabel.value"
    :panel-prefix="`guide-${id}`"
    :shortcuts-enabled="false"
    presentation="guide"
    @enter="interaction.handlePopoverEnter"
    @leave="interaction.handlePopoverLeave"
    @navigate-whole="explanation.navigateWhole"
    @navigate-component="explanation.navigateComponent"
    @select-whole-form="explanation.selectWholeForm"
    @select-component-form="explanation.selectComponentForm"
    @back-whole="explanation.backWhole"
    @back-component="explanation.backComponent"
  />
  <GrammarPopover
    :show="explanation.renderGate.value.grammar"
    :tag="explanation.grammarTag.value"
    :anchor="explanation.grammarAnchorRect.value"
    presentation="guide"
    @enter="interaction.handlePopoverEnter"
    @leave="interaction.handlePopoverLeave"
    @open-dictionary="openGrammarDictionary"
  />
</template>

<style scoped>
.guide-example { display: block; min-width: 0; margin: 0; }
.guide-example__text { min-width: 0; font: 21px/1.9 var(--font-ja); overflow-wrap: anywhere; }
.guide-example__fallback { color: var(--text-primary); }
.guide-example__translation { margin: 2px 0 0; color: var(--text-secondary); font: 13px/1.6 var(--font-ui); }
</style>
