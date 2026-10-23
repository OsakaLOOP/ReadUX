import { readFile, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const sourcePath = process.argv[2];
if (!sourcePath) throw new Error("用法：node scripts/import_grammar_guide_source.mjs <源 MDX 路径>");
const sourceFile = resolve(sourcePath);
const source = await readFile(sourceFile, "utf8");

function readBalanced(sourceText, openIndex) {
  let depth = 0;
  let quote = "";
  let escaped = false;
  for (let index = openIndex; index < sourceText.length; index += 1) {
    const character = sourceText[index];
    if (quote) {
      if (escaped) escaped = false;
      else if (character === "\\") escaped = true;
      else if (character === quote) quote = "";
      continue;
    }
    if (character === "\"" || character === "'") { quote = character; continue; }
    if (character === "{") depth += 1;
    if (character === "}" && --depth === 0) return index;
  }
  throw new Error("GuideExampleList 的 items 表达式未闭合。");
}

function parseLiteral(text) {
  let cursor = 0;
  const fail = (message) => { throw new Error("items 表达式第 " + (cursor + 1) + " 列：" + message); };
  const whitespace = () => { while (/\s/.test(text[cursor] ?? "")) cursor += 1; };
  function string() {
    const delimiter = text[cursor++];
    let value = "";
    while (cursor < text.length) {
      const character = text[cursor++];
      if (character === delimiter) return value;
      if (character !== "\\") { value += character; continue; }
      const escaped = text[cursor++];
      const replacements = { n: "\n", r: "\r", t: "\t", b: "\b", f: "\f", v: "\v", "0": "\0" };
      if (escaped in replacements) value += replacements[escaped];
      else if (escaped === "u") {
        const digits = text.slice(cursor, cursor + 4);
        if (!/^[0-9a-fA-F]{4}$/.test(digits)) fail("Unicode 转义无效。");
        value += String.fromCharCode(Number.parseInt(digits, 16));
        cursor += 4;
      } else if (escaped === "x") {
        const digits = text.slice(cursor, cursor + 2);
        if (!/^[0-9a-fA-F]{2}$/.test(digits)) fail("十六进制转义无效。");
        value += String.fromCharCode(Number.parseInt(digits, 16));
        cursor += 2;
      } else value += escaped;
    }
    fail("字符串未闭合。");
  }
  function identifier() {
    const start = cursor;
    while (/[\w$]/.test(text[cursor] ?? "")) cursor += 1;
    if (cursor === start) fail("预期属性名。");
    return text.slice(start, cursor);
  }
  function value() {
    whitespace();
    const character = text[cursor];
    if (character === "\"" || character === "'") return string();
    if (character === "[") {
      cursor += 1;
      const items = [];
      whitespace();
      while (text[cursor] !== "]") {
        items.push(value());
        whitespace();
        if (text[cursor] === "]") break;
        if (text[cursor++] !== ",") fail("数组项之间缺少逗号。");
        whitespace();
      }
      if (text[cursor++] !== "]") fail("数组未闭合。");
      return items;
    }
    if (character === "{") {
      cursor += 1;
      const object = {};
      whitespace();
      while (text[cursor] !== "}") {
        const key = text[cursor] === "\"" || text[cursor] === "'" ? string() : identifier();
        whitespace();
        if (text[cursor++] !== ":") fail("属性名后缺少冒号。");
        object[key] = value();
        whitespace();
        if (text[cursor] === "}") break;
        if (text[cursor++] !== ",") fail("对象属性之间缺少逗号。");
        whitespace();
      }
      if (text[cursor++] !== "}") fail("对象未闭合。");
      return object;
    }
    const token = identifier();
    if (token === "true") return true;
    if (token === "false") return false;
    if (token === "null") return null;
    fail("不支持的值 " + token + "。");
  }
  const result = value();
  whitespace();
  if (cursor !== text.length) fail("表达式中含有未处理内容。");
  return result;
}

function markdownBlocks(markdown) {
  const blocks = [];
  let lines = [];
  let kind = "paragraph";
  const flush = () => {
    if (lines.length) blocks.push({ kind, source: lines.join("\n") });
    lines = [];
  };
  for (const line of markdown.replace(/\r/g, "").split("\n")) {
    if (!line.trim()) { flush(); continue; }
    const next = /^\|/.test(line) ? "table"
      : /^\s*[-*] /.test(line) ? "list"
        : /^\s*\d+[.)] /.test(line) ? "ordered-list"
          : /^>/.test(line) ? "quote"
            : /^#{1,6} /.test(line) ? "heading" : "paragraph";
    if (lines.length && next !== kind) flush();
    if (!lines.length) kind = next;
    lines.push(line);
  }
  flush();
  return blocks;
}

function normalizeJapanese(sourceJa) {
  const pattern = /([\u3400-\u9fff々〆ヵヶ]+)\[([^\]]+)\]/gu;
  const ruby = [];
  let plain = "";
  let cursor = 0;
  for (const match of sourceJa.matchAll(pattern)) {
    plain += sourceJa.slice(cursor, match.index) + match[1];
    const end = Array.from(plain).length;
    const start = end - Array.from(match[1]).length;
    ruby.push({ range: [start, end], base: match[1], reading: match[2] });
    cursor = match.index + match[0].length;
  }
  plain += sourceJa.slice(cursor);
  if (/[\[\]]/.test(plain)) throw new Error("例句含有无法识别的假名注音：" + sourceJa);
  return { ja: plain, ruby };
}

