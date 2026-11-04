import { describe, expect, it } from "vitest";
import { partRole, programRows, programText, programTitle, rosterWho } from "./program";
import { emptyRoster, type Service } from "./roster";
import type { SetItem } from "./types";

const doc = emptyRoster();
doc.people = [
  { id: "p1", name: "Pr. Marcos", phone: "", roles: ["pregacao"], away: [] },
  { id: "p2", name: "Ana", phone: "", roles: ["oracao"], away: [] },
];
const service: Service = {
  id: "s1",
  date: "2026-11-07",
  time: "09:00",
  name: "Culto de sábado",
  templateId: "sabado",
  roles: ["pregacao", "oracao", "leitura"],
  assigned: { pregacao: ["p1"], oracao: ["p2"] },
  note: "Santa Ceia",
};

describe("programa do culto", () => {
  it("liga o momento à função da escala", () => {
    expect(partRole("Sermão")).toBe("pregacao");
    expect(partRole("Oração intercessora")).toBe("oracao");
    expect(partRole("Momento infantil")).toBe("infantil");
    expect(partRole("Dízimos e ofertas")).toBe("ofertorio");
    expect(partRole("Informativo Mundial das Missões")).toBe("es-missao");
    expect(partRole("Bênção final")).toBeNull();
    expect(rosterWho(doc, service, "Sermão")).toBe("Pr. Marcos");
    expect(rosterWho(doc, service, "Leitura bíblica")).toBe("");
    expect(rosterWho(doc, null, "Sermão")).toBe("");
  });

  it("a anotação vale mais que a escala", () => {
    const items: SetItem[] = [
      { kind: "part", id: "a", title: "Oração inicial" },
      { kind: "song", id: 12, title: "Castelo forte" },
      { kind: "part", id: "b", title: "Sermão", minutes: 40, note: "Pr. visitante" },
    ];
    const rows = programRows(items, doc, service);
    expect(rows.map((r) => r.who)).toEqual(["Ana", "", "Pr. visitante"]);
    const text = programText(rows, programTitle(service, "2026-11-07"), "IASD Central");
    expect(text).toContain("*Culto de sábado* — IASD Central");
    expect(text).toContain("Santa Ceia");
    expect(text).toContain("1. Oração inicial — Ana");
    expect(text).toContain("2. Hino: Castelo forte");
    expect(text).toContain("3. Sermão — Pr. visitante");
  });
});
