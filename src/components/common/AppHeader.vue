<script setup lang="ts">
import { ArrowLeft, BadgeInfo, X } from "@lucide/vue";
import readingMarkSource from "../../../public/branding/readux-mark.svg?raw";
import readingSource from "../../../public/branding/readux-reading.svg?raw";
import { nextTick, onMounted, ref, watch } from "vue";

const props = withDefaults(
  defineProps<{
    showBack?: boolean;
    backLabel?: string;
    collapseBrand?: boolean;
    title?: string;
    description?: string;
    coverSrc?: string;
    coverKey?: string;
    readingMode?: boolean;
  }>(),
  {
    showBack: false,
    backLabel: "返回",
    collapseBrand: false,
    title: "",
    description: "",
    coverSrc: "",
    coverKey: "",
    readingMode: false,
  },
);

const emit = defineEmits<{ back: [] }>();
const showAbout = ref(false);
const readingMarkHost = ref<HTMLSpanElement | null>(null);
const readingMark = ref<SVGSVGElement | null>(null);
let readingSvg: SVGSVGElement | null = null;
let bookAnimations: Animation[] = [];
let animationRun = Promise.resolve();
let animationGeneration = 0;
let coverRevision = 0;
const coverColorStorageKey = "readux.cover-representative-colors";
const sharedTransitionIds = [
  "kabuto-helmet",
  "kuwagata-left",
  "kuwagata-right",
  "katana-blade-a",
  "katana-highlight-a",
  "katana-blade-b",
  "katana-highlight-b",
] as const;
let readingMarkVisible = false;

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

function updateReadingCover() {
  if (!readingSvg) return;
  const image = readingSvg.querySelector<SVGImageElement>("#book-front-cover-image");
  const revision = ++coverRevision;
  readingSvg.style.removeProperty("--book-back-color");
  if (!props.coverSrc) {
    image?.removeAttribute("href");
    return;
  }
  image?.setAttribute("href", props.coverSrc);
  const storedColor = readStoredCoverColor(props.coverKey);
  if (storedColor) {
    readingSvg.style.setProperty("--book-back-color", storedColor);
    return;
  }

  const source = new Image();
  source.onload = () => {
    if (revision !== coverRevision || !readingSvg) return;
    try {
      const canvas = document.createElement("canvas");
      canvas.width = 32;
      canvas.height = 32;
      const context = canvas.getContext("2d", { willReadFrequently: true });
      if (!context) return;
      context.drawImage(source, 0, 0, canvas.width, canvas.height);
      const { data } = context.getImageData(0, 0, canvas.width, canvas.height);
      const colors = new Map<number, { count: number; red: number; green: number; blue: number }>();
      for (let index = 0; index < data.length; index += 4) {
        const red = data[index];
        const green = data[index + 1];
        const blue = data[index + 2];
        if (data[index + 3] < 160 || (red + green + blue) / 3 < 18 || (red + green + blue) / 3 > 246) continue;
        const key = (red >> 4) << 8 | (green >> 4) << 4 | (blue >> 4);
        const bucket = colors.get(key) || { count: 0, red: 0, green: 0, blue: 0 };
        bucket.count += 1;
        bucket.red += red;
        bucket.green += green;
        bucket.blue += blue;
        colors.set(key, bucket);
      }
      const dominant = [...colors.values()].sort((left, right) => right.count - left.count)[0];
      if (!dominant) return;
      const channel = (value: number) => Math.round(value / dominant.count).toString(16).padStart(2, "0");
      const color = `#${channel(dominant.red)}${channel(dominant.green)}${channel(dominant.blue)}`;
      readingSvg.style.setProperty("--book-back-color", color);
      storeCoverColor(props.coverKey, color);
    } catch {
      readingSvg.style.removeProperty("--book-back-color");
    }
  };
  source.src = props.coverSrc;
}

function setSharedTransitionNames(svg: SVGSVGElement, prefix: string) {
  for (const id of sharedTransitionIds) {
    const node = svg.querySelector<SVGGraphicsElement>(`#${id}`);
    if (node) node.style.viewTransitionName = `${prefix}-${id}`;
  }
}

function setReadingVisibility(visible: boolean) {
  readingMarkVisible = visible;
  if (readingMark.value) readingMark.value.style.display = visible ? "none" : "block";
  if (readingSvg) readingSvg.style.display = visible ? "block" : "none";
}

