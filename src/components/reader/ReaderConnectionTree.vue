<script setup lang="ts">
import { computed, nextTick, shallowRef } from "vue";
import type { ConnectionDiagram } from "../../reader/morphologyBubble";
import { layoutConnectionRules, layoutSentenceChain } from "../../reader/morphologyGraph";
import ReaderMorphologyGraph from "./ReaderMorphologyGraph.vue";
const props = defineProps<{ diagram: ConnectionDiagram; maxHeight: number; scrollMode?: "split" | "flow" }>();
const emit = defineEmits<{ resize: [] }>();
const expanded = shallowRef<string | null>(null);
const rules = computed(() => layoutConnectionRules(props.diagram, expanded.value));
const sentence = computed(() => layoutSentenceChain(props.diagram.sentence));
const sentenceHeight = shallowRef<number>();
const rulesHeight = shallowRef<number>();
const contentHeights = computed(() => ({
  sentence: sentence.value.nodes.length ? sentenceHeight.value ?? sentence.value.height : 0,
  rules: rules.value.nodes.length ? rulesHeight.value ?? rules.value.height : 0,
}));
// 标题占 21 px；两栏之间的间隔、上内边距及边框共占 16 px。
const chromeHeight = computed(() => (Number(Boolean(sentence.value.nodes.length)) + Number(Boolean(rules.value.nodes.length))) * 21
  + (sentence.value.nodes.length && rules.value.nodes.length ? 16 : 0));
const naturalHeight = computed(() => contentHeights.value.sentence + contentHeights.value.rules + chromeHeight.value);
const viewportHeights = computed(() => {
  const heights = contentHeights.value;
  if (props.scrollMode === "flow") return heights;
  const available = Math.max(0, props.maxHeight - chromeHeight.value);
  if (heights.sentence + heights.rules <= available) return heights;
  if (!sentence.value.nodes.length) return { sentence: 0, rules: available };
  if (!rules.value.nodes.length) return { sentence: available, rules: 0 };
  // 优先完整显示较短的一栏，另一栏至少保留可用图示高度的三分之一。
  const shorter = Math.min(heights.sentence, heights.rules, available * 2 / 3);
  return heights.sentence <= heights.rules
    ? { sentence: shorter, rules: available - shorter }
    : { sentence: available - shorter, rules: shorter };
});
async function resizeGraph(section: 'sentence' | 'rules', height: number) {
  if (section === 'sentence') sentenceHeight.value = height;
  else rulesHeight.value = height;
  await nextTick();
  emit('resize');
}
function toggle(key: string) {
  expanded.value = expanded.value === key ? null : key;
}
</script>

<template>
  <div class="connection-diagrams" :class="{ 'connection-diagrams--flow': scrollMode === 'flow' }" :data-natural-width="Math.ceil(Math.max(sentence.width, rules.width))" :data-natural-height="Math.ceil(naturalHeight)">
    <section v-if="sentence.nodes.length" aria-label="接续方式">
      <h3>接续方式</h3>
      <ReaderMorphologyGraph :layout="sentence" :height="viewportHeights.sentence" :flow="scrollMode === 'flow'" label="完整活用链" @resize="resizeGraph('sentence', $event)" />
    </section>
    <section v-if="rules.nodes.length" aria-label="接续规则">
      <h3>接续规则 <span v-if="diagram.status === 'candidate'">候选</span></h3>
      <ReaderMorphologyGraph :layout="rules" :height="viewportHeights.rules" :flow="scrollMode === 'flow'" label="接续规则树" @toggle="toggle" @resize="resizeGraph('rules', $event)" />
    </section>
  </div>
</template>

<style scoped>
.connection-diagrams { display: grid; gap: 8px; min-height: 0; overflow: hidden; }
.connection-diagrams--flow { overflow: visible; }
.connection-diagrams > section { min-width: 0; min-height: 0; }
.connection-diagrams > section + section { padding-top: 7px; border-top: 1px solid var(--glass-border); }
h3 { display: flex; gap: 8px; margin: 0 0 3px; color: var(--text-secondary); font: 600 12px/1.5 var(--font-ui); }
h3 span { color: var(--text-muted); font-weight: 400; }
</style>
