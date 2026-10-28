import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { morphologyConnectionDiagram, morphologyEntry, morphologyLocalTransition } from "./morphologyBubble.ts";
import { layoutConnectionRules, layoutSentenceChain } from "./morphologyGraph.ts";

const snapshots = JSON.parse(readFileSync(new URL("../grammar-guide/lesson-stage11-analysis.json", import.meta.url), "utf8")).examples;

function diagramFor(id, kind) {
  const document = snapshots[id].document;
  const owner = document.morphology.chains.find((chain) => chain.operators.some((operator) => operator.kind === kind));
  const operator = owner.operators.find((operator) => operator.kind === kind);
  return morphologyConnectionDiagram(operator, owner, document, owner.status);
}

test("词干保留辞典形到实际词形，后接成分采用局部变化", () => {
  assert.equal(morphologyLocalTransition("高い → 高かっ（い→かっ）", true), "高い → 高かっ");
  assert.equal(morphologyLocalTransition("読まない → 読まなかった（ない→なかった）"), "ない → なかった");
});

test("缩约节点和词条变化保留连接器、清浊及实际活用", () => {
  for (const [id, kind, expanded, surface] of [
    ["M-complex-001", "contracted_te_shimau", "てしまう", "ちゃう"],
    ["M-regression-134", "contracted_te_shimau", "でしまう", "じゃっ"],
    ["M-regression-099", "contracted_te_iru", "ている", "てる"],
    ["M-regression-162", "contracted_te_oku", "でおく", "どく"],
  ]) {
    const diagram = diagramFor(id, kind);
    const part = diagram.sentence.find((part) => part.selected);
    assert.equal(part.caption, expanded);
    assert.equal(part.surface, surface);
    assert.ok(part.form.includes("缩略形式"));
    const node = layoutSentenceChain(diagram.sentence).nodes.find((node) => node.matched);
    assert.equal(node.lines[0].text, `补助 ${expanded} → ${surface}`);
    const document = snapshots[id].document;
    const operator = document.morphology.chains.flatMap((chain) => chain.operators).find((op) => op.kind === kind);
    assert.equal(morphologyEntry(operator, document).transition, `${expanded} → ${surface}`);
  }
});

test("名词性变化经实际连接器进入なる，形容词连用形直接接なる", () => {
  const connectorRules = new Set();
  let directCount = 0;
  for (const { document } of Object.values(snapshots)) {
    for (const owner of document.morphology.chains) {
      for (const operator of owner.operators.filter((op) => ["ni_naru", "adjectival_naru"].includes(op.kind))) {
        const diagram = morphologyConnectionDiagram(operator, owner, document, owner.status);
        assert.equal(diagram.target, "なる");
        const layout = layoutConnectionRules(diagram, null);
        if (operator.kind === "ni_naru") {
          connectorRules.add(operator.connection.rule);
          assert.deepEqual(diagram.branches.map((branch) => branch.label), ["に"]);
          const connector = layout.nodes.find((node) => node.kind === "connector");
          assert.equal(connector.lines[0].text, "に");
          assert.ok(connector.matched);
          assert.ok(layout.links.some((link) => link.source === connector.id && link.target === "target" && link.matched));
          for (const group of layout.nodes.filter((node) => node.kind === "group")) {
            assert.ok(layout.links.some((link) => link.source === group.id && link.target === connector.id));
          }
          const current = diagram.branches.flatMap((branch) => branch.groups).flatMap((group) => group.paths).find((path) => path.matched);
          assert.equal(current.actual, operator.connection.predecessor_surface);
        } else {
          directCount++;
          assert.deepEqual(diagram.branches.map((branch) => branch.label), [""]);
          assert.ok(layout.nodes.every((node) => node.kind !== "connector"));
          assert.ok(layout.links.some((link) => link.target === "target" && link.matched));
        }
      }
    }
  }
  assert.deepEqual([...connectorRules].sort(), ["judgement_adverbial", "nominal_change"]);
  assert.ok(directCount > 0);
});

