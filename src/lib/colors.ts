// Cores e fundos do telão (avisos, tela de espera, relógio, recado do púlpito).
//
// Um fundo é guardado como texto:
//   "#rrggbb"                cor lisa
//   "radial:#centro,#borda"  degradê circular, mais claro no meio
//   "gradient"               o azul do 7Hinos (de antes, igual a um radial)

export interface Swatch {
  value: string;
  label: string;
}

export const TEXT_COLORS: Swatch[] = [
  { value: "#ffffff", label: "Branco" },
  { value: "#f1e4c6", label: "Creme" },
  { value: "#ffe9a8", label: "Amarelo claro" },
  { value: "#d9dee5", label: "Cinza claro" },
  { value: "#cfe8ff", label: "Azul claro" },
  { value: "#111111", label: "Preto (para fundo claro)" },
];

export const ACCENT_COLORS: Swatch[] = [
  { value: "#ffc94a", label: "Dourado" },
  { value: "#ffffff", label: "Branco" },
  { value: "#f1e4c6", label: "Creme" },
  { value: "#8fd0f2", label: "Azul claro" },
  { value: "#ff8a80", label: "Vermelho claro" },
  { value: "#9be3a8", label: "Verde claro" },
  { value: "#c9b6ff", label: "Lilás" },
];

/** Degradês circulares: centro mais claro, bordas quase pretas. */
export const RADIAL_BACKGROUNDS: Swatch[] = [
  { value: "radial:#1d4560,#050b11", label: "Azul do 7Hinos" },
  { value: "radial:#2f6f9f,#06121e", label: "Azul céu" },
  { value: "radial:#16525a,#031012", label: "Petróleo" },
  { value: "radial:#1f4a35,#04100a", label: "Verde floresta" },
  { value: "radial:#5a1d2a,#100407", label: "Vinho" },
  { value: "radial:#552255,#0f0510", label: "Púrpura" },
  { value: "radial:#3f2a5c,#0a0613", label: "Roxo" },
  { value: "radial:#6b4c1a,#120c03", label: "Dourado" },
  { value: "radial:#4a3426,#0e0906", label: "Café" },
  { value: "radial:#3a3f47,#08090b", label: "Grafite" },
];

export const SOLID_BACKGROUNDS: Swatch[] = [
  { value: "#000000", label: "Preto" },
  { value: "#173b53", label: "Azul-marinho" },
  { value: "#0f2a44", label: "Azul-noite" },
  { value: "#4a1c24", label: "Vinho" },
  { value: "#1f3d2b", label: "Verde-escuro" },
  { value: "#2e1f45", label: "Roxo-escuro" },
  { value: "#3b2a1a", label: "Marrom" },
  { value: "#2a2d33", label: "Grafite" },
];

const HEX = /^#[0-9a-f]{6}$/i;

export const isHex = (value: string) => HEX.test(value);

/** Os dois tons de um degradê circular, ou null se não for um. */
export function radialStops(value: string): [string, string] | null {
  if (value === "gradient") return ["#1d4560", "#050b11"];
  const match = /^radial:(#[0-9a-f]{6}),(#[0-9a-f]{6})$/i.exec(value);
  return match ? [match[1], match[2]] : null;
}

export const radialValue = (center: string, edge: string) => `radial:${center},${edge}`;

/** O CSS de um fundo guardado. Valores desconhecidos viram preto. */
export function backgroundCss(value: string): string {
  const stops = radialStops(value);
  if (stops) {
    const [center, edge] = stops;
    return `radial-gradient(ellipse at 50% 35%, ${center} 0%, ${mix(center, edge, 0.35)} 70%, ${edge} 100%)`;
  }
  return isHex(value) ? value : "#000000";
}

/** Mistura duas cores #rrggbb (`t` = quanto da primeira). */
export function mix(a: string, b: string, t: number): string {
  const [ra, ga, ba] = rgb(a);
  const [rb, gb, bb] = rgb(b);
  return hex(ra * t + rb * (1 - t), ga * t + gb * (1 - t), ba * t + bb * (1 - t));
}

export function rgb(value: string): [number, number, number] {
  const n = parseInt(value.slice(1), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

export function hex(r: number, g: number, b: number): string {
  const part = (v: number) =>
    Math.round(Math.min(255, Math.max(0, v)))
      .toString(16)
      .padStart(2, "0");
  return `#${part(r)}${part(g)}${part(b)}`;
}

/** HSV (h 0–360, s e v 0–1) ↔ #rrggbb, para o seletor de cor. */
export function hsvToHex(h: number, s: number, v: number): string {
  const f = (n: number) => {
    const k = (n + h / 60) % 6;
    return v - v * s * Math.max(0, Math.min(k, 4 - k, 1));
  };
  return hex(f(5) * 255, f(3) * 255, f(1) * 255);
}

export function hexToHsv(value: string): [number, number, number] {
  const [r, g, b] = rgb(isHex(value) ? value : "#000000").map((c) => c / 255);
  const max = Math.max(r, g, b);
  const d = max - Math.min(r, g, b);
  let h = 0;
  if (d) {
    if (max === r) h = ((g - b) / d) % 6;
    else if (max === g) h = (b - r) / d + 2;
    else h = (r - g) / d + 4;
  }
  return [(h * 60 + 360) % 360, max ? d / max : 0, max];
}

/** Aceita "#abc", "abc", "#aabbcc" ou "aabbcc"; devolve #aabbcc ou null. */
export function normalizeHex(input: string): string | null {
  let v = input.trim().replace(/^#/, "").toLowerCase();
  if (/^[0-9a-f]{3}$/.test(v)) v = [...v].map((c) => c + c).join("");
  return /^[0-9a-f]{6}$/.test(v) ? `#${v}` : null;
}
