import assert from "node:assert/strict";
import { mkdir, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || process.argv[2] || "playwright");
const browser = await chromium.launch({ channel: "msedge", headless: true });
const page = await browser.newPage();
const errors = [];
page.on("pageerror", (error) => errors.push(error.message));
page.on("console", (message) => {
  if (["error", "warning"].includes(message.type())) errors.push(message.text());
});
try {
  await page.goto(process.env.READUX_TEST_URL || "http://127.0.0.1:1422/", { waitUntil: "networkidle" });
  const original = await page.locator('link[rel="icon"]').getAttribute("href");
  await page.getByRole("button", { name: "打开书籍", exact: true }).click();
  await page.waitForFunction(() => document.querySelector('link[rel="icon"]').href.startsWith("data:image/png"));
  const png = await page.locator('link[rel="icon"]').getAttribute("href");
  await mkdir(".agents/header-animation", { recursive: true });
  await writeFile(".agents/header-animation/reading-taskbar.png", Buffer.from(png.split(",")[1], "base64"));

  const result = await page.evaluate(async () => {
    const { renderReadingIcon } = await import("/src/reader/readingIcon.ts");
    const { updateTaskbarIcon } = await import("/src/services/taskbarIcon.ts");
    const svg = document.querySelector(".reading-mark svg");
    const first = await renderReadingIcon(svg);
    const clone = svg.cloneNode(true);
    const cover = document.createElement("canvas");
    cover.width = 32;
    cover.height = 48;
    const context = cover.getContext("2d");
    context.fillStyle = "#ef258a";
    context.fillRect(0, 0, 32, 48);
    clone.querySelector("image").setAttribute("href", cover.toDataURL());
    const second = await renderReadingIcon(clone);
    let changed = 0;
    let visible = 0;
    let transparent = 0;
    for (let index = 0; index < first.rgba.length; index += 4) {
      if (first.rgba[index + 3]) visible++;
      else transparent++;
      if (first.rgba[index] !== second.rgba[index] || first.rgba[index + 1] !== second.rgba[index + 1]) changed++;
    }

    // 检查桌面 IPC 的完整像素数据，以及快速返回时迟到结果的处理。
    const calls = [];
    window.isTauri = true;
    window.__TAURI_INTERNALS__ = { invoke: async (command, payload) => { calls.push({ command, payload }); } };
    await updateTaskbarIcon(svg);
    await updateTaskbarIcon(null);
    await Promise.all([updateTaskbarIcon(svg), updateTaskbarIcon(null)]);
    window.isTauri = false;
    delete window.__TAURI_INTERNALS__;
    return { changed, visible, transparent, size: first.size, calls };
  });
  assert.equal(result.size, 256);
  assert.ok(result.visible > 10000 && result.transparent > 10000, "图标应有完整图形与透明背景");
  assert.ok(result.changed > 1000, "封面变化应反映在任务栏图标中");
  assert.equal(result.calls[0].command, "set_reading_taskbar_icon");
  assert.equal(result.calls[0].payload.rgba.length, 256 * 256 * 4);
  assert.equal(result.calls.at(-1).payload.rgba, null, "快速返回后应保持默认图标");
  assert.equal(result.calls.length, 3, "迟到的阅读图标应取消更新");
  await page.getByRole("button", { name: "关闭书籍", exact: true }).click();
  await page.waitForFunction((href) => document.querySelector('link[rel="icon"]').getAttribute("href") === href, original);
  assert.deepEqual(errors, []);
  console.log("阅读 SVG 最终态、封面变化、透明像素、桌面 IPC 及默认图标恢复验证通过。");
} finally {
  await browser.close();
}
