import { describe, expect, it } from "vitest";
import {
  conflicts,
  csv,
  emptyRoster,
  generateMonth,
  normalizeRoster,
  printTables,
  serviceRoles,
  shortDate,
  shortTime,
  suggest,
  whatsappText,
  type Person,
  type RosterDoc,
} from "./roster";

const person = (id: string, name: string, roles: string[], away: string[] = []): Person => ({ id, name, phone: "", roles, away });

function church(): RosterDoc {
  const doc = emptyRoster();
  doc.people = [
    person("ana", "Ana", ["pregacao", "oracao"]),
    person("bia", "Bia", ["pregacao", "recepcao"]),
    person("caio", "Caio", ["pregacao", "recepcao", "plataforma"], ["2026-11-07"]),
    person("davi", "Davi", ["recepcao", "plataforma"]),
    person("eva", "Eva", ["recepcao"]),
    person("fabio", "Fábio", ["recepcao"]),
  ];
  return doc;
}

describe("cultos do mês", () => {
  it("cria sábado 9h, quarta 19h30 e domingo 18h", () => {
    const nov = generateMonth(emptyRoster(), 2026, 11);
    // Novembro de 2026: 4 sábados, 4 quartas e 5 domingos.
    expect(nov.filter((s) => s.templateId === "sabado")).toHaveLength(4);
    expect(nov.filter((s) => s.templateId === "quarta")).toHaveLength(4);
    expect(nov.filter((s) => s.templateId === "domingo")).toHaveLength(5);
    const sab = nov.find((s) => s.templateId === "sabado")!;
    expect(sab.date).toBe("2026-11-07");
    expect(sab.time).toBe("09:00");
  });

  it("não duplica o que já existe", () => {
    const doc = emptyRoster();
    doc.services = generateMonth(doc, 2026, 11);
    expect(generateMonth(doc, 2026, 11)).toHaveLength(0);
  });

  it("função desligada some do culto", () => {
    const doc = emptyRoster();
    const s = generateMonth(doc, 2026, 11)[0];
    expect(serviceRoles(doc, s).some((r) => r.id === "sonoplastia")).toBe(true);
    doc.roles.find((r) => r.id === "sonoplastia")!.active = false;
    expect(serviceRoles(doc, s).some((r) => r.id === "sonoplastia")).toBe(false);
  });
});

describe("conflitos", () => {
  it("avisa quem não pode, quem já está no culto e quem pregou há pouco", () => {
    const doc = church();
    doc.services = generateMonth(doc, 2026, 11);
    const sab7 = doc.services.find((s) => s.date === "2026-11-07")!;
    expect(conflicts(doc, sab7, "pregacao", "caio")[0].kind).toBe("away");
    sab7.assigned = { oracao: ["ana"], "es-licao": ["bia"] };
    expect(conflicts(doc, sab7, "pregacao", "ana").map((c) => c.kind)).toContain("double");
    // Escola Sabatina e pregação são grupos diferentes: não é conflito.
    expect(conflicts(doc, sab7, "pregacao", "bia").map((c) => c.kind)).not.toContain("double");
    const qua11 = doc.services.find((s) => s.date === "2026-11-11")!;
    qua11.assigned = { pregacao: ["bia"] };
    const sab14 = doc.services.find((s) => s.date === "2026-11-14")!;
    expect(conflicts(doc, sab14, "pregacao", "bia").map((c) => c.kind)).toContain("recent");
    expect(conflicts(doc, sab14, "pregacao", "ana")).toEqual([]);
    // Duas semanas depois já não é "recente".
    const qua25 = doc.services.find((s) => s.date === "2026-11-25")!;
    expect(conflicts(doc, qua25, "pregacao", "bia")).toEqual([]);
  });
});