async function switchReadingMark(visible: boolean) {
  if (!readingMark.value || !readingSvg || readingMarkVisible === visible) {
    setReadingVisibility(visible);
    return;
  }
  const transitionDocument = document as Document & {
    startViewTransition?: (update: () => void) => { updateCallbackDone: Promise<void> };
  };
  if (!transitionDocument.startViewTransition || window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
    setReadingVisibility(visible);
    return;
  }
  const transition = transitionDocument.startViewTransition(() => setReadingVisibility(visible));
  await transition.updateCallbackDone.catch(() => undefined);
}

async function playBookAnimation(open: boolean) {
  const generation = ++animationGeneration;
  await nextTick();
  if (!readingSvg) return;
  updateReadingCover();
  if (open) await switchReadingMark(true);
  const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  if (reducedMotion) {
    if (!bookAnimations.length) {
      bookAnimations = readingSvg.querySelector<SVGGElement>("#book-root")
        ?.getAnimations({ subtree: true }) ?? [];
    }
    for (const animation of bookAnimations) {
      animation.pause();
      const duration = Number(animation.effect?.getTiming().duration) || 0;
      animation.currentTime = open ? duration : 0;
    }
    readingSvg.style.setProperty("--book-play-state", "paused");
    readingSvg.classList.remove("is-paused");
    if (!open) await switchReadingMark(false);
    return;
  }

  readingSvg.style.setProperty("--book-play-state", "paused");
  readingSvg.classList.add("is-paused");
  if (!bookAnimations.length) {
    await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    bookAnimations = readingSvg.querySelector<SVGGElement>("#book-root")
      ?.getAnimations({ subtree: true }) ?? [];
    for (const animation of bookAnimations) {
      animation.pause();
      const duration = Number(animation.effect?.getTiming().duration) || 0;
      animation.currentTime = props.readingMode ? duration : 0;
    }
  }
  readingSvg.style.setProperty("--book-play-state", "running");
  readingSvg.classList.remove("is-paused");
  const animations = bookAnimations;
  for (const animation of animations) {
    const duration = Number(animation.effect?.getTiming().duration) || 0;
    if (open && animation.currentTime === null) animation.currentTime = 0;
    if (open && animation.currentTime === 0) animation.playbackRate = 1;
    if (!open && animation.currentTime === null) animation.currentTime = duration;
    if (!open && animation.currentTime === 0) continue;
    if (open && animation.currentTime === duration) continue;
    animation.playbackRate = open ? 1 : -1;
    animation.play();
  }
  animationRun = Promise.all(animations.map((animation) => animation.finished.catch(() => undefined)))
    .then(() => {
      if (generation !== animationGeneration) return;
      for (const animation of animations) animation.pause();
      readingSvg?.style.setProperty("--book-play-state", "paused");
      readingSvg?.classList.add("is-paused");
    });
  await animationRun;
  if (!open) await switchReadingMark(false);
}

onMounted(() => {
  const markParsed = new DOMParser().parseFromString(readingMarkSource, "image/svg+xml").documentElement;
  const readingParsed = new DOMParser().parseFromString(readingSource, "image/svg+xml").documentElement;
  if (!(markParsed instanceof SVGSVGElement) || !(readingParsed instanceof SVGSVGElement) || !readingMarkHost.value) return;
  readingMark.value = document.importNode(markParsed, true) as SVGSVGElement;
  readingSvg = document.importNode(readingParsed, true) as SVGSVGElement;
  readingMark.value.classList.add("app-header__mark-svg");
  readingSvg.classList.add("app-header__reading-svg", "is-paused");
  readingSvg.style.setProperty("--book-play-state", "paused");
  readingSvg.setAttribute("focusable", "false");
  readingMark.value.setAttribute("focusable", "false");
  setSharedTransitionNames(readingMark.value, "readux-shared");
  setSharedTransitionNames(readingSvg, "readux-shared");
  readingMarkHost.value.replaceChildren(readingMark.value, readingSvg);
  setReadingVisibility(props.readingMode);
  updateReadingCover();
  void nextTick().then(() => {
    const root = readingSvg?.querySelector<SVGGElement>("#book-root");
    bookAnimations = root?.getAnimations({ subtree: true }) ?? [];
    for (const animation of bookAnimations) {
      animation.pause();
      const duration = Number(animation.effect?.getTiming().duration) || 0;
      animation.currentTime = props.readingMode ? duration : 0;
    }
  });
});

