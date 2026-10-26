// A programação do culto como lista pura: cada operação devolve uma lista
// nova, que vai inteira para as preferências.

import type { SetItem } from "./types";
import { tagInfo } from "./videoTags";

const weekly = (tag: string | null | undefined) => (tagInfo(tag)?.weekly ? tag : null);

export function sameItem(a: SetItem, b: SetItem): boolean {
  if (a.kind === "song" && b.kind === "song") return a.id === b.id;
  if (a.kind === "deck" && b.kind === "deck") return a.id === b.id;
  if (a.kind === "part" && b.kind === "part") return a.id === b.id;
  if (a.kind === "video" && b.kind === "video") {
    // "Informativo de hoje" é um lugar na programação, não um arquivo.
    const wa = weekly(a.tag);
    const wb = weekly(b.tag);
    return wa || wb ? wa === wb : a.path === b.path;
  }
  return false;
}

export function contains(list: SetItem[], item: SetItem): boolean {
  return list.some((i) => sameItem(i, item));
}

/** Acrescenta no fim; o mesmo hino ou vídeo não entra duas vezes. */
export function addItem(list: SetItem[], item: SetItem): SetItem[] {
  return contains(list, item) ? list : [...list, item];
}

/** Acrescenta momentos da liturgia (que podem se repetir: duas orações). */
export function addParts(list: SetItem[], parts: SetItem[]): SetItem[] {
  return [...list, ...parts];
}

/** Muda um item no lugar (a anotação, o tempo de um momento). */
export function updateAt(list: SetItem[], index: number, patch: Partial<SetItem>): SetItem[] {
  return list.map((item, i) => (i === index ? ({ ...item, ...patch } as SetItem) : item));
}

/** Chave estável para listas na tela. */
export function itemKey(item: SetItem): string {
  if (item.kind === "video") return `video|${item.path}|${item.tag ?? ""}`;
  return `${item.kind}|${item.id}`;
}

export function removeAt(list: SetItem[], index: number): SetItem[] {
  return list.filter((_, i) => i !== index);
}

/** Troca de lugar com o vizinho (delta -1 sobe, +1 desce). */
export function moveItem(list: SetItem[], index: number, delta: number): SetItem[] {
  const target = index + delta;
  if (index < 0 || index >= list.length || target < 0 || target >= list.length) return list;
  const next = [...list];
  [next[index], next[target]] = [next[target], next[index]];
  return next;
}

/** O que vem depois do item atual na programação, se houver. */
export function nextAfter(list: SetItem[], current: SetItem | null): SetItem | null {
  if (!current) return null;
  const index = list.findIndex((i) => sameItem(i, current));
  return index >= 0 ? (list[index + 1] ?? null) : null;
}
