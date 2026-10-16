import assert from "node:assert/strict";
import test from "node:test";
import { findTextMatches } from "./find.ts";

test("findTextMatches returns prepared-text coordinates", () => {
  assert.deepEqual(findTextMatches("彼女は風を見る", "風を見る"), [{ start: 3, end: 7 }]);
});

test("findTextMatches keeps overlapping matches", () => {
  assert.deepEqual(findTextMatches("あああ", "ああ"), [{ start: 0, end: 2 }, { start: 1, end: 3 }]);
});
