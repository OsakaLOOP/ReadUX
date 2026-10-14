# ReadUX

## **Reading, brought back to where it belongs.**

A thoughtful redux of the Japanese desktop reader, to make the last one you'll ever need—integrating deep NLP, authoritative dictionaries, and grammar analysis all in one workspace. Load and analyze once, and let your mind flow with zero manual highlighting or lookup latency.

---

## Turn Any Book Into Your Own Textbook

The best way to learn Japanese is to read what you actually care about. Yet jumping from textbooks to real novels or essays often feels like hitting a wall—not because you lack motivation, but because raw Japanese text isn't built for learners.

Japanese has no spaces. Words conjugate, kanas fuse into compounds, sometimes stay as okurigana and sometimes are absorbed into kanji. With traditional readers or pop-up tools, you are forced to manually highlight text or guess where an unknown word begins and ends, or even end up inputing it for yourself—how to determine a word when you don't even know it yet?

**ReadUX bridges that gap.** Drop in any EPUB or text you want to read, and ReadUX transforms it into approachable, structured reading material:

- **No Manual Highlighting or Guessing:** Every word, compound expression, and grammar pattern is pre-segmented with high linguistic accuracy. Just click—ReadUX already knows what belongs together with furigana markups considered.
- **Load Once, Read Effortlessly:** Instead of running laggy lookups on the fly, ReadUX analyzes the entire book or chapter **once** upon import and caches it locally, and when the first page is done it will immediately be set on your screen. When you sit down to read, every definition and grammatical breakdown responds in 0ms.

---

## Core Features

### 1. High-Accuracy NLP: UniDic + GiNZA (Beyond Legacy IPADic)

Most Japanese learning tools still rely on **IPADic**—a dictionary standard from the early 2000s that struggles with modern vocabulary,orthographic variants, and complex inflections. ReadUX takes a much more rigorous approach:

- **UniDic Morphology (via Vibrato):** Uses the modern **UniDic** standard developed by NINJAL, providing consistent lemma resolution across spelling variations, historical kanji (_kyūjitai_), and intricate verb/adjective conjugations.
- **GiNZA Syntactic & Dependency Parsing:** Integrates **GiNZA** for true sentence-level comprehension—accurately resolving phrase boundaries (_bunsetsu_), clause structures, and dependency graphs where flat tokenizers fail.
- **Smart Word Formation & Furigana Alignment:** Reconstructs meaningful lexical units, compound words, and multi-word idioms while cleanly stripping and aligning EPUB ruby annotations so furigana never breaks word segmentation.

### 2. All-in-One Dictionary & Grammar Companion

- **Multi-Dictionary Aggregation:** Look up words across authoritative monolingual and bilingual dictionaries (with built-in adapters for _Daijirin_, _Shogakukan_, _Crown_, and `.kdict` bundles) in a single, clean inline view—no constant context switching between apps.
- **In-Context Grammar Breakdown:** Automatically detects and explains grammar patterns right inside the sentence you are reading, backed by a curated catalog of rules, usages, and nuances.
- **Optional LLM Sense Disambiguation:** When a word has multiple subtle meanings, an optional context-aware LLM helper can highlight the exact definition that fits the current sentence. [TODO]

### 3. Continuous Exposure Tracking & Study Loop

- **Passive Familiarity Profiling:** As you read, ReadUX quietly tracks your exposure to vocabulary—helping you see what is brand new and what you have already met dozens of times across different books.
- **Structured Export & Anki Integration _(Roadmap / TODO)_:** Export analyzed sentences and lexical data to JSON today, with seamless one-click **Anki card generation** and long-term cross-book mastery tracking actively in development.

### 4. BERT Large Inference Service

ReadUX aims to provide a shared BERT large inference service for full-book linguistic analysis. The service will batch and analyze text units, return structured results to the reader, and support concurrent reading sessions while preserving the UniDic alignment and analysis interfaces. [TODO]

---

## Architecture Overview

ReadUX is built in Rust as a modular workspace:

- `crates/readux-nlp` — The linguistic pipeline: UniDic/Vibrato tokenization, GiNZA syntactic/dependency integration, bunsetsu & clause graphs, furigana (`ruby`) alignment, kyūjitai handling, and grammar matching.
- `crates/readux-core` — Core application logic: EPUB import, ahead-of-time analysis engine & local caching (`analysis_cache`), multi-dictionary adapters, learner exposure profiles, and data export.
- `crates/readux-quality-audit` — Regression and baseline auditing to ensure segmentation and grammar accuracy never degrade across updates. [TODO]

---

## Roadmap & TODO

- [x] Ahead-of-time book/chapter analysis with persistent local caching
- [x] UniDic + GiNZA linguistic pipeline with furigana-safe alignment
- [x] Multi-dictionary aggregation (_Daijirin_, _Shogakukan_, _Crown_) & inline grammar catalog
- [ ] Reading exposure & familiarity scoring engine across the complete library
- [ ] **Anki Integration:** Direct export of mined words, context sentences, and grammar notes to Anki
- [ ] **Continuous Learning Dashboard:** Long-term vocabulary/kanji growth visualization across your entire library
- [ ] **BERT Large Inference Service:** Shared inference for full-book analysis and concurrent reading sessions (Half done)
- [ ] **LLM Sense Disambiguation:** Context-aware selection of the most relevant dictionary sense
- [ ] **Quality Audit:** Regression and baseline auditing integrated into the application workflow

---

## Getting Started

```bash
# Clone the repository
git clone https://github.com/OsakaLOOP/ReadUX.git
cd ReadUX

# Build in release mode
cargo build --release

# Run the test & linguistic quality audit suite
cargo test --workspace
```
