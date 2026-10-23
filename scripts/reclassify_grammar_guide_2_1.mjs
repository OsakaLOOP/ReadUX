import { readFile, rename, rm, writeFile } from "node:fs/promises";
import { execFileSync } from "node:child_process";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const guide = resolve(root, "src/grammar-guide");
const sourceStem = "lesson-1-1";
const targetStem = "lesson-2-1";
const targetPackage = resolve(guide, targetStem + ".json");
const sourceAnalysis = resolve(guide, sourceStem + "-analysis.json");
const targetAnalysis = resolve(guide, targetStem + "-analysis.json");
const sourceDictionary = resolve(guide, sourceStem + "-dictionary.json");
const targetDictionary = resolve(guide, targetStem + "-dictionary.json");

for (const path of [targetPackage, targetAnalysis, targetDictionary]) {
  try { await readFile(path); throw new Error("目标资源已存在，停止迁移：" + path); }
  catch (error) { if (error.code !== "ENOENT") throw error; }
}

const packageData = JSON.parse(execFileSync("git", ["show", "HEAD:src/grammar-guide/" + sourceStem + ".json"], { cwd: root, encoding: "utf8" }));
for (const example of packageData.examples) {
  if (!/^1\.1-\d{2}(?:-[a-z])?$/.test(example.id)) continue;
}
function reclassify(value) {
  if (typeof value === "string") return value.replace(/1\.1-(\d{2})(?=-[a-z]|\b)/g, "2.1-$1");
  if (Array.isArray(value)) return value.map(reclassify);
  if (value && typeof value === "object") {
    return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, reclassify(item)]));
  }
  return value;
}
const migrated = reclassify(packageData);
migrated.lessonId = "grammar-2.1";
migrated.sourceMarkdown = migrated.sourceMarkdown.replace("### 1.1 认识实词、虚词、助词", "### 2.1 认识实词、虚词、助词");
migrated.generatedFrom = {
  ...migrated.generatedFrom,
  reclassifiedFrom: "grammar-1.1",
  section: "2.1",
};
const expectedIds = new Set(migrated.examples.map((example) => example.id));
const placedIds = migrated.placements.flatMap((placement) => placement.exampleIds);
if (migrated.lessonId !== "grammar-2.1" || migrated.examples.length !== 70 || migrated.originalExamples.length !== 19
  || expectedIds.size !== 70 || placedIds.length !== 70 || placedIds.some((id) => !expectedIds.has(id))) {
  throw new Error("旧课节改号后的例句编号或插入关系校验失败。");
}
await writeFile(targetPackage, JSON.stringify(migrated, null, 2) + "\n", "utf8");
const analysis = JSON.parse(await readFile(sourceAnalysis, "utf8"));
analysis.source = targetStem + ".json";
analysis.examples = Object.fromEntries(Object.entries(analysis.examples).map(([id, snapshot]) => [
  id.replace(/^1\.1-/, "2.1-"), snapshot,
]));
if (Object.keys(analysis.examples).length !== 89 || Object.keys(analysis.examples).some((id) => id.startsWith("1.1-"))) {
  throw new Error("旧课节分析快照的例句编号迁移失败。");
}
await writeFile(targetAnalysis, JSON.stringify(analysis) + "\n", "utf8");
await rm(sourceAnalysis);
await rename(sourceDictionary, targetDictionary);
console.log("Reclassified the existing guide package as grammar-2.1 with 70 examples and 19 original examples.");