describe("sugerir", () => {
  it("distribui de forma justa e respeita quem não pode", () => {
    const doc = church();
    const month = generateMonth(doc, 2026, 11);
    const filled = suggest(doc, month);
    const preachers = filled.map((s) => s.assigned.pregacao?.[0]).filter(Boolean);
    // 13 cultos, 3 pregadores: ninguém prega mais de 5 vezes.
    const counts = new Map<string, number>();
    for (const p of preachers) counts.set(p!, (counts.get(p!) ?? 0) + 1);
    expect(Math.max(...counts.values()) - Math.min(...counts.values())).toBeLessThanOrEqual(1);
    const sab7 = filled.find((s) => s.date === "2026-11-07")!;
    expect(sab7.assigned.pregacao).not.toContain("caio");
    // Ninguém em duas funções no mesmo culto.
    for (const s of filled) {
      const ids = Object.values(s.assigned).flat();
      expect(new Set(ids).size).toBe(ids.length);
    }
    // Recepção pede duas pessoas.
    expect(filled.find((s) => s.date === "2026-11-08")!.assigned.recepcao).toHaveLength(2);
  });

  it("com pouca gente, completa com quem já está no dia em outro grupo", () => {
    const doc = emptyRoster();
    doc.people = [person("rita", "Rita", ["es-direcao", "recepcao"]), person("lia", "Lia", ["recepcao"])];
    const sab = generateMonth(doc, 2026, 11).filter((s) => s.date === "2026-11-07");
    const filled = suggest(doc, sab)[0];
    expect(filled.assigned["es-direcao"]).toEqual(["rita"]);
    // Recepção pede 2: Lia e, na segunda passada, Rita (outro grupo).
    expect(filled.assigned.recepcao?.sort()).toEqual(["lia", "rita"]);
  });

  it("não mexe no que já foi escolhido", () => {
    const doc = church();
    const month = generateMonth(doc, 2026, 11);
    month[0].assigned = { pregacao: ["ana"] };
    expect(suggest(doc, month)[0].assigned.pregacao).toEqual(["ana"]);
    expect(month[0].assigned.recepcao).toBeUndefined();
  });
});

describe("exportar", () => {
  it("texto do WhatsApp, planilha e tabelas para imprimir", () => {
    const doc = church();
    doc.services = suggest(doc, generateMonth(doc, 2026, 11));
    const text = whatsappText(doc, doc.services, "Novembro de 2026", "IASD Central");
    expect(text).toContain("📅 *Escala de Novembro de 2026* — IASD Central");
    expect(text).toContain("*Sáb 07/11 · 9h · Culto de sábado*");
    expect(text).toMatch(/• Pregação: (Ana|Bia)/);
    const sheet = csv(doc, doc.services);
    expect(sheet.startsWith("﻿Data;Horário;Culto;Observação;Pregação")).toBe(true);
    expect(sheet.split("\r\n").length).toBe(15);
    const extra = { ...doc.services[0], id: "x", templateId: null, name: "Culto jovem", date: "2026-11-20", time: "19:30" };
    const tables = printTables(doc, [...doc.services, extra]);
    expect(tables.map((t) => t.title)).toEqual([
      "Culto de sábado · Sábado 9h — Culto",
      "Culto de sábado · Sábado 9h — Escola Sabatina",
      "Culto de sábado · Sábado 9h — Apoio",
      "Culto de oração · Quarta 19h30",
      "Culto de domingo · Domingo 18h",
      "Programações especiais",
    ]);
  });

  it("datas e horas curtas", () => {
    expect(shortDate("2026-11-07")).toBe("Sáb 07/11");
    expect(shortTime("19:30")).toBe("19h30");
    expect(shortTime("09:00")).toBe("9h");
  });

  it("lê documento antigo ou vazio", () => {
    expect(normalizeRoster(null).roles.length).toBeGreaterThan(10);
    const doc = normalizeRoster({ people: [{ id: "a", name: "Ana" }] });
    expect(doc.people[0]).toEqual({ id: "a", name: "Ana", phone: "", roles: [], away: [] });
  });
});
