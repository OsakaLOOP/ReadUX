<script setup lang="ts" generic="T extends { key: string; text: string }">
defineProps<{
  grouped?: boolean;
  merged?: boolean;
  runs: { key: string; reading?: string; parts: T[] }[];
}>();
</script>

<template>
  <span class="reader-capsule" :class="{ 'reader-capsule--group': grouped, 'reader-capsule--merged': merged }">
    <component v-for="run in runs" :key="run.key" :is="run.reading ? 'ruby' : 'span'">
      <slot v-for="part in run.parts" :key="part.key" :part="part">{{ part.text }}</slot>
      <rt v-if="run.reading">{{ run.reading }}</rt>
    </component>
  </span>
</template>

<style scoped>
.reader-capsule { display: inline; }
.reader-capsule--group { border-radius: var(--radius-sm); box-decoration-break: clone; -webkit-box-decoration-break: clone; }
.reader-capsule--group:hover { outline: 1px solid var(--border-color); }
.reader-capsule--merged { outline: 1px solid var(--accent-color); }
rt { user-select: none; }
</style>
