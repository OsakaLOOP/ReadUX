<script setup lang="ts">
import { computed } from "vue";
import { PanelBottomOpen, PanelsTopLeft } from "@lucide/vue";

const props = defineProps<{ docked: boolean; narrow?: boolean }>();
const emit = defineEmits<{ toggle: [] }>();
const label = computed(() => props.docked ? "切换为悬浮气泡" : "切换为固定面板");
</script>

<template>
  <button
    type="button"
    class="morphology-layout-toggle"
    :class="{ 'morphology-layout-toggle--active': docked }"
    :aria-label="label"
    :aria-pressed="docked"
    :title="label"
    @click.stop="emit('toggle')"
  >
    <svg v-if="!docked" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" aria-hidden="true">
      <rect x="4.8" y="7.2" width="5.6" height="14.4" rx="0.8" />
      <rect x="13.6" y="7.2" width="5.6" height="5.6" rx="0.8" />
      <rect x="13.6" y="16" width="5.6" height="5.6" rx="0.8" />
      <rect x="8.4" y="2.8" width="7.2" height="1.6" rx="0.8" fill="currentColor" stroke="none" />
    </svg>
    <PanelsTopLeft v-else-if="!narrow" :size="17" class="morphology-layout-toggle__panel" aria-hidden="true" />
    <PanelBottomOpen v-else :size="17" aria-hidden="true" />
  </button>
</template>

<style scoped>
.morphology-layout-toggle {
  display: grid;
  width: 34px;
  height: 34px;
  flex: 0 0 auto;
  place-items: center;
  padding: 0;
  border: 1px solid transparent;
  border-radius: 5px;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background-color 140ms ease, color 140ms ease, border-color 140ms ease;
}
.morphology-layout-toggle svg { width: 20px; height: 20px; }
.morphology-layout-toggle__panel { transform: scaleX(-1); }
.morphology-layout-toggle:hover,
.morphology-layout-toggle--active {
  border-color: var(--border-color);
  background: var(--bg-secondary);
  color: var(--accent-color);
}
.morphology-layout-toggle:focus-visible { outline: 2px solid var(--accent-color); outline-offset: 2px; }
@media (prefers-reduced-motion: reduce) {
  .morphology-layout-toggle { transition: none; }
}
</style>
