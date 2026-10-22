<script setup lang="ts">
import type { DictionaryLookup } from "../../types";
import DictionaryExamples from "../dictionary/DictionaryExamples.vue";

defineProps<{ lookup: DictionaryLookup }>();
</script>

<template>
  <article class="particle-grammar">
    <header>
      <strong>{{ lookup.entries[0]?.headword ?? lookup.query }}</strong>
      <span>{{ ({ 格助詞: "格助词", 係助詞: "係助词", 副助詞: "副助词", 接続助詞: "接续助词", 終助詞: "终助词", 準体助詞: "准体助词", 助動詞: "助动词" } as Record<string, string>)[lookup.pos?.sub1 ?? ""] ?? "助词" }}</span>
      <span v-if="lookup.observed_form && lookup.observed_form !== lookup.entries[0]?.headword">本句：<i lang="ja">{{ lookup.observed_form }}</i></span>
    </header>
    <section v-for="sense in lookup.entries[0]?.senses ?? []" :key="sense.sense_id">
      <h3>{{ sense.heading }}</h3>
      <p v-for="(gloss, index) in sense.glosses" :key="index" v-html="gloss.html" />
      <p v-for="(note, index) in sense.notes" :key="`note-${index}`" v-html="note.html" />
      <DictionaryExamples v-if="sense.examples.length" :examples="sense.examples" />
    </section>
  </article>
</template>

<style scoped>
.particle-grammar { display: grid; gap: 8px; }
.particle-grammar header { display: flex; align-items: baseline; gap: 8px; }
.particle-grammar header strong { color: var(--accent-color); font: 700 1rem/1.4 var(--font-ja); }
.particle-grammar header span { color: var(--text-muted); font: 12px/1.4 var(--font-ui); }
.particle-grammar header i { font-family: var(--font-ja); font-style: normal; }
.particle-grammar section + section { padding-top: 8px; border-top: 1px solid color-mix(in srgb, #337eb7 25%, var(--border-color)); }
.particle-grammar h3 { margin: 0; color: var(--text-primary); font: 600 13px/1.5 var(--font-ui); }
.particle-grammar p { margin: 3px 0 0; font: 13px/1.6 var(--font-ui); }
</style>
