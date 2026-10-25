import { describe, expect, it } from "vitest";
import { elapsedMs, timerDisplay, timerText } from "./timer";

describe("cronômetro", () => {
  it("soma o tempo corrido só enquanto corre", () => {
    expect(elapsedMs({ elapsedMs: 1000, startedAt: null, limitMs: null }, 99_999)).toBe(1000);
    expect(elapsedMs({ elapsedMs: 1000, startedAt: 10_000, limitMs: null }, 15_000)).toBe(6000);
  });

  it("formata minutos, horas e tempo estourado", () => {
    expect(timerText(65_000)).toBe("01:05");
    expect(timerText(3_723_000)).toBe("1:02:03");
    expect(timerText(-61_000)).toBe("-01:01");
  });

  it("com tempo combinado, mostra o que falta e avisa", () => {
    const base = { elapsedMs: 0, startedAt: 0, limitMs: 30 * 60_000 };
    expect(timerDisplay(base, 10 * 60_000)).toMatchObject({ label: "Faltam", text: "20:00", over: false, warn: false });
    expect(timerDisplay(base, 26 * 60_000).warn).toBe(true);
    expect(timerDisplay(base, 31 * 60_000)).toMatchObject({ label: "Passou", text: "01:00", over: true, warn: false });
    expect(timerDisplay({ ...base, limitMs: null }, 65_000)).toMatchObject({ label: "Tempo", text: "01:05", limited: false });
  });
});
