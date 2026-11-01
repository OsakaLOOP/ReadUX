<script setup lang="ts" generic="T extends { key: string; text: string }">
import type { CapsuleActiveRange } from "../../reader/capsuleRuns";

const props = defineProps<{
  grouped?: boolean;
  merged?: boolean;
  activeRanges?: CapsuleActiveRange[];
  runs: { key: string; reading?: string; parts: T[] }[];
}>();
function partHighlight(part: T) {
  const range = (part as T & { range?: [number, number] }).range;
  const activePart = range && props.activeRanges?.find((item) => range[0] < item.range[1] && range[1] > item.range[0]);
  if (!activePart) return undefined;
  const group = activePart.group ?? `${activePart.range[0]}:${activePart.range[1]}`;
  return highlightStyle(group);
}
function highlightStyle(group?: string) {
  const active = props.activeRanges?.filter((item) => group ? (item.group ?? `${item.range[0]}:${item.range[1]}`) === group : true) ?? [];
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
  <span class="reader-capsule" :class="{ 'reader-capsule--group': grouped, 'reader-capsule--merged': merged, 'reader-capsule--active': activeRanges?.length }">
    <template v-for="run in runs" :key="run.key">
      <component :is="run.reading ? 'ruby' : 'span'">
        <span v-for="part in run.parts" :key="part.key" class="reader-capsule__query-object" :style="partHighlight(part)">
          <slot :part="part">{{ part.text }}</slot>
        </span>
        <rt v-if="run.reading">{{ run.reading }}</rt>
      </component>
    </template>
  </span>
</template>

<style scoped>
.reader-capsule { display: inline; }
.reader-capsule--group { border-radius: var(--radius-sm); box-decoration-break: clone; -webkit-box-decoration-break: clone; }
.reader-capsule--group:hover,
.reader-capsule--group.reader-capsule--active { outline: 1px solid var(--border-color); }
.reader-capsule--merged { outline: 1px solid var(--accent-color); }
.reader-capsule__query-object { border-radius: 3px; box-decoration-break: clone; -webkit-box-decoration-break: clone; }
rt { user-select: none; }
</style>
