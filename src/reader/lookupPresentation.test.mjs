import assert from "node:assert/strict";
import test from "node:test";
import { dictionaryLookupFromSearch, displayReading, readerCapsuleRanges, readerChains, readerMorphologyHits, readerRubyAnnotations } from "./lookupPresentation.ts";

test("作者注音显示为平假名，读音匹配形式保持原样", () => {
  assert.equal(displayReading("カタカナー"), "かたかなー");
});

test("胶囊范围包含词汇核心自身的完整活用链", () => {
  const targets = [
    { id: "nedaru", char_range: [0, 2], lexical_core_ids: ["root"], source_formation_ids: [] },
  ];
  const document = {
    text: "ねだられる", morphemes: [], formation: { nodes: [] },
    morphology: { chains: [
      { chain_id: "root", role: "lexical", parent_chain_id: null, char_range: [0, 5] },
      { chain_id: "passive", role: "functional", parent_chain_id: "root", char_range: [2, 5] },
    ] },
  };
  assert.deepEqual(readerCapsuleRanges(targets, document), [{ id: "nedaru", range: [0, 5] }]);
});

test("ruby 显示范围采用验证后的最小词语范围", () => {
  assert.deepEqual(readerRubyAnnotations(
    [{ base: "以来放課後", reading: "ホウカゴ", char_range: [0, 5] }],
    [{ base: "放課後", ruby_reading: "ホウカゴ", char_range: [2, 5], candidate_char_range: [0, 5] }],
  ), [{ base: "放課後", reading: "ホウカゴ", char_range: [2, 5] }]);
});

test("每个查询目标只有一个胶囊身份，并覆盖其连续附属活用链", () => {
  const targets = [
    { id: "whole", char_range: [0, 4], lexical_core_ids: ["core"], source_formation_ids: [] },
  ];
  const document = {
    text: "読んでいる", morphemes: [], formation: { nodes: [] },
    morphology: { chains: [
      { chain_id: "core", role: "lexical", parent_chain_id: null, char_range: [0, 4] },
      { chain_id: "aux", role: "functional", parent_chain_id: "core", char_range: [4, 6] },
    ] },
  };
  assert.deepEqual(readerCapsuleRanges(targets, document), [{ id: "whole", range: [0, 6] }]);
});

test("解释助词ん与相邻过去判断链共用胶囊边界", () => {
  const targets = [
    { id: "n", char_range: [5, 6], lexical_core_ids: [], source_formation_ids: [] },
  ];
  const document = {
    text: "そうだったん", formation: { nodes: [] },
    morphemes: [{ surface: "ん", lemma: "の", pos: ["助詞"], char_range: [5, 6] }],
    morphology: { chains: [
      { chain_id: "past-copula", role: "functional", parent_chain_id: null, surface_form: "だった", lookup_form: "だ", char_range: [2, 5] },
    ] },
  };
  assert.deepEqual(readerCapsuleRanges(targets, document), [{ id: "n", range: [2, 6] }]);
});

test("サ变て形说明显示原形到整体活用形", () => {
  const document = {
    text: "力説して", formation: { nodes: [] },
    morphology: {
      chains: [{ chain_id: "sahen", role: "lexical", parent_chain_id: null, anchor_range: [0, 3],
        surface_form: "力説して", lookup_form: "力説する", operators: [
          { operator_id: "te", kind: "te_connection", label: "て形", description: "て形：て", char_range: [3, 4] },
        ] }],
      occurrences: [{ id: "occurrence:te", chain_id: "sahen", operator_ids: ["te"], kind: "te_connection",
        char_range: [3, 4], hit_ranges: [[3, 4]], candidates: [] }],
    },
  };
  const [hit] = readerMorphologyHits([
    { id: "whole", char_range: [0, 4], lexical_core_ids: ["sahen"], source_formation_ids: [] },
  ], document);
  assert.equal(hit.detail.title, "て形");
  assert.equal(hit.detail.description, "力説する → 力説して");
});

test("按当前查询目标引用展示词形、连接和成员", () => {
  const document = {
    morphemes: [
      { id: "m0", surface: "読ん", pos: ["動詞"] },
      { id: "m1", surface: "で", pos: ["助詞"] },
    ],
    formation: { nodes: [] },
    morphology: { chains: [{
      chain_id: "chain-1", role: "lexical", surface_form: "読んで", lemma_form: "読む",
      dictionary_form: "読む", lookup_form: "読む", base_lexeme: "読む",
      final_state: { conjugation_type: "五段", conjugation_form: "連用形" },
      connection_forms: ["て接続"], morpheme_indices: [0, 1], core_morpheme_indices: [0],
      operators: [{ label: "音便", description: "撥音便", output_state: "連用形" }],
    }] },
  };
  const [chain] = readerChains({ lexical_core_ids: ["chain-1"], source_formation_ids: [], morpheme_ids: ["m0"] }, document);
  assert.equal(chain.surface, "読んで");
  assert.equal(chain.lemma, "読む");
  assert.equal(chain.conjugation, "五段活用 · 连用形");
  assert.deepEqual(chain.members, [{ surface: "読ん", role: "核心" }, { surface: "で", role: "助詞" }]);
  assert.deepEqual(chain.connections, ["て接続"]);
  assert.deepEqual(chain.steps, [{ label: "音便", description: "撥音便", normalizedForm: undefined }]);
});

