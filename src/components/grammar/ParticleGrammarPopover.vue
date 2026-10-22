<script setup lang="ts">
import { computed } from "vue";
import type { DictionaryLookup } from "../../types";
import ParticleGrammarBlock from "./ParticleGrammarBlock.vue";

const props = defineProps<{ show: boolean; x: number; y: number; lookup: DictionaryLookup | null }>();
const emit = defineEmits<{ enter: [event: PointerEvent]; leave: [event: PointerEvent] }>();
const style = computed(() => {
  const width = Math.min(390, window.innerWidth - 24);
  const left = Math.max(12, Math.min(window.innerWidth - width - 12, props.x));
  const top = Math.min(window.innerHeight - 12, props.y);
  return { left: `${left}px`, top: `${top}px`, width: `${width}px` };
});
</script>

<template>
  <Transition name="fade">
    <aside v-if="show && lookup" class="particle-popover" :style="style" data-explanation-panel="particle-grammar"
      role="dialog" aria-label="助词语法查询" @pointerenter="emit('enter', $event)" @pointerleave="emit('leave', $event)">
      <ParticleGrammarBlock :lookup="lookup" />
    </aside>
  </Transition>
</template>

<style scoped>
.particle-popover { position: fixed; z-index: 1010; box-sizing: border-box; max-height: min(72vh,620px); overflow: auto; padding: 12px 14px; border: 1px solid color-mix(in srgb,#337eb7 30%,var(--border-color)); border-radius: var(--radius-md); background: var(--glass-bg); box-shadow: var(--shadow-md); backdrop-filter: var(--glass-filter); color: var(--text-primary); font: .86rem/1.55 var(--font-ui); }
.fade-enter-active,.fade-leave-active{transition:opacity 120ms ease}.fade-enter-from,.fade-leave-to{opacity:0}
</style>
