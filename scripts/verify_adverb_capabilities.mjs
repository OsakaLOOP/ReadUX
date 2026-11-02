/** 使用 Rust 的真实分词验收产物核对查询与接续树投影。 */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { morphologyConnectionDiagram, morphologyEntry, morphologyStemDiagram } from "../src/reader/morphologyBubble.ts";
import { readerMorphologyHits, readerTargetPresentation } from "../src/reader/lookupPresentation.ts";
import { layoutConnectionRules, layoutSentenceChain } from "../src/reader/morphologyGraph.ts";

const path = process.argv[2] || ".agents/analysis/adverbs/acceptance.json";
const documents = JSON.parse(readFileSync(path, "utf8"));
let operators = 0;
for (const document of documents) {
  const targets = document.morphology.chains.filter((chain) => !chain.parent_chain_id && chain.role === "lexical")
    .map((chain) => ({ id: chain.chain_id, char_range: chain.char_range, surface: chain.surface_form,
      lexical_core_ids: [chain.chain_id], source_formation_ids: [],
      morpheme_ids: chain.morpheme_indices.map((index) => document.morphemes[index].id), lookup_forms: chain.query_forms }));
  const hits = readerMorphologyHits(targets, document);
  for (const chain of document.morphology.chains) {
    for (const operator of chain.operators.filter((op) => op.executor === "adverbial_predicate")) {
      operators++;
      const entry = morphologyEntry(operator, document);
      assert.ok(entry.summary && entry.examples.length, `${document.text}: 活用词条正文与例句`);
      assert.equal(entry.headword, operator.kind === "adverb_to" ? "と" : "する");
      if (operator.kind !== "adverb_to") assert.notEqual(entry.conjugation, "无活用");
      const diagram = morphologyConnectionDiagram(operator, chain, document, chain.status);
      assert.equal(diagram.diagnostics.length, 0);
      assert.ok(diagram.branches.some((branch) => branch.matched && branch.groups.some(
        (group) => group.paths.some((item) => item.matched))), `${document.text}: 接续树命中`);
      const tree = layoutConnectionRules(diagram, null);
      const sentence = layoutSentenceChain(diagram.sentence);
      for (const node of [...tree.nodes, ...sentence.nodes]) {
        assert.ok(Number.isFinite(node.x) && Number.isFinite(node.y) && node.width > 0 && node.height > 0);
      }
      assert.ok(hits.some((hit) => hit.detail.id === `occurrence:${operator.operator_id}`), `${document.text}: 活用查询入口`);
      if (operator.kind === "adverb_to_suru") {
        assert.ok(operator.connection.connector_operator_id);
        assert.ok(diagram.branches.some((branch) => branch.label === "と"));
      }
    }
    const token = document.morphemes[chain.anchor_morpheme];
    if (token.pos[0] !== "副詞" || chain.adverb_capabilities == null) continue;
    const diagram = morphologyStemDiagram(chain, document, chain.anchor_range);
    const stem = diagram.sentence.find((part) => part.unit === `${chain.chain_id}:core`);
    if ((chain.adverb_capabilities & 30) !== 0) {
      assert.ok(stem && stem.form !== "无活用", `${document.text}: 可活用词干`);
      const target = targets.find((item) => item.id === chain.chain_id);
      if (target) assert.match(readerTargetPresentation(target, document).inflectionCapability, /^可活用/);
    } else if (stem) {
      assert.equal(stem.form, "无活用", `${document.text}: 不可活用词干`);
    }
  }
}
assert.ok(operators >= 5, "验收应覆盖三个新增规则");
console.log(`${documents.length} 条真实分词记录、${operators} 个新增接续的词条、查询入口与图示投影通过。`);
