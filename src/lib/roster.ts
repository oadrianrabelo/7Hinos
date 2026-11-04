// Escala da igreja: quem faz o quê em cada culto do mês.
// Funções puras e testadas; o estado e a gravação ficam em state/roster.

export interface RosterRole {
  id: string;
  name: string;
  /** Quantas pessoas a função pede por culto (ex.: recepção 2). */
  slots: number;
  /** Desligada: não aparece na escala (ex.: igreja sem sonoplastia). */
  active: boolean;
  /** Grupo para organizar a tela e a impressão. */
  group: "Culto" | "Escola Sabatina" | "Apoio";
}

export interface Person {
  id: string;
  name: string;
  phone: string;
  /** Funções que a pessoa faz. */
  roles: string[];
  /** Dias em que não pode ("2026-11-07"). */
  away: string[];
}

/** Um culto que se repete toda semana. */
export interface ServiceTemplate {
  id: string;
  name: string;
  /** 0 = domingo … 6 = sábado. */
  weekday: number;
  time: string;
  roles: string[];
}

/** Um culto (ou programação) num dia. */
export interface Service {
  id: string;
  date: string;
  time: string;
  name: string;
  templateId: string | null;
  roles: string[];
  /** função → pessoas (ids), na ordem das vagas. */
  assigned: Record<string, string[]>;
  /** Tema, observações ("Santa Ceia", "Dia do jovem"). */
  note: string;
}

export interface RosterDoc {
  version: 1;
  /** Nome da igreja, no alto da folha impressa e do texto do WhatsApp. */
  church: string;
  roles: RosterRole[];
  people: Person[];
  templates: ServiceTemplate[];
  services: Service[];
  /** Como imprimir (lembrado entre um mês e outro). */
  print: PrintOptions;
}

export interface PrintOptions {
  /** "list": uma linha por culto, por data. "grouped": uma tabela por tipo de culto. */
  layout: "list" | "grouped";
  /** Funções na folha (vazio = todas as que aparecem no mês). */
  roles: string[];
}

const role = (id: string, name: string, group: RosterRole["group"], slots = 1, active = true): RosterRole => ({
  id,
  name,
  slots,
  active,
  group,
});

/** As funções de uma igreja adventista; dá para desligar e criar outras. */
export const DEFAULT_ROLES: RosterRole[] = [
  role("pregacao", "Pregação", "Culto"),
  role("plataforma", "Plataforma", "Culto", 2),
  role("oracao", "Oração", "Culto"),
  role("leitura", "Leitura bíblica", "Culto"),
  role("louvor", "Louvor (direção)", "Culto"),
  role("especial", "Música especial", "Culto"),
  role("ofertorio", "Ofertório", "Culto"),
  role("infantil", "História infantil", "Culto"),
  role("es-direcao", "Direção da Escola Sabatina", "Escola Sabatina"),
  role("es-licao", "Recapitulação da lição", "Escola Sabatina"),
  role("es-missao", "Informativo missionário", "Escola Sabatina"),
  role("recepcao", "Recepção", "Apoio", 2),
  role("diaconato", "Diaconato", "Apoio", 2),
  role("sonoplastia", "Sonoplastia", "Apoio"),
  role("projecao", "Projeção", "Apoio"),
];

/** Os cultos fixos: sábado 9h, quarta 19h30 e domingo 18h. */
export const DEFAULT_TEMPLATES: ServiceTemplate[] = [
  {
    id: "sabado",
    name: "Culto de sábado",
    weekday: 6,
    time: "09:00",
    roles: ["es-direcao", "es-licao", "es-missao", "pregacao", "plataforma", "oracao", "leitura", "louvor", "especial", "ofertorio", "infantil", "recepcao", "diaconato", "sonoplastia", "projecao"],
  },
  {
    id: "quarta",
    name: "Culto de oração",
    weekday: 3,
    time: "19:30",
    roles: ["pregacao", "plataforma", "louvor", "especial", "recepcao", "sonoplastia", "projecao"],
  },
  {
    id: "domingo",
    name: "Culto de domingo",
    weekday: 0,
    time: "18:00",
    roles: ["pregacao", "plataforma", "louvor", "especial", "recepcao", "sonoplastia", "projecao"],
  },
];

