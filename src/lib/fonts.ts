// Fontes dos slides: as embutidas no 7Hinos, as instaladas no computador e
// arquivos de fonte que a pessoa adicionar. Guardadas por nome:
//   "condensed" | "sans" | "serif"   (as três primeiras, de sempre)
//   "montserrat", "lora"…            (embutidas)
//   "system:Arial"                   (instalada no computador)
//   "file:minha-fonte.ttf"           (arquivo em user-media)

import { userMediaSrc } from "./media";

export interface BuiltinFont {
  id: string;
  name: string;
  family: string;
  kind: "condensada" | "sem serifa" | "com serifa" | "títulos";
}

export const BUILTIN_FONTS: BuiltinFont[] = [
  { id: "condensed", name: "Barlow Condensed", family: '"Barlow Condensed", "Arial Narrow", sans-serif', kind: "condensada" },
  { id: "oswald", name: "Oswald", family: '"Oswald Variable", "Arial Narrow", sans-serif', kind: "condensada" },
  { id: "bebas", name: "Bebas Neue", family: '"Bebas Neue", "Arial Narrow", sans-serif', kind: "títulos" },
  { id: "sans", name: "Inter", family: '"Inter Variable", system-ui, sans-serif', kind: "sem serifa" },
  { id: "montserrat", name: "Montserrat", family: '"Montserrat Variable", sans-serif', kind: "sem serifa" },
  { id: "nunito", name: "Nunito", family: '"Nunito Variable", sans-serif', kind: "sem serifa" },
  { id: "poppins", name: "Poppins", family: "Poppins, sans-serif", kind: "sem serifa" },
  { id: "serif", name: "Georgia", family: 'Georgia, "Times New Roman", serif', kind: "com serifa" },
  { id: "lora", name: "Lora", family: '"Lora Variable", Georgia, serif', kind: "com serifa" },
  { id: "playfair", name: "Playfair Display", family: '"Playfair Display Variable", Georgia, serif', kind: "com serifa" },
  { id: "merriweather", name: "Merriweather", family: "Merriweather, Georgia, serif", kind: "com serifa" },
  { id: "sourceserif", name: "Source Serif", family: '"Source Serif 4 Variable", Georgia, serif', kind: "com serifa" },
  { id: "robotoslab", name: "Roboto Slab", family: '"Roboto Slab Variable", Georgia, serif', kind: "com serifa" },
];

const FALLBACK = BUILTIN_FONTS[0].family;

/** Nome de família CSS para registrar um arquivo de fonte. */
const fileFamily = (file: string) => `7h-${file.replace(/[^a-z0-9]/gi, "-")}`;

/** O `font-family` CSS de uma fonte guardada. */
export function fontFamily(id: string | null | undefined): string {
  if (!id) return FALLBACK;
  if (id.startsWith("system:")) return `"${id.slice(7).replace(/"/g, "")}", ${FALLBACK}`;
  if (id.startsWith("file:")) return `"${fileFamily(id.slice(5))}", ${FALLBACK}`;
  return BUILTIN_FONTS.find((f) => f.id === id)?.family ?? FALLBACK;
}

/** Nome para mostrar na tela. */
export function fontName(id: string | null | undefined): string {
  if (!id) return BUILTIN_FONTS[0].name;
  if (id.startsWith("system:")) return id.slice(7);
  if (id.startsWith("file:")) return id.slice(5).replace(/\.(ttf|otf|woff2?)$/i, "");
  return BUILTIN_FONTS.find((f) => f.id === id)?.name ?? BUILTIN_FONTS[0].name;
}

const loaded = new Set<string>();

/**
 * Arquivos de fonte precisam ser registrados em cada janela que os usa (o
 * controle e cada tela). As embutidas e as do sistema já estão disponíveis.
 */
export function ensureFont(id: string | null | undefined) {
  if (!id?.startsWith("file:") || loaded.has(id) || typeof FontFace === "undefined") return;
  const src = userMediaSrc(id.slice(5));
  if (!src) return;
  loaded.add(id);
  const face = new FontFace(fileFamily(id.slice(5)), `url("${src}")`);
  face
    .load()
    .then((f) => document.fonts.add(f))
    .catch(() => loaded.delete(id));
}