test("分词与单体とか均显示一个完整插入单元", () => {
  for (const id of ["M-complex-010", "M-complex-011", "M-regression-034"]) {
    const document = snapshots[id].document;
    for (const owner of document.morphology.chains) {
      for (const operator of owner.operators.filter((op) => op.kind === "exemplification")) {
        const diagram = morphologyConnectionDiagram(operator, owner, document, owner.status);
        const inserted = diagram.sentence.filter((part) => part.unit === operator.operator_id);
        assert.equal(inserted.length, 1);
        assert.equal(inserted[0].surface, "とか");
        assert.equal(inserted[0].caption, "とか");
        assert.equal(inserted[0].form, "无活用");
        assert.deepEqual(inserted[0].range, operator.char_range);
        const single = structuredClone(document);
        const [start, end] = operator.source_morpheme_range;
        single.morphemes[start].surface = "とか";
        single.morphemes[start].char_range = operator.char_range;
        single.morphemes.splice(start + 1, end - start - 1);
        for (const chain of single.morphology.chains) {
          const shift = (index) => index > start ? index - (end - start - 1) : index;
          chain.morpheme_range = chain.morpheme_range.map(shift);
          chain.anchor_morpheme = shift(chain.anchor_morpheme);
          for (const op of chain.operators) op.source_morpheme_range = op.source_morpheme_range.map(shift);
        }
        const singleOwner = single.morphology.chains.find((chain) => chain.chain_id === owner.chain_id);
        const singleOperator = singleOwner.operators.find((op) => op.operator_id === operator.operator_id);
        const singleDiagram = morphologyConnectionDiagram(singleOperator, singleOwner, single, singleOwner.status);
        const content = (parts) => parts.map(({ id, ...part }) => part);
        assert.deepEqual(content(singleDiagram.sentence), content(diagram.sentence));
        const ordered = [...diagram.sentence].sort((a, b) => a.range[0] - b.range[0]);
        for (let index = 1; index < ordered.length; index++) assert.equal(ordered[index - 1].range[1], ordered[index].range[0]);
      }
    }
  }
});

test("词内意向形采用词汇辞典形，独立词尾采用自身原型", () => {
  const diagram = diagramFor("M-complex-003", "volitional");
  const part = diagram.sentence.find((part) => part.selected);
  assert.equal(part.caption, "作る");
  assert.equal(part.surface, "作ろう");
  assert.deepEqual(part.range, [3, 6]);
  const node = layoutSentenceChain(diagram.sentence).nodes.find((node) => node.matched);
  assert.equal(node.lines[0].text, "意向形 作る → 作ろう");
  const document = structuredClone(snapshots["M-complex-003"].document);
  const owner = document.morphology.chains.find((chain) => chain.operators.some((op) => op.kind === "volitional"));
  const operator = owner.operators.find((op) => op.kind === "volitional");
  const [start] = operator.source_morpheme_range;
  // 用相同的句中范围验证词干与独立词尾分词时的展示身份。
  document.morphemes.splice(start, 1,
    { ...document.morphemes[start], surface: "作ろ", char_range: [3, 5] },
    { ...document.morphemes[start], surface: "う", lemma: "う", pos: ["助動詞"], char_range: [5, 6] });
  for (const chain of document.morphology.chains) {
    const shift = (index) => index > start ? index + 1 : index;
    chain.morpheme_range = chain.morpheme_range.map(shift);
    chain.anchor_morpheme = shift(chain.anchor_morpheme);
    for (const op of chain.operators) op.source_morpheme_range = op.source_morpheme_range.map(shift);
  }
  operator.source_morpheme_range = [start + 1, start + 2];
  operator.char_range = [5, 6];
  const split = morphologyConnectionDiagram(operator, owner, document, owner.status);
  assert.equal(split.sentence.find((part) => part.selected).caption, "う");
  assert.equal(split.sentence.map((part) => part.surface).join(""), diagram.sentence.map((part) => part.surface).join(""));
});

