// A tela de espera pode ir passando: a de sempre (logo, texto), os
// aniversariantes da semana e os avisos marcados. A vez de cada uma sai do
// relógio, então o telão, o retorno e a prévia trocam juntos.
//
// Mesma regra de `src-tauri/src/projection/idle.rs`.

import type { Birthday, IdleScreen, Notice } from "./types";

export type IdlePage = { kind: "base" } | { kind: "text"; title: string; body: string };

export const BIRTHDAYS_TITLE = "Aniversariantes da semana";

const pad = (n: number) => String(n).padStart(2, "0");

/** Domingo a sábado da semana de `today`. */
export function weekOf(today: Date): Date[] {
  const sunday = new Date(today.getFullYear(), today.getMonth(), today.getDate() - today.getDay());
  return Array.from({ length: 7 }, (_, i) => new Date(sunday.getFullYear(), sunday.getMonth(), sunday.getDate() + i));
}

const isLeap = (year: number) => new Date(year, 1, 29).getMonth() === 1;

/** "05/11 · Ana Souza", na ordem dos dias. Quem nasceu em 29/02 aparece no dia 28 nos anos sem 29. */
export function birthdaysOfWeek(birthdays: Birthday[], today: Date): string[] {
  const lines: string[] = [];
  for (const day of weekOf(today)) {
    const d = day.getDate();
    const m = day.getMonth() + 1;
    const leapFill = m === 2 && d === 28 && !isLeap(day.getFullYear());
    const names = birthdays
      .map((b) => ({ ...b, name: b.name.trim() }))
      .filter((b) => b.name && ((b.month === m && b.day === d) || (leapFill && b.month === 2 && b.day === 29)))
      .map((b) => b.name)
      .sort((a, b) => a.toLowerCase().localeCompare(b.toLowerCase()));
    for (const name of names) lines.push(`${pad(d)}/${pad(m)} · ${name}`);
  }
  return lines;
}

export function idlePages(idle: IdleScreen, notices: Notice[], birthdays: Birthday[], today: Date): IdlePage[] {
  const pages: IdlePage[] = [{ kind: "base" }];
  if (idle.birthdays) {
    const lines = birthdaysOfWeek(birthdays, today);
    if (lines.length) pages.push({ kind: "text", title: BIRTHDAYS_TITLE, body: lines.join("\n") });
  }
  for (const n of notices) {
    if (!n.idle || (!n.title.trim() && !n.body.trim())) continue;
    pages.push({ kind: "text", title: n.title.trim(), body: n.body.trim() });
  }
  return pages;
}

/** A tela da vez em `now` (ms) e quantos ms faltam para a próxima (`null` quando só há uma). */
export function currentIdlePage(
  idle: IdleScreen,
  notices: Notice[],
  birthdays: Birthday[],
  now: number,
): { page: IdlePage; next: number | null } {
  const pages = idlePages(idle, notices, birthdays, new Date(now));
  if (pages.length === 1) return { page: pages[0], next: null };
  const every = (idle.rotateSecs || 12) * 1000;
  const slot = Math.floor(now / every);
  return { page: pages[slot % pages.length], next: Math.max(1, Math.ceil((slot + 1) * every - now)) };
}
