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
  identity?: string;
  conjugation?: string;
  transition?: string;
  contained?: boolean;
  docked?: boolean;
}>();

const emit = defineEmits<{ enter: [event: PointerEvent]; leave: [event: PointerEvent] }>();
</script>

<template>
  <Transition name="fade">
    <aside
      v-if="show"
      class="reader-explanation"
      :class="{ 'reader-explanation--contained': contained, 'reader-explanation--docked': docked }"
      :style="docked
        ? { position: 'relative', left: 'auto', top: 'auto', width: '100%', height: 'auto', maxHeight: 'none' }
        : { left: `${x}px`, top: `${y}px`, width: `${width}px`, maxHeight: `${maxHeight}px` }"
      :role="docked ? 'region' : 'dialog'"
      :aria-label="label || title"
      @pointerenter="emit('enter', $event)"
      @pointerleave="emit('leave', $event)"
      @wheel.stop
    >
      <h2 v-if="docked" class="dock-section-title">{{ contained ? '接续' : '活用' }}</h2>
      <header v-if="title || surface || identity || conjugation || transition || $slots.actions" :class="{ 'reader-explanation__header--morphology': identity || conjugation || transition }">
        <div class="reader-explanation__heading">
          <strong>{{ title }}</strong><span v-if="surface" lang="ja">{{ surface }}</span>
        </div>
        <div v-if="identity || conjugation || transition" class="reader-explanation__meta" aria-label="词类与活用">
          <span v-if="identity || conjugation">{{ [identity, conjugation].filter(Boolean).join(' · ') }}</span>
          <span v-if="transition" class="reader-explanation__transition" lang="ja">{{ transition }}</span>
        </div>
        <slot name="actions" />
      </header>
      <slot :available="Math.max(0, maxHeight - 20)" />
    </aside>
  </Transition>
</template>

<style scoped>
.reader-explanation { position: fixed; z-index: 1001; box-sizing: border-box; width: min(310px, calc(100vw - 24px)); overflow: auto; overflow-wrap: anywhere; overscroll-behavior: contain; padding: 9px 10px; border: 1px solid var(--glass-border); border-radius: var(--radius-md); background: var(--glass-bg); backdrop-filter: var(--glass-filter); box-shadow: var(--shadow-md); color: var(--text-primary); font: .84rem/1.5 var(--font-ui); }
.reader-explanation { scrollbar-width: none; overflow-anchor: none; }
.reader-explanation--contained { overflow: hidden; }
.reader-explanation--docked { flex: 0 0 auto; overflow: visible; padding: 20px; border: 0; border-bottom: 1px solid #dce1e8; border-radius: 0; background: #fff; box-shadow: none; backdrop-filter: none; }
.dock-section-title { margin: 0 0 14px; color: var(--text-secondary); font: 600 13px/1.5 var(--font-ui); }
.reader-explanation::-webkit-scrollbar { display: none; }
.reader-explanation header { display: flex; align-items: flex-start; justify-content: space-between; gap: 6px; margin-bottom: 6px; }
.reader-explanation header .reader-explanation__heading { display: grid; min-width: 0; gap: 1px; }
.reader-explanation header strong { color: var(--accent-color); font: 600 15px/1.5 var(--font-ui); }
.reader-explanation header :deep(.morphology-layout-toggle) { margin-left: auto; }
.reader-explanation__heading span { color: var(--text-primary); font: 600 15px/1.5 var(--font-ja); }
.reader-explanation__header--morphology { display: grid !important; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); align-items: stretch !important; gap: 6px 10px !important; padding-bottom: 6px; border-bottom: 1px solid var(--border-color); }
.reader-explanation:has(.morphology-entry:empty) .reader-explanation__header--morphology { margin-bottom: 0; padding-bottom: 0; border-bottom: 0; }
.reader-explanation__meta { display: grid; align-content: center; gap: 3px; min-width: 0; border-left: 1px solid var(--border-color); padding-left: 10px; color: var(--text-secondary); font-size: 12px; }
.reader-explanation__transition { color: var(--accent-color); font: 500 14px/1.5 var(--font-ja); }
.fade-enter-active, .fade-leave-active { transition: opacity 120ms ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
@media (prefers-reduced-motion: reduce) { .fade-enter-active, .fade-leave-active { transition: none; } }
</style>
