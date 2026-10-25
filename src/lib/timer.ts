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

export interface TimerView {
  /** "Faltam", "Passou" ou "Tempo" (sem tempo combinado). */
  label: string;
  text: string;
  /** Há tempo combinado: o retorno mostra em vermelho. */
  limited: boolean;
  warn: boolean;
  over: boolean;
}

/**
 * O que mostrar: com tempo combinado, quanto falta (e quanto passou depois
 * de estourar); sem, o tempo corrido.
 */
export function timerDisplay(timer: Timer, now: number): TimerView {
  const elapsed = elapsedMs(timer, now);
  if (timer.limitMs === null) return { label: "Tempo", text: timerText(elapsed), limited: false, warn: false, over: false };
  const left = timer.limitMs - elapsed;
  const over = left < 0;
  return {
    label: over ? "Passou" : "Faltam",
    text: timerText(Math.abs(left)),
    limited: true,
    warn: !over && left <= 5 * 60_000,
    over,
  };
}
