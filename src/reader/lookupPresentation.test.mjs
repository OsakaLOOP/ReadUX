import assert from "node:assert/strict";
import test from "node:test";
import { dictionaryLookupFromSearch, displayReading, readerCapsuleRanges, readerChains, readerMorphologyHits as projectMorphologyHits, readerRubyAnnotations, readerTargetPresentation } from "./lookupPresentation.ts";
import { posDisplayLabel } from "../explanation/morphologyView.ts";

// 历史查询样例提供气泡内容投影需要的分析字段。
function readerMorphologyHits(targets, source) {
  const document = structuredClone(source);
  document.source ??= { tokens: [] };
  document.morphemes ??= [];
  for (const chain of document.morphology?.chains ?? []) {
    for (const operator of chain.operators ?? []) {
      operator.evidence ??= [];
      operator.source_morpheme_range ??= [0, 0];
    }
  }
  return projectMorphologyHits(targets, document);
}

test("表屈折形查辞书形时仍以目标锚点语素显示词性", () => {
  const document = {
    morphemes: [{ id: "m0", source_index: 0, pos: ["動詞", "一般"], lemma: "知る", char_range: [0, 2] }],
    source: { tokens: [{ index: 0, fields: [{ name: "cType", value: "五段-ラ行" }] }] },
    morphology: { chains: [{ chain_id: "知る", anchor_morpheme: 0 }] },
  };
  const presentation = readerTargetPresentation({
    char_range: [0, 2], surface: "知っ", morpheme_ids: ["m0"], lexical_core_ids: ["知る"], lookup_forms: [{ form: "知る" }],
  }, document);
  assert.deepEqual(presentation.pos, { major: "動詞", sub1: "一般", sub2: "", sub3: "" });
  assert.equal(posDisplayLabel(presentation.pos, presentation.conjugationType), "用言 · 動詞 · 五段");
});

test("缩略形式标签仅用于实际缩约，规范原型继续作为副标题", () => {
  for (const [kind, prototype, contracted] of [
    ["contracted_te_iru", "ている", true], ["contracted_te_shimau", "てしまう", true], ["contracted_te_oku", "ておく", true],
    ["te_iru", "ている", false], ["te_yuku", "てゆく", false], ["tari_suru", "たりする", false],
    ["passive_potential", "られる", false], ["tara_condition", "たら", false], ["copula", "だ", false], ["deshou_modality", "でしょう", false],
  ]) {
    const document = {
      text: "読んでる", morphemes: [], formation: { nodes: [] }, morphology: {
        chains: [{ chain_id: "core", lookup_form: "読む", surface_form: "読ん", operators: [
          { operator_id: "op", kind, normalized_form: prototype, char_range: [2, 4] },
        ] }],
        occurrences: [{ id: "occ", chain_id: "core", kind, operator_ids: ["op"], hit_ranges: [[2, 4]] }],
      },
    };
    const [hit] = readerMorphologyHits([{ id: "word", char_range: [0, 2], lexical_core_ids: ["core"], source_formation_ids: [] }], document);
    assert.deepEqual(hit.detail.tags, contracted ? ["缩略形式"] : [], kind);
    assert.equal(hit.detail.surface, prototype);
  }
});

test("条件词尾的显示覆盖完整たら与だら", () => {
  for (const [text, lemma, ending] of [["なれたら", "なる", "たら"], ["読んだら", "読む", "だら"]]) {
    const operator = { operator_id: "condition", kind: "tara_condition", label: "条件", char_range: [2, 4], source_morpheme_range: [1, 2] };
    const document = {
      text, morphemes: [{ surface: text.slice(0, 2), lemma }, { surface: ending, lemma: "た", pos: ["助動詞"] }],
      formation: { nodes: [] }, morphology: {
        chains: [{ chain_id: "core", char_range: [0, 4], lookup_form: lemma, surface_form: text, operators: [operator] }],
        occurrences: [{ id: "condition", chain_id: "core", operator_ids: ["condition"], hit_ranges: [[2, 4]] }],
      },
    };
    const hits = readerMorphologyHits([{ id: "word", char_range: [0, 2], lexical_core_ids: ["core"], source_formation_ids: [] }], document);
    assert.equal(hits.length, 1);
    assert.equal(hits[0].detail.title, "条件形");
    assert.equal(hits[0].detail.surface, ending);
    assert.deepEqual(hits[0].range, [2, 4]);
  }
});

