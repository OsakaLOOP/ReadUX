import assert from "node:assert/strict";
import { mkdir } from "node:fs/promises";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || process.argv[2] || "playwright");
await mkdir(".agents/header-animation", { recursive: true });
const browser = await chromium.launch({ channel: "msedge", headless: true });
try {
  for (const viewport of [{ width: 1440, height: 960 }, { width: 390, height: 844 }]) {
    const page = await browser.newPage({ viewport });
    const errors = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.addInitScript(() => {
      const book = {
        id: "return-check", title: "返回流程验证", author: "作者", language: "ja",
        sourceName: "check.epub", chapterCount: 1, totalCharacters: 100,
        progressOffset: 0, progressPercent: 0, createdAt: "2026-10-18T00:00:00Z",
        tags: [], coverPath: "/branding/readux-preview-cover.svg",
      };
      window.iconStates = [];
      window.isTauri = true;
      window.__TAURI_INTERNALS__ = {
        convertFileSrc: (path) => path,
        invoke: async (command, payload) => {
          if (command === "reader_library") return { books: [book], path: "验证书库" };
          if (command === "reader_book_summary") return book;
          // 保持打开请求待完成，验证分析准备期间的返回操作。
          if (command === "reader_open_book") return new Promise(() => {});
          if (command === "nlp_request") return { result: { settings: { analysis_timing_enabled: false } }, error: null };
          if (command === "set_reading_taskbar_icon") window.iconStates.push(Boolean(payload.rgba));
          return null;
        },
      };
    });
    await page.goto(process.env.READUX_TEST_URL || "http://127.0.0.1:1423/");
    await page.getByRole("heading", { name: "书架", exact: true }).waitFor();
    await page.getByRole("button", { name: /^返回流程验证/ }).first().click();
    await page.waitForFunction(() => document.querySelector(".reading-mark svg")?.getCurrentTime() >= 2.2);
    await page.waitForFunction(() => window.iconStates.at(-1) === true);
    const cover = await page.locator(".reading-mark image").getAttribute("href");
    await page.getByRole("button", { name: "返回书架", exact: true }).click();
    await page.getByRole("heading", { name: "书架", exact: true }).waitFor({ timeout: 1000 });
    assert.equal(await page.locator(".reader-progress").count(), 0, "返回后应立即移除加载页");
    const time = await page.locator(".reading-mark svg").evaluate((svg) => svg.getCurrentTime());
    assert.ok(time > 0 && time <= 2.2, "书架显示时 header 应仍在倒放");
    assert.equal(await page.locator(".reading-mark image").getAttribute("href"), cover, "倒放期间保留封面");
    assert.equal(await page.evaluate(() => window.iconStates.at(-1)), false, "返回时恢复默认图标");
    assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
    await page.screenshot({ path: `.agents/header-animation/return-${viewport.width}.png` });
    assert.deepEqual(errors, []);
    await page.close();
  }
  console.log("桌面与窄屏的书架即时返回、加载页移除、封面倒放及图标恢复验证通过。");
} finally {
  await browser.close();
}
