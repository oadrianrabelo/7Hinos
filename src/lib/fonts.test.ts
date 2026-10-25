import { describe, expect, it } from "vitest";
import { fontFamily, fontName } from "./fonts";

describe("fontes", () => {
  it("as três de sempre continuam iguais", () => {
    expect(fontFamily("condensed")).toContain("Barlow Condensed");
    expect(fontFamily("sans")).toContain("Inter");
    expect(fontFamily("serif")).toContain("Georgia");
  });

  it("embutidas, do computador e de arquivo", () => {
    expect(fontFamily("lora")).toContain("Lora Variable");
    expect(fontFamily("system:Segoe UI")).toMatch(/^"Segoe UI", /);
    expect(fontFamily("file:Minha Fonte.ttf")).toMatch(/^"7h-Minha-Fonte-ttf"/);
    expect(fontName("file:Minha Fonte.ttf")).toBe("Minha Fonte");
    expect(fontName("system:Arial")).toBe("Arial");
  });

  it("nome desconhecido cai na condensada", () => {
    expect(fontFamily("nao-existe")).toContain("Barlow Condensed");
    expect(fontFamily(null)).toContain("Barlow Condensed");
  });
});
