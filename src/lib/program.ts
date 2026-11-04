// Programa do culto: a ordem do "Culto de hoje" com quem faz cada momento,
// tirado da escala. Para imprimir e mandar no grupo do WhatsApp.

import { assignedNames, fullDate, shortTime, type RosterDoc, type Service } from "./roster";
import type { SetItem } from "./types";

/** Palavras do nome do momento → função da escala. A primeira que bate vale. */
const PART_ROLES: [RegExp, string][] = [
  [/serm[aã]o|mensagem|prega[cç][aã]o|palavra/i, "pregacao"],
  [/infantil|crian[cç]as|hist[oó]ria/i, "infantil"],
  [/d[ií]zimos?|ofertas?|ofert[oó]rio/i, "ofertorio"],
  [/m[uú]sica especial|especial/i, "especial"],
  [/leitura/i, "leitura"],
  [/informativo|miss[oõ]es|mission[aá]rio/i, "es-missao"],
  [/li[cç][aã]o/i, "es-licao"],
  [/escola sabatina/i, "es-direcao"],
  [/ora[cç][aã]o/i, "oracao"],
  [/louvor|cânticos|cantico/i, "louvor"],
  [/boas[- ]vindas|abertura|plataforma/i, "plataforma"],
];

/** A função da escala de um momento ("Sermão" → pregação), se houver. */
export function partRole(title: string): string | null {
  return PART_ROLES.find(([re]) => re.test(title))?.[1] ?? null;
}

/** Quem faz o momento segundo a escala do culto (vazio se ninguém). */
export function rosterWho(doc: RosterDoc, service: Service | null, title: string): string {
  if (!service) return "";
  const role = partRole(title);
  if (!role || !service.roles.includes(role)) return "";
  return assignedNames(doc, service.assigned[role]);
}

export interface ProgramRow {
  /** "Hino", "Momento"… */
  kind: string;
  title: string;
  /** Quem faz: a anotação do item ou, sem ela, a escala. */
  who: string;
  minutes: number | null;
}

const KIND: Record<SetItem["kind"], string> = { song: "Hino", deck: "Apresentação", video: "Vídeo", part: "" };

export function programRows(items: SetItem[], doc: RosterDoc, service: Service | null): ProgramRow[] {
  return items.map((item) => {
    const note = item.note?.trim() ?? "";
    const who = note || (item.kind === "part" ? rosterWho(doc, service, item.title) : "");
    return {
      kind: KIND[item.kind],
      title: item.title,
      who,
      minutes: item.kind === "part" ? (item.minutes ?? null) : null,
    };
  });
}

/** Título do programa: culto da escala de hoje ou só a data. */
export function programTitle(service: Service | null, today: string): { title: string; subtitle: string } {
  if (!service) return { title: "Programa do culto", subtitle: fullDate(today) };
  const extra = service.note.trim() ? ` · ${service.note.trim()}` : "";
  return { title: service.name, subtitle: `${fullDate(service.date)} · ${shortTime(service.time)}${extra}` };
}

/** Texto para o grupo do WhatsApp. */
export function programText(rows: ProgramRow[], head: { title: string; subtitle: string }, church = ""): string {
  const lines = [`⛪ *${head.title}*${church ? ` — ${church}` : ""}`, head.subtitle, ""];
  rows.forEach((r, i) => {
    const what = r.kind ? `${r.kind}: ${r.title}` : r.title;
    lines.push(`${i + 1}. ${what}${r.who ? ` — ${r.who}` : ""}`);
  });
  return lines.join("\n").trim() + "\n";
}
