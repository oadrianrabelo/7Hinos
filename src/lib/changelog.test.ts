import { describe, expect, it } from "vitest";
import { formatDate, parseChangelog } from "./changelog";

const sample = `# Changelog

## [1.3.0](https://github.com/x/compare/v1.2.0...v1.3.0) (2026-10-25)


### ✨ Novas Funcionalidades

* apresentações e lição ([c9a348e](https://github.com/x/commit/c9a348ef))
* **telas:** perfis e reabrir ([085e0f5](https://github.com/x/commit/085e0f53))
* apresentações e lição ([8bef5bb](https://github.com/x/commit/8bef5bb9))

### 🔧 Manutenção

* Cargo.lock na versão 1.2.0 ([afef903](https://github.com/x/commit/afef903b))

## 0.4.0 (2026-09-01)

### Correções

* retorno vazio (#24)
`;

describe("changelog", () => {
  it("reads versions, sections and items without commit links", () => {
    const [latest, old] = parseChangelog(sample);
    expect(latest.version).toBe("1.3.0");
    expect(latest.date).toBe("2026-10-25");
    expect(latest.sections[0]).toMatchObject({ title: "Novas Funcionalidades", icon: "✨", technical: false });
    expect(latest.sections[0].items).toEqual([
      { scope: null, text: "Apresentações e lição" },
      { scope: "telas", text: "Perfis e reabrir" },
    ]);
    expect(latest.sections[1].technical).toBe(true);
    expect(old.sections[0].items[0].text).toBe("Retorno vazio");
  });

  it("formats dates in Portuguese", () => {
    expect(formatDate("2026-10-25")).toBe("25 de outubro de 2026");
    expect(formatDate(null)).toBe("");
  });
});
