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
const originalReadings = {
  "遅刻の訳を聞く": "ちこくのわけをきく", "飲むわけにはいかない": "のむわけにはいかない", "そういうわけで": "そういうわけで",
  "大事な事を忘れた": "だいじなことをわすれた", "泳ぐことができる": "およぐことができる", "行くことにする": "いくことにする",
  "甘い物を食べる": "あまいものをたべる", "昔はよく遊んだものだ": "むかしはよくあそんだものだ", "静かな所に住む": "しずかなところにすむ",
  "今食べているところだ": "いまたべているところだ", "部屋に居る": "へやにいる", "食べている": "たべている", "映画を見る": "えいがをみる",
  "食べてみる": "たべてみる", "席に着く": "せきにつく", "味がつく": "あじがつく", "これについて": "これについて",
  "部屋にいる": "へやにいる",
};
const originalTranslations = {
  "遅刻の訳を聞く": "询问迟到的理由。", "飲むわけにはいかない": "不能喝。", "そういうわけで": "因此。",
  "大事な事を忘れた": "忘了一件重要的事。", "泳ぐことができる": "会游泳。", "行くことにする": "决定去。",
  "甘い物を食べる": "吃甜食。", "昔はよく遊んだものだ": "过去常常玩，真令人怀念。", "静かな所に住む": "住在安静的地方。",
  "今食べているところだ": "现在正在吃。", "部屋に居る": "在房间里。", "食べている": "正在吃。", "映画を見る": "看电影。",
  "食べてみる": "试着吃。", "席に着く": "坐到座位上。", "味がつく": "汤有了味道。", "これについて": "关于这个。", "部屋にいる": "在房间里。",
};

