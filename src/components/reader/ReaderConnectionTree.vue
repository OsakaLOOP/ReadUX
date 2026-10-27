<script setup lang="ts">
import { computed, shallowRef } from "vue";
import type { ConnectionDiagram } from "../../reader/morphologyBubble";
import { layoutConnectionRules, layoutSentenceChain } from "../../reader/morphologyGraph";
import ReaderMorphologyGraph from "./ReaderMorphologyGraph.vue";
const props = defineProps<{ diagram: ConnectionDiagram }>();
const emit = defineEmits<{ resize: [] }>();
const expanded = shallowRef<string | null>(null);
const rules = computed(() => layoutConnectionRules(props.diagram, expanded.value));
const sentence = computed(() => layoutSentenceChain(props.diagram.sentence));
function toggle(key: string) {
  expanded.value = expanded.value === key ? null : key;
}
</script>

<template>
  <div class="connection-diagrams" :data-natural-width="Math.ceil(Math.max(sentence.width, rules.width))">
    <section v-if="sentence.nodes.length" aria-label="句中关系">
      <h3>句中关系</h3>
      <ReaderMorphologyGraph :layout="sentence" label="完整活用链" @resize="emit('resize')" />
    </section>
    <section v-if="rules.nodes.length" aria-label="前接规则">
      <h3>前接规则 <span v-if="diagram.status === 'candidate'">候选</span></h3>
      <ReaderMorphologyGraph :layout="rules" label="前接规则树" @toggle="toggle" @resize="emit('resize')" />
    </section>
  </div>
</template>

<style scoped>
.connection-diagrams { display: grid; gap: 8px; }
.connection-diagrams > section { min-width: 0; }
.connection-diagrams > section + section { padding-top: 7px; border-top: 1px solid var(--glass-border); }
h3 { display: flex; gap: 8px; margin: 0 0 3px; color: var(--text-secondary); font: 600 12px/1.5 var(--font-ui); }
h3 span { color: var(--text-muted); font-weight: 400; }
</style>
