<script setup lang="ts">
import { ArrowLeft, BadgeInfo, X } from "@lucide/vue";
import { ref, watch } from "vue";
import ReadingMark from "./ReadingMark.vue";

const props = withDefaults(
  defineProps<{
    showBack?: boolean;
    backLabel?: string;
    collapseBrand?: boolean;
    title?: string;
    description?: string;
    readingMode?: boolean;
    coverSrc?: string;
    coverKey?: string;
    animateIdentity?: boolean;
    taskbarIcon?: boolean;
  }>(),
  {
    showBack: false,
    backLabel: "返回",
    collapseBrand: false,
    title: "",
    description: "",
    readingMode: false,
    coverSrc: "",
    coverKey: "",
    animateIdentity: false,
    taskbarIcon: false,
  },
);

const emit = defineEmits<{ back: [] }>();
const showAbout = ref(false);
const wordmarkHref = "/branding/readux-wordmark.svg#readux-wordmark";
const markRef = ref<InstanceType<typeof ReadingMark> | null>(null);
const bookTitle = ref(props.title);
const bookDescription = ref(props.title ? props.description : "");
const brandDescription = ref(props.title ? "" : props.description);
watch(() => [props.title, props.description], () => {
  if (props.title) {
    bookTitle.value = props.title;
    bookDescription.value = props.description;
  } else {
    brandDescription.value = props.description;
  }
});
defineExpose({ playBookAnimation: (open: boolean) => markRef.value?.playBookAnimation(open) ?? Promise.resolve() });
</script>

<template>
  <header
    class="app-header"
    :class="{
      'app-header--collapsible': collapseBrand,
      'app-header--has-title': Boolean(title),
      'app-header--animated': animateIdentity,
      'app-header--back-visible': showBack,
    }"
  >
    <div class="app-header__identity">
      <button
        v-if="showBack || animateIdentity"
        class="app-header__back"
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
      <ReadingMark
        ref="markRef"
        class="app-header__brand-logo"
        :reading="props.readingMode"
        :cover-src="props.coverSrc"
        :cover-key="props.coverKey"
        :taskbar-icon="taskbarIcon"
      />
      <svg
        v-if="!collapseBrand || animateIdentity"
        class="app-header__wordmark"
        viewBox="0 0 357.25 120"
        role="img"
        :aria-hidden="collapseBrand"
        aria-label="ReadUX"
        focusable="false"
      >
        <use :href="wordmarkHref" />
      </svg>
      <span v-if="title || description || animateIdentity" class="app-header__divider" aria-hidden="true" />
      <div class="app-header__copy">
        <div v-if="title || animateIdentity" class="app-header__page-identity" :aria-hidden="!title">
          <strong>{{ animateIdentity ? bookTitle : title }}</strong>
          <span v-if="animateIdentity ? bookDescription : description">{{ animateIdentity ? bookDescription : description }}</span>
        </div>
        <span v-if="(!title && description) || animateIdentity" class="app-header__brand-description" :aria-hidden="Boolean(title)">{{
          animateIdentity ? brandDescription : description
        }}</span>
      </div>
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
  width: 32px;
  height: 32px;
  flex: 0 0 auto;
  place-items: center;
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
}

.app-header__back:hover,
.app-header__back:focus-visible {
  outline: 0;
  color: var(--accent-color);
}

.app-header__brand-logo {
  display: block;
  flex: 0 0 auto;
  width: 44px;
  height: 44px;
}

.app-header__wordmark {
  display: inline-flex;
  flex: 0 0 auto;
  width: 126px;
  height: 38px;
}

.app-header__divider {
  width: 1px;
  height: 22px;
  flex: 0 0 auto;
  background: var(--border-color);
}

.app-header__copy {
  display: grid;
  min-width: 0;
  align-items: center;
  perspective: 600px;
}

.app-header__copy > * { grid-area: 1 / 1; }

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

.app-header--animated .app-header__back {
  width: 0;
  margin-right: -8px;
  overflow: hidden;
  opacity: 0;
  visibility: hidden;
  transform: translateX(-20px);
  transition: width 220ms ease, margin-right 220ms ease, opacity 140ms ease, transform 180ms ease, visibility 0s 180ms;
}

.app-header--animated.app-header--back-visible .app-header__back {
  width: 32px;
  margin-right: 0;
  opacity: 1;
  visibility: visible;
  transform: translateX(0);
  transition: width 220ms ease, margin-right 220ms ease;
  animation: header-back-enter 260ms both;
}

.app-header--animated .app-header__wordmark {
  transition: width 220ms cubic-bezier(.2, .65, .3, 1), margin-right 220ms ease, opacity 140ms ease;
}

.app-header--animated.app-header--collapsible .app-header__wordmark {
  width: 0;
  margin-right: -8px;
  opacity: 0;
}

.app-header--animated .app-header__divider {
  transform: scaleY(.8);
  transition: transform 220ms ease;
}

.app-header--animated.app-header--has-title .app-header__divider { transform: scaleY(1); }

.app-header--animated .app-header__page-identity {
  opacity: 0;
  transform: translateY(12px) rotateX(-65deg);
  transform-origin: 50% 100%;
  transition: opacity 120ms ease, transform 160ms ease;
  pointer-events: none;
}

.app-header--animated.app-header--has-title .app-header__page-identity {
  opacity: 1;
  transform: translateY(0) rotateX(0);
  transition: opacity 360ms 260ms ease, transform 460ms 260ms cubic-bezier(.2, .65, .3, 1);
  pointer-events: auto;
}

.app-header--animated .app-header__brand-description {
  opacity: 1;
  transform: translateY(0);
  transition: opacity 220ms 220ms ease, transform 220ms 220ms ease;
}

.app-header--animated.app-header--has-title .app-header__brand-description {
  opacity: 0;
  transform: translateY(-6px);
  transition: opacity 100ms ease, transform 160ms ease;
  pointer-events: none;
}

@keyframes header-back-enter {
  0% { opacity: 0; transform: translateX(-20px); }
  70% { opacity: 1; transform: translateX(3px); }
  100% { opacity: 1; transform: translateX(0); }
}

@media (prefers-reduced-motion: reduce) {
  .app-header.app-header--animated :is(.app-header__back, .app-header__wordmark, .app-header__divider, .app-header__page-identity, .app-header__brand-description) {
    transition: none;
    animation: none;
  }
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
</style>
