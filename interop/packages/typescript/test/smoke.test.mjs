import assert from "node:assert/strict";
import test from "node:test";

import {
  Calendar,
  CivilDate,
  Limudim,
  TractateCode,
  ZmanPresets,
} from "../lib/index.mjs";

test("gregorian/hebrew round trip via Calendar", () => {
  const calendar = new Calendar();
  const civil = new CivilDate({ year: 2024, month: 1, day: 20 });

  const hebrew = calendar.gregorianToHebrew(civil);
  assert.ok(hebrew);
  assert.equal(hebrew.year, 5784);
  assert.equal(hebrew.month, 5);
  assert.equal(hebrew.day, 10);

  const back = calendar.hebrewToGregorian(hebrew);
  assert.ok(back);
  assert.equal(back.year, civil.year);
  assert.equal(back.month, civil.month);
  assert.equal(back.day, civil.day);
});

test("daf yomi bavli via Limudim", () => {
  const limudim = new Limudim();
  const daf = limudim.dafYomiBavli(2017, 12, 28);

  assert.ok(daf);
  assert.equal(daf.tractate, TractateCode.Shevuos);
  assert.equal(daf.page, 30);
});

test("preset count via ZmanPresets", () => {
  const presets = new ZmanPresets();
  assert.equal(presets.presetCount(), 167);
});
