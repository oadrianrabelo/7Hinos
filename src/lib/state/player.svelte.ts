// O player de hinos: toca o áudio na janela de controle e decide qual slide
// está no palco. É o único lugar que manda slides para as telas.

import { api, errorMessage } from "../api";
import { mediaSrc, mp3Fallback } from "../media";
import { slideIndexAt, slideTimes, type PlayMode } from "../slides";
import type { Scene, Song } from "../types";
import { live } from "./live.svelte";
import { outputs } from "./outputs.svelte";
import { toasts } from "./toasts.svelte";
import { video } from "./video.svelte";

const audio = new Audio();
audio.preload = "auto";

class Player {
  song = $state<Song | null>(null);
  /** Música escolhida durante uma apresentação, esperando a vez. */
  queued = $state<Song | null>(null);
  mode = $state<PlayMode>("voice");
  index = $state(0);
  /** A música está no palco (as telas mostram os slides). */
  live = $state(false);
  /** Tela preta temporária, sem parar a música. */
  blackout = $state(false);
  playing = $state(false);
  loading = $state(false);
  time = $state(0);
  duration = $state(0);
  volume = $state(1);

  #times: (number | null)[] = [];
  #frame = 0;

  constructor() {
    audio.addEventListener("play", () => {
      this.playing = true;
      this.#tick();
    });
    audio.addEventListener("pause", () => (this.playing = false));
    audio.addEventListener("loadedmetadata", () => (this.duration = audio.duration || 0));
    audio.addEventListener("waiting", () => (this.loading = true));
    audio.addEventListener("playing", () => (this.loading = false));
    audio.addEventListener("ended", () => this.#finished());
    audio.addEventListener("error", () => {
      const src = audio.getAttribute("src");
      if (!src) return;
      const fallback = mp3Fallback(src);
      if (fallback) {
        const at = audio.currentTime;
        audio.src = fallback;
        audio.currentTime = at;
        void audio.play().catch(() => {});
        return;
      }
      this.loading = false;
      toasts.error("Não foi possível tocar o áudio. Confira a internet ou baixe o álbum.");
      this.#switchToLyrics();
    });
  }

  get slide() {
    return this.song?.slides[this.index] ?? null;
  }

  get nextSlide() {
    return this.song?.slides[this.index + 1] ?? null;
  }

  get synced() {
    return this.#times.some((t) => t !== null);
  }

  get hasAudio() {
    return this.mode !== "lyrics";
  }

  /**
   * Carrega uma música no painel sem projetar. Durante uma apresentação ela
   * fica na fila, para um clique sem querer não interromper o culto.
   */
  async open(id: number) {
    if (this.song?.id === id) return this.song;
    try {
      const song = await api.song(id);
      if (this.live) this.queued = song;
      else this.#load(song);
      return song;
    } catch (e) {
      toasts.error(errorMessage(e));
      return null;
    }
  }

  /** Recarrega a música do painel depois de editada (se não estiver no ar). */
  async refresh(id: number) {
    if (this.song?.id !== id || this.live) return;
    try {
      this.#load(await api.song(id));
    } catch (e) {
      toasts.error(errorMessage(e));
    }
  }

  /** Encerra a apresentação atual e traz a música da fila para o painel. */
  openQueued() {
    const song = this.queued;
    if (!song) return;
    this.stop();
    this.#load(song);
  }

  /** Troca já para a música da fila e começa a apresentar (cantado, se tiver). */
  async startQueued() {
    const song = this.queued;
    if (!song) return;
    this.openQueued();
    await this.start(song.audio ? "voice" : "lyrics");
  }

  /** Troca de música e já começa a apresentar (modo simples). */
  async present(song: Song, mode: PlayMode) {
    if (this.live) this.stop();
    this.#load(song);
    await this.start(mode);
  }

  #load(song: Song) {
    this.song = song;
    this.queued = null;
    this.index = 0;
    this.mode = song.audio ? "voice" : "lyrics";
  }

