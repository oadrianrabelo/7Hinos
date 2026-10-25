import { describe, expect, it } from "vitest";
import { countdownText, nextTimeOfDay } from "./clock";

describe("countdown", () => {
  it("formats what is left and stops at zero", () => {
    expect(countdownText(300_000, 1_000)).toBe("04:59");
    expect(countdownText(3_723_000, 0)).toBe("1:02:03");
    expect(countdownText(1_000, 5_000)).toBe("00:00");
  });

  it("finds the next time of day", () => {
    const now = new Date(2026, 9, 25, 18, 0);
    expect(new Date(nextTimeOfDay("19:30", now)!).getHours()).toBe(19);
    expect(new Date(nextTimeOfDay("08:00", now)!).getDate()).toBe(26);
    expect(nextTimeOfDay("25:00", now)).toBeNull();
    expect(nextTimeOfDay("abc", now)).toBeNull();
  });
});
