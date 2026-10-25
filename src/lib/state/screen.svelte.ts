// Avisos, relógio e contagem regressiva no telão.

import { api } from "../api";
import type { Notice } from "../types";
import { live } from "./live.svelte";
import { outputs } from "./outputs.svelte";
import { settings } from "./settings.svelte";

class ScreenPresenter {
  notice = $state<Notice | null>(null);
  blackout = $state(false);

  /** Projeta um aviso salvo. */
  showNotice(notice: Notice) {
    live.claim("text");
    this.notice = notice;
    this.blackout = false;
    void outputs.ensureOpen();
    this.#pushNotice();
  }

  showClock() {
    live.claim("clock");
    void outputs.ensureOpen();
    void api.show({ kind: "clock", until: null, label: "" });
  }

  /** Contagem regressiva até `until` (epoch em ms). */
  startCountdown(until: number, label: string) {
    live.claim("clock");
    void outputs.ensureOpen();
    void api.show({ kind: "clock", until, label });
  }

  /** Volta à tela de espera (ou preta, se pedido). */
  rest(black = false) {
    live.release("text");
    live.release("clock");
    this.notice = null;
    void api.show({ kind: black ? "black" : "idle" });
  }

  #pushNotice() {
    if (!this.notice) return;
    void api.show(
      this.blackout
        ? { kind: "black" }
        : {
            kind: "slide",
            title: this.notice.title,
            slide: {
              kind: "text",
              text: this.notice.body,
              aux: this.notice.title || null,
              image: null,
              imagePosition: 5,
              repeat: false,
              timeMs: null,
              instrumentalTimeMs: null,
            },
          },
    );
  }

  // Atalhos de teclado enquanto um aviso ou o relógio está no ar.
  readonly presenter = {
    stop: () => {
      this.notice = null;
      // Quem assumiu o palco já mandou a própria cena.
    },
    next: () => {},
    previous: () => {},
    toggleBlackout: () => {
      if (live.source !== "text") return;
      this.blackout = !this.blackout;
      this.#pushNotice();
    },
  };

  async saveNotice(notice: Notice) {
    const list = settings.current.notices;
    const id = notice.id || Math.max(0, ...list.map((n) => n.id)) + 1;
    const next = list.some((n) => n.id === id) ? list.map((n) => (n.id === id ? { ...notice, id } : n)) : [...list, { ...notice, id }];
    await settings.save({ notices: next });
    return id;
  }

  deleteNotice(id: number) {
    return settings.save({ notices: settings.current.notices.filter((n) => n.id !== id) });
  }
}

export const screen = new ScreenPresenter();
live.register("text", screen.presenter);
live.register("clock", screen.presenter);