export function emptyRoster(): RosterDoc {
  return {
    version: 1,
    church: "",
    print: { layout: "list", roles: [] },
    roles: DEFAULT_ROLES.map((r) => ({ ...r })),
    people: [],
    templates: DEFAULT_TEMPLATES.map((t) => ({ ...t, roles: [...t.roles] })),
    services: [],
  };
}

/** Lê o que veio do disco, completando o que faltar (versões antigas). */
export function normalizeRoster(raw: unknown): RosterDoc {
  const base = emptyRoster();
  if (!raw || typeof raw !== "object") return base;
  const doc = raw as Partial<RosterDoc>;
  return {
    version: 1,
    church: typeof doc.church === "string" ? doc.church : "",
    print: {
      layout: doc.print?.layout === "grouped" ? "grouped" : "list",
      roles: Array.isArray(doc.print?.roles) ? doc.print.roles : [],
    },
    roles: Array.isArray(doc.roles) && doc.roles.length ? doc.roles : base.roles,
    people: Array.isArray(doc.people)
      ? doc.people.map((p: Partial<Person>) => ({
          id: p.id ?? newId("p"),
          name: p.name ?? "",
          phone: p.phone ?? "",
          roles: p.roles ?? [],
          away: p.away ?? [],
        }))
      : [],
    templates: Array.isArray(doc.templates) ? doc.templates : base.templates,
    services: Array.isArray(doc.services)
      ? doc.services.map((s: Partial<Service>) => ({
          id: s.id ?? newId("s"),
          date: s.date ?? "",
          time: s.time ?? "",
          name: s.name ?? "",
          templateId: s.templateId ?? null,
          roles: s.roles ?? [],
          assigned: s.assigned ?? {},
          note: s.note ?? "",
        }))
      : [],
  };
}

export function newId(prefix: string): string {
  return `${prefix}-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`;
}

// --- Datas (sempre "AAAA-MM-DD", no calendário local) -----------------------

export function isoDate(d: Date): string {
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}

export function parseDate(iso: string): Date {
  const [y, m, d] = iso.split("-").map(Number);
  return new Date(y, m - 1, d);
}

export const monthKey = (iso: string) => iso.slice(0, 7);

const WEEKDAYS = ["Domingo", "Segunda", "Terça", "Quarta", "Quinta", "Sexta", "Sábado"];
const WEEKDAYS_SHORT = ["Dom", "Seg", "Ter", "Qua", "Qui", "Sex", "Sáb"];
const WEEKDAYS_LONG = ["Domingo", "Segunda-feira", "Terça-feira", "Quarta-feira", "Quinta-feira", "Sexta-feira", "Sábado"];

/** "03/11/2026" */
export function fullDate(iso: string): string {
  const [y, m, d] = iso.split("-");
  return `${d}/${m}/${y}`;
}

export const weekdayLong = (iso: string) => WEEKDAYS_LONG[parseDate(iso).getDay()] ?? "";
const MONTHS = ["janeiro", "fevereiro", "março", "abril", "maio", "junho", "julho", "agosto", "setembro", "outubro", "novembro", "dezembro"];

export const weekdayName = (n: number) => WEEKDAYS[n] ?? "";

export function monthTitle(year: number, month: number): string {
  const name = MONTHS[month - 1] ?? "";
  return `${name.charAt(0).toUpperCase()}${name.slice(1)} de ${year}`;
}

/** "Sáb 07/11" */
export function shortDate(iso: string): string {
  const d = parseDate(iso);
  return `${WEEKDAYS_SHORT[d.getDay()]} ${String(d.getDate()).padStart(2, "0")}/${String(d.getMonth() + 1).padStart(2, "0")}`;
}

/** "9h", "19h30" */
export function shortTime(time: string): string {
  const [h, m] = time.split(":");
  if (!h) return "";
  return m && m !== "00" ? `${Number(h)}h${m}` : `${Number(h)}h`;
}

const daysBetween = (a: string, b: string) => Math.round((parseDate(b).getTime() - parseDate(a).getTime()) / 86_400_000);

export function sortServices(services: Service[]): Service[] {
  return [...services].sort((a, b) => (a.date + a.time).localeCompare(b.date + b.time));
}

export function servicesOfMonth(doc: RosterDoc, year: number, month: number): Service[] {
  const key = `${year}-${String(month).padStart(2, "0")}`;
  return sortServices(doc.services.filter((s) => monthKey(s.date) === key));
}

