import type { Slide } from "./types";

export type PlayMode = "voice" | "playback" | "lyrics";

/** Momento (ms) em que cada slide começa no modo escolhido, ou null se não houver. */
export function slideTimes(slides: Slide[], mode: PlayMode): (number | null)[] {
  if (mode === "lyrics") return slides.map(() => null);
  const times = slides.map((s) => (mode === "voice" ? s.timeMs : s.instrumentalTimeMs));
  // Sem tempos de verdade (tudo zero depois da capa) não há como sincronizar.
  const synced = times.slice(1).some((t) => t !== null && t > 0);
  return synced ? times : slides.map(() => null);
}

/** Índice do slide que deve estar na tela no instante `ms`. */
export function slideIndexAt(times: (number | null)[], ms: number): number | null {
  let index: number | null = null;
  for (let i = 0; i < times.length; i++) {
    const t = times[i];
    if (t === null) continue;
    if (t <= ms) index = i;
    else break;
  }
  return index;
}

/**
 * Tamanho da letra para caber na tela sem medir nada: largura pela linha mais
 * longa, altura pelo número de linhas. Devolve unidades de container (cqw/cqh).
 */
export function fitText(text: string, kind: Slide["kind"], scale = 1): { cqw: number; cqh: number } {
  const lines = text.split("\n").filter((l) => l.trim().length > 0);
  const count = Math.max(1, lines.length);
  const longest = Math.max(8, ...lines.map((l) => l.trim().length));
  const cap = kind === "title" ? 14 : 12;
  const byHeight = Math.min(cap, 78 / (count * 1.12));
  const byWidth = 90 / (longest * 0.5);
  return { cqw: round(byWidth * scale), cqh: round(byHeight * scale) };
}

const round = (n: number) => Math.round(n * 100) / 100;

/** Grade 3×3 do LouvorJA (1 = topo-esquerda, 5 = centro, 9 = base-direita). */
export function backgroundPosition(position: number): string {
  const p = position >= 1 && position <= 9 ? position - 1 : 4;
  const y = ["top", "center", "bottom"][Math.floor(p / 3)];
  const x = ["left", "center", "right"][p % 3];
  return `${x} ${y}`;
}
