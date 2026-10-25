// Quais músicas tocam sem internet. Pergunta ao backend em lotes, guarda a
// resposta e só mostra o aviso quando o computador está de fato sem conexão.

import { api } from "../api";
import { catalog } from "./catalog.svelte";

class Availability {
  #known = $state<Record<number, boolean>>({});
  #pending = new Set<number>();
  #timer: ReturnType<typeof setTimeout> | null = null;
  online = $state(typeof navigator === "undefined" ? true : navigator.onLine);

  constructor() {
    if (typeof window === "undefined") return;
    window.addEventListener("online", () => (this.online = true));
    window.addEventListener("offline", () => (this.online = false));
  }

  /** Sem internet agora (o sistema diz que não, ou o LouvorJA não responde). */
  get offline() {
    return !this.online || !!catalog.status?.lastError;
  }

  /** `true`/`false` quando já sabemos; `undefined` enquanto pergunta. */
  get(id: number): boolean | undefined {
    const known = this.#known[id];
    if (known === undefined) this.#ask(id);
    return known;
  }

  clear() {
    this.#known = {};
  }

  #ask(id: number) {
    this.#pending.add(id);
    this.#timer ??= setTimeout(async () => {
      const ids = [...this.#pending];
      this.#pending.clear();
      this.#timer = null;
      const available = new Set(await api.songsAvailable(ids).catch(() => ids));
      for (const id of ids) this.#known[id] = available.has(id);
    }, 60);
  }
}

export const availability = new Availability();
