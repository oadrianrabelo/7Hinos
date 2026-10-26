import { describe, expect, it } from "vitest";
import { available, drawFrame, numberRange, parseNames, pick, sampleCandidates } from "./draw";

describe("draw", () => {
  it("builds number ranges in either order", () => {
    expect(numberRange(1, 5)).toEqual(["1", "2", "3", "4", "5"]);
    expect(numberRange(3, 1)).toEqual(["1", "2", "3"]);
  });

  it("reads names one per line without duplicates", () => {
    expect(parseNames(" Ana \n\nJoão\nana\r\nMaria ")).toEqual(["Ana", "João", "Maria"]);
  });

  it("does not repeat unless allowed", () => {
    expect(available(["1", "2", "3"], ["2"], false)).toEqual(["1", "3"]);
    expect(available(["1", "2", "3"], ["2"], true)).toEqual(["1", "2", "3"]);
    expect(pick([])).toBeNull();
    expect(pick(["a", "b", "c"], () => 0.99)).toBe("c");
  });

  it("samples a big pool keeping the result", () => {
    const pool = numberRange(1, 1000);
    const sample = sampleCandidates(pool, "777");
    expect(sample.length).toBe(60);
    expect(sample).toContain("777");
  });

  it("rolls and then lands on the result", () => {
    const scene = { result: "B", candidates: ["A", "B", "C", "D"], startedAt: 1000, durationMs: 4000 };
    expect(drawFrame(scene, 1500).done).toBe(false);
    expect(drawFrame(scene, 5000)).toEqual({ text: "B", done: true });
    const seen = new Set(Array.from({ length: 40 }, (_, i) => drawFrame(scene, 1000 + i * 80).text));
    expect(seen.size).toBeGreaterThan(1);
  });
});
