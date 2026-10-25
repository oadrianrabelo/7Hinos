import { convertFileSrc } from "@tauri-apps/api/core";
import type { Media } from "./types";

/** Arquivo baixado tem preferência; sem ele, usa o endereço da internet. */
export function mediaSrc(media: Media | null | undefined): string | null {
  if (!media) return null;
  return media.path ? convertFileSrc(media.path) : media.url;
}

export function fileSrc(path: string): string {
  return convertFileSrc(path);
}
