<script setup lang="ts">
import type { ReaderChainSummary } from "../../reader/lookupPresentation";
defineProps<{ chains: ReaderChainSummary[] }>();
</script>

<template>
  <div class="reader-chain-summary">
    <div v-for="chain in chains" :key="chain.id" class="reader-chain">
      <div class="reader-chain__form"><strong>{{ chain.lookupForm || chain.lemma }}</strong><span v-if="chain.surface !== (chain.lookupForm || chain.lemma)">→ {{ chain.surface }}</span></div>
      <small v-if="chain.conjugation">{{ chain.conjugation }}</small>
      <div v-for="(step, index) in chain.steps.filter((item) => item.label !== chain.conjugation)" :key="index" class="reader-chain__step"><b>{{ step.label }}</b><span v-if="step.description && step.description !== step.label">{{ step.description }}</span></div>
    </div>
  </div>
</template>

<style scoped>
.reader-chain-summary { display: grid; gap: 8px; font: .78rem/1.5 var(--font-ui); }
.reader-chain__form { display: flex; flex-wrap: wrap; gap: 4px; font-family: var(--font-ja); }
.reader-chain__form strong { color: var(--accent-color); }
.reader-chain small { color: var(--text-muted); }
.reader-chain__step { display: flex; flex-wrap: wrap; gap: 3px 8px; }
.reader-chain__step b { color: var(--text-secondary); font-weight: 600; }
</style>
