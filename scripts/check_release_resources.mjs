import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const configPath = resolve(root, "src-tauri/tauri.conf.json");
const config = JSON.parse(readFileSync(configPath, "utf8"));
const requireGenerated = process.argv.includes("--require-generated");
const requiredBundles = [
  "data/dict-sources/starter.kdict",
  "data/dict-sources/daijirin.kdict",
  "data/dict-sources/shogakukan.kdict",
  "data/dict-sources/crown.kdict",
];
const resources = config.bundle?.resources ?? [];
const resourceSources = Array.isArray(resources) ? resources : Object.keys(resources);

for (const relativePath of requiredBundles) {
  if (!existsSync(resolve(root, relativePath))) throw new Error(`发行资源缺失：${relativePath}`);
  try {
    execFileSync("git", ["-C", root, "ls-files", "--error-unmatch", "--", relativePath], { stdio: "ignore" });
  } catch {
    throw new Error(`发行资源未被 Git 跟踪：${relativePath}`);
  }
}

const dictionaryResources = resourceSources.filter((resource) => resource.includes("../data/dict-sources/") && /\.dic$/i.test(resource));
if (dictionaryResources.length > 0) throw new Error(`词典源资源不应使用 .dic：${dictionaryResources.join(", ")}`);
if (requireGenerated) {
  for (const relativePath of ["release-resources/nlp/cwj.dic", "release-resources/nlp/csj.dic", "release-resources/python", "release-resources/python-manifest.json"]) {
    if (!existsSync(resolve(root, relativePath))) throw new Error(`下载生成的发行依赖缺失：${relativePath}`);
  }
  if (existsSync(resolve(root, "release-resources/python/pyvenv.cfg"))) {
    throw new Error("安装包 Python 必须采用独立解释器，当前资源仍为虚拟环境。");
  }
  if (!["python.exe", "bin/python3.11", "bin/python3"].some((path) => existsSync(resolve(root, "release-resources/python", path)))) {
    throw new Error("安装包缺少独立 Python 解释器。");
  }
}
const version = JSON.parse(readFileSync(resolve(root, "package.json"), "utf8")).version;
if (process.env.GITHUB_REF?.startsWith("refs/tags/v") && process.env.GITHUB_REF !== `refs/tags/v${version}`) {
  throw new Error(`发布标签与安装包版本 ${version} 不一致。`);
}
console.log(`已确认 ${requiredBundles.length} 个 .kdict 资源存在且已被 Git 跟踪。`);
