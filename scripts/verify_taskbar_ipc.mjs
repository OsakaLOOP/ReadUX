import { readFile } from "node:fs/promises";
import { createRequire } from "node:module";

const [endpoint, iconPath, modulePath, action] = process.argv.slice(2);
const require = createRequire(import.meta.url);
const { chromium } = require(modulePath || process.env.PLAYWRIGHT_MODULE || "playwright");
const browser = await chromium.connectOverCDP(endpoint);
try {
  const page = browser.contexts().flatMap((context) => context.pages())[0];
  await page.waitForFunction(() => location.origin !== "null" && window.__TAURI_INTERNALS__?.invoke && document.readyState !== "loading");
  console.log(`窗口地址：${page.url()}`);
  const png = action === "restore" ? null : `data:image/png;base64,${(await readFile(iconPath)).toString("base64")}`;
  const result = await page.evaluate(async (src) => {
    const load = (src) => new Promise((resolve, reject) => {
      const image = new Image();
      image.crossOrigin = "anonymous";
      image.onload = () => resolve(image);
      image.onerror = () => reject(new Error("窗口中的图标或封面加载失败"));
      image.src = src;
    });
    const canvas = document.createElement("canvas");
    canvas.width = canvas.height = 256;
    const context = canvas.getContext("2d");
    let rgba = null;
    if (src) {
      context.drawImage(await load(src), 0, 0, 256, 256);
      rgba = Array.from(context.getImageData(0, 0, 256, 256).data);
    }
    await window.__TAURI_INTERNALS__.invoke("set_reading_taskbar_icon", { rgba, size: 256 });
    const result = { reading: Boolean(src), pixelBytes: rgba?.length ?? 0 };
    if (src) {
      const library = await window.__TAURI_INTERNALS__.invoke("reader_library");
      const cover = library.books.find((book) => book.coverPath);
      if (cover) {
        const url = window.__TAURI_INTERNALS__.convertFileSrc(cover.coverPath, "asset");
        context.clearRect(0, 0, 256, 256);
        context.drawImage(await load(url), 0, 0, 256, 256);
        result.coverPixelsReadable = context.getImageData(0, 0, 1, 1).data.length === 4;
      }
    }
    return result;
  }, png);
  console.log(JSON.stringify(result));
} finally {
  await browser.close();
}
