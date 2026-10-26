import { describe, expect, it } from "vitest";
import { backgroundCss, hexToHsv, hsvToHex, mix, normalizeHex, radialStops } from "./colors";

describe("colors", () => {
  it("draws solid, radial and the old default", () => {
    expect(backgroundCss("#173b53")).toBe("#173b53");
    expect(backgroundCss("radial:#ffffff,#000000")).toContain("radial-gradient(");
    expect(backgroundCss("gradient")).toBe(backgroundCss("radial:#1d4560,#050b11"));
    expect(backgroundCss("lixo")).toBe("#000000");
  });

  it("reads radial stops", () => {
    expect(radialStops("radial:#112233,#000000")).toEqual(["#112233", "#000000"]);
    expect(radialStops("#112233")).toBeNull();
  });

  it("mixes and converts", () => {
    expect(mix("#ffffff", "#000000", 0.5)).toBe("#808080");
    for (const c of ["#ff0000", "#1d4560", "#ffc94a", "#000000", "#ffffff"]) {
      expect(hsvToHex(...hexToHsv(c))).toBe(c);
    }
  });

  it("normalizes typed hex", () => {
    expect(normalizeHex("abc")).toBe("#aabbcc");
    expect(normalizeHex(" #A1B2C3 ")).toBe("#a1b2c3");
    expect(normalizeHex("xyz")).toBeNull();
  });
});