const frontMatter = source.match(/^---\r?\n[\s\S]*?\r?\n---\r?\n/);
if (!frontMatter) throw new Error("源 MDX 缺少 YAML front matter。");
const body = source.slice(frontMatter[0].length).replace(/^import\s+GuideExampleList\s+from\s+["'][^"']+["'];\s*\r?\n/m, "");
const components = [];
const componentStart = /<GuideExampleList\b/g;
let match;
while ((match = componentStart.exec(body))) {
  const attributeStart = body.indexOf("items={", match.index);
  if (attributeStart < 0) throw new Error("第 " + match.index + " 个 GuideExampleList 缺少 items 属性。");
  const openIndex = attributeStart + "items=".length;
  const closeIndex = readBalanced(body, openIndex);
  const closingTag = body.indexOf("/>", closeIndex + 1);
  if (closingTag < 0 || body.slice(closeIndex + 1, closingTag).trim()) throw new Error("GuideExampleList 必须使用自闭合写法。");
  const items = parseLiteral(body.slice(openIndex + 1, closeIndex));
  if (!Array.isArray(items) || items.some((item) => !item || typeof item !== "object")) throw new Error("GuideExampleList 的 items 必须是对象数组。");
  components.push({ start: match.index, end: closingTag + 2, items });
  componentStart.lastIndex = closingTag + 2;
}
if (!components.length) throw new Error("源 MDX 中没有 GuideExampleList。");

const examples = [];
const placements = [];
const sourceParts = [];
let sourceCursor = 0;
for (const component of components) {
  sourceParts.push(body.slice(sourceCursor, component.start));
  const markdownSoFar = sourceParts.join("");
  const blocks = markdownBlocks(markdownSoFar);
  if (!blocks.length) throw new Error("例句组前没有可用的 Markdown 段落锚点。");
  const group = "G" + String(placements.length + 1).padStart(2, "0");
  const items = [];
  const exampleIds = [];
  for (const [index, item] of component.items.entries()) {
    if (item.type === "explanation") {
      if (typeof item.text !== "string" || !item.text.trim()) throw new Error(group + " 的讲解项内容为空。");
      items.push({ type: "explanation", id: group + "-explanation-" + String(index + 1).padStart(2, "0"), text: item.text });
      continue;
    }
    if (item.type !== "example" || typeof item.id !== "string" || typeof item.ja !== "string" || typeof item.zh !== "string" || typeof item.note !== "string") {
      throw new Error(group + " 第 " + (index + 1) + " 项不符合源例句结构。");
    }
    const normalized = normalizeJapanese(item.ja);
    const example = {
      id: item.id,
      group,
      ja: normalized.ja,
      sourceJa: item.ja,
      kana: "",
      zh: item.zh,
      note: item.note,
      focus: [],
      rubySource: "author",
      ruby: normalized.ruby,
      ...(typeof item.source === "string" ? { source: item.source } : {}),
    };
    examples.push(example);
    exampleIds.push(item.id);
    items.push({ type: "example", id: item.id });
  }
  if (!exampleIds.length) throw new Error(group + " 没有例句项。");
  placements.push({ group, title: "例句组 " + (placements.length + 1), exampleIds, layout: "list", anchor: blocks.at(-1).source, items });
  sourceCursor = component.end;
}
sourceParts.push(body.slice(sourceCursor));
const sourceMarkdown = sourceParts.join("");
const sourceBlocks = markdownBlocks(sourceMarkdown).map((block) => block.source);
if (new Set(examples.map((example) => example.id)).size !== examples.length) throw new Error("源 MDX 中存在重复例句编号。");
if (placements.some((placement) => !sourceBlocks.includes(placement.anchor))) throw new Error("例句组锚点没有对应的正文块。");
if (examples.length !== 43 || placements.length !== 16) throw new Error("源稿数量异常：" + examples.length + " 条例句、" + placements.length + " 组。");

const packageData = {
  lessonId: "grammar-1.1",
  title: "句法入门",
  version: 1,
  status: "reviewed",
  sourceMarkdown,
  placements,
  originalExamplesAnchor: "",
  originalExamples: [],
  examples,
  generatedFrom: {
    source: sourceFile.endsWith("jpn-grammar-01.mdx") ? "astro-paper/src/content/posts/jpn-grammar-01.mdx" : sourceFile,
    sourceSha256: createHash("sha256").update(source).digest("hex"),
    sections: "1.1–1.3",
    exampleCount: String(examples.length),
    groupCount: String(placements.length),
    generation: "scripts/import_grammar_guide_source.mjs",
  },
};
const output = resolve(root, "src/grammar-guide/lesson-1-1.json");
await writeFile(output, JSON.stringify(packageData, null, 2) + "\n", "utf8");
console.log("Imported " + examples.length + " examples in " + placements.length + " source groups to " + output);
