// Controle dos vídeos projetados. O estado vai para o palco e cada tela
// escolhida toca a sua cópia, sincronizada pelo relógio.

import { api } from "../api";
import { expectedPosition } from "../videoSync";
import type { VideoItem, VideoState } from "../types";

class VideoControl {
  state = $state<VideoState | null>(null);
  /** Preenchidos pela prévia na janela de controle. */
  duration = $state(0);
  position = $state(0);

  async init() {
    this.state = (await api.stage()).video;
  }

  play(item: VideoItem, targets: string[], options: { looping?: boolean; volume?: number } = {}) {
    if (targets.length === 0) return;
    this.duration = 0;
    this.#commit({
      path: item.path,
      title: item.title,
      playing: true,
      position: 0,
      updatedAt: Date.now(),
      looping: options.looping ?? this.state?.looping ?? false,
      volume: options.volume ?? this.state?.volume ?? 1,
      targets,
    });
  }

  toggle() {
    const s = this.state;
    if (!s) return;
    // Se acabou, apertar play recomeça do início.
    const now = this.#now(s);
    const atEnd = !s.looping && this.duration > 0 && now >= this.duration - 0.05;
    this.#commit({ ...s, playing: !s.playing, position: atEnd ? 0 : now, updatedAt: Date.now() });
  }

  seek(seconds: number) {
    const s = this.state;
    if (!s) return;
    this.#commit({ ...s, position: Math.max(0, seconds), updatedAt: Date.now() });
  }

  setLooping(looping: boolean) {
    const s = this.state;
    if (s) this.#commit({ ...s, position: this.#now(s), updatedAt: Date.now(), looping });
  }

  setVolume(volume: number) {
    const s = this.state;
    if (s) this.#commit({ ...s, position: this.#now(s), updatedAt: Date.now(), volume });
  }

  setTargets(targets: string[]) {
    const s = this.state;
    if (!s) return;
    if (targets.length === 0) return this.stop();
    this.#commit({ ...s, position: this.#now(s), updatedAt: Date.now(), targets });
  }

  /** O vídeo terminou (sem repetição): fica parado no fim. */
  ended() {
    const s = this.state;
    if (s?.playing && !s.looping) {
      this.#commit({ ...s, playing: false, position: this.duration, updatedAt: Date.now() });
    }
  }

  stop() {
    this.state = null;
    this.position = 0;
    void api.setVideo(null);
  }

  /** Mantém em dia com mudanças feitas por outra janela ou ao fechar telas. */
  receive(state: VideoState | null) {
    this.state = state;
  }

  #now(s: VideoState) {
    return expectedPosition(s, Date.now(), this.duration || Infinity);
  }

  #commit(state: VideoState) {
    this.state = state;
    void api.setVideo($state.snapshot(state));
  }
}

export const video = new VideoControl();
