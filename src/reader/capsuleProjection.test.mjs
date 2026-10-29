import assert from "node:assert/strict";
import test from "node:test";
import { groupCapsuleRuns, stemEndingHighlights } from "./capsuleRuns.ts";
import { morphologyHighlightForToken } from "./lookupPresentation.ts";
import { volitionalStemAndEnding } from "./japaneseInflectionRanges.ts";

const parts = [
  { key: "prepare", text: "準備", range: [8, 10] },
  { key: "shi", text: "し", range: [10, 11] },
  { key: "ta", text: "た", range: [11, 12] },
];
const runs = [{ key: "plain", parts }];

test("整体词范围由一个查询容器覆盖全部片段", () => {
  const groups = groupCapsuleRuns(runs, [{ range: [8, 12], tone: "blue" }]);
  assert.equal(groups.length, 1);
  assert.equal(groups[0].kind, "runs");
  assert.equal(groups[0].active, true);
  assert.deepEqual(groups[0].runs[0].parts.map((part) => part.text), ["準備", "し", "た"]);
});

test("活用子范围只包裹对应连续片段", () => {
  const groups = groupCapsuleRuns(runs, [{ range: [10, 12], tone: "grammar" }]);
  assert.equal(groups.length, 1);
  assert.equal(groups[0].kind, "partial");
  assert.deepEqual(groups[0].segments.map((segment) => ({ active: segment.active, text: segment.parts.map((part) => part.text).join("") })), [
    { active: false, text: "準備" },
    { active: true, text: "した" },
  ]);
});

test("相邻完整 ruby run 合并进同一个查询容器", () => {
  const groupedRuns = [
    { key: "first", reading: "じゅんび", parts: [parts[0]] },
    { key: "second", parts: parts.slice(1) },
  ];
  const groups = groupCapsuleRuns(groupedRuns, [{ range: [8, 12], tone: "blue" }]);
  assert.equal(groups.length, 1);
  assert.equal(groups[0].kind, "runs");
  assert.equal(groups[0].active, true);
  assert.equal(groups[0].runs.length, 2);
});

test("词干蓝色与语法黄色连续组合成一个活用词容器", () => {
  const inflected = [
    { key: "koroshi-kanji", text: "殺", range: [2, 3] },
    { key: "koroshi-kana", text: "し", range: [3, 4] },
    { key: "mashi-1", text: "ま", range: [4, 5] },
    { key: "mashi-2", text: "し", range: [5, 6] },
    { key: "ta", text: "た", range: [6, 7] },
  ];
  const coloredRanges = [
    { range: [2, 4], tone: "blue" },
    { range: [4, 6], tone: "grammar" },
    { range: [6, 7], tone: "grammar" },
  ];
  const groups = groupCapsuleRuns([{ key: "inflected", parts: inflected }], coloredRanges);
  assert.equal(groups.length, 1);
  assert.equal(groups[0].kind, "runs");
  assert.equal(groups[0].active, true);
  assert.deepEqual(groups[0].runs[0].parts.map((part) => part.text).join(""), "殺しました");
});

test("同一图元素的词干与首个活用假名共用一个高亮对象", () => {
  const diagram = { sentence: [
    { unit: "stem", range: [0, 2], selected: false },
    { unit: "ending", range: [2, 3], selected: true },
  ] };
  const routed = morphologyHighlightForToken(diagram, [2, 3], "fallback");
  assert.equal(routed.unit, "ending");
  assert.deepEqual(routed.range, [2, 3]);
  const groups = groupCapsuleRuns([{ key: "all", parts: [
    { key: "stem", text: "読", range: [0, 1] },
    { key: "stem2", text: "み", range: [1, 2] },
    { key: "ending", text: "た", range: [2, 3] },
    { key: "ending2", text: "い", range: [3, 4] },
  ] }], [
    { range: [0, 2], tone: "blue", group: "stem-ending" },
    { range: [2, 4], tone: "grammar", group: "stem-ending" },
  ]);
  assert.equal(groups.length, 1);
  assert.equal(groups[0].kind, "runs");
});

test("相邻的不同接续图元素保持独立高亮对象", () => {
  const groups = groupCapsuleRuns([{ key: "all", parts: [
    { key: "a", text: "て", range: [0, 1] },
    { key: "b", text: "いる", range: [1, 3] },
  ] }], [
    { range: [0, 1], tone: "grammar", group: "te" },
    { range: [1, 3], tone: "grammar", group: "iru" },
  ]);
  assert.equal(groups.length, 1);
  assert.equal(groups[0].kind, "partial");
  assert.deepEqual(groups[0].segments.map((segment) => segment.activeGroup), ["te", "iru"]);
});

test("点击词尾时显示蓝色词干和黄色连续尾缀", () => {
  const parts = [
    { range: [2, 3], role: "stem" },
    { range: [3, 4], role: "stem" },
    { range: [4, 6], role: "grammar" },
    { range: [6, 7], role: "grammar" },
    { range: [9, 11], role: "grammar" },
  ];
  assert.deepEqual(stemEndingHighlights(parts, [6, 7]), [
    { range: [2, 4], tone: "blue" },
    { range: [4, 6], tone: "grammar" },
    { range: [6, 7], tone: "grammar" },
  ]);
  assert.equal(stemEndingHighlights([
    { range: [2, 3], role: "stem" },
    { range: [3, 4], role: "stem" },
    { range: [5, 7], role: "grammar" },
  ], [5, 7]), null);
});

test("整形态素的五段意志形保留词干与末尾う的边界", () => {
  assert.deepEqual(volitionalStemAndEnding("作ろう", [3, 6]), {
    stem: [3, 5],
    ending: [5, 6],
  });
});
