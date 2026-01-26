import { describe, expect, it } from "vitest";
import { clampOrderRange } from "./music-loop";

describe("music-loop", () => {
  it("clamps and orders", () => {
    expect(clampOrderRange({ start: 5, end: 2 }, 10)).toEqual({ start: 5, end: 5 });
    expect(clampOrderRange({ start: -1, end: 999 }, 3)).toEqual({ start: 0, end: 2 });
  });
});
