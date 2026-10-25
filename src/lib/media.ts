import { invoke } from "@tauri-apps/api/core";
import type { Media } from "./types";

// Arquivos do computador chegam à interface pelo servidor local de mídia do
// backend (ver `media_server.rs`): o WebView do Linux não toca áudio e vídeo
// pelo protocolo próprio do Tauri, e por HTTP funciona em todo lugar.
let base = "";

/** Carregado uma vez, antes de montar a interface. */
export async function initMedia() {
  base = await invoke<string>("media_base").catch(() => "");
}

export function fileSrc(path: string): string {
  return `${base}/${encodeURIComponent(path)}`;
}

/** Arquivo no computador tem preferência; sem ele, usa o endereço da internet. */
export function mediaSrc(media: Media | null | undefined): string | null {
  if (!media) return null;
  return media.path ? fileSrc(media.path) : media.url;
}