test("构词来源关联活用链；搜索只展示所选表记的词条", () => {
  const document = {
    morphemes: [],
    formation: { nodes: [{ id: "word-1", word: { chain_ids: ["chain-1"] } }] },
    morphology: { chains: [{ chain_id: "chain-1", surface_form: "読んだ", lemma_form: "読む", lookup_form: "読む", final_state: {}, connection_forms: [], morpheme_indices: [], core_morpheme_indices: [], operators: [] }] },
  };
  assert.equal(readerChains({ lexical_core_ids: [], source_formation_ids: ["word-1"], morpheme_ids: [] }, document)[0].lemma, "読む");
  const output = {
    forms: [{ form_id: "a" }, { form_id: "b" }], selected_form_id: "b", dictionary_names: ["辞典"],
    groups: [{ form_id: "a", entries: [{ headword: "甲" }] }, { form_id: "b", entries: [{ headword: "乙" }] }],
  };
  assert.deepEqual(dictionaryLookupFromSearch(output, "読む").entries.map((entry) => entry.headword), ["乙"]);
});

test("活用部分的两个命中范围共用核心词查询目标", () => {
  const document = {
    text: "読んでいる", formation: { nodes: [] },
    morphology: {
      chains: [
        { chain_id: "core", parent_chain_id: null, anchor_range: [0, 2], operators: [] },
        { chain_id: "aux", parent_chain_id: "core", operators: [{ operator_id: "op", kind: "te_iru", label: "ている形式", description: "ている形式：いる", normalized_form: null }] },
      ],
      occurrences: [{ id: "occ", chain_id: "aux", kind: "te_iru", char_range: [2, 5], hit_ranges: [[2, 3], [3, 5]], operator_ids: ["op"], candidates: [] }],
    },
  };
  const hits = readerMorphologyHits([{ id: "word", char_range: [0, 2], lexical_core_ids: ["core"], source_formation_ids: [] }], document);
  assert.deepEqual(hits.map((hit) => [hit.range, hit.targetId]), [
    [[2, 3], "word"], [[3, 5], "word"],
  ]);
  assert.equal(hits[0].detail.title, "ている形式");
  assert.equal(hits[0].detail.surface, "でいる");
  const [preferred] = readerMorphologyHits([
    { id: "inner", char_range: [0, 2], lexical_core_ids: ["core"], source_formation_ids: [] },
    { id: "outer", char_range: [0, 5], lexical_core_ids: ["core"], source_formation_ids: [] },
  ], document);
  assert.equal(preferred.targetId, "inner");
});

test("所属补助链参与词典气泡的整体活用概要", () => {
  const document = {
    morphemes: [{ id: "core-m", surface: "読ん", pos: ["動詞"] }, { id: "aux-m", surface: "いる", pos: ["動詞"] }],
    formation: { nodes: [] },
    morphology: { chains: [
      { chain_id: "core", parent_chain_id: null, role: "lexical", surface_form: "読んで", lemma_form: "読む", lookup_form: "読む", final_state: { conjugation_type: "五段", conjugation_form: "連用形" }, connection_forms: [], morpheme_indices: [0], core_morpheme_indices: [0], operators: [] },
      { chain_id: "aux", parent_chain_id: "core", role: "functional", surface_form: "いる", lemma_form: "いる", lookup_form: "いる", final_state: {}, connection_forms: [], morpheme_indices: [1], core_morpheme_indices: [1], operators: [{ kind: "te_iru", label: "ている形式", description: "継続" }] },
    ] },
  };
  const chains = readerChains({ lexical_core_ids: ["core"], source_formation_ids: [], morpheme_ids: ["core-m"] }, document);
  assert.deepEqual(chains.map((chain) => [chain.surface, chain.formName]), [["読んで", ""], ["いる", "ている形式"]]);
});

test("普通词尾活用生成单体说明而保留词典目标", () => {
  const document = {
    text: "読ん", formation: { nodes: [] },
    morphology: {
      chains: [{ chain_id: "core", parent_chain_id: null, anchor_range: [0, 2], surface_form: "読ん", lemma_form: "読む", dictionary_form: "読む",
        final_state: { conjugation_type: "五段", conjugation_form: "連用形", category: "動詞", form: "continuative" },
        operators: [{ operator_id: "conj", kind: "conjugation", char_range: [0, 2] }] }],
      occurrences: [],
    },
  };
  const [hit] = readerMorphologyHits([{ id: "word", char_range: [0, 2], lexical_core_ids: ["core"], source_formation_ids: [] }], document);
  assert.equal(hit.targetId, "word");
  assert.equal(hit.detail.title, "五段活用 · 连用形");
  assert.equal(hit.detail.description, "読む → 読ん");
});
