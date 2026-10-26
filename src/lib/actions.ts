// Ações de alto nível compartilhadas pelas páginas, para que cada modo de
// interface decida o que "escolher uma música" significa.

import { decks } from "./state/decks.svelte";
import { nav } from "./state/nav.svelte";
import { toasts } from "./state/toasts.svelte";
import { outputs } from "./state/outputs.svelte";
import { player } from "./state/player.svelte";
import { settings } from "./state/settings.svelte";
import { setlist } from "./state/setlist.svelte";
import { video } from "./state/video.svelte";
import type { SetItem, SongSummary, VideoItem } from "./types";
import { resolveVideo } from "./videoTags";

/** Modo simples abre a página da música; o completo carrega no painel. */
export function selectSong(song: Pick<SongSummary, "id">) {
  if (settings.current.mode === "simple") nav.push({ name: "song", id: song.id });
  else void player.open(song.id);
}

/**
 * Toca direto com voz (duplo clique / Enter no modo completo). Durante uma
 * apresentação o primeiro duplo clique põe na fila, para um clique sem
 * querer não interromper o culto; o segundo, na mesma música, troca já.
 */
export async function playSong(song: Pick<SongSummary, "id">) {
  if (settings.current.mode === "simple") return selectSong(song);
  if (player.live) {
    if (player.queued?.id === song.id) return player.startQueued();
    const queued = await player.open(song.id);
    if (queued && player.queued?.id === song.id)
      toasts.show(`“${queued.title}” é o próximo. Clique duplo de novo para trocar já.`, "info", {
        label: "Começar já",
        run: () => void player.startQueued(),
      });
    return;
  }
  const loaded = await player.open(song.id);
  if (loaded) await player.start(loaded.audio ? "voice" : "lyrics");
}

/** Começa agora, mesmo com outra música no ar (duplo clique na programação). */
export async function playNow(item: SetItem) {
  if (item.kind !== "song" || settings.current.mode === "simple") return openSetItem(item);
  setlist.mark(item);
  if (player.live) {
    if (player.song?.id === item.id) return;
    if (player.queued?.id !== item.id) await player.open(item.id);
    return player.startQueued();
  }
  const loaded = await player.open(item.id);
  if (loaded) await player.start(loaded.audio ? "voice" : "lyrics");
}

/**
 * Exibe um vídeo nas telas indicadas (ou em todas as abertas). Sem nenhuma
 * aberta, abre no projetor/TV; com uma tela só, abre numa janela.
 */
export async function playVideo(item: VideoItem, targets: string[] = []) {
  if (!outputs.isProjecting && !(await outputs.ensureOpen())) await outputs.open(null);
  const open = outputs.outputs.map((o) => o.label);
  // Telões primeiro: o primeiro alvo é o que toca o som.
  const chosen = outputs.forVideo.map((o) => o.label).filter((l) => targets.includes(l) && open.includes(l));
  video.play(item, chosen.length ? chosen : outputs.forVideo.map((o) => o.label));
}

/** Abre um item da programação do culto. */
export async function openSetItem(item: SetItem) {
  setlist.mark(item);
  if (item.kind === "part") return setlist.startPart(item);
  if (item.kind === "song") return selectSong({ id: item.id });
  if (item.kind === "deck") return openDeck(item.id);
  return playVideo(resolveVideo(item, settings.current.videos));
}

/** Abre uma apresentação na página Apresentações (a primeira página vai já para o telão). */
export async function openDeck(id: string) {
  if (!decks.list.length) await decks.load();
  const deck = decks.list.find((d) => d.id === id);
  if (!deck) return toasts.error("Essa apresentação não existe mais. Importe de novo em Apresentações.");
  decks.open(deck);
  nav.push({ name: "decks" });
  decks.show(0);
}
