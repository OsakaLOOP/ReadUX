<script setup lang="ts">
import { computed } from "vue";
import type { ReaderMorphologyDetail } from "../../reader/lookupPresentation";
import DictionaryExamples from "../dictionary/DictionaryExamples.vue";
import { morphologyLocalTransition } from "../../reader/morphologyBubble";
const props = defineProps<{ detail: ReaderMorphologyDetail }>();
const escapeHtml = (text: string) => text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
const examples = computed(() => (props.detail.entry?.examples ?? []).map((example) => ({
  source: { lang: 'ja', html: escapeHtml(example.ja) },
  translation: example.zh ? { lang: 'zh-CN', html: escapeHtml(example.zh) } : null,
})));
</script>

<template>
  <article class="morphology-entry">
    <p v-if="detail.entry" class="morphology-entry__identity">
      {{ [detail.entry.identity, detail.entry.conjugation].filter(Boolean).join(' · ') }}
      <span v-if="detail.status === 'candidate' || detail.status === 'pending'" class="morphology-entry__status">候选</span>
    </p>
    <p v-if="detail.entry?.transition" class="morphology-entry__transition" lang="ja">{{ detail.entry.transition }}</p>
    <p v-else-if="!detail.entry && detail.description" class="morphology-entry__transition" lang="ja">{{ morphologyLocalTransition(detail.description, true) }}</p>
    <p v-if="detail.entry?.summary" class="morphology-entry__summary">{{ detail.entry.summary }}</p>
    <DictionaryExamples v-if="examples.length" :examples="examples" />
    <details v-if="detail.candidates.length || detail.connection?.diagnostics.length" class="morphology-entry__diagnostics">
      <summary>识别详情</summary>
      <p v-if="detail.candidates.length">候选含义：{{ detail.candidates.join('、') }}</p>
      <p v-for="diagnostic in detail.connection?.diagnostics ?? []" :key="diagnostic">{{ diagnostic }}</p>
    </details>
  </article>
</template>

<style scoped>
.morphology-entry { display: grid; gap: 6px; }
.morphology-entry p { margin: 0; }
.morphology-entry__identity { color: var(--text-secondary); font-size: 12px; }
.morphology-entry__status { margin-left: 8px; padding: 1px 5px; border: 1px solid var(--glass-border); border-radius: 4px; color: var(--text-muted); font-size: 11px; }
.morphology-entry__transition { color: var(--accent-color); font: 500 15px/1.6 var(--font-ja); }
.morphology-entry__summary { font: 13px/1.7 var(--font-ui); }
.morphology-entry__diagnostics { color: var(--text-muted); font-size: 12px; }
.morphology-entry__diagnostics summary { cursor: pointer; }
</style>