test("实际递归链的节点内部固定两行，每次折行后从左向右连接", () => {
  const diagram = diagramFor("M11-39", "te_iru");
  assert.equal(diagram.sentence.map((part) => part.surface).join(""), "読んでしまっていた");
  const layout = layoutSentenceChain(diagram.sentence);
  const nodes = new Map(layout.nodes.map((node) => [node.id, node]));
  for (const node of layout.nodes) assert.equal(node.lines.length, 2);
  for (const link of layout.links) {
    const source = nodes.get(link.source);
    const target = nodes.get(link.target);
    if (link.vertical) {
      assert.equal(source.x + source.width / 2, target.x + target.width / 2);
      assert.ok(target.y > source.y);
    } else {
      assert.ok(target.x >= source.x + source.width);
    }
  }
  const first = layout.nodes[0].lines[0];
  assert.equal(first.runs.find((run) => run.role === "form").lang, "ja");
  const support = layout.nodes.find((node) => node.lines[0].text.includes("补助"));
  assert.equal(support.lines[0].runs[0].lang, "zh-CN");
  assert.equal(support.lines[0].text, "补助 しまう → しまっ");
  assert.equal(first.text, "読む → 読ん");
});

test("前接只连接词性节点，未显示的单条条件可展开，完整显示的当前路径无重复入口", () => {
  const diagram = diagramFor("M11-39", "te_iru");
  const initial = layoutConnectionRules(diagram, null);
  assert.equal(new Set(initial.nodes.filter((node) => node.kind === 'group').map((node) => node.width)).size, 1);
  for (const link of initial.links) {
    assert.ok(!link.source.startsWith("path:"));
    assert.ok(!link.target.startsWith("path:"));
  }
  const groups = diagram.branches.flatMap((branch) => branch.groups);
  for (const group of groups) {
    const node = initial.nodes.find((node) => node.id === `group:${group.id}`);
    assert.equal(Boolean(node.action), group.paths.some((path) => !path.matched));
  }
  const multi = initial.nodes.filter((node) => node.action);
  for (const node of multi) {
    const expanded = layoutConnectionRules(diagram, node.action.key);
    assert.equal(expanded.width, initial.width);
    assert.equal(expanded.nodes.filter((node) => node.action?.expanded).length, 1);
    assert.ok(expanded.nodes.every((node) => !node.action || node.action.count >= 1));
    const current = expanded.nodes.flatMap((node) => node.list ?? []).filter((row) => row.matched);
    assert.equal(current.length, 1);
  }
});

test("前接图各父节点按子节点范围自然居中，末个分类只计算标题", () => {
  const path = (id, matched) => ({ id, label: id, condition: `${id}接续条件`, matched, actual: "" });
  const diagram = {
    target: "ません",
    branches: [
      { id: "te", label: "て", matched: true, groups: [
        { id: "verbs", label: "动词", paths: [path("当前路径", true), path("展开路径", false)] },
        { id: "adjectives", label: "形容词", paths: [path("形容词路径", true)] },
      ] },
      { id: "de", label: "で", matched: false, groups: [
        { id: "nouns", label: "名词", paths: [path("名词路径", true)] },
      ] },
    ],
  };
  const layout = layoutConnectionRules(diagram, "verbs");
  const categories = layout.nodes.filter((node) => node.kind === "group").sort((a, b) => a.y - b.y);
  assert.ok(categories[0].height > 28);
  for (const parent of layout.nodes.filter((node) => node.kind !== "group")) {
    const children = layout.links.filter((link) => link.target === parent.id)
      .map((link) => layout.nodes.find((node) => node.id === link.source)).sort((a, b) => a.y - b.y);
    const first = children[0];
    const last = children.at(-1);
    const bottom = last.y + (last.kind === "group" ? 28 : last.height);
    assert.equal(parent.y + parent.height / 2, (first.y + bottom) / 2);
  }
});
