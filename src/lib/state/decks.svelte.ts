// Apresentações (PDF, imagens, lição): importar e passar as páginas no telão.

import { api, errorMessage } from "../api";
import { renderPdf } from "../pdf";
import type { Deck, Lesson, Scene } from "../types";
import { live } from "./live.svelte";
import { outputs } from "./outputs.svelte";
import { toasts } from "./toasts.svelte";
import { video } from "./video.svelte";

const fileTitle = (path: string) =>
  (path.split(/[\\/]/).pop() ?? "Apresentação").replace(/\.[^.]+$/, "").replace(/[_-]+/g, " ").trim();

class Decks {
  list = $state<Deck[]>([]);
  #dir = "";
  /** A apresentação aberta no painel. */
  deck = $state<Deck | null>(null);
  index = $state(0);
  live = $state(false);
  blackout = $state(false);
  /** Importação em andamento, para mostrar o progresso. */
  importing = $state<{ title: string; done: number; total: number } | null>(null);

  async load() {
    [this.list, this.#dir] = await Promise.all([api.decks(), api.decksDir()]);
  }

  pagePath(deck: Deck, i: number): string {
    return `${this.#dir}/${deck.id}/${deck.pages[i]}`;
  }

  async importPdf(path: string, title = fileTitle(path), source = "pdf"): Promise<Deck | null> {
    this.importing = { title, done: 0, total: 0 };
    let deck: Deck | null = null;
    try {
      const data = await api.readPdf(path);
      deck = await api.createDeck(title, source);
      const id = deck.id;
      await renderPdf(data, async (jpeg, page, total) => {
        this.importing = { title, done: page, total };
        deck = await api.addDeckPage(id, jpeg, "jpg");
      });
      toasts.show(`“${title}” pronta: ${deck.pages.length} páginas.`, "success");
      return deck;
    } catch (e) {
      // Sem páginas, a apresentação pela metade não serve para nada.
      if (deck && !deck.pages.length) await api.deleteDeck(deck.id).catch(() => {});
      toasts.error(`Não deu para abrir o PDF: ${errorMessage(e)}`);
      return null;
    } finally {
      this.importing = null;
      await this.load();
    }
  }

  /** PowerPoint/Impress: vira PDF pelo LibreOffice, se houver. */
  async importOffice(path: string): Promise<Deck | null> {
    this.importing = { title: fileTitle(path), done: 0, total: 0 };
    try {
      const pdf = await api.officeToPdf(path);
      return await this.importPdf(pdf, fileTitle(path));
    } catch (e) {
      this.importing = null;
      toasts.error(errorMessage(e));
      return null;
    }
  }

  async importImages(paths: string[]): Promise<Deck | null> {
    try {
      const sorted = [...paths].sort((a, b) => a.localeCompare(b, "pt-BR", { numeric: true }));
      const deck = await api.importImages(paths.length === 1 ? fileTitle(paths[0]) : "Imagens", sorted);
      await this.load();
      return deck;
    } catch (e) {
      toasts.error(errorMessage(e));
      return null;
    }
  }

  async importLesson(quarter: string, lesson: Lesson): Promise<Deck | null> {
    const title = `Lição ${lesson.number} — ${lesson.title}`;
    this.importing = { title, done: 0, total: 0 };
    try {
      const path = await api.downloadLesson(quarter, lesson);
      return await this.importPdf(path, title, "lesson");
    } catch (e) {
      this.importing = null;
      toasts.error(errorMessage(e));
      return null;
    }
  }

  async rename(deck: Deck, title: string) {
    const renamed = await api.renameDeck(deck.id, title);
    if (this.deck?.id === deck.id) this.deck = renamed;
    await this.load();
  }

  async remove(deck: Deck) {
    if (this.deck?.id === deck.id) {
      this.stop();
      this.deck = null;
    }
    await api.deleteDeck(deck.id);
    await this.load();
  }

  /** Abre no painel, sem ir para o telão. */
  open(deck: Deck) {
    if (this.deck?.id !== deck.id) this.index = 0;
    this.deck = deck;
  }

  /** Mostra a página `i` no telão. */
  show(i: number) {
    const deck = this.deck;
    if (!deck?.pages.length) return;
    this.index = Math.max(0, Math.min(deck.pages.length - 1, i));
    if (!this.live) {
      if (video.state) video.stop();
      live.claim("deck");
      this.live = true;
      void outputs.ensureOpen();
    }
    this.blackout = false;
    this.#push();
  }

  next() {
    if (!this.live) return this.show(this.index);
    if (this.deck && this.index < this.deck.pages.length - 1) this.show(this.index + 1);
  }

  previous() {
    if (this.live && this.index > 0) this.show(this.index - 1);
  }

  toggleBlackout() {
    if (!this.live) return;
    this.blackout = !this.blackout;
    this.#push();
  }

  stop() {
    if (!this.live) return;
    this.live = false;
    this.blackout = false;
    live.release("deck");
    void api.show({ kind: "idle" });
  }

  #push() {
    const deck = this.deck;
    if (!deck) return;
    const scene: Scene = this.blackout
      ? { kind: "black" }
      : {
          kind: "image",
          src: this.pagePath(deck, this.index),
          title: deck.title,
          page: this.index + 1,
          total: deck.pages.length,
          next: this.index + 1 < deck.pages.length ? this.pagePath(deck, this.index + 1) : null,
        };
    void api.show(scene);
  }

  // Quem assume o palco (um hino, a Bíblia) encerra a apresentação.
  readonly presenter = {
    stop: () => {
      this.live = false;
      this.blackout = false;
    },
    next: () => this.next(),
    previous: () => this.previous(),
    toggleBlackout: () => this.toggleBlackout(),
  };
}

export const decks = new Decks();
live.register("deck", decks.presenter);
