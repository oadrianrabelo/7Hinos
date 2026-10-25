// Cronômetro do pregador: funções puras, testadas.

import type { Timer } from "./types";

export function elapsedMs(timer: Timer, now: number): number {
  return timer.elapsedMs + (timer.startedAt !== null ? Math.max(0, now - timer.startedAt) : 0);
}

/** "12:05", "1:02:03"; com `-` quando passou do tempo combinado. */
export function timerText(ms: number): string {
  const negative = ms < 0;
  const total = Math.floor(Math.abs(ms) / 1000);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = String(total % 60).padStart(2, "0");
  const text = h > 0 ? `${h}:${String(m).padStart(2, "0")}:${s}` : `${String(m).padStart(2, "0")}:${s}`;
  return negative ? `-${text}` : text;
}

/**
 * O que mostrar: com tempo combinado, o que falta (negativo depois de
 * passar); sem, o tempo corrido.
 */
export function timerDisplay(timer: Timer, now: number): { text: string; over: boolean; warn: boolean } {
  const elapsed = elapsedMs(timer, now);
  if (timer.limitMs === null) return { text: timerText(elapsed), over: false, warn: false };
  const left = timer.limitMs - elapsed;
  return { text: timerText(left), over: left < 0, warn: left >= 0 && left <= 5 * 60_000 };
}
