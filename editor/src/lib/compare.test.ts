import { describe, expect, it } from "vitest";

import { clamp01, wipeToSplitX } from "./compare";

describe("compare", () => {
  it("clamps wipe", () => {
    expect(clamp01(-1)).toBe(0);
    expect(clamp01(0.5)).toBe(0.5);
    expect(clamp01(2)).toBe(1);
    expect(clamp01(Number.NaN)).toBe(0);
    expect(clamp01(Number.POSITIVE_INFINITY)).toBe(0);
  });

  it("maps wipe to split pixel", () => {
    expect(wipeToSplitX(0, 100)).toBe(0);
    expect(wipeToSplitX(1, 100)).toBe(100);
    expect(wipeToSplitX(0.5, 101)).toBe(51);
  });
});
