<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, shallowRef, useId, watch } from "vue";
import { interpolateNumber } from "d3-interpolate";
import { morphologyGraphLink, type GraphNode, type MorphologyGraphLayout } from "../../reader/morphologyGraph";

const props = defineProps<{ layout: MorphologyGraphLayout; label: string }>();
const emit = defineEmits<{ toggle: [key: string]; resize: [] }>();
const current = shallowRef(props.layout);
const markerId = `morphology-arrow-${useId().replace(/:/g, '')}`;
const nodeMap = computed(() => new Map(current.value.nodes.map((node) => [node.id, node])));
let frame = 0;

function textY(node: GraphNode, index: number) {
  const contentHeight = node.lines.reduce((sum, line) => sum + (line.kind === 'secondary' ? 15 : 18), 0);
  const height = node.kind === 'group' ? 28 : node.height;
  return (height - contentHeight) / 2 + 13 + node.lines.slice(0, index).reduce((sum, line) => sum + (line.kind === 'secondary' ? 15 : 18), 0);
}

function listGuides(node: GraphNode) {
  let y = 35;
  const branches = (node.list ?? []).map((row) => {
    const line = `M12,${y + 12}H23`;
    y += row.height;
    return line;
  });
  const last = node.list?.[node.list.length - 1];
  const bottom = last ? y - last.height + 12 : 28;
  return `M12,28V${bottom} ${branches.join(' ')}`;
}

function currentGuide(node: GraphNode) {
  let y = 35;
  for (const row of node.list ?? []) {
    if (row.matched) return `M12,28V${y + 12}H22`;
    y += row.height;
  }
  return '';
}

watch(() => props.layout, (next) => {
  cancelAnimationFrame(frame);
  const before = current.value;
  if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) { current.value = next; void nextTick(() => emit('resize')); return; }
  const previous = new Map(before.nodes.map((node) => [node.id, node]));
  const nextIds = new Set(next.nodes.map((node) => node.id));
  const tweens = next.nodes.map((node) => {
    const source = previous.get(node.id) ?? { ...node, opacity: 0 };
    return { node, x: interpolateNumber(source.x, node.x), y: interpolateNumber(source.y, node.y),
      width: interpolateNumber(source.width, node.width), height: interpolateNumber(source.height, node.height), opacity: interpolateNumber(source.opacity, 1) };
  });
  const exiting = before.nodes.filter((node) => !nextIds.has(node.id));
  const oldLinks = before.links.filter((link) => !next.links.some((item) => item.id === link.id));
  const width = interpolateNumber(before.width, next.width);
  const height = interpolateNumber(before.height, next.height);
  const start = performance.now();
  const animate = (time: number) => {
    const progress = Math.min(1, (time - start) / 280);
    const eased = 1 - Math.pow(1 - progress, 3);
    current.value = progress === 1 ? next : {
      width: width(eased), height: height(eased), links: [...next.links, ...oldLinks],
      nodes: [...tweens.map((tween) => ({ ...tween.node, x: tween.x(eased), y: tween.y(eased),
        width: tween.width(eased), height: tween.height(eased), opacity: tween.opacity(eased) })),
        ...exiting.map((node) => ({ ...node, opacity: node.opacity * (1 - eased) }))],
    };
    void nextTick(() => emit('resize'));
    if (progress < 1) frame = requestAnimationFrame(animate);
  };
  frame = requestAnimationFrame(animate);
});
onBeforeUnmount(() => cancelAnimationFrame(frame));
</script>

<template>
  <div class="morphology-graph" tabindex="0" :aria-label="`${label}，可横向滚动`">
    <svg :width="current.width" :height="current.height" :viewBox="`0 0 ${current.width} ${current.height}`" role="group" :aria-label="label">
      <defs>
        <marker :id="markerId" viewBox="0 0 7 7" refX="6" refY="3.5" markerWidth="7" markerHeight="7" markerUnits="userSpaceOnUse" orient="auto-start-reverse">
          <path d="M1,1 L6,3.5 L1,6" fill="none" stroke="context-stroke" stroke-width="1.1" />
        </marker>
      </defs>
      <g class="morphology-graph__edges" aria-hidden="true">
        <g v-for="link in current.links" :key="link.id" :class="{ 'is-matched': link.matched, 'is-fold': link.vertical }">
          <path :d="morphologyGraphLink(link, nodeMap)" :marker-end="link.matched ? `url(#${markerId})` : undefined" />
          <text v-if="link.label && nodeMap.get(link.source) && nodeMap.get(link.target)"
            :x="nodeMap.get(link.source)!.x + nodeMap.get(link.source)!.width / 2 + 9"
            :y="(nodeMap.get(link.source)!.y + nodeMap.get(link.source)!.height + nodeMap.get(link.target)!.y) / 2 + 3">{{ link.label }}</text>
        </g>
      </g>
      <g v-for="node in current.nodes" :key="node.id" class="morphology-graph__node"
        :class="[`is-${node.kind}`, { 'is-matched': node.matched, 'is-insertion': node.inserted, 'is-action': node.action }]"
        :transform="`translate(${node.x},${node.y})`" :opacity="node.opacity"
        :data-node-id="node.id" :data-matched="node.matched ? '' : undefined"
        role="group" :aria-label="node.lines.map(line => line.text).join('，')">
        <title>{{ node.lines.map(line => line.text).join(' · ') }}</title>
        <rect :width="node.width" :height="node.kind === 'group' ? 28 : node.height" rx="6"
          :role="node.action ? 'button' : undefined" :tabindex="node.action ? 0 : undefined"
          :aria-label="node.action ? `${node.action.expanded ? '收起' : '展开'}${node.lines[0].text}的其他接续路径` : undefined"
          :aria-expanded="node.action ? node.action.expanded : undefined"
          @click="node.action && emit('toggle', node.action.key)"
          @keydown.enter.prevent="node.action && emit('toggle', node.action.key)"
          @keydown.space.prevent="node.action && emit('toggle', node.action.key)" />
        <text v-for="(line, index) in node.lines" :key="index" :x="node.width / 2"
          :y="textY(node, index)" text-anchor="middle"
          :class="`line-${line.kind}`" :lang="line.kind === 'actual' ? 'ja' : 'zh-CN'">
          <template v-if="line.runs"><tspan v-for="(run, runIndex) in line.runs" :key="runIndex" :lang="run.lang" :class="`run-${run.role}`">{{ run.text }}</tspan></template>
          <template v-else>{{ line.text }}</template>
        </text>
        <g v-if="node.action" transform="translate(8, 7)" class="morphology-graph__toggle" aria-hidden="true">
          <path :d="node.action.expanded ? 'M0,5L4,9L8,5' : 'M2,3L6,7L2,11'" />
          <text x="12" y="11">{{ node.action.count }}</text>
        </g>
        <template v-if="node.list?.length">
          <path class="morphology-graph__list-stem" :d="listGuides(node)" aria-hidden="true" />
          <path v-if="currentGuide(node)" class="morphology-graph__current-stem" :d="currentGuide(node)" :marker-end="`url(#${markerId})`" aria-hidden="true" />
          <foreignObject x="24" y="35" :width="node.width - 24" :height="Math.max(0, node.height - 35)" @click.stop>
            <div class="morphology-rule-list" role="list" :aria-label="`${node.lines[0].text}的细分类`">
              <div v-for="row in node.list" :key="row.id" class="morphology-rule-list__row" :class="{ 'is-current': row.matched }" role="listitem" :data-path-id="row.id" :data-current-path="row.matched ? '' : undefined">
                <div class="morphology-rule-list__heading"><span>{{ row.label }}</span><b v-if="row.actual" lang="ja">{{ row.actual }}</b></div>
                <div v-if="row.condition" class="morphology-rule-list__condition">{{ row.condition }}</div>
              </div>
            </div>
          </foreignObject>
        </template>
      </g>
    </svg>
  </div>