/** Os cultos fixos do mês que ainda não existem na escala. */
export function generateMonth(doc: RosterDoc, year: number, month: number): Service[] {
  const out: Service[] = [];
  const days = new Date(year, month, 0).getDate();
  for (let day = 1; day <= days; day++) {
    const date = new Date(year, month - 1, day);
    const iso = isoDate(date);
    for (const t of doc.templates) {
      if (t.weekday !== date.getDay()) continue;
      if (doc.services.some((s) => s.date === iso && s.templateId === t.id)) continue;
      out.push({
        id: `${t.id}-${iso}`,
        date: iso,
        time: t.time,
        name: t.name,
        templateId: t.id,
        roles: [...t.roles],
        assigned: {},
        note: "",
      });
    }
  }
  return out;
}

/** Funções que aparecem num culto (só as ligadas, na ordem das funções). */
export function serviceRoles(doc: RosterDoc, service: Service): RosterRole[] {
  return doc.roles.filter((r) => r.active && service.roles.includes(r.id));
}

/** Vagas vazias de um culto (cada função conta as vagas que pede). */
export function openSlots(doc: RosterDoc, service: Service): number {
  return serviceRoles(doc, service).reduce((n, r) => {
    const filled = (service.assigned[r.id] ?? []).filter(Boolean).length;
    return n + Math.max(0, Math.max(1, r.slots) - filled);
  }, 0);
}

// --- Conflitos -------------------------------------------------------------

export interface Conflict {
  kind: "away" | "double" | "sameDay" | "recent";
  message: string;
}

/** Problemas de pôr `personId` na função `roleId` deste culto. */
export function conflicts(doc: RosterDoc, service: Service, roleId: string, personId: string): Conflict[] {
  const person = doc.people.find((p) => p.id === personId);
  if (!person) return [];
  const out: Conflict[] = [];
  if (person.away.includes(service.date)) out.push({ kind: "away", message: `${person.name} avisou que não pode neste dia.` });
  // Duas funções do mesmo grupo no mesmo culto (pregar e orar, por
  // exemplo) é demais; dirigir a Escola Sabatina e depois recepcionar é normal.
  const group = doc.roles.find((r) => r.id === roleId)?.group;
  const otherRoles = Object.entries(service.assigned)
    .filter(([r, ids]) => r !== roleId && ids.includes(personId) && doc.roles.find((x) => x.id === r)?.group === group)
    .map(([r]) => doc.roles.find((x) => x.id === r)?.name ?? r);
  if (otherRoles.length) out.push({ kind: "double", message: `${person.name} já está em ${otherRoles.join(", ")} neste culto.` });
  const sameDay = doc.services.filter(
    (s) => s.id !== service.id && s.date === service.date && Object.values(s.assigned).some((ids) => ids.includes(personId)),
  );
  if (sameDay.length) out.push({ kind: "sameDay", message: `${person.name} também está no ${sameDay[0].name} deste dia.` });
  // Pregar duas vezes na mesma semana cansa a pessoa e a igreja.
  if (roleId === "pregacao") {
    const recent = doc.services.find(
      (s) =>
        s.id !== service.id &&
        s.date !== service.date &&
        s.assigned[roleId]?.includes(personId) &&
        Math.abs(daysBetween(s.date, service.date)) < 7,
    );
    if (recent) out.push({ kind: "recent", message: `${person.name} também prega em ${shortDate(recent.date)}.` });
  } else {
    // A mesma função no culto do mesmo tipo logo antes ou logo depois
    // (recepção dois sábados seguidos, por exemplo).
    // Só avisa se houver mais gente para revezar; senão não tem jeito.
    const role = doc.roles.find((r) => r.id === roleId);
    // (quem já está em outra função deste culto não conta como alternativa).
    const busy = new Set(Object.entries(service.assigned).flatMap(([r, ids]) => (r === roleId ? [] : ids)));
    const able = doc.people.filter((p) => p.roles.includes(roleId) && !p.away.includes(service.date) && !busy.has(p.id)).length;
    const hit = able > Math.max(1, role?.slots ?? 1) ? neighbor(doc, service, roleId, personId) : null;
    if (hit) {
      out.push({ kind: "recent", message: `${person.name} também está em ${role?.name ?? roleId} no culto ${hit.date < service.date ? "anterior" : "seguinte"} (${shortDate(hit.date)}).` });
    }
  }
  return out;
}

