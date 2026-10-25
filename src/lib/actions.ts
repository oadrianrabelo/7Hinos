// Ações de alto nível compartilhadas pelas páginas, para que cada modo de
// interface decida o que "escolher uma música" significa.

import { nav } from "./state/nav.svelte";
import { player } from "./state/player.svelte";
import { settings } from "./state/settings.svelte";
import type { SongSummary } from "./types";

/** Modo simples abre a página da música; o completo carrega no painel. */
export function selectSong(song: SongSummary) {
  if (settings.current.mode === "simple") nav.push({ name: "song", id: song.id });
  else void player.open(song.id);
}

/** Toca direto com voz (duplo clique / Enter no modo completo). */
export async function playSong(song: SongSummary) {
  if (settings.current.mode === "simple") return selectSong(song);
  // Durante uma apresentação a música vai para a fila, nunca interrompe.
  if (player.live) return void player.open(song.id);
  const loaded = await player.open(song.id);
  if (loaded) await player.start(loaded.audio ? "voice" : "lyrics");
}
