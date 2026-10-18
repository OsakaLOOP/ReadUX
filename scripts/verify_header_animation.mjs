import assert from "node:assert/strict";
import { mkdir } from "node:fs/promises";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || process.argv[2] || "playwright");
const browser = await chromium.launch({ channel: "msedge", headless: true });
const output = ".agents/header-animation";
await mkdir(output, { recursive: true });
const page = await browser.newPage({ viewport: { width: 1440, height: 960 } });
const errors = [];
page.on("pageerror", (error) => { errors.push(error.message); console.error(error.stack); });
page.on("console", (message) => {
  if (message.type() === "error") { errors.push(message.text()); console.error(message.text()); }
});
page.on("response", (response) => {
  if (response.status() >= 400) console.error(response.status(), response.url());
});

async function waitForTime(expected) {
  await page.waitForFunction((target) => {
    const svg = document.querySelector(".reading-mark svg");
    return svg && Math.abs(svg.getCurrentTime() - target) < 0.000001;
  }, expected);
}

async function frame() {
  return page.locator(".reading-mark svg").evaluate((svg) => {
    const blade = svg.querySelector('[id$="-katana-blade-a"]');
    const helmet = svg.querySelector('[id$="-kabuto-helmet"]');
    const book = svg.querySelector('[id$="-book-root"]');
    const box = blade.getBBox();
    return {
      time: svg.getCurrentTime(),
      helmet: helmet.getTotalLength(),
      blade: { x: box.x, y: box.y, width: box.width, height: box.height },
      bookOpacity: Number(getComputedStyle(book).opacity),
      bookX: book.getCTM().e,
    };
  });
}

try {
  const started = Date.now();
  await page.goto(process.env.READUX_TEST_URL || "http://127.0.0.1:1422/", { waitUntil: "networkidle" });
  await page.getByRole("heading", { name: "书架", exact: true }).waitFor({ timeout: 15000 });
  assert.deepEqual(errors, [], "前端挂载应成功");
  console.log(`首次打开书架：${Date.now() - started} ms`);
  assert.equal(await page.locator(".app-header").count(), 1);
  assert.equal(await page.locator(".reading-mark svg").count(), 1);
  const closed = await frame();
  assert.equal(closed.time, 0);
  assert.equal(closed.bookOpacity, 0);
  await page.evaluate(() => {
    window.__testedHeader = document.querySelector(".app-header");
    window.__testedHelmet = document.querySelector('[id$="-kabuto-helmet"]');
  });
  await page.screenshot({ path: `${output}/desktop-library.png` });

  await page.getByRole("button", { name: "打开《窓辺の一日》", exact: true }).click();
  const sequence = await page.evaluate(() => {
    const animations = (selector) => document.querySelector(selector).getAnimations();
    const arrow = animations(".app-header__back").find((animation) => animation instanceof CSSAnimation);
    const divider = animations(".app-header__divider").find((animation) => animation.transitionProperty === "transform");
    const title = animations(".app-header__page-identity").find((animation) => animation.transitionProperty === "transform");
    return { arrow: arrow.effect.getComputedTiming().endTime, divider: divider.effect.getComputedTiming().endTime,
      titleDelay: title.effect.getTiming().delay, titleDuration: title.effect.getTiming().duration };
  });
  assert.ok(sequence.titleDelay >= Math.max(sequence.arrow, sequence.divider), "箭头与竖线就位后，书名才开始翻入");
  assert.equal(sequence.titleDuration, 460);
  await page.waitForTimeout(150);
  const middle = await frame();
  assert.ok(middle.time > 0 && middle.time < 2.2);
  assert.notEqual(middle.helmet, closed.helmet, "眼睛应连续变形");
  assert.notDeepEqual(middle.blade, closed.blade, "刀身应同步移动");
  assert.ok(middle.bookOpacity > 0, "书本应同步进入");
  await page.screenshot({ path: `${output}/desktop-entering.png` });
  await page.waitForFunction(() => document.querySelector(".reading-mark svg").getCurrentTime() >= 1.05);
  const settledMark = await frame();
  await waitForTime(2.2);
  const opened = await frame();
  assert.notEqual(middle.helmet, opened.helmet, "中间姿态应有实际插值");
  assert.ok(opened.blade.width < closed.blade.width / 2);
  assert.equal(settledMark.helmet, opened.helmet, "眼睛应在约 1 秒时完成变换");
  assert.deepEqual(settledMark.blade, opened.blade, "刀身应在约 1 秒时完成变换");
  assert.notEqual(settledMark.bookX, opened.bookX, "眼睛和刀身就位后，书本应继续原有动画");
  await page.screenshot({ path: `${output}/desktop-reading.png` });

  await page.getByRole("button", { name: "关闭书籍", exact: true }).click();
  await page.waitForTimeout(650);
  const closing = await frame();
  assert.ok(closing.time > 0 && closing.time < 2.2);
  await page.getByRole("button", { name: "打开书籍", exact: true }).click();
  const reversed = await frame();
  assert.ok(Math.abs(reversed.time - closing.time) < 0.15, "反向播放应保留当前进度");
  await waitForTime(2.2);
  await page.getByRole("button", { name: "关闭书籍", exact: true }).click();
  await waitForTime(0);
  const restored = await frame();
  assert.deepEqual(restored, closed, "关闭后应恢复默认徽标");
  assert.ok(await page.evaluate(() => window.__testedHeader === document.querySelector(".app-header")
    && window.__testedHelmet === document.querySelector('[id$="-kabuto-helmet"]')), "动画期间应复用原节点");

  await page.setViewportSize({ width: 390, height: 844 });
  await page.screenshot({ path: `${output}/mobile-library.png` });
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.getByRole("button", { name: "打开书籍", exact: true }).click();
  await waitForTime(2.2);
  await page.locator(".app-header__page-identity strong").evaluate((title) => {
    title.textContent = "これは長い本の題名で、窓辺から始まる物語を最後まで読むための表示確認です";
  });
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), "长书名应适应窄屏宽度");
  await page.screenshot({ path: `${output}/mobile-reading.png` });
  await page.getByRole("button", { name: "关闭书籍", exact: true }).click();
  await waitForTime(0);
  assert.deepEqual(errors, []);
  console.log("前端挂载、单一 header、同步变形、反向播放和窄屏验证通过。");
} finally {
  await browser.close();
}