/** O culto do mesmo tipo logo antes ou logo depois em que a pessoa faz a mesma função. */
function neighbor(doc: RosterDoc, service: Service, roleId: string, personId: string, services = doc.services): Service | null {
  const same = sortServices(
    services.filter((s) => s.id !== service.id && (service.templateId ? s.templateId === service.templateId : s.name === service.name)),
  );
  const before = [...same].reverse().find((s) => s.date < service.date);
  const after = same.find((s) => s.date > service.date);
  return [before, after].find((s) => s?.assigned[roleId]?.includes(personId)) ?? null;
}

// --- Sugestão -------------------------------------------------------------

/**
 * Preenche as vagas vazias dos cultos com quem faz a função, de forma justa:
 * primeiro quem fez menos vezes no mês, depois quem está há mais tempo sem
 * fazer. Nunca põe quem avisou que não pode. Primeiro tenta não repetir
 * ninguém no mesmo dia; se faltar gente, completa com quem já está no dia
 * em outro grupo de funções (Escola Sabatina e recepção, por exemplo).
 * Devolve os cultos novos (não mexe nos dados).
 */
export function suggest(doc: RosterDoc, services: Service[]): Service[] {
  const result = sortServices(services).map((s) => ({ ...s, assigned: Object.fromEntries(Object.entries(s.assigned).map(([k, v]) => [k, [...v]])) }));
  const all = () => [...doc.services.filter((s) => !result.some((r) => r.id === s.id)), ...result];
  const groupOf = (roleId: string) => doc.roles.find((r) => r.id === roleId)?.group;
  for (const pass of ["strict", "relaxed"] as const) for (const service of result) {
    for (const r of serviceRoles(doc, service)) {
      const filled = (service.assigned[r.id] ?? []).filter(Boolean);
      const want = Math.max(1, r.slots);
      while (filled.length < want) {
        const today = all().filter((s) => s.date === service.date);
        const busyToday = new Set(today.flatMap((s) => Object.values(s.assigned).flat()));
        // Na segunda passada: só quem não está em outra função do mesmo grupo.
        const busySameGroup = new Set(
          today.flatMap((s) =>
            Object.entries(s.assigned)
              .filter(([roleId]) => groupOf(roleId) === r.group)
              .flatMap(([, ids]) => ids),
          ),
        );
        const candidates = doc.people.filter(
          (p) =>
            p.roles.includes(r.id) &&
            !p.away.includes(service.date) &&
            !filled.includes(p.id) &&
            !(pass === "strict" ? busyToday : busySameGroup).has(p.id),
        );
        if (!candidates.length) break;
        const stats = (p: Person) => {
          const times = all().filter((s) => monthKey(s.date) === monthKey(service.date) && s.assigned[r.id]?.includes(p.id));
          const last = all()
            .filter((s) => s.date < service.date && s.assigned[r.id]?.includes(p.id))
            .map((s) => s.date)
            .sort()
            .pop();
          // Evita a mesma função em dois cultos do mesmo tipo seguidos.
          const row = neighbor(doc, service, r.id, p.id, all()) ? 1 : 0;
          return { count: times.length, row, gap: last ? daysBetween(last, service.date) : 9_999 };
        };
        candidates.sort((a, b) => {
          const sa = stats(a);
          const sb = stats(b);
          return sa.count - sb.count || sa.row - sb.row || sb.gap - sa.gap || a.name.localeCompare(b.name, "pt-BR");
        });
        filled.push(candidates[0].id);
        service.assigned[r.id] = [...filled];
      }
    }
  }
  return result;
}

// --- Exportar ---------------------------------------------------------------

const names = (doc: RosterDoc, ids: string[] | undefined) =>
  (ids ?? [])
    .map((id) => doc.people.find((p) => p.id === id)?.name)
    .filter(Boolean)
    .join(" e ");

/** Texto pronto para colar no grupo do WhatsApp. */
export function whatsappText(doc: RosterDoc, services: Service[], title: string, church = ""): string {
  const lines = [`📅 *Escala de ${title}*${church ? ` — ${church}` : ""}`, ""];
  for (const s of sortServices(services)) {
    lines.push(`*${shortDate(s.date)} · ${shortTime(s.time)} · ${s.name}*${s.note ? ` (${s.note})` : ""}`);
    for (const r of serviceRoles(doc, s)) {
      const who = names(doc, s.assigned[r.id]);
      if (who) lines.push(`• ${r.name}: ${who}`);
    }
    lines.push("");
  }
  return lines.join("\n").trim() + "\n";
}

