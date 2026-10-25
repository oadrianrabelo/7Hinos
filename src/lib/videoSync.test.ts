import { describe, expect, it } from "vitest";
import { expectedPosition, needsSeek } from "./videoSync";
import type { VideoState } from "./types";

const video = (patch: Partial<VideoState> = {}): VideoState => ({
  path: "/v.mp4",
  title: "v",
  playing: true,
  position: 10,
  updatedAt: 1_000,
  looping: false,
  volume: 1,
  targets: ["output-1"],
  ...patch,
});

describe("expectedPosition", () => {
  it("advances while playing", () => {
    expect(expectedPosition(video(), 3_500)).toBeCloseTo(12.5);
  });
  it("stays put while paused", () => {
    expect(expectedPosition(video({ playing: false }), 99_000)).toBe(10);
  });
  it("clamps to the end or wraps when looping", () => {
    expect(expectedPosition(video(), 31_000, 30)).toBe(30);
    expect(expectedPosition(video({ looping: true }), 31_000, 30)).toBeCloseTo(10);
  });
});

describe("needsSeek", () => {
  it("ignores small drift", () => {
    expect(needsSeek(10, 10.2)).toBe(false);
    expect(needsSeek(10, 11)).toBe(true);
  });
});