test("みたい和よう之后的に、な、だ分别显示对应功能与形式", () => {
  for (const word of ["みたい", "よう"]) {
    for (const [ending, kind, title] of [["に", "adverbial_ni", "連用修飾"], ["な", "attributive_na", "連体修飾"], ["だ", "copula", "断定"]]) {
      const range = [word.length, word.length + 1];
      const text = word + ending;
      const document = {
        text, formation: { nodes: [] },
        morphemes: [{ surface: word, lemma: word, pos: ["形状詞"] }, { surface: ending, lemma: "だ", pos: ["助動詞"] }],
        morphology: {
          chains: [{ chain_id: "core", char_range: [0, text.length], lookup_form: word, surface_form: text,
            operators: [{ operator_id: "ending", kind, char_range: range, source_morpheme_range: [1, 2] }] }],
          occurrences: [{ id: "ending", chain_id: "core", operator_ids: ["ending"], hit_ranges: [range] }],
        },
      };
      const targets = [{ id: "word", char_range: [0, word.length], lexical_core_ids: ["core"], source_formation_ids: [] }];
      const hits = readerMorphologyHits(targets, document);
      assert.equal(hits.length, 1);
      assert.equal(hits[0].detail.title, title);
      assert.equal(hits[0].detail.surface, ending);
      assert.ok(hits[0].detail.description.includes(`→ ${text}`));
      assert.deepEqual(readerCapsuleRanges(targets, document)[0].range, [0, text.length]);
    }
  }
});

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

test("句末助词ん保持独立于谓语胶囊", () => {
  const targets = [
    { id: "suru", char_range: [0, 1], lexical_core_ids: ["predicate"], source_formation_ids: [] },
    { id: "ending", char_range: [2, 3], lexical_core_ids: [], source_formation_ids: [] },
  ];
  const document = {
    text: "したん", formation: { nodes: [] },
    morphemes: [{ surface: "し", lemma: "為る", pos: ["動詞"], char_range: [0, 1] },
      { surface: "た", lemma: "た", pos: ["助動詞"], char_range: [1, 2] },
      { surface: "ん", lemma: "の", pos: ["助詞", "準体助詞"], char_range: [2, 3] }],
    morphology: { chains: [
      { chain_id: "predicate", role: "lexical", parent_chain_id: null, surface_form: "した", lookup_form: "する", char_range: [0, 2], core_morpheme_indices: [0] },
      { chain_id: "ending", role: "functional", parent_chain_id: null, surface_form: "ん", char_range: [2, 3], core_morpheme_indices: [2] },
    ] },
  };
  assert.deepEqual(readerCapsuleRanges(targets, document), [{ id: "suru", range: [0, 2] }, { id: "ending", range: [2, 3] }]);
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
  assert.equal(hit.detail.description, "力説する → 力説して（する→して）");
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
  assert.equal(chain.conjugation, "五段活用 - 連用形");
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
  assert.equal(hits[0].detail.title, "進行・状態継続");
  assert.equal(hits[0].detail.surface, "ている");
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
  assert.equal(hit.detail.title, "動詞・五段活用");
  assert.equal(hit.detail.conjugation, "連用形");
  assert.equal(hit.detail.description, "読む → 読ん（む→ん）");
});

