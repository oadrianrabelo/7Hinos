import { describe, expect, it } from "vitest";
import { formatBytes, formatTime, formatWhen } from "./format";

describe("format", () => {
  it("formats times and sizes", () => {
    expect(formatTime(65)).toBe("1:05");
    expect(formatTime(3725)).toBe("1:02:05");
    expect(formatTime(NaN)).toBe("0:00");
    expect(formatBytes(1536)).toBe("1,5 KB");
    expect(formatBytes(734_003_200)).toBe("700 MB");
  });

  it("describes when something happened", () => {
    const now = new Date(2026, 9, 25, 15, 0);
    const at = (d: Date) => d.getTime() / 1000;
    expect(formatWhen(at(new Date(2026, 9, 25, 14, 32)), now)).toMatch(/^hoje às 14:32$/);
    expect(formatWhen(at(new Date(2026, 9, 24, 9, 5)), now)).toMatch(/^ontem às 09:05$/);
    expect(formatWhen(at(new Date(2026, 9, 12, 18, 0)), now)).toMatch(/^12\/10\/2026 às 18:00$/);
  });
});
