import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { posDisplayLabel } from "../src/explanation/morphologyView.ts";
import { readerTargetPresentation } from "../src/reader/lookupPresentation.ts";

const binary = process.argv[2] || "target/debug/readux-nlp.exe";
const cases = [
  ["体言 · 名詞", "猫", "名詞"], ["体言 · 名詞", "学生", "名詞"],
  ["体言 · 代名詞", "私", "代名詞"], ["体言 · 代名詞", "これ", "代名詞"],
  ["体言 · 数詞", "三", "名詞"], ["体言 · 数詞", "一つ", "名詞"],
  ["用言 · 動詞 · 五段", "読む", "動詞"], ["用言 · 動詞 · 五段", "書く", "動詞"],
  ["用言 · 動詞 · 下一段", "食べる", "動詞"], ["用言 · 動詞 · 下一段", "寝る", "動詞"],
  ["用言 · 動詞 · 上一段", "見る", "動詞"], ["用言 · 動詞 · 上一段", "起きる", "動詞"],
  ["用言 · 動詞 · サ変", "勉強する", "動詞"], ["用言 · 動詞 · サ変", "研究する", "動詞"],
  ["用言 · 動詞 · カ変", "来る", "動詞"], ["用言 · 動詞 · カ変", "来ない", "動詞"],
  ["用言 · 形容詞 · イ形", "高い", "形容詞"], ["用言 · 形容詞 · イ形", "美しい", "形容詞"],
  ["用言 · 形容動詞 · ナ形", "静か", "形状詞"], ["用言 · 形容動詞 · ナ形", "穏やか", "形状詞"],
  ["付属語 · 助動詞 · ない", "読まない", "助動詞"], ["付属語 · 助動詞 · ない", "食べない", "助動詞"],
  ["付属語 · 助動詞 · たい", "読みたい", "助動詞"], ["付属語 · 助動詞 · たい", "食べたい", "助動詞"],
  ["付属語 · 助詞 · 格助詞", "が", "助詞"], ["付属語 · 助詞 · 格助詞", "を", "助詞"],
  ["付属語 · 助詞 · 接続助詞", "けれども", "助詞"], ["付属語 · 助詞 · 接続助詞", "けど", "助詞"],
  ["独立語 · 副詞", "とても", "副詞"], ["独立語 · 副詞", "すぐ", "副詞"],
  ["独立語 · 連体詞", "この", "連体詞"], ["独立語 · 連体詞", "あらゆる", "連体詞"],
  ["独立語 · 接続詞", "しかし", "接続詞"], ["独立語 · 接続詞", "そして", "接続詞"],
  ["独立語 · 感動詞", "ああ", "感動詞"], ["独立語 · 感動詞", "もしもし", "感動詞"],
  ["接辞 · 接頭辞", "お茶", "接頭辞"], ["接辞 · 接頭辞", "お金", "接頭辞"],
  ["接辞 · 接尾辞", "私たち", "接尾辞"], ["接辞 · 接尾辞", "子供たち", "接尾辞"],
  ["その他 · 補助記号", "。", "補助記号"], ["その他 · 補助記号", "、", "補助記号"],
];

const requests = cases.map(([, text]) => JSON.stringify({ command: "analyze", text: `${text}。`, register: "cwj" }));
const processResult = spawnSync(binary, ["stdio"], {
  input: `${requests.join("\n")}\n`, encoding: "utf8", maxBuffer: 128 * 1024 * 1024,
});
if (processResult.error) throw processResult.error;
if (processResult.status !== 0) throw new Error(processResult.stderr || `NLP 服务退出：${processResult.status}`);
const responses = processResult.stdout.trim().split(/\r?\n/).map((line) => JSON.parse(line));
assert.equal(responses.length, cases.length, "NLP 响应数量与用例不一致");
const categoryCounts = new Map();
for (const [expected] of cases) categoryCounts.set(expected, (categoryCounts.get(expected) ?? 0) + 1);
for (const [category, count] of categoryCounts) assert.equal(count, 2, `${category}：用例数量应为 2`);
assert.equal(
  posDisplayLabel({ major: "名詞", sub1: "普通名詞", sub2: "", sub3: "" }, "", [{ label: "動サ変" }]),
  "体言 · 名詞",
  "NLP 词性应覆盖冲突的词典词性标识",
);

for (const [[expected, text, major], response] of cases.map((item, index) => [item, responses[index]])) {
  assert.equal(response.error, null, `${text}：NLP 返回错误`);
  const document = response.result;
  const morphemeIndex = document.morphemes.findIndex((morpheme) => morpheme.pos?.[0] === major);
  assert.notEqual(morphemeIndex, -1, `${text}：未找到 ${major} 语素`);
  const morpheme = document.morphemes[morphemeIndex];
  const sourceToken = document.source.tokens.find((token) => JSON.stringify(token.char_range) === JSON.stringify(morpheme.char_range));
  const conjugationType = sourceToken?.fields?.[4]?.value || "";
  const pos = { major: morpheme.pos[0], sub1: morpheme.pos[1] || "", sub2: morpheme.pos[2] || "", sub3: morpheme.pos[3] || "" };
  const display = posDisplayLabel(pos, conjugationType);
  assert.equal(display, expected, `${text}：词性显示不符`);
  console.log(`通过：${text} → ${display}`);
}

console.log(`端到端词性临时检查通过：${cases.length} 个真实词，每类 2 个用例。`);

let guideTargetCount = 0;
for (const lesson of ["lesson-1-1", "lesson-2-1"]) {
  const snapshot = JSON.parse(await readFile(new URL(`../src/grammar-guide/${lesson}-analysis.json`, import.meta.url), "utf8"));
  for (const [exampleId, example] of Object.entries(snapshot.examples)) {
    for (const target of [...example.group.outer_targets, ...example.group.inner_targets]) {
      if (target.decision === "grammar") continue;
      const presentation = readerTargetPresentation(target, example.document);
      assert.ok(presentation, `${lesson} ${exampleId} ${target.surface}：指南词条没有关联到 NLP 词性`);
      const display = posDisplayLabel(presentation.pos, presentation.conjugationType);
      assert.ok(display && display !== "整体", `${lesson} ${exampleId} ${target.surface}：词性显示无效`);
      guideTargetCount += 1;
    }
  }
}
console.log(`指南词性覆盖检查通过：${guideTargetCount} 个词条目标均可从 NLP 结果取得词性。`);
