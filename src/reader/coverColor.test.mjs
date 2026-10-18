import assert from "node:assert/strict";
import test from "node:test";
import { representativeCoverColor } from "./coverColor.ts";

function pixels(...groups) {
  return new Uint8ClampedArray(groups.flatMap(([count, color]) => Array.from({ length: count }, () => color).flat()));
}

test("鲜艳色获得更高权重，黑白灰背景仍参与计算", () => {
  for (const background of [[255, 255, 255, 255], [0, 0, 0, 255], [128, 128, 128, 255], [144, 140, 140, 255]]) {
    assert.equal(representativeCoverColor(pixels([100, background], [10, [240, 30, 60, 255]])), "#f01e3c");
  }
});

test("纯黑、纯白和灰度封面均使用封面颜色", () => {
  for (const value of [0, 8, 128, 248, 255]) {
    const hex = value.toString(16).padStart(2, "0");
    assert.equal(representativeCoverColor(pixels([10, [value, value, value, 255]])), `#${hex.repeat(3)}`);
  }
  assert.equal(representativeCoverColor(pixels([100, [255, 255, 255, 255]], [20, [128, 128, 128, 255]])), "#808080");
});

test("透明度连续影响权重，全透明图片没有可见代表色", () => {
  assert.equal(representativeCoverColor(pixels([10, [240, 30, 60, 1]])), "#f01e3c");
  assert.equal(representativeCoverColor(pixels([100, [255, 0, 0, 0]], [1, [128, 128, 128, 255]])), "#808080");
  assert.equal(representativeCoverColor(pixels([1, [255, 0, 0, 0]])), null);
});