watch(() => [props.coverSrc, props.coverKey], updateReadingCover);
defineExpose({ playBookAnimation });
</script>

<template>
  <header
    class="app-header"
    :class="{
      'app-header--collapsible': collapseBrand,
      'app-header--has-title': Boolean(title),
    }"
  >
    <div class="app-header__identity">
      <button
        class="app-header__back"
        :class="{ 'app-header__back--visible': showBack }"
        type="button"
        :title="backLabel"
        :aria-label="backLabel"
        :aria-hidden="!showBack"
        :tabindex="showBack ? 0 : -1"
        :disabled="!showBack"
        @click="emit('back')"
      >
        <ArrowLeft :size="19" aria-hidden="true" />
      </button>
      <span ref="readingMarkHost" class="app-header__reading-mark" aria-hidden="true" />
      <svg
        class="app-header__wordmark"
        :class="{ 'app-header__wordmark--collapsed': collapseBrand }"
        viewBox="0 0 357.25 120"
        role="img"
        aria-label="ReadUX"
        focusable="false"
      >
        <use href="/branding/readux-wordmark.svg#readux-wordmark" />
      </svg>
      <span
        class="app-header__divider"
        :class="{ 'app-header__divider--visible': title || description }"
        aria-hidden="true"
      />
      <Transition name="app-header-title">
        <div v-if="title" key="title" class="app-header__page-identity">
          <strong>{{ title }}</strong>
          <span v-if="description">{{ description }}</span>
        </div>
        <span v-else-if="description" key="description" class="app-header__brand-description">{{ description }}</span>
      </Transition>
      <slot name="version" />
    </div>
    <div class="app-header__actions">
      <slot name="actions" />
      <button
        v-if="!collapseBrand"
        class="app-header__about"
        type="button"
        title="关于 ReadUX"
        aria-label="关于 ReadUX"
        @click="showAbout = true"
      >
        <BadgeInfo :size="18" stroke-width="1.8" aria-hidden="true" />
      </button>
    </div>
  </header>
  <Teleport to="body">
    <div
      v-if="showAbout"
      class="about-dialog-backdrop"
      role="presentation"
      @click.self="showAbout = false"
      @keydown.esc="showAbout = false"
    >
      <section
        class="about-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="about-dialog-title"
      >
        <header class="about-dialog__header">
          <div>
            <h2 id="about-dialog-title">关于 ReadUX</h2>
          </div>
          <button
            class="about-dialog__close"
            type="button"
            title="关闭"
            aria-label="关闭关于窗口"
            @click="showAbout = false"
          >
            <X :size="18" aria-hidden="true" />
          </button>
        </header>
        <div class="about-dialog__body">
          <p>
            ReadUX 是面向日语原文沉浸式阅读与快捷语言语法分析的本地桌面应用, 以
            redux——整合完善你需要的一切日语阅读功能为目标, 并提供良好、快捷的 UX
            交互. 目前完成的主要功能有: 快速 NLP 分析; 智能分词和悬浮查词;
            语法活用识别.
          </p>
          <h3>关于作者</h3>
          你可以叫我 Loopo, 这是
          <a
            href="https://github.com/OsakaLOOP"
            target="_blank"
            rel="noreferrer"
            >我的Github</a
          >, 和
          <a href="https://www.loopo.cc/" target="_blank" rel="noreferrer"
            >我的博客</a
          >, 以及<a
            href="https://ifdian.net/a/OsakaLOOP"
            target="_blank"
            rel="noreferrer"
            >我的爱发电主页</a
          >.
          <h3>许可证</h3>
          <p>
            项目代码采用 MIT License, 文本见发行包中的
            <code>LICENSE</code> 文件。
          </p>
          <h3>致谢</h3>
          <ul>
            <li>
              <a
                href="https://clrd.ninjal.ac.jp/unidic/"
                target="_blank"
                rel="noreferrer"
                >UniDic</a
              >：感谢国立国语研究所及相关维护团队提供日语词法资源。UniDic
              资源遵循官方的 GPL v2.0、LGPL v2.1 与 BSD 许可条件。
            </li>
            <li>
              <a
                href="https://github.com/megagonlabs/ginza"
                target="_blank"
                rel="noreferrer"
                >GiNZA</a
              >：感谢 Megagon Labs 提供基于 spaCy 与 Sudachi
              的日语依存分析工具，GiNZA 项目采用 MIT License。
            </li>
            <li>
              <a
                href="https://forum.freemdict.com/"
                target="_blank"
                rel="noreferrer"
                >FreeMdict 论坛</a
              >：感谢论坛及其用户制作的 MDict 词典与分享的技术讨论。
            </li>
          </ul>
          <p class="about-dialog__note">
            第三方程序、模型、词典资源及其派生文件不属于本项目 MIT 许可范围.
          </p>
        </div>
      </section>
    </div>
  </Teleport>