/** Planilha (CSV com ";", que o Excel em português abre direto). */
export function csv(doc: RosterDoc, services: Service[]): string {
  const sorted = sortServices(services);
  const roles = doc.roles.filter((r) => r.active && sorted.some((s) => s.roles.includes(r.id)));
  const cell = (v: string) => (/[;"\n]/.test(v) ? `"${v.replace(/"/g, '""')}"` : v);
  const rows = [["Data", "Horário", "Culto", "Observação", ...roles.map((r) => r.name)]];
  for (const s of sorted) {
    rows.push([shortDate(s.date), shortTime(s.time), s.name, s.note, ...roles.map((r) => (s.roles.includes(r.id) ? names(doc, s.assigned[r.id]) : ""))]);
  }
  return "﻿" + rows.map((r) => r.map(cell).join(";")).join("\r\n") + "\r\n";
}

/**
 * Para imprimir: uma tabela por tipo de culto (datas nas linhas, funções nas
 * colunas) e as programações especiais juntas no fim.
 */
export interface PrintRow {
  id: string;
  date: string;
  weekday: string;
  /** Nome do culto quando ele é especial ("Culto Jovem"); "-" num culto comum. */
  kind: string;
  time: string;
  cells: { name: string; pending: boolean }[];
  note: string;
}

/**
 * Para imprimir em lista (o modelo que os irmãos entendem de primeira): uma
 * linha por culto, em ordem de data, com as funções escolhidas.
 */
export function printList(doc: RosterDoc, services: Service[], roleIds: string[]): { title: string; roles: RosterRole[]; rows: PrintRow[] } {
  const sorted = sortServices(services);
  const used = doc.roles.filter((r) => r.active && sorted.some((s) => s.roles.includes(r.id)));
  const roles = roleIds.length ? used.filter((r) => roleIds.includes(r.id)) : used;
  const title = roles.length === 1 ? `Escala de ${roles[0].name}` : "Escala";
  const rows = sorted.map((s) => {
    const template = doc.templates.find((t) => t.id === s.templateId);
    const special = !template || s.name !== template.name;
    return {
      id: s.id,
      date: fullDate(s.date),
      weekday: weekdayLong(s.date),
      kind: special ? s.name : "-",
      time: s.time,
      cells: roles.map((r) => {
        if (!s.roles.includes(r.id)) return { name: "-", pending: false };
        const who = names(doc, s.assigned[r.id]);
        return who ? { name: who, pending: false } : { name: "(A definir)", pending: true };
      }),
      note: s.note.trim() || "-",
    };
  });
  return { title, roles, rows };
}

export function printTables(doc: RosterDoc, services: Service[]): { title: string; roles: RosterRole[]; services: Service[] }[] {
  const sorted = sortServices(services);
  const groups: { title: string; roles: RosterRole[]; services: Service[] }[] = [];
  const byTemplate = new Map<string, Service[]>();
  const extras: Service[] = [];
  for (const s of sorted) {
    if (s.templateId && doc.templates.some((t) => t.id === s.templateId)) {
      byTemplate.set(s.templateId, [...(byTemplate.get(s.templateId) ?? []), s]);
    } else {
      extras.push(s);
    }
  }
  const rolesOf = (list: Service[]) => doc.roles.filter((r) => r.active && list.some((s) => s.roles.includes(r.id)));
  // Muitas funções não cabem numa linha da folha: separa por grupo.
  const push = (title: string, list: Service[]) => {
    const roles = rolesOf(list);
    if (roles.length <= 9) return groups.push({ title, roles, services: list });
    for (const g of ["Culto", "Escola Sabatina", "Apoio"] as const) {
      const part = roles.filter((r) => r.group === g);
      if (part.length) groups.push({ title: `${title} — ${g}`, roles: part, services: list });
    }
  };
  for (const t of doc.templates) {
    const list = byTemplate.get(t.id);
    if (list?.length) push(`${t.name} · ${weekdayName(t.weekday)} ${shortTime(t.time)}`, list);
  }
  if (extras.length) push("Programações especiais", extras);
  return groups;
}

/** Os cultos de hoje com quem faz o quê (para o Culto de hoje). */
export function todayServices(doc: RosterDoc, today = isoDate(new Date())): Service[] {
  return sortServices(doc.services.filter((s) => s.date === today));
}

export const assignedNames = names;
