import { readFile } from "node:fs/promises";
import { gzipSync } from "node:zlib";

export function grammarGuideAssets() {
  return {
    name: "readux-grammar-guide-assets",
    enforce: "pre",
    apply: "build",
    async load(id) {
      const match = id.match(/[/\\]grammar-guide[/\\](lesson-[^/\\]+-(?:analysis|dictionary)\.json)\?url$/);
      if (!match) return;
      const filename = id.slice(0, -4);
      this.addWatchFile(filename);
      const reference = this.emitFile({
        type: "asset",
        name: `${match[1]}.gz`,
        source: gzipSync(await readFile(filename), { level: 9 }),
      });
      return `export default import.meta.ROLLUP_FILE_URL_${reference};`;
    },
  };
}