test("可能动词的两个气泡统一从词元推导当前词形", () => {
  const state = { category: "動詞", form: "irrealis", conjugation_type: "下一段-ラ行", conjugation_form: "未然形-一般" };
  const document = {
    text: "立ち回れない", formation: { nodes: [] }, morphemes: [{ id: "m0", surface: "立ち回れ", pos: ["動詞"], char_range: [0, 4] }],
    morphology: { chains: [{ chain_id: "core", role: "lexical", char_range: [0, 6], anchor_range: [0, 4],
      surface_form: "立ち回れない", lemma_form: "立ち回る", dictionary_form: "立ち回れる", lookup_form: "立ち回る",
      core_morpheme_indices: [0], morpheme_indices: [0], final_state: state, operators: [
        { operator_id: "conj", kind: "conjugation", char_range: [0, 4], input_state: "立ち回る", state_before: { category: "動詞", form: "irrealis", conjugation_type: "下一段-ラ行", conjugation_form: "未然形-一般" }, state_after: state, evidence: ["unidic:potential"] },
      ] }], occurrences: [] },
  };
  const target = { id: "word", char_range: [0, 4], lexical_core_ids: ["core"], source_formation_ids: [], morpheme_ids: ["m0"] };
  const [hit] = readerMorphologyHits([target], document);
  const [chain] = readerChains(target, document);
  assert.equal(hit.detail.title, "動詞・下一段活用");
  assert.equal(hit.detail.conjugation, "未然形 - 可能形");
  assert.equal(hit.detail.description, "立ち回る → 立ち回れ（る→れ）");
  assert.equal(chain.lookupForm, "立ち回る");
  assert.equal(chain.steps[0].description, hit.detail.description);
});

test("复合活用链如読まなかった按现实原型展示标题并逐步承接原型推导", () => {
  const document = {
    text: "読まなかった",
    formation: { nodes: [] },
    morphemes: [
      { id: "m0", surface: "読ま", lemma: "読む", pos: ["動詞"] },
      { id: "m1", surface: "なかっ", lemma: "ない", pos: ["助動詞"] },
      { id: "m2", surface: "た", lemma: "た", pos: ["助動詞"] },
    ],
    morphology: {
      chains: [{
        chain_id: "c0", role: "lexical", parent_chain_id: null, anchor_range: [0, 2], char_range: [0, 6],
        surface_form: "読まなかった", lemma_form: "読む", dictionary_form: "読む", lookup_form: "読む",
        core_morpheme_indices: [0], morpheme_indices: [0, 1, 2],
        final_state: { category: "助動詞", form: "terminal", conjugation_type: "助動詞-タ", conjugation_form: "基本形" },
        connection_forms: [],
        operators: [
          { operator_id: "op-neg", kind: "negative", label: "否定", source_morpheme_range: [1, 2], char_range: [2, 5],
            state_before: { category: "動詞", form: "irrealis", conjugation_type: "五段-マ行", conjugation_form: "未然形-一般" },
            state_after: { category: "助動詞", form: "continuative", conjugation_type: "助動詞-ナイ", conjugation_form: "連用形-促音便" } },
          { operator_id: "op-past", kind: "past", label: "过去", source_morpheme_range: [2, 3], char_range: [5, 6],
            state_before: { category: "助動詞", form: "continuative", conjugation_type: "助動詞-ナイ", conjugation_form: "連用形-促音便" },
            state_after: { category: "助動詞", form: "terminal", conjugation_type: "助動詞-タ", conjugation_form: "基本形" } },
        ],
      }],
      occurrences: [
        { id: "occ-neg", chain_id: "c0", kind: "negative", char_range: [2, 5], hit_ranges: [[2, 5]], operator_ids: ["op-neg"], candidates: [] },
        { id: "occ-past", chain_id: "c0", kind: "past", char_range: [5, 6], hit_ranges: [[5, 6]], operator_ids: ["op-past"], candidates: [] },
      ],
    },
  };

  const hits = readerMorphologyHits([{ id: "target-yomu", char_range: [0, 6], lexical_core_ids: ["c0"], source_formation_ids: [] }], document);
  const negHit = hits.find((h) => h.detail.id === "occ-neg");
  const pastHit = hits.find((h) => h.detail.id === "occ-past");
  const coreHit = hits.find((h) => h.detail.id.endsWith("core-conjugation"));

  // 1. 核心词未然形屈折命中
  assert.ok(coreHit);
  assert.equal(coreHit.detail.title, "動詞・五段活用");
  assert.equal(coreHit.detail.conjugation, "未然形");
  assert.equal(coreHit.detail.description, "読む → 読ま（む→ま）");

  // 2. 否定算子命中：title 为概念标题 否定形，surface 为现实原型 ない，推导终点以ない原型结束
  assert.ok(negHit);
  assert.equal(negHit.detail.title, "未然形 - ない形");
  assert.equal(negHit.detail.surface, "ない");
  assert.equal(negHit.detail.description, "読ま → 読まない");
  assert.equal(negHit.detail.entry.transition, "読ま → 読まない");

  // 3. 过去算子命中：title 为概念标题 过去，surface 为现实原型 た，起点从上一步的原型読まない开始，终点到当前实际形式読まなかった
  assert.ok(pastHit);
  assert.equal(pastHit.detail.title, "連用形 - た形");
  assert.equal(pastHit.detail.surface, "た");
  assert.equal(pastHit.detail.description, "読まない → 読まなかった（ない→なかった）");

  // 4. 词典活用链 steps 完整对应
  const [chain] = readerChains({ lexical_core_ids: ["c0"], source_formation_ids: [], morpheme_ids: ["m0"] }, document);
  assert.equal(chain.steps.length, 3);
  assert.equal(chain.steps[0].label, "未然形");
  assert.equal(chain.steps[0].description, "読む → 読ま（む→ま）");
  assert.equal(chain.steps[1].label, "未然形 - ない形");
  assert.equal(chain.steps[1].description, "読ま → 読まない");
  assert.equal(chain.steps[2].label, "連用形 - た形");
  assert.equal(chain.steps[2].description, "読まない → 読まなかった（ない→なかった）");
});

