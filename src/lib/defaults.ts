// Os valores que o 7Hinos traz de fábrica (iguais aos de settings.rs), para
// os botões "Voltar ao padrão" de cada estilo.

import type { MonitorOptions, ScreenStyle, SlideStyle } from "./types";

export const SLIDE_DEFAULTS: SlideStyle = {
  font: "condensed",
  bibleFont: "",
  uppercase: true,
  textScale: 1,
  showImages: true,
  transitionMs: 450,
};

export const SCREEN_DEFAULTS: ScreenStyle = {
  font: "condensed",
  textColor: "#ffffff",
  accentColor: "#ffc94a",
  background: "gradient",
  backgroundImage: null,
};

/** O que o retorno mostra (sem o recado, que tem o próprio botão). */
export const MONITOR_DEFAULTS = {
  showNext: true,
  showClock: true,
  showTitle: true,
  showBackground: false,
  textScale: 1,
} satisfies Partial<MonitorOptions>;

/** Aparência do recado do púlpito no retorno. */
export const MESSAGE_DEFAULTS = {
  messageFont: "",
  messageColor: "#000000",
  messageBackground: "#ffd36b",
} satisfies Partial<MonitorOptions>;

export const UI_DEFAULTS = { theme: "system", uiScale: 1 } as const;

/** Se `current` já tem todos os valores de `defaults`. */
export function isDefault<T extends object>(current: T, defaults: Partial<T>): boolean {
  return Object.entries(defaults).every(([key, value]) => {
    const now = current[key as keyof T] as unknown;
    return typeof value === "number" && typeof now === "number" ? Math.abs(now - value) < 1e-6 : now === value;
  });
}
