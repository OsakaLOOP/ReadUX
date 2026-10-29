<script setup lang="ts" generic="T extends { key: string; text: string }">
import { computed } from "vue";
import { groupCapsuleRuns, type CapsuleActiveRange } from "../../reader/capsuleRuns";

const props = defineProps<{
  grouped?: boolean;
  merged?: boolean;
  activeRanges?: CapsuleActiveRange[];
  runs: { key: string; reading?: string; parts: T[] }[];
}>();
const groups = computed(() => groupCapsuleRuns(props.runs, props.activeRanges));
function highlightStyle(group?: string) {
  const active = props.activeRanges?.filter((item) => group ? item.group === group : true) ?? [];
  const blue = active.filter((item) => item.tone === "blue");
  const grammar = active.filter((item) => item.tone === "grammar");
  if (!blue.length) return {
    background: "color-mix(in srgb, #e2bb38 26%, transparent)",
    boxShadow: "0 0 0 2px color-mix(in srgb, #d4a91c 24%, transparent), 0 0 9px color-mix(in srgb, #d4a91c 32%, transparent)",
  };
  if (!grammar.length) return {
    background: "color-mix(in srgb, #378bc4 18%, transparent)",
    boxShadow: "0 0 0 2px color-mix(in srgb, #378bc4 22%, transparent), 0 0 9px color-mix(in srgb, #378bc4 28%, transparent)",
  };
  const bounds = active.reduce<[number, number]>((range, item) => [Math.min(range[0], item.range[0]), Math.max(range[1], item.range[1])], [Infinity, -Infinity]);
  const percent = (offset: number) => Math.max(0, Math.min(100, (offset - bounds[0]) / (bounds[1] - bounds[0]) * 100));
  const stops = active.flatMap((item) => {
    const color = item.tone === "blue" ? "color-mix(in srgb, #378bc4 18%, transparent)" : "color-mix(in srgb, #e2bb38 26%, transparent)";
    return [`${color} ${percent(item.range[0])}%`, `${color} ${percent(item.range[1])}%`];
  });
  return {
    background: `linear-gradient(90deg, ${stops.join(", ")})`,
    boxShadow: "0 0 0 2px color-mix(in srgb, #458bb2 20%, #d4a91c 16%), 0 0 9px color-mix(in srgb, #458bb2 24%, #d4a91c 20%)",
  };
}
</script>

<template>
  <span class="reader-capsule" :class="{ 'reader-capsule--group': grouped, 'reader-capsule--merged': merged }">
    <template v-for="group in groups" :key="group.key">
      <span v-if="group.active" class="reader-capsule__query-object" :style="highlightStyle(group.activeGroup)">
        <component v-for="run in group.runs" :key="run.key" :is="run.reading ? 'ruby' : 'span'">
          <slot v-for="part in run.parts" :key="part.key" :part="part">{{ part.text }}</slot>
          <rt v-if="run.reading">{{ run.reading }}</rt>
        </component>
      </span>
      <template v-else>
      <component v-for="run in group.runs" :key="run.key" :is="run.reading ? 'ruby' : 'span'">
        <slot v-for="part in run.parts" :key="part.key" :part="part">{{ part.text }}</slot>
        <rt v-if="run.reading">{{ run.reading }}</rt>
      </component>
      </template>
    </template>
  </span>
</template>

<style scoped>
.reader-capsule { display: inline; }
.reader-capsule--group { border-radius: var(--radius-sm); box-decoration-break: clone; -webkit-box-decoration-break: clone; }
.reader-capsule--group:hover { outline: 1px solid var(--border-color); }
.reader-capsule--merged { outline: 1px solid var(--accent-color); }
.reader-capsule__query-object { border-radius: 3px; box-decoration-break: clone; -webkit-box-decoration-break: clone; }
rt { user-select: none; }
</style>