</template>

<style scoped>
.app-header {
  z-index: 10;
  display: flex;
  min-height: 58px;
  flex: 0 0 auto;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 24px;
  border-bottom: 1px solid var(--border-color);
  background: var(--glass-bg);
  backdrop-filter: var(--glass-filter);
}

.app-header__identity {
  display: flex;
  min-width: 0;
  flex: 1 1 auto;
  align-items: center;
  gap: 8px;
  overflow: visible;
}

.app-header__back {
  display: grid;
  width: 0;
  height: 32px;
  flex: 0 0 auto;
  place-items: center;
  margin-right: -8px;
  overflow: hidden;
  border: 0;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  opacity: 0;
  transform: translateX(-7px);
  transition: width 420ms cubic-bezier(0.22, 1, 0.36, 1),
    margin-right 420ms cubic-bezier(0.22, 1, 0.36, 1),
    opacity 220ms ease, transform 420ms cubic-bezier(0.22, 1, 0.36, 1);
}

.app-header__back--visible {
  width: 32px;
  margin-right: 0;
  opacity: 1;
  transform: translateX(0);
}

.app-header__back:hover,
.app-header__back:focus-visible {
  outline: 0;
  color: var(--accent-color);
}

.app-header__reading-mark {
  display: block;
  flex: 0 0 auto;
  width: 48px;
  height: 44px;
}

.app-header__reading-mark :deep(svg) {
  display: block;
  width: 100%;
  height: 100%;
}

:global(::view-transition-group(readux-shared-kabuto-helmet)),
:global(::view-transition-group(readux-shared-kuwagata-left)),
:global(::view-transition-group(readux-shared-kuwagata-right)),
:global(::view-transition-group(readux-shared-katana-blade-a)),
:global(::view-transition-group(readux-shared-katana-highlight-a)),
:global(::view-transition-group(readux-shared-katana-blade-b)),
:global(::view-transition-group(readux-shared-katana-highlight-b)) {
  animation-duration: 620ms;
  animation-timing-function: cubic-bezier(0.22, 1, 0.36, 1);
}

:global(::view-transition-old(readux-shared-kabuto-helmet)),
:global(::view-transition-new(readux-shared-kabuto-helmet)),
:global(::view-transition-old(readux-shared-kuwagata-left)),
:global(::view-transition-new(readux-shared-kuwagata-left)),
:global(::view-transition-old(readux-shared-kuwagata-right)),
:global(::view-transition-new(readux-shared-kuwagata-right)),
:global(::view-transition-old(readux-shared-katana-blade-a)),
:global(::view-transition-new(readux-shared-katana-blade-a)),
:global(::view-transition-old(readux-shared-katana-highlight-a)),
:global(::view-transition-new(readux-shared-katana-highlight-a)),
:global(::view-transition-old(readux-shared-katana-blade-b)),
:global(::view-transition-new(readux-shared-katana-blade-b)),
:global(::view-transition-old(readux-shared-katana-highlight-b)),
:global(::view-transition-new(readux-shared-katana-highlight-b)) {
  mix-blend-mode: normal;
}

.app-header__wordmark {
  display: inline-flex;
  flex: 0 0 auto;
  width: 126px;
  height: 38px;
  opacity: 1;
  transition: width 420ms cubic-bezier(0.22, 1, 0.36, 1), opacity 220ms ease;
}

.app-header__wordmark--collapsed {
  width: 0;
  opacity: 0;
  pointer-events: none;
}

.app-header__divider {
  width: 1px;
  height: 22px;
  flex: 0 0 auto;
  margin-right: 2px;
  background: var(--border-color);
  opacity: 0;
  transform: scaleY(0.72);
  transition: opacity 220ms ease, transform 420ms cubic-bezier(0.22, 1, 0.36, 1);
}