const anchorMarkers = {
  P01: "**组成单元**", P02: "我将采取一个对汉语母语者来说", P03: "其中, 实词和虚词都是",
  P04: "其中, 实词和虚词都是", P05: "其中, 实词和虚词都是", P06: "其中, 实词和虚词都是",
  P07: "实词和虚词最明显的区别在于", P08: "[[虚词的来源和演变", P09: "而助词是标记性的",
  P10: "传统上可能把助词和虚词统称为", P11: "此外, 感动词", P12: "| **訳**",
};
const blockKind = (line) => /^\|/.test(line) ? "table" : /^\s*[-*] /.test(line) ? "list" : /^>/.test(line) ? "quote" : /^#{1,6} /.test(line) ? "heading" : "paragraph";
function markdownBlocks(markdown) {
  const blocks = [];
  let lines = [];
  let kind = "paragraph";
  for (const line of markdown.replace(/\r/g, "").split("\n")) {
    if (!line.trim()) {
      if (lines.length) blocks.push({ kind, source: lines.join("\n") });
      lines = [];
      continue;
    }
    const next = blockKind(line);
    if (lines.length && next !== kind) {
      blocks.push({ kind, source: lines.join("\n") });
      lines = [];
    }
    if (!lines.length) kind = next;
    lines.push(line);
  }
  if (lines.length) blocks.push({ kind, source: lines.join("\n") });
  return blocks;
}
const sourceBlocks = markdownBlocks(sourceMarkdown);
function uniqueSourceAnchor(marker) {
  const matches = sourceBlocks.filter((block) => block.source.includes(marker));
  if (matches.length !== 1) throw new Error(`原稿锚点「${marker}」匹配到 ${matches.length} 个正文块。`);
  return matches[0].source;
}
const groupTitles = {};
const markers = [
  ["。", "other", "标点"], ["！", "other", "标点"], ["？", "other", "标点"], ["、", "other", "标点"],
  ["然而", "other", "接续词"], ["しかし", "other", "接续词"], ["はい", "other", "感动词"], ["ああ", "other", "感动词"],
  ["について", "particle", "复合助词"], ["わけで", "functional", "形式名词接续表达"], ["ので", "functional", "接续表达"],
  ["この", "content", "连体词"], ["これ", "content", "代词"],
  ["ところ", "functional", "形式名词"], ["わけ", "functional", "形式名词"],
  ["こと", "functional", "形式名词"], ["もの", "functional", "形式名词"], ["だけ", "particle", "副助词"],
  ["ない", "functional", "否定助动词"], ["たい", "functional", "愿望助动词"], ["いる", "functional", "补助动词"],
  ["みる", "functional", "补助动词"], ["ね", "other", "终助词"], ["よ", "other", "终助词"],
  ["とても", "content", "程度副词"],
  ["て", "particle", "接续助词"], ["が", "particle", "格助词"], ["を", "particle", "格助词"],
  ["は", "particle", "系助词"], ["に", "particle", "格助词"], ["で", "particle", "格助词"],
  ["と", "particle", "格助词／并列助词"], ["の", "particle", "格助词"], ["も", "particle", "系助词"],
  ["から", "particle", "格助词"], ["た", "functional", "过去助动词"], ["だ", "functional", "判断／过去助动词"],
];
function splitParts(ja, id, note, source = "") {
  const chars = Array.from(ja);
  const parts = [];
  let cursor = 0;
  for (let index = 0; index < chars.length;) {
    const marker = markers.find(([text]) => {
      if (chars.slice(index, index + Array.from(text).length).join("") !== text) return false;
      if (text === "ない" && note.includes("つなぐ") && chars.slice(index + 2, index + 3).join("") === "で") return false;
      if (["はい", "ああ", "しかし"].includes(text) && index > 0 && !/[。！？、]/.test(chars[index - 1])) return false;
      return true;
    });
    if (!marker) { index += 1; continue; }
    if (index > cursor) parts.push({ start: cursor, end: index, teachingClass: "content", classLabel: "实词／词汇核心", term: "词汇成分" });
    const length = Array.from(marker[0]).length;
    let teachingClass = marker[1];
    let classLabel = marker[2];
    if (marker[0] === "で" && ["ん", "い", "ぎ", "び", "み"].includes(chars[index - 1])) classLabel = "接续助词";
    if (marker[0] === "だ" && ["ん", "っ"].includes(chars[index - 1])) classLabel = "过去助动词（浊音形式）";
    if (marker[0] === "いる" && (/存在/.test(note) || (source === "居る" && ja.startsWith("部屋")) || (source === "UD 列表" && ja === "部屋にいる"))) {
      teachingClass = "content";
      classLabel = "存在动词";
    }
    if (marker[0] === "みる" && (/视觉动词/.test(note) || (source === "見る" && ja.startsWith("映画")))) {
      teachingClass = "content";
      classLabel = "动词";
    }
    if (marker[0] === "と") classLabel = "并列助词";
    parts.push({ start: index, end: index + length, teachingClass, classLabel, term: classLabel });
    index += length;
    cursor = index;
  }
  if (cursor < chars.length) parts.push({ start: cursor, end: chars.length, teachingClass: "content", classLabel: "实词／词汇核心", term: "词汇成分" });
  if (!parts.length) parts.push({ start: 0, end: chars.length, teachingClass: "other", classLabel: "整句观察", term: "整句" });
  return parts.map((part, index) => {
    const text = chars.slice(part.start, part.end).join("");
    const partId = `${id}-part-${index + 1}`;
    return {
      id: partId, range: [part.start, part.end], text, teachingClass: part.teachingClass,
      classLabel: part.classLabel, term: part.term, focus: /[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}]/u.test(text) && note.includes(text), detailId: `${partId}-detail`,
      sourceNote: note,
    };
  });
}
const examples = [];
let group = "";
for (const line of spec.split(/\r?\n/)) {
  const heading = line.match(/^### (P\d\d) (.+)$/);
  if (heading) { group = heading[1]; groupTitles[group] = heading[2]; }
  const row = line.match(/^\| (E\d\d) \| (.*?) \| (.*?) \| (.*?) \|$/);
  if (!row) continue;
  const [, id, ja, zh, note] = row;
  const length = Array.from(ja).length;
  const kana = readings[id] ?? "";
  const parts = splitParts(ja, id, note);
  const morphology = parts.filter((part) => part.teachingClass === "functional").map((part) => ({ form: part.text, label: part.classLabel, note }));
  const details = Object.fromEntries(parts.map((part) => [part.detailId, { id: part.detailId, label: part.text, sourceLabel: "1.1 教学标注", teachingClass: part.classLabel, note, dictionaryEntryIds: [] }]));
  examples.push({
    id, group, ja, kana, zh, note, focus: parts.filter((part) => part.focus).map((part) => part.text),
    capsules: [{ id: `${id}-whole`, range: [0, length], partIds: parts.map((part) => part.id) }],
    parts: parts.map(({ sourceNote, ...part }) => part),
    ruby: [],
    targets: [
      { id: `${id}-target`, range: [0, length], detailId: `${id}-detail`, defaultEntryId: null, entryIds: [], childTargetIds: parts.map((part) => part.id) },
      ...parts.map((part) => ({ id: part.id, range: part.range, detailId: part.detailId, defaultEntryId: null, entryIds: [], childTargetIds: [] })),
    ],
    detail: { id: `${id}-detail`, label: "例句整体", sourceLabel: "原稿例句规格", teachingClass: "整句观察", note, morphology, dictionaryEntryIds: [] },
    details, dictionaryStatus: "no-record", dictionaryEntries: [], morphology,
  });
}
if (examples.length !== 59) throw new Error(`预期 59 条补充例句，实际读取到 ${examples.length} 条。`);

const placementGroups = new Map();
for (const example of examples) {
  const marker = anchorMarkers[example.group];
  if (!marker) throw new Error(`例句 ${example.id} 缺少插入锚点。`);
  const item = placementGroups.get(example.group) ?? { group: example.group, title: groupTitles[example.group], exampleIds: [], layout: example.group === "P01" ? "contrast" : "list", anchor: uniqueSourceAnchor(marker) };
  if (!item.anchor) throw new Error(`插入组 ${example.group} 的原文锚点缺失。`);
  item.exampleIds.push(example.id);
  placementGroups.set(example.group, item);
}
const originals = [];
const originalExamplesAnchor = sourceBlocks.find((block) => block.kind === "table" && block.source.includes("遅刻の**訳**を聞く"))?.source;
if (!originalExamplesAnchor) throw new Error("未找到原稿表格锚点。");
for (const line of spec.split(/\r?\n/)) {
  const row = line.match(/^\| (訳|事|物|所|居る|見る|付く|UD 列表) \| (.*?) \|$/);
  if (!row) continue;
  const [, source, text] = row;
  const fragments = text.split(/[；;]/).flatMap((part) => part.split(/\s*\/\s*/)).map((part) => part.trim()).filter(Boolean);
  for (const ja of fragments) {
    const id = `O${String(originals.length + 1).padStart(2, "0")}`;
    const contentUse = (source === "訳" && ja.startsWith("遅刻")) || (source === "事" && ja.startsWith("大事")) || (source === "物" && ja.startsWith("甘い")) || (source === "所" && ja.startsWith("静かな")) || (source === "居る" && ja.startsWith("部屋")) || (source === "見る" && ja.startsWith("映画")) || source === "付く" || (source === "UD 列表" && ja.startsWith("部屋"));
    const note = `${source}：原稿中的例句；${contentUse ? "本处保留词汇意义。" : "本处按原稿说明标注功能用法。"}${ja.includes("ない") ? "ない在本例用于否定。" : ""}`;
    const parts = splitParts(ja, id, note, source).map(({ sourceNote, ...part }) => part);
    const morphology = parts.filter((part) => part.teachingClass === "functional").map((part) => ({ form: part.text, label: part.classLabel, note }));
    const details = Object.fromEntries(parts.map((part) => [part.detailId, { id: part.detailId, label: part.text, sourceLabel: "原稿对照表", teachingClass: part.classLabel, note, dictionaryEntryIds: [] }]));
    originals.push({ id, group: "原稿例", source, ja, kana: originalReadings[ja] ?? "", zh: originalTranslations[ja] ?? "", focus: parts.filter((part) => part.focus).map((part) => part.text), note,
      capsules: [{ id: `${id}-whole`, range: [0, Array.from(ja).length], partIds: parts.map((part) => part.id) }], parts, ruby: [],
      targets: [{ id: `${id}-target`, range: [0, Array.from(ja).length], detailId: `${id}-detail`, defaultEntryId: null, entryIds: [], childTargetIds: parts.map((part) => part.id) }, ...parts.map((part) => ({ id: part.id, range: part.range, detailId: part.detailId, defaultEntryId: null, entryIds: [], childTargetIds: [] }))],
      detail: { id: `${id}-detail`, label: "原稿例句", sourceLabel: source, teachingClass: "整句观察", note, morphology, dictionaryEntryIds: [] }, details,
      dictionaryStatus: "no-record", dictionaryEntries: [], morphology });
  }
}
if (originals.length !== 19 || originals.some((item) => !item.kana)) throw new Error("原稿 19 个例句的读音或覆盖数量不完整。");
const packageData = {
  lessonId: "grammar-2.1", title: "认识实词、虚词、助词", version: 1, status: "reviewed", sourceMarkdown,
  placements: [...placementGroups.values()].sort((left, right) => left.afterBlock - right.afterBlock),
  originalExamplesAnchor, originalExamples: originals, examples,
  generatedFrom: { specification: "docs/grammar_guide_1_1.md", sourceText: "原稿原文代码块", generation: "scripts/build_grammar_guide_package.mjs", review: "例句、译文与观察说明逐条复核；成分标签依照展示设计的 5.1 映射生成，词典未命中保存为固定无记录状态。" },
};
await writeFile(resolve(root, "src/grammar-guide/lesson-2-1.json"), `${JSON.stringify(packageData, null, 2)}\n`, "utf8");
console.log(`Generated ${examples.length} supplementary examples and ${originals.length} original capsules.`);
