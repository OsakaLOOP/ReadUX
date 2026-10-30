/** 使用正式阅读器解析 Markdown，供静态书库生成器准备正文。 */
import { build } from 'esbuild';

const bundle = await build({
  entryPoints: ['src/reader/document.ts'], bundle: true, write: false,
  platform: 'node', format: 'esm',
});
const { compileReaderDocument } = await import('data:text/javascript;base64,' + Buffer.from(bundle.outputFiles[0].text).toString('base64'));
let input = '';
process.stdin.setEncoding('utf8');
for await (const part of process.stdin) input += part;
const document = compileReaderDocument(JSON.parse(input));
process.stdout.write(JSON.stringify({ text: document.analysisText, chapters: document.chapters }));