.app-header__divider--visible {
  opacity: 1;
  transform: scaleY(1);
}

.app-header__brand-description {
  min-width: 0;
  overflow: hidden;
  color: var(--text-muted);
  font-size: 0.75rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.app-header__page-identity {
  display: flex;
  min-width: 0;
  max-width: min(46vw, 760px);
  flex-direction: column;
  line-height: 1.25;
}

.app-header-title-enter-active,
.app-header-title-leave-active {
  transition: opacity 220ms ease, transform 420ms cubic-bezier(0.22, 1, 0.36, 1);
}

.app-header-title-enter-from {
  opacity: 0;
  transform: translateY(9px);
}

.app-header-title-leave-to {
  opacity: 0;
  transform: translateY(9px);
}

.app-header__page-identity strong,
.app-header__page-identity span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.app-header__page-identity strong {
  color: var(--text-primary);
  font-size: 0.86rem;
}

.app-header__page-identity span {
  color: var(--text-muted);
  font-size: 0.72rem;
}

.app-header__actions {
  display: flex;
  min-width: 0;
  flex: 0 1 auto;
  align-items: center;
  gap: 4px;
  overflow: visible;
}

.app-header__about,
.about-dialog__close {
  display: grid;
  width: 32px;
  height: 32px;
  flex: 0 0 auto;
  place-items: center;
  border: 0;
  border-radius: 50%;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
}

.app-header__about:hover,
.app-header__about:focus-visible,
.about-dialog__close:hover,
.about-dialog__close:focus-visible {
  outline: 0;
  background: var(--accent-light);
  color: var(--accent-color);
}

.about-dialog-backdrop {
  position: fixed;
  z-index: 100;
  inset: 0;
  display: grid;
  place-items: center;
  padding: 20px;
  background: color-mix(in srgb, #111 38%, transparent);
}

.about-dialog {
  width: min(560px, 100%);
  max-height: min(720px, calc(100vh - 40px));
  overflow: hidden;
  border: 1px solid var(--border-color);
  border-radius: 12px;
  background: var(--bg-primary);
  box-shadow: 0 18px 60px color-mix(in srgb, #000 22%, transparent);
}

.about-dialog__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 18px 20px 14px;
  border-bottom: 1px solid var(--border-color);
}

.about-dialog__header h2,
.about-dialog__header p,
.about-dialog__body h3,
.about-dialog__body p {
  margin: 0;
}

.about-dialog__header h2 {
  color: var(--text-primary);
  font-size: 1rem;
}
.about-dialog__header p {
  margin-top: 4px;
  color: var(--text-muted);
  font-size: 0.72rem;
}
.about-dialog__body {
  max-height: calc(min(720px, 100vh - 40px) - 82px);
  overflow: auto;
  padding: 18px 20px 20px;
  color: var(--text-secondary);
  font-size: 0.8rem;
  line-height: 1.65;
}
.about-dialog__body h3 {
  margin-top: 18px;
  margin-bottom: 6px;
  color: var(--text-primary);
  font-size: 0.82rem;
}
.about-dialog__body ul {
  display: grid;
  gap: 9px;
  margin: 8px 0 0;
  padding-left: 20px;
}
.about-dialog__body a {
  color: var(--accent-color);
}
.about-dialog__body code {
  padding: 1px 4px;
  border-radius: 3px;
  background: var(--bg-secondary);
  font-size: 0.75rem;
}
.about-dialog__note {
  margin-top: 18px !important;
  color: var(--text-muted);
  font-size: 0.72rem;
}

@media (max-width: 520px) {
  .about-dialog-backdrop {
    padding: 10px;
  }
  .about-dialog {
    max-height: calc(100vh - 20px);
  }
  .about-dialog__body {
    max-height: calc(100vh - 102px);
  }
}

@media (max-width: 820px) {
  .app-header {
    padding-right: 12px;
    padding-left: 12px;
  }

  .app-header--collapsible .app-header__page-identity {
    max-width: 34vw;
  }
}

@media (prefers-reduced-motion: reduce) {
  .app-header__back,
  .app-header__wordmark,
  .app-header__divider,
  .app-header-title-enter-active,
  .app-header-title-leave-active {
    transition-duration: 0.01ms !important;
  }

  .app-header__reading-mark :deep(svg *) {
    transition-duration: 0.01ms !important;
  }
}
</style>
