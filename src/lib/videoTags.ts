import type { SetItem, VideoItem } from "./types";

// Tipos de vídeo. Os semanais (Informativo Mundial das Missões, Provai e
// Vede) têm um só "de hoje" por vez e ganham botão próprio.

export interface VideoTag {
  id: string;
  label: string;
  /** Para etiquetas e listas curtas. */
  short: string;
  weekly: boolean;
  tint: string;
}

export const VIDEO_TAGS: VideoTag[] = [
  { id: "missoes", label: "Informativo Mundial das Missões", short: "Informativo das Missões", weekly: true, tint: "#34a853" },
  { id: "provai", label: "Provai e Vede", short: "Provai e Vede", weekly: true, tint: "#ff9500" },
  { id: "abertura", label: "Abertura", short: "Abertura", weekly: false, tint: "#1f6a90" },
  { id: "anuncio", label: "Anúncio", short: "Anúncio", weekly: false, tint: "#30b0c7" },
  { id: "alerta", label: "Alerta para os irmãos", short: "Alerta", weekly: false, tint: "#ff3b30" },
];

export const WEEKLY_TAGS = VIDEO_TAGS.filter((t) => t.weekly);

export function tagInfo(id: string | null | undefined): VideoTag | null {
  return VIDEO_TAGS.find((t) => t.id === id) ?? null;
}

const normalize = (s: string) =>
  s
    .normalize("NFD")
    .replace(/\p{Diacritic}/gu, "")
    .toLowerCase();

/** Adivinha o tipo semanal pelo nome do arquivo ("Informativo 25-10.mp4"). */
export function guessTag(path: string): string | null {
  const name = normalize(path.split(/[\\/]/).pop() ?? "");
  if (/informativo|missoes|mundial das missoes/.test(name)) return "missoes";
  if (/provai/.test(name)) return "provai";
  return null;
}

type VideoSetItem = Extract<SetItem, { kind: "video" }>;

/** O tipo de um vídeo do culto: o guardado no item ou o do vídeo na lista. */
export function itemTag(item: VideoSetItem, videos: VideoItem[]): VideoTag | null {
  return tagInfo(item.tag ?? videos.find((v) => v.path === item.path)?.tag);
}

/**
 * O vídeo a tocar: num tipo semanal, o "de hoje" daquele tipo (o culto pode ter
 * sido montado antes de chegar o vídeo da semana).
 */
export function resolveVideo(item: VideoSetItem, videos: VideoItem[]): VideoItem {
  const tag = tagInfo(item.tag);
  const today = tag?.weekly ? videos.find((v) => v.tag === tag.id) : undefined;
  return today ?? { path: item.path, title: item.title };
}