test("研究しつくしてる整条复杂连用复合链聚合为单一连续胶囊", () => {
  const document = {
    text: "研究しつくしてる",
    morphemes: [
      { id: "m0", surface: "研究", char_range: [0, 2], pos: ["名詞"] },
      { id: "m1", surface: "し", char_range: [2, 3], pos: ["動詞"] },
      { id: "m2", surface: "つくし", char_range: [3, 6], pos: ["動詞"] },
      { id: "m3", surface: "てる", char_range: [6, 8], pos: ["助動詞"] },
    ],
    formation: { nodes: [] },
    morphology: {
      chains: [
        { chain_id: "c0", role: "lexical", parent_chain_id: null, anchor_range: [0, 3], char_range: [0, 3], core_morpheme_indices: [0, 1], morpheme_indices: [0, 1], lookup_form: "研究する", surface_form: "研究し", operators: [] },
        { chain_id: "c1", role: "functional", parent_chain_id: "c0", anchor_range: [3, 6], char_range: [3, 6], core_morpheme_indices: [2], morpheme_indices: [2], lookup_form: "尽くす", surface_form: "つくし", operators: [{ operator_id: "op-comp", kind: "completion", label: "彻底完成", char_range: [3, 6] }] },
        { chain_id: "c2", role: "functional", parent_chain_id: "c1", anchor_range: [6, 8], char_range: [6, 8], core_morpheme_indices: [3], morpheme_indices: [3], lookup_form: "ている", surface_form: "てる", operators: [{ operator_id: "op-teiru", kind: "te_iru", label: "ている形式", char_range: [6, 8] }] },
      ],
      occurrences: [
        { id: "occ-comp", chain_id: "c1", kind: "completion", char_range: [3, 6], hit_ranges: [[3, 6]], operator_ids: ["op-comp"], candidates: [] },
        { id: "occ-teiru", chain_id: "c2", kind: "te_iru", char_range: [6, 8], hit_ranges: [[6, 8]], operator_ids: ["op-teiru"], candidates: [] },
      ],
    },
  };
  const targets = [
    { id: "target-kenkyu", char_range: [0, 3], lexical_core_ids: ["c0"], source_formation_ids: [] },
  ];
  const capsules = readerCapsuleRanges(targets, document);
  assert.equal(capsules.length, 1);
  assert.deepEqual(capsules[0].range, [0, 8]);
});

