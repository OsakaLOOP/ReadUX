<script setup lang="ts">
import { computed } from "vue";
import type { DictionaryLookup } from "../../types";
import ParticleGrammarBlock from "./ParticleGrammarBlock.vue";

const props = defineProps<{
  show: boolean;
  x: number;
  y: number;
  lookup: DictionaryLookup | null;
  docked?: boolean;
  panelId?: string;
}>();
const emit = defineEmits<{
  enter: [event: PointerEvent];
  leave: [event: PointerEvent];
}>();
const style = computed(() => {
  if (props.docked) return {};
  const width = Math.min(390, window.innerWidth - 24);
  const left = Math.max(12, Math.min(window.innerWidth - width - 12, props.x));
  const top = Math.min(window.innerHeight - 12, props.y);
  return { left: `${left}px`, top: `${top}px`, width: `${width}px` };
});
const particleKind = computed(
  () =>
    (
      ({
        格助詞: "格助词",
        係助詞: "係助词",
        副助詞: "副助词",
        接続助詞: "接续助词",
        終助詞: "终助词",
        準体助詞: "准体助词",
        助動詞: "助动词",
      }) as Record<string, string>
    )[props.lookup?.pos?.sub1 ?? ""] ?? "助词",
);
</script>

<template>
  <Transition name="fade">
    <aside
      v-if="show && lookup"
      class="particle-popover"
      :class="{ 'particle-popover--docked': docked }"
      :style="style"
      :data-explanation-panel="panelId || 'particle-grammar'"
      :role="docked ? 'region' : 'dialog'"
      aria-label="助词语法查询"
      @pointerenter="emit('enter', $event)"
      @pointerleave="emit('leave', $event)"
    >
      <h2 v-if="docked" class="dock-section-title">词典</h2>
      <header class="particle-popover__header">
        <div>
          <strong>{{ lookup.entries[0]?.headword ?? lookup.query }}</strong
          ><span>{{ particleKind }}</span>
        </div>
        <div><span>无活用</span></div>
      </header>
      <ParticleGrammarBlock :lookup="lookup" :show-header="false" />
    </aside>
  </Transition>
</template>

<style scoped>
.particle-popover {
  position: fixed;
  z-index: 1010;
  box-sizing: border-box;
  max-height: min(72vh, 620px);
  overflow: auto;
  padding: 12px 14px;
  border: 1px solid color-mix(in srgb, #337eb7 30%, var(--border-color));
  border-radius: var(--radius-md);
  background: var(--glass-bg);
  box-shadow: var(--shadow-md);
  backdrop-filter: var(--glass-filter);
  color: var(--text-primary);
  font: 0.86rem/1.55 var(--font-ui);
}
.fade-enter-active,
.fade-leave-active {
  transition: opacity 120ms ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
.particle-popover--docked {
  position: relative;
  width: 100%;
  max-height: none;
  flex: 0 0 auto;
  overflow: visible;
  padding: 20px;
  border: 0;
  border-bottom: 1px solid #dce1e8;
  border-radius: 0;
  background: #fff;
  box-shadow: none;
  backdrop-filter: none;
}
.dock-section-title {
  margin: 0 0 14px;
  color: var(--text-secondary);
  font: 600 13px/1.5 var(--font-ui);
}
.particle-popover__header {
  display: grid;
  gap: 6px;
  margin-bottom: 10px;
  padding-bottom: 9px;
  border-bottom: 1px solid var(--border-color);
}
.particle-popover__header > div {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 0.85fr);
  gap: 12px;
  align-items: baseline;
}
.particle-popover__header strong {
  color: var(--accent-color);
  font: 700 1.1rem/1.4 var(--font-ja);
}
.particle-popover__header span {
  min-width: 0;
  color: var(--text-secondary);
  font: 12px/1.4 var(--font-ui);
}
.particle-popover__header span[lang="ja"] {
  color: var(--text-primary);
  font: 600 14px/1.4 var(--font-ja);
}
</style>
