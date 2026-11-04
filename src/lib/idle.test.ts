import { describe, expect, it } from "vitest";
import { BIRTHDAYS_TITLE, birthdaysOfWeek, currentIdlePage, idlePages } from "./idle";
import type { Birthday, IdleScreen, Notice } from "./types";

const person = (name: string, day: number, month: number): Birthday => ({ id: 0, name, day, month });
const idle = (patch: Partial<IdleScreen> = {}): IdleScreen => ({ image: null, text: "", showClock: false, birthdays: false, rotateSecs: 10, ...patch });

describe("aniversariantes da semana", () => {
  it("vai de domingo a sábado, na ordem dos dias", () => {
    const people = [person("Zeca", 7, 11), person("Ana", 2, 11), person("Bia", 7, 11), person("Fora", 9, 11), person("Antes", 31, 10)];
    expect(birthdaysOfWeek(people, new Date(2026, 10, 4))).toEqual(["02/11 · Ana", "07/11 · Bia", "07/11 · Zeca"]);
  });

  it("atravessa o ano novo e cuida do 29 de fevereiro", () => {
    const people = [person("Réveillon", 1, 1), person("Bissexto", 29, 2)];
    expect(birthdaysOfWeek(people, new Date(2026, 11, 30))).toEqual(["01/01 · Réveillon"]);
    expect(birthdaysOfWeek(people, new Date(2027, 2, 1))).toEqual(["28/02 · Bissexto"]);
    expect(birthdaysOfWeek(people, new Date(2028, 1, 29))).toEqual(["29/02 · Bissexto"]);
  });
});

describe("tela de espera passando", () => {
  const notices: Notice[] = [
    { id: 1, title: "Santa Ceia", body: "Sábado", idle: true },
    { id: 2, title: "Só no botão", body: "x" },
  ];

  it("fica parada quando só há a tela de sempre", () => {
    expect(currentIdlePage(idle(), notices.slice(1), [], 0)).toEqual({ page: { kind: "base" }, next: null });
  });

  it("troca pelo relógio", () => {
    const today = new Date(2026, 10, 4, 12).getTime();
    const base = Math.floor(today / 30_000) * 30_000;
    const cfg = idle({ birthdays: true });
    const people = [person("Ana", 4, 11)];
    expect(idlePages(cfg, notices, people, new Date(base))).toHaveLength(3);
    expect(currentIdlePage(cfg, notices, people, base + 2_500)).toEqual({ page: { kind: "base" }, next: 7_500 });
    expect(currentIdlePage(cfg, notices, people, base + 10_000).page).toMatchObject({ title: BIRTHDAYS_TITLE });
    expect(currentIdlePage(cfg, notices, people, base + 25_000).page).toMatchObject({ title: "Santa Ceia" });
  });
});
