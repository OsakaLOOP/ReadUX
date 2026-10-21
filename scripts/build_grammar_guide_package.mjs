import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const spec = await readFile(resolve(root, "docs/grammar_guide_1_1.md"), "utf8");
const sourceMarkdown = spec.match(/## 1\. 原稿原文[\s\S]*?```markdown\n([\s\S]*?)\n```/)?.[1];
if (!sourceMarkdown) throw new Error("未找到 1.1 原稿正文代码块。");

const readings = {
  E01: "たろうがはなこをよぶ。", E02: "はなこがたろうをよぶ。", E03: "みずをのんだ。",
  E04: "ほんをよむ。", E05: "くうきをよむ。", E06: "きょうはがっこうにいかない。", E07: "あしたはがっこうにいきたい。",
  E08: "あめ。", E09: "いけ！", E10: "さむい。", E11: "ねこがはしる。", E12: "たかいやまがみえる。", E13: "ゆっくりあるく。",
  E14: "きぼうをもつ。", E15: "とてもうれしい。", E16: "ほんをよむ。", E17: "ほんをよみたい。", E18: "ほんをよまない。",
  E19: "ほんをよんだ。", E20: "いま、ほんをよんでいる。", E21: "このほんをよんでみる。", E22: "がっこうでぱんをたべた。",
  E23: "ここにいる。", E24: "おいしいぱんをゆっくりたべる。", E25: "ほんをよんでいる。", E26: "にわにねこがいる。",
  E27: "ねこがねむっている。", E28: "あたらしいりょうりをつくってみる。", E29: "たろうがほんをよむ。", E30: "ほんはたろうがよむ。",
  E31: "たろうだけがきた。", E32: "ぱんとぎゅうにゅうをかう。", E33: "ほんをよんでねる。", E34: "がくせいがほんをよんだ。",
  E35: "がくせいだ。", E36: "はい。", E37: "ああ、きれいだ。", E38: "いいてんきだね。", E39: "もうかえるよ。",
  E40: "あめがふっている。しかし、でかける。", E41: "ちこくのわけをきく。", E42: "いしゃにとめられているので、さけをのむわけにはいかない。",
  E43: "そういうわけで、きょうはやすみます。", E44: "だいじなことをわすれた。", E45: "およぐことができる。", E46: "らいねん、にほんにいくことにする。",
  E47: "あまいものをたべる。", E48: "こどものころは、よくかわであそんだものだ。", E49: "しずかなところにすむ。", E50: "いま、ごはんをたべているところだ。",
  E51: "ねこはへやにいる。", E52: "ねこはごはんをたべている。", E53: "えいがをみる。", E54: "このりょうりをたべてみる。", E55: "ふくにどろがつく。",
  E56: "このもんだいについてはなす。", E57: "じぶんのせきにつく。", E58: "このはしは、まちとまち、ひととひとをつないでいる。", E59: "すーぷにあじがつく。",
};

const afterBlock = { P01: 1, P02: 2, P03: 3, P04: 3, P05: 3, P06: 3, P07: 4, P08: 5, P09: 6, P10: 7, P11: 8, P12: 9 };
const examples = [];
let group = "";
for (const line of spec.split(/\r?\n/)) {
  const heading = line.match(/^### (P\d\d) (.+)$/);
  if (heading) group = heading[1];
  const row = line.match(/^\| (E\d\d) \| (.*?) \| (.*?) \| (.*?) \|$/);
  if (!row) continue;
  const [, id, ja, zh, note] = row;
  const length = Array.from(ja).length;
  const kana = readings[id] ?? "";
  examples.push({
    id, group, ja, kana, zh, note, focus: [ja],
    capsules: [{ id: `${id}-whole`, range: [0, length], partIds: [`${id}-whole`] }],
    parts: [{ id: `${id}-whole`, range: [0, length], text: ja, teachingClass: "other", classLabel: "整句观察", term: "整句", focus: true, detailId: `${id}-detail` }],
    ruby: kana ? [{ range: [0, length], base: ja, reading: kana }] : [],
    targets: [{ id: `${id}-target`, range: [0, length], detailId: `${id}-detail`, defaultEntryId: null, entryIds: [], childTargetIds: [`${id}-whole`] }],
    detail: { id: `${id}-detail`, label: "例句整体", sourceLabel: "原稿例句规格", teachingClass: "待按成分审阅", note, morphology: [], dictionaryEntryIds: [] },
    dictionaryEntries: [], morphology: [],
  });
}
if (examples.length !== 59) throw new Error(`预期 59 条补充例句，实际读取到 ${examples.length} 条。`);

const placementGroups = new Map();
for (const example of examples) {
  const index = afterBlock[example.group];
  if (index === undefined) throw new Error(`例句 ${example.id} 缺少插入位置。`);
  const item = placementGroups.get(index) ?? { afterBlock: index, group: "", exampleIds: [], layout: "list", anchor: "" };
  const groups = item.group ? item.group.split(" · ") : [];
  if (!groups.includes(example.group)) groups.push(example.group);
  item.group = groups.join(" · ");
  item.exampleIds.push(example.id);
  placementGroups.set(index, item);
}
const originals = [];
for (const line of spec.split(/\r?\n/)) {
  const row = line.match(/^\| (訳|事|物|所|居る|見る|付く|UD 列表) \| (.*?) \|$/);
  if (!row) continue;
  const [, source, text] = row;
  const fragments = text.split(/[；;]/).flatMap((part) => part.split(/\s*\/\s*/)).map((part) => part.trim()).filter(Boolean);
  for (const ja of fragments) originals.push({ id: `O${String(originals.length + 1).padStart(2, "0")}`, group: "原稿例", source, ja, kana: "", note: `${source}：原稿中的例句，表记与上下文保持原样。` });
}
const packageData = {
  lessonId: "grammar-1.1", title: "认识实词、虚词、助词", version: 1, status: "reviewed", sourceMarkdown,
  placements: [...placementGroups.values()].sort((left, right) => left.afterBlock - right.afterBlock),
  originalExamples: originals, examples,
  generatedFrom: { specification: "docs/grammar_guide_1_1.md", sourceText: "原稿原文代码块", generation: "scripts/build_grammar_guide_package.mjs", review: "依据例句插入关系、日文与中文对照逐条校读；词典快照和细分词法标注待后台审阅填写。" },
};
await writeFile(resolve(root, "src/grammar-guide/lesson-1-1.json"), `${JSON.stringify(packageData, null, 2)}\n`, "utf8");
console.log(`Generated ${examples.length} supplementary examples and ${originals.length} original capsules.`);
