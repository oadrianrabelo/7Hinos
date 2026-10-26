// Apresentações (PDF, imagens, lição): importar e passar as páginas no telão.

import { api, errorMessage } from "../api";
import { renderPdf } from "../pdf";
import type { Deck, Lesson, PageOverlay, PageTransition, Scene, Stroke } from "../types";
import { live } from "./live.svelte";
import { outputs } from "./outputs.svelte";
import { toasts } from "./toasts.svelte";
import { video } from "./video.svelte";

export type DeckTool = "none" | "zoom" | "hand" | "spotlight" | "marker" | "pen" | "laser" | "eraser";

const TRANSITION_KEY = "7hinos.deckTransition";

function savedTransition(): PageTransition {
  try {
    const t = localStorage.getItem(TRANSITION_KEY);
    if (t === "none" || t === "fade" || t === "slide" || t === "zoom") return t;
  } catch {
    // Sem armazenamento: usa o padrão.
  }
  return "fade";
}

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

  // Ferramentas do apresentador. Os desenhos ficam guardados por página
  // enquanto a apresentação estiver aberta; zoom, foco e laser valem só para
  // a página atual.
  tool = $state<DeckTool>("none");
  penColor = $state("#ff3b30");
  markerColor = $state("#ffe14d");
  transition = $state<PageTransition>(savedTransition());
  marks = $state<Record<number, Stroke[]>>({});
  zoom = $state<PageOverlay["zoom"]>(null);
  spotlight = $state<PageOverlay["spotlight"]>(null);
  laser = $state<PageOverlay["laser"]>(null);
  /**
   * "Ajustar à largura" vale para as próximas páginas também (PDF em pé):
   * guarda a escala e cada página nova começa do alto, na largura da tela.
   */
  widthScale = $state<number | null>(null);
  #direction = 1;
  #frame = 0;
  #progressTimer: ReturnType<typeof setTimeout> | undefined;

  get overlay(): PageOverlay {
    return {
      zoom: this.zoom,
      spotlight: this.spotlight,
      laser: this.laser,
      strokes: this.marks[this.index] ?? [],
      transition: this.transition,
      dir: this.#direction,
    };
  }

  setTransition(t: PageTransition) {
    this.transition = t;
    try {
      localStorage.setItem(TRANSITION_KEY, t);
    } catch {
      // Só não lembra na próxima vez.
    }
  }

  /** Vai para a página `i` (no telão, se estiver no ar), guardando onde parou. */
  go(i: number) {
    const deck = this.deck;
    if (!deck?.pages.length) return;
    const target = Math.max(0, Math.min(deck.pages.length - 1, i));
    if (target !== this.index) this.#turnTo(target);
    this.index = target;
    this.#saveProgress();
    if (this.live) this.#push();
  }

  /** Ao trocar de página: zera foco e laser; o zoom só fica se for "na largura". */
  #turnTo(target: number) {
    this.#direction = target > this.index ? 1 : -1;
    this.zoom = this.widthScale ? { x: 0.5, y: 0, scale: this.widthScale } : null;
    this.spotlight = null;
    this.laser = null;
  }

  #saveProgress() {
    const deck = this.deck;
    if (!deck) return;
    clearTimeout(this.#progressTimer);
    const page = this.index;
    this.#progressTimer = setTimeout(() => {
      void api
        .deckProgress(deck.id, page)
        .then((saved) => {
          this.list = this.list.map((d) => (d.id === saved.id ? saved : d));
        })
        .catch(() => {});
    }, 600);
  }

  // Desenho e laser mudam muitas vezes por segundo: manda no máximo um
  // quadro por vez para as telas.
  pushOverlay() {
    if (!this.live || this.#frame) return;
    this.#frame = requestAnimationFrame(() => {
      this.#frame = 0;
      this.#push();
    });
  }

  setZoom(zoom: PageOverlay["zoom"]) {
    this.zoom = zoom;
    // Voltar à página inteira desliga o "na largura" das próximas páginas.
    if (!zoom) this.widthScale = null;
    this.pushOverlay();
  }

  /** Página em pé na largura da tela, e as próximas também. */
  fitToWidth(zoom: PageOverlay["zoom"]) {
    this.widthScale = zoom?.scale ?? null;
    this.zoom = zoom;
    this.pushOverlay();
  }

  setSpotlight(spotlight: PageOverlay["spotlight"]) {
    this.spotlight = spotlight;
    this.pushOverlay();
  }

  setLaser(laser: PageOverlay["laser"]) {
    this.laser = laser;
    this.pushOverlay();
  }

  /** Começa um traço novo (caneta ou marca-texto) na página atual. */
  beginStroke(stroke: Stroke) {
    this.marks[this.index] = [...(this.marks[this.index] ?? []), stroke];
    this.pushOverlay();
  }

  /** Atualiza o último traço enquanto a pessoa desenha. */
  extendStroke(points: [number, number][]) {
    const list = this.marks[this.index];
    if (!list?.length) return;
    this.marks[this.index] = [...list.slice(0, -1), { ...list[list.length - 1], points }];
    this.pushOverlay();
  }

  eraseStroke(i: number) {
    const list = this.marks[this.index] ?? [];
    this.marks[this.index] = list.filter((_, j) => j !== i);
    this.pushOverlay();
  }

  undo() {
    const list = this.marks[this.index] ?? [];
    if (!list.length) return;
    this.marks[this.index] = list.slice(0, -1);
    this.pushOverlay();
  }

  /** Limpa desenhos, foco e zoom da página atual. */
  clearPage() {
    this.marks[this.index] = [];
    this.zoom = null;
    this.widthScale = null;
    this.spotlight = null;
    this.laser = null;
    this.pushOverlay();
  }

  get hasMarks() {
    return (this.marks[this.index]?.length ?? 0) > 0 || !!this.zoom || !!this.spotlight;
  }

  async load() {
    [this.list, this.#dir] = await Promise.all([api.decks(), api.decksDir()]);
  }

  pagePath(deck: Deck, i: number): string {
    return `${this.#dir}/${deck.id}/${deck.pages[i]}`;
  }

  async importPdf(path: string, title = fileTitle(path), source = "pdf", quiet = false): Promise<Deck | null> {
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
      if (!quiet) toasts.show(`“${title}” pronta: ${deck.pages.length} páginas.`, "success");
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

  async importLesson(quarter: string, lesson: Lesson, quiet = false): Promise<Deck | null> {
    const title = `Lição ${lesson.number} — ${lesson.title}`;
    this.importing = { title, done: 0, total: 0 };
    try {
      const path = await api.downloadLesson(quarter, lesson);
      return await this.importPdf(path, title, "lesson", quiet);
    } catch (e) {
      this.importing = null;
      toasts.error(errorMessage(e));
      return null;
    }
  }

  /** Lote em andamento ("Baixar todas"), para o botão mostrar o progresso. */
  batch = $state<{ done: number; total: number } | null>(null);

  /**
   * Baixa e prepara todas as lições do trimestre que ainda não estão aqui,
   * uma de cada vez. Se uma falhar, segue com as outras.
   */
  async importAllLessons(quarter: string, lessons: Lesson[], ready: (l: Lesson) => boolean) {
    const missing = lessons.filter((l) => !ready(l));
    if (!missing.length) return toasts.show("Todas as lições do trimestre já estão aqui.", "success");
    let ok = 0;
    this.batch = { done: 0, total: missing.length };
    try {
      for (const lesson of missing) {
        if (await this.importLesson(quarter, lesson, true)) ok++;
        this.batch = { done: this.batch.done + 1, total: missing.length };
      }
    } finally {
      this.batch = null;
    }
    toasts.show(
      ok === missing.length ? `${ok} lições prontas para usar sem internet.` : `${ok} de ${missing.length} lições prontas; tente de novo as que faltaram.`,
      ok === missing.length ? "success" : "info",
    );
  }

  /** A lição aberta por último, para continuar de onde parou. */
  get lastLesson(): Deck | null {
    return this.list.filter((d) => d.source === "lesson" && d.openedAt > 0).sort((a, b) => b.openedAt - a.openedAt)[0] ?? null;
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

  /** Abre no painel, sem ir para o telão, na página onde parou da última vez. */
  open(deck: Deck) {
    if (this.deck?.id !== deck.id) {
      this.marks = {};
      this.zoom = null;
      this.spotlight = null;
      this.laser = null;
      this.widthScale = null;
      this.tool = "none";
      const resume = deck.lastPage > 0 && deck.lastPage < deck.pages.length ? deck.lastPage : 0;
      this.index = resume;
      if (resume) {
        toasts.show(`Continuando da página ${resume + 1}, onde parou.`, "info", { label: "Do começo", run: () => this.go(0) }, 6000);
      }
    }
    this.deck = deck;
    this.#saveProgress();
  }

  /** Mostra a página `i` no telão. */
  show(i: number) {
    const deck = this.deck;
    if (!deck?.pages.length) return;
    const target = Math.max(0, Math.min(deck.pages.length - 1, i));
    if (target !== this.index) this.#turnTo(target);
    this.index = target;
    this.#saveProgress();
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
          overlay: this.overlay,
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
