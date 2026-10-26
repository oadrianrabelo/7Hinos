// Momentos da liturgia e modelos de programa, para montar o culto rápido.

import type { SetItem } from "./types";

export type Part = Extract<SetItem, { kind: "part" }>;

/** Momentos comuns, na ordem em que costumam aparecer. */
export const COMMON_PARTS: { title: string; minutes?: number }[] = [
  { title: "Boas-vindas" },
  { title: "Oração inicial" },
  { title: "Leitura bíblica" },
  { title: "Momento infantil", minutes: 10 },
  { title: "Informativo Mundial das Missões" },
  { title: "Dízimos e ofertas" },
  { title: "Oração intercessora" },
  { title: "Música especial" },
  { title: "Sermão", minutes: 40 },
  { title: "Apelo" },
  { title: "Oração final" },
  { title: "Bênção final" },
  { title: "Avisos" },
];

export const TEMPLATES: { name: string; parts: { title: string; minutes?: number }[] }[] = [
  {
    name: "Culto divino",
    parts: [
      { title: "Boas-vindas" },
      { title: "Oração inicial" },
      { title: "Leitura bíblica" },
      { title: "Momento infantil", minutes: 10 },
      { title: "Dízimos e ofertas" },
      { title: "Oração intercessora" },
      { title: "Música especial" },
      { title: "Sermão", minutes: 40 },
      { title: "Bênção final" },
    ],
  },
  {
    name: "Escola Sabatina",
    parts: [
      { title: "Boas-vindas" },
      { title: "Oração inicial" },
      { title: "Informativo Mundial das Missões", minutes: 5 },
      { title: "Estudo da lição", minutes: 40 },
      { title: "Oração final" },
    ],
  },
  {
    name: "Culto jovem",
    parts: [
      { title: "Boas-vindas" },
      { title: "Oração inicial" },
      { title: "Louvor" },
      { title: "Dinâmica", minutes: 10 },
      { title: "Mensagem", minutes: 25 },
      { title: "Oração final" },
    ],
  },
];

let counter = 0;

/** Um momento novo, com id único (o mesmo momento pode aparecer duas vezes). */
export function makePart(title: string, minutes?: number | null, note = ""): Part {
  counter = (counter + 1) % 1296;
  const id = `${Date.now().toString(36)}${counter.toString(36).padStart(2, "0")}`;
  return { kind: "part", id, title: title.trim() || "Momento", minutes: minutes ?? null, ...(note ? { note } : {}) };
}
