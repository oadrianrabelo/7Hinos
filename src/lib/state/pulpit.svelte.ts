// Recado e cronômetro para quem está no púlpito. O estado vive no palco
// (backend); aqui só montamos as mudanças.

import { api } from "../api";
import { elapsedMs } from "../timer";
import type { Pulpit, Timer } from "../types";
import { stage } from "./stage.svelte";

class PulpitControl {
  get current(): Pulpit {
    return stage.current.pulpit;
  }

  #set(patch: Partial<Pulpit>) {
    return api.setPulpit({ ...this.current, ...patch });
  }

  say(message: string) {
    const text = message.trim();
    return this.#set({ message: text || null });
  }

  clearMessage() {
    return this.#set({ message: null });
  }

  get running() {
    return this.current.timer?.startedAt != null;
  }

  start(limitMs: number | null = this.current.timer?.limitMs ?? null) {
    const timer = this.current.timer;
    if (timer?.startedAt != null) return;
    return this.#setTimer({ elapsedMs: timer?.elapsedMs ?? 0, startedAt: Date.now(), limitMs });
  }

  pause() {
    const timer = this.current.timer;
    if (!timer || timer.startedAt === null) return;
    return this.#setTimer({ ...timer, elapsedMs: elapsedMs(timer, Date.now()), startedAt: null });
  }

  /** Zera e esconde o cronômetro do retorno. */
  reset() {
    return this.#set({ timer: null });
  }

  setLimit(limitMs: number | null) {
    const timer = this.current.timer;
    return this.#setTimer(timer ? { ...timer, limitMs } : { elapsedMs: 0, startedAt: null, limitMs });
  }

  #setTimer(timer: Timer) {
    return this.#set({ timer });
  }
}

export const pulpit = new PulpitControl();
