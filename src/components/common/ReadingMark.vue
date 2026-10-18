<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, useId, watch } from "vue";
import source from "../../branding/readux-animated.svg?raw";
import { createMarkPlayback } from "../../reader/markPlayback";
import { representativeCoverColor } from "../../reader/coverColor";
import { updateTaskbarIcon } from "../../services/taskbarIcon";

const props = withDefaults(defineProps<{ reading?: boolean; coverSrc?: string; coverKey?: string; taskbarIcon?: boolean }>(), {
  reading: false, coverSrc: "", coverKey: "", taskbarIcon: false,
});
const host = ref<HTMLSpanElement | null>(null);
const prefix = `reading-mark-${useId()}`;
let svg: SVGSVGElement | null = null;
let playback: ReturnType<typeof createMarkPlayback> | null = null;
let coverRevision = 0;
let presentationRevision = 0;
const coverColorStorageKey = "readux.cover-representative-colors-v2";

function readStoredCoverColor(key: string) {
  if (!key) return null;
  try {
    const stored = JSON.parse(localStorage.getItem(coverColorStorageKey) || "{}") as Record<string, unknown>;
    return typeof stored[key] === "string" ? stored[key] : null;
  } catch {
    return null;
  }
}

function storeCoverColor(key: string, color: string) {
  if (!key) return;
  try {
    const stored = JSON.parse(localStorage.getItem(coverColorStorageKey) || "{}") as Record<string, unknown>;
    stored[key] = color;
    localStorage.setItem(coverColorStorageKey, JSON.stringify(stored));
  } catch {
    // 本地存储不可用时仍保留当前 SVG 的颜色。
  }
}

async function updateReadingCover() {
  if (!svg) return;
  const image = svg.querySelector<SVGImageElement>(`#${prefix}-book-front-cover-image`);
  const revision = ++coverRevision;
  svg.style.removeProperty("--book-back-color");
  if (!props.coverSrc) {
    image?.removeAttribute("href");
    return;
  }
  image?.setAttribute("href", props.coverSrc);
  const storedColor = readStoredCoverColor(props.coverKey);
  if (storedColor) {
    svg.style.setProperty("--book-back-color", storedColor);
    return;
  }

  const source = new Image();
  source.crossOrigin = "anonymous";
  const loaded = await new Promise<boolean>((resolve) => {
    source.onload = () => resolve(true);
    source.onerror = () => resolve(false);
    source.src = props.coverSrc;
  });
  if (!loaded || revision !== coverRevision || !svg) return;
  try {
    const canvas = document.createElement("canvas");
    canvas.width = 32;
    canvas.height = 32;
    const context = canvas.getContext("2d", { willReadFrequently: true });
    if (!context) return;
    context.drawImage(source, 0, 0, canvas.width, canvas.height);
    const { data } = context.getImageData(0, 0, canvas.width, canvas.height);
    const color = representativeCoverColor(data);
    if (!color) return;
    svg.style.setProperty("--book-back-color", color);
    storeCoverColor(props.coverKey, color);
  } catch {
    svg.style.removeProperty("--book-back-color");
  }
}

async function updatePresentation() {
  const revision = ++presentationRevision;
  if (!props.reading) {
    // 关闭期间保留封面，供同一 SVG 倒放。
    ++coverRevision;
    if (props.taskbarIcon) void updateTaskbarIcon(null);
    return;
  }
  await updateReadingCover();
  if (revision !== presentationRevision || !props.taskbarIcon || !props.reading || !svg) return;
  await updateTaskbarIcon(svg);
}

function playBookAnimation(open: boolean) {
  return playback?.play(open) ?? Promise.resolve();
}

onMounted(() => {
  const parsed = new DOMParser().parseFromString(source, "image/svg+xml").documentElement;
  svg = document.importNode(parsed, true) as unknown as SVGSVGElement;
  // 每个实例独立命名引用；同一动画始终操作原节点。
  for (const node of svg.querySelectorAll("[id]")) {
    node.id = `${prefix}-${node.id}`;
  }
  for (const node of svg.querySelectorAll("*")) {
    for (const attribute of [...node.attributes]) {
      if (attribute.value.startsWith("url(#")) {
        node.setAttribute(attribute.name, attribute.value.replace("url(#", `url(#${prefix}-`));
      }
    }
  }
  host.value?.replaceChildren(svg);
  playback = createMarkPlayback(svg);
  void updatePresentation();
  if (props.reading) void playBookAnimation(true);
});
watch(() => props.reading, (open) => { void playBookAnimation(open); });
watch(() => [props.coverSrc, props.coverKey, props.reading], () => { void updatePresentation(); });
onBeforeUnmount(() => {
  ++coverRevision;
  ++presentationRevision;
  playback?.dispose();
  if (props.taskbarIcon) void updateTaskbarIcon(null);
});
defineExpose({ playBookAnimation });
</script>

<template>
  <span ref="host" class="reading-mark" aria-hidden="true" />
</template>

<style scoped>
.reading-mark { display: block; flex: 0 0 auto; }
.reading-mark :deep(svg) { display: block; width: 100%; height: 100%; }
</style>
