import assert from "node:assert/strict";
import test from "node:test";
import { isNarrowMorphologyViewport, morphologyLayoutMode, morphologySheetSizes } from "./morphologyLayout.ts";

test("固定布局在窄屏切换为抽屉，宽屏使用侧栏", () => {
  assert.equal(isNarrowMorphologyViewport(960), true);
  assert.equal(isNarrowMorphologyViewport(961), false);
  assert.equal(morphologyLayoutMode(true, true), "sheet");
  assert.equal(morphologyLayoutMode(true, false), "panel");
  assert.equal(morphologyLayoutMode(false, true), "modal");
  assert.equal(morphologyLayoutMode(false, false), "modal");
});

test("长词条的默认高度覆盖词典入口，最高位置保留屏幕空间", () => {
  const size = morphologySheetSizes(844, 650, 1200);
  assert.equal(size.minimum, 80);
  assert.equal(size.preferred, 844 * .55);
  assert.equal(size.maximum, 748);
});

test("短内容按完整内容高度限制抽屉，最高位置不留多余空白", () => {
  const size = morphologySheetSizes(844, 170, 210);
  assert.equal(size.maximum, 232);
  assert.equal(size.preferred, 232);
});
