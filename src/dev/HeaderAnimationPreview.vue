<script setup lang="ts">
import { BookOpen, BookX, GraduationCap } from "@lucide/vue";
import { ref } from "vue";
import AppHeader from "../components/common/AppHeader.vue";
import GrammarGuidePanel from "../components/grammar/GrammarGuidePanel.vue";
import "../styles/main.css";

const reading = ref(false);
const grammarGuideOpen = ref(true);
</script>

<template>
  <div class="preview-app">
    <AppHeader
      animate-identity
      taskbar-icon
      :show-back="reading"
      :collapse-brand="reading"
      :reading-mode="reading"
      back-label="返回书架"
      :title="reading ? '窓辺の一日' : ''"
      :description="reading ? 'サンプル' : '日语生肉阅读助手'"
      cover-src="/branding/readux-preview-cover.svg"
      cover-key="header-preview"
      @back="reading = false"
    >
      <template #actions>
        <button
          class="preview-guide"
          type="button"
          title="打开日文语法指南"
          aria-label="打开日文语法指南"
          @click="grammarGuideOpen = true"
        >
          <GraduationCap :size="17" aria-hidden="true" />
          <span>语法指南</span>
        </button>
        <button
          class="preview-toggle"
          type="button"
          :title="reading ? '关闭书籍' : '打开书籍'"
          :aria-label="reading ? '关闭书籍' : '打开书籍'"
          @click="reading = !reading"
        >
          <BookX v-if="reading" :size="20" aria-hidden="true" />
          <BookOpen v-else :size="20" aria-hidden="true" />
        </button>
      </template>
    </AppHeader>
    <main class="preview-workspace">
      <article v-if="reading" class="preview-reading" lang="ja">
        <h1>窓辺の一日</h1>
        <p>朝の光が、机の上に置かれた本を照らしていた。窓を開けると、遠くから電車の音が聞こえた。</p>
        <p>読みかけのページを開く。昨日は気づかなかった言葉が、今日は少し違って見える。</p>
        <p>風が通り過ぎるまで、私はその一行をゆっくり読んでいた。</p>
      </article>
      <section v-else class="preview-library">
        <h1>书架</h1>
        <button class="preview-book" type="button" aria-label="打开《窓辺の一日》" @click="reading = true">
          <img src="/branding/readux-preview-cover.svg" width="150" height="210" alt="窓辺の一日" />
          <strong lang="ja">窓辺の一日</strong>
          <span>サンプル</span>
        </button>
      </section>
    </main>
    <GrammarGuidePanel :show="grammarGuideOpen" @close="grammarGuideOpen = false" />
  </div>
</template>

<style scoped>
.preview-app { display: flex; height: 100dvh; flex-direction: column; background: var(--bg-primary); color: var(--text-primary); }
.preview-workspace { min-height: 0; flex: 1; overflow: auto; padding: 32px 24px; }
.preview-library, .preview-reading { width: min(100%, 1000px); margin: 0 auto; }
.preview-workspace h1 { margin: 0 0 28px; font-size: 1.5rem; }
.preview-book { display: flex; width: 180px; flex-direction: column; align-items: flex-start; gap: 8px; border: 0; padding: 0; background: transparent; text-align: left; color: inherit; cursor: pointer; }
.preview-book img { display: block; width: 150px; height: 210px; object-fit: cover; border-radius: 3px; box-shadow: 0 3px 10px #0002; }
.preview-book strong { margin-top: 6px; font-size: 1rem; }
.preview-book span { font-size: .85rem; color: var(--text-secondary); }
.preview-reading { max-width: 720px; line-height: 2.2; }
.preview-reading p { margin: 0 0 1.4em; font-family: var(--font-ja); }
.preview-toggle { display: grid; width: 32px; height: 32px; flex: 0 0 auto; place-items: center; border: 0; background: transparent; color: var(--text-secondary); cursor: pointer; }
.preview-toggle:hover { color: var(--accent-color); }
.preview-guide { display: inline-flex; min-height: 34px; align-items: center; gap: 6px; padding: 0 9px; border: 1px solid var(--border-color); border-radius: 5px; background: var(--bg-primary); color: var(--text-secondary); cursor: pointer; font-size: 12px; }
.preview-guide:hover { border-color: var(--accent-color); color: var(--accent-color); }
@media (max-width: 520px) { .preview-workspace { padding: 24px 16px; } }
</style>
