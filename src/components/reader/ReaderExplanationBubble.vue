<script setup lang="ts">
defineProps<{
  show: boolean;
  x: number;
  y: number;
  width: number;
  maxHeight: number;
  title: string;
  surface: string;
  label?: string;
}>();

const emit = defineEmits<{ enter: [event: PointerEvent]; leave: [event: PointerEvent] }>();
</script>

<template>
  <Transition name="fade">
    <aside
      v-if="show"
      class="reader-explanation"
      :style="{ left: `${x}px`, top: `${y}px`, width: `${width}px`, maxHeight: `${maxHeight}px` }"
      role="dialog"
      :aria-label="label || title"
      @pointerenter="emit('enter', $event)"
      @pointerleave="emit('leave', $event)"
      @wheel.stop
    >
      <header>
        <div><strong>{{ title }}</strong><span v-if="surface" lang="ja">{{ surface }}</span></div>
      </header>
      <slot />
    </aside>
  </Transition>
</template>

<style scoped>
.reader-explanation { position: fixed; z-index: 1001; box-sizing: border-box; width: min(310px, calc(100vw - 24px)); overflow: auto; overflow-wrap: anywhere; overscroll-behavior: contain; padding: 9px 10px; border: 1px solid var(--glass-border); border-radius: var(--radius-md); background: var(--glass-bg); backdrop-filter: var(--glass-filter); box-shadow: var(--shadow-md); color: var(--text-primary); font: .84rem/1.5 var(--font-ui); }
.reader-explanation { scrollbar-width: none; overflow-anchor: none; }
.reader-explanation::-webkit-scrollbar { display: none; }
.reader-explanation header { display: flex; align-items: flex-start; justify-content: space-between; gap: 6px; margin-bottom: 6px; }
.reader-explanation header div { display: grid; min-width: 0; gap: 1px; }
.reader-explanation header strong { color: var(--accent-color); font: 600 15px/1.5 var(--font-ui); }
.reader-explanation header span { color: var(--text-secondary); font: 14px/1.5 var(--font-ja); }
.fade-enter-active, .fade-leave-active { transition: opacity 120ms ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
@media (prefers-reduced-motion: reduce) { .fade-enter-active, .fade-leave-active { transition: none; } }
</style>
