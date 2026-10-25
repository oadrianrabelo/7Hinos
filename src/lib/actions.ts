// Ações de alto nível compartilhadas pelas páginas, para que cada modo de
// interface decida o que "escolher uma música" significa.

import { nav } from "./state/nav.svelte";
import { outputs } from "./state/outputs.svelte";
import { player } from "./state/player.svelte";
import { settings } from "./state/settings.svelte";
import { video } from "./state/video.svelte";
import type { SetItem, SongSummary, VideoItem } from "./types";
import { resolveVideo } from "./videoTags";

/** Modo simples abre a página da música; o completo carrega no painel. */
export function selectSong(song: Pick<SongSummary, "id">) {
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

/**
 * Exibe um vídeo nas telas indicadas (ou em todas as abertas). Sem nenhuma
 * aberta, abre no projetor/TV; com uma tela só, abre numa janela.
 */
export async function playVideo(item: VideoItem, targets: string[] = []) {
  if (!outputs.isProjecting && !(await outputs.ensureOpen())) await outputs.open(null);
  const open = outputs.outputs.map((o) => o.label);
  const chosen = targets.filter((t) => open.includes(t));
  video.play(item, chosen.length ? chosen : outputs.audience.map((o) => o.label));
}

/** Abre um item da programação do culto. */
export function openSetItem(item: SetItem) {
  if (item.kind === "song") return selectSong({ id: item.id });
  return playVideo(resolveVideo(item, settings.current.videos));
}