  /** Começa a apresentação no modo escolhido. */
  async start(mode: PlayMode) {
    const song = this.song;
    if (!song) return;
    // Um vídeo no telão cobriria a letra e tocaria junto: sai antes.
    if (video.state) {
      video.after = null;
      video.stop();
    }
    const source = mode === "voice" ? song.audio : mode === "playback" ? song.instrumental : null;
    this.mode = source ? mode : "lyrics";
    this.#times = slideTimes(song.slides, this.mode);
    this.index = 0;
    live.claim("song");
    this.live = true;
    this.blackout = false;
    this.time = 0;
    this.duration = (song.durationMs ?? 0) / 1000;
    this.#push();
    void outputs.ensureOpen();
    void api.songPlayed(song.id).catch(() => {});

    audio.pause();
    if (source) {
      this.loading = true;
      audio.src = mediaSrc(source) ?? "";
      audio.currentTime = 0;
      audio.volume = this.volume;
      try {
        await audio.play();
      } catch {
        // O evento "error" do elemento já avisa a pessoa.
      }
    } else {
      audio.removeAttribute("src");
      audio.load();
    }
  }

  stop() {
    if (!this.live) return;
    audio.pause();
    audio.removeAttribute("src");
    audio.load();
    cancelAnimationFrame(this.#frame);
    this.live = false;
    this.blackout = false;
    this.playing = false;
    this.loading = false;
    this.time = 0;
    this.index = 0;
    this.#push();
    live.release("song");
  }

  togglePlay() {
    if (!this.live || !this.hasAudio) return;
    if (audio.paused) void audio.play();
    else audio.pause();
  }

  toggleBlackout() {
    if (!this.live) return;
    this.blackout = !this.blackout;
    this.#push();
  }

  next() {
    this.go(this.index + 1);
  }

  previous() {
    this.go(this.index - 1);
  }

  /** Vai para um slide. Com áudio sincronizado, a música acompanha. */
  go(index: number) {
    const song = this.song;
    if (!song) return;
    const target = Math.max(0, Math.min(song.slides.length - 1, index));
    if (!this.live) {
      this.index = target;
      return;
    }
    const time = this.#times[target];
    if (this.hasAudio && time !== null && time !== undefined) {
      audio.currentTime = time / 1000 + 0.05;
    }
    this.index = target;
    this.blackout = false;
    this.#push();
  }

  seek(seconds: number) {
    if (!this.hasAudio) return;
    audio.currentTime = Math.max(0, Math.min(seconds, this.duration || seconds));
    this.#sync();
  }

  setVolume(volume: number) {
    this.volume = Math.max(0, Math.min(1, volume));
    audio.volume = this.volume;
  }

  #tick = () => {
    cancelAnimationFrame(this.#frame);
    const loop = () => {
      this.#sync();
      if (!audio.paused) this.#frame = requestAnimationFrame(loop);
    };
    loop();
  };

  #sync() {
    this.time = audio.currentTime;
    const index = slideIndexAt(this.#times, audio.currentTime * 1000);
    if (index !== null && index !== this.index) {
      this.index = index;
      this.blackout = false;
      this.#push();
    }
  }

  #finished() {
    this.playing = false;
    this.blackout = true;
    this.#push();
  }

  #switchToLyrics() {
    audio.removeAttribute("src");
    this.mode = "lyrics";
    this.#times = slideTimes([], "lyrics");
    this.playing = false;
  }

  #push() {
    const slide = this.slide;
    // Fora do ar, o telão volta à tela de espera; "tela preta" é escolha explícita.
    const scene: Scene = !this.live
      ? { kind: "idle" }
      : this.blackout || !slide || !this.song
        ? { kind: "black" }
        : {
            kind: "slide",
            slide: $state.snapshot(slide),
            title: this.song.title,
            next: $state.snapshot(this.nextSlide),
          };
    void api.show(scene);
  }
}

export const player = new Player();
live.register("song", player);