test("訪ねてきたりしない包含たりする语法化句型并聚合为单一连续胶囊", () => {
  const document = {
    text: "訪ねてきたりしない",
    morphemes: [
      { id: "m0", surface: "訪ね", char_range: [0, 2], pos: ["動詞"] },
      { id: "m1", surface: "て", char_range: [2, 3], pos: ["助詞"] },
      { id: "m2", surface: "き", char_range: [3, 4], pos: ["動詞"] },
      { id: "m3", surface: "たり", char_range: [4, 6], pos: ["助詞"] },
      { id: "m4", surface: "し", char_range: [6, 7], pos: ["動詞"] },
      { id: "m5", surface: "ない", char_range: [7, 9], pos: ["助動詞"] },
    ],
    formation: { nodes: [] },
    morphology: {
      chains: [
        { chain_id: "c0", role: "lexical", parent_chain_id: null, anchor_range: [0, 3], char_range: [0, 3], core_morpheme_indices: [0], morpheme_indices: [0, 1], lookup_form: "訪ねる", surface_form: "訪ねて", operators: [{ operator_id: "op-te", kind: "te_connection", label: "て形", char_range: [2, 3] }] },
        { chain_id: "c1", role: "functional", parent_chain_id: "c0", anchor_range: [3, 6], char_range: [3, 6], core_morpheme_indices: [2], morpheme_indices: [2, 3], lookup_form: "くる", surface_form: "きたり", operators: [{ operator_id: "op-kuru", kind: "te_kuru", label: "てくる", char_range: [3, 4] }, { operator_id: "op-tari", kind: "enumerative", label: "列举", char_range: [4, 6] }] },
        { chain_id: "c2", role: "functional", parent_chain_id: "c1", anchor_range: [6, 9], char_range: [6, 9], core_morpheme_indices: [4], morpheme_indices: [4, 5], lookup_form: "する", surface_form: "しない", operators: [{ operator_id: "op-tarisuru", kind: "tari_suru", label: "たりする", char_range: [6, 7] }, { operator_id: "op-neg", kind: "negative", label: "否定", char_range: [7, 9], state_before: { category: "動詞", form: "irrealis", conjugation_type: "サ行変格", conjugation_form: "未然形-一般" } }] },
      ],
      occurrences: [
        { id: "occ-tarisuru", chain_id: "c2", kind: "tari_suru", char_range: [6, 7], hit_ranges: [[6, 7]], operator_ids: ["op-tarisuru"], candidates: [] },
        { id: "occ-neg", chain_id: "c2", kind: "negative", char_range: [7, 9], hit_ranges: [[7, 9]], operator_ids: ["op-neg"], candidates: [] },
      ],
    },
  };
  const targets = [
    { id: "target-tazuneru", char_range: [0, 3], lexical_core_ids: ["c0"], source_formation_ids: [] },
  ];
  const capsules = readerCapsuleRanges(targets, document);
  assert.equal(capsules.length, 1);
  assert.deepEqual(capsules[0].range, [0, 9]);

  const hits = readerMorphologyHits(targets, document);
  const tariSuruHit = hits.find((h) => h.detail.id === "occ-tarisuru");
  assert.ok(tariSuruHit);
  assert.equal(tariSuruHit.detail.title, "列挙・例示");
  assert.equal(tariSuruHit.detail.surface, "たりする");

  const negHit = hits.find((h) => h.detail.id === "occ-neg");
  assert.ok(negHit);
  assert.equal(negHit.detail.title, "未然形 - ない形");
  assert.equal(negHit.detail.surface, "ない");
});
