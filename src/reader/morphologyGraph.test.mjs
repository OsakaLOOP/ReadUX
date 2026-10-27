import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { morphologyConnectionDiagram, morphologyLocalTransition } from "./morphologyBubble.ts";
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
