import { invoke } from "@tauri-apps/api/core";
import type { Media } from "./types";

// Arquivos do computador chegam à interface pelo servidor local de mídia do
// backend (ver `media_server.rs`): o WebView do Linux não toca áudio e vídeo
// pelo protocolo próprio do Tauri, e por HTTP funciona em todo lugar.
let base = "";
let userDir = "";

/** Carregado uma vez, antes de montar a interface. */
export async function initMedia() {
  [base, userDir] = await Promise.all([
    invoke<string>("media_base").catch(() => ""),
    invoke<string>("user_media_dir").catch(() => ""),
  ]);
}

/** Um arquivo da igreja (logo, fundo) guardado dentro do app. */
export function userMediaSrc(name: string | null | undefined): string | null {
  return name && userDir ? fileSrc(`${userDir}/${name}`) : null;
}

export function fileSrc(path: string): string {
  return `${base}/${encodeURIComponent(path)}`;
}

/** Arquivo no computador tem preferência; sem ele, usa o endereço da internet. */
export function mediaSrc(media: Media | null | undefined): string | null {
  if (!media) return null;
  return media.path ? fileSrc(media.path) : media.url;
}

/**
 * O LouvorJA guarda o MP3 original de cada áudio publicado em Opus. Quando o
 * sistema não toca Opus (macOS antigo, Linux sem plugins), tentamos o MP3.
 */
export function mp3Fallback(src: string): string | null {
  const match = /^https:\/\/files\.louvorja\.com\.br\/(musics\/.+)\.opus$/i.exec(src);
  return match ? `https://mp3.louvorja.com.br/${match[1]}.mp3` : null;
}