</template>

<style scoped>
.morphology-graph { min-width: 0; max-width: 100%; overflow-x: auto; overscroll-behavior-inline: contain; scrollbar-width: none; }
.morphology-graph::-webkit-scrollbar { display: none; }
svg { display: block; margin: 0 auto; overflow: visible; font: 12px/1.4 var(--font-ui); }
.morphology-graph__edges path { fill: none; stroke: color-mix(in srgb, var(--text-muted) 45%, var(--border-color)); stroke-width: 1.1; }
.morphology-graph__edges .is-matched path { stroke: var(--accent-color); stroke-width: 1.6; }
.morphology-graph__edges .is-fold path { stroke: color-mix(in srgb, var(--accent-color) 65%, var(--border-color)); stroke-width: 1.6; }
.morphology-graph__edges text { fill: var(--text-muted); font-size: 10px; }
.morphology-graph__node rect { fill: var(--bg-primary, #fff); stroke: var(--border-color); stroke-width: 1; }
.morphology-graph__node.is-matched rect { stroke: var(--accent-color); stroke-width: 1.5; }
.morphology-graph__node.is-insertion rect { stroke-dasharray: 4 3; fill: color-mix(in srgb, var(--bg-primary, #fff) 94%, var(--accent-color)); }
.morphology-graph__node text { pointer-events: none; fill: var(--text-primary); }
.morphology-graph__node .line-main { font-weight: 500; font-size: 12px; }
.morphology-graph__node.is-sentence .line-main, .morphology-graph__node.is-connector .line-main, .morphology-graph__node.is-target .line-main { font: 600 15px var(--font-ja); }
.morphology-graph__node .line-secondary { fill: var(--text-secondary); font-size: 11px; }
.morphology-graph__node .line-actual { fill: var(--accent-color); font: 500 12px var(--font-ja); }
.morphology-graph__node .run-label { fill: var(--text-secondary); font: 12px var(--font-ui); }
.morphology-graph__node .run-form { font: 600 15px var(--font-ja); }
.morphology-graph__node .run-arrow { fill: var(--text-muted); font: 13px var(--font-ui); }
.morphology-graph__node.is-target rect, .morphology-graph__node.is-connector.is-matched rect { stroke: var(--accent-color); }
.morphology-graph__node.is-action > rect { cursor: pointer; outline: none; }
.morphology-graph__node.is-action:hover rect, .morphology-graph__node.is-action:focus-visible rect { stroke: var(--accent-color); stroke-width: 2; }
.morphology-graph__toggle text { fill: var(--text-muted); font: 11px var(--font-ui); }
.morphology-graph__toggle { pointer-events: none; }
.morphology-graph__toggle path, .morphology-graph__list-stem { fill: none; stroke: var(--text-muted); stroke-width: 1; }
.morphology-graph__current-stem { fill: none; stroke: var(--accent-color); stroke-width: 1.6; }
.morphology-rule-list { box-sizing: border-box; font: 12px/18px var(--font-ui); color: var(--text-primary); }
.morphology-rule-list__row { padding: 3px 0; }
.morphology-rule-list__heading { display: flex; align-items: baseline; gap: 4px; }
.morphology-rule-list__condition { color: var(--text-secondary); font-size: 11px; line-height: 15px; overflow-wrap: anywhere; }
.morphology-rule-list__heading b { margin-left: auto; color: var(--accent-color); font: 500 12px/18px var(--font-ja); white-space: nowrap; }
.morphology-rule-list__row.is-current .morphology-rule-list__heading { color: var(--accent-color); }
</style>
