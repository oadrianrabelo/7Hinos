// A Bíblia na tela: navegar por livro/capítulo, projetar versículos e passar
// para o próximo com as setas (atravessando capítulos e livros).

import { api, errorMessage, on } from "../api";
import { formatReference, parseReference } from "../reference";
import type { BibleBook, BibleVersion, Scene, Slide, Verse } from "../types";
import { live } from "./live.svelte";
import { outputs } from "./outputs.svelte";
import { settings } from "./settings.svelte";
import { toasts } from "./toasts.svelte";

class BiblePresenter {
  versions = $state<BibleVersion[]>([]);
  books = $state<BibleBook[]>([]);
  version = $state<number | null>(null);
  book = $state<number | null>(null);
  chapter = $state<number | null>(null);
  verses = $state<Verse[]>([]);
  /** Versículo selecionado (índice em `verses`) e, opcionalmente, o fim do trecho. */
  index = $state<number | null>(null);
  rangeEnd = $state<number | null>(null);
  live = $state(false);
  blackout = $state(false);
  loaded = $state(false);
  downloading = $state(false);
  progress = $state<{ phase: string; done: number; total: number } | null>(null);
  #listening = false;

  get installed() {
    return this.versions.length > 0;
  }

  get bookInfo(): BibleBook | null {
    return this.books.find((b) => b.number === this.book) ?? null;
  }

  get versionInfo(): BibleVersion | null {
    return this.versions.find((v) => v.id === this.version) ?? null;
  }

  async load() {
    this.versions = await api.bibleVersions().catch(() => []);
    if (!this.versions.length) {
      this.loaded = true;
      return;
    }
    this.books = await api.bibleBooks().catch(() => []);
    const saved = settings.value?.bibleVersion;
    this.version = this.versions.some((v) => v.id === saved) ? saved! : this.versions[0].id;
    this.loaded = true;
  }

  async setVersion(id: number) {
    this.version = id;
    void settings.save({ bibleVersion: id });
    if (this.book && this.chapter) await this.open(this.book, this.chapter, this.index);
  }

  /** Abre um capítulo; com `verseIndex`, já seleciona um versículo. */
  async open(book: number, chapter: number, verseIndex: number | null = null) {
    if (this.version === null) return;
    this.book = book;
    this.chapter = chapter;
    this.verses = await api.bibleChapter(this.version, book, chapter).catch(() => []);
    this.index = verseIndex !== null && verseIndex < this.verses.length ? verseIndex : null;
    this.rangeEnd = null;
    if (this.live && this.index !== null) this.#push();
  }

  /** Vai para uma referência digitada. Devolve `false` se não entendeu. */
  async goTo(text: string, project = false): Promise<boolean> {
    const ref = parseReference(text, this.books);
    if (!ref) return false;
    await this.open(ref.book, ref.chapter);
    if (ref.verse !== null) {
      const start = this.verses.findIndex((v) => v.number === ref.verse);
      if (start >= 0) {
        this.index = start;
        const end = ref.verseEnd ? this.verses.findIndex((v) => v.number === ref.verseEnd) : -1;
        this.rangeEnd = end > start ? end : null;
        if (project) this.show(start, this.rangeEnd);
      }
    }
    return true;
  }

  /** Projeta um versículo (ou um trecho até `end`). */
  show(index: number, end: number | null = null) {
    if (!this.verses[index]) return;
    this.index = index;
    this.rangeEnd = end;
    if (!this.live) {
      live.claim("bible");
      this.live = true;
      void outputs.ensureOpen();
    }
    this.blackout = false;
    this.#push();
  }

  async next() {
    if (this.index === null) return this.show(0);
    const from = this.rangeEnd ?? this.index;
    if (from + 1 < this.verses.length) return this.show(from + 1);
    // Fim do capítulo: segue para o próximo (ou o próximo livro).
    const book = this.bookInfo;
    if (!book || this.chapter === null) return;
    if (this.chapter < book.chapters) await this.open(book.number, this.chapter + 1);
    else if (book.number < 66) await this.open(book.number + 1, 1);
    else return;
    this.show(0);
  }

  async previous() {
    if (this.index === null) return;
    if (this.index > 0) return this.show(this.index - 1);
    const book = this.bookInfo;
    if (!book || this.chapter === null) return;
    if (this.chapter > 1) await this.open(book.number, this.chapter - 1);
    else if (book.number > 1) {
      const before = this.books.find((b) => b.number === book.number - 1);
      if (!before) return;
      await this.open(before.number, before.chapters);
    } else return;
    this.show(this.verses.length - 1);
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
    void api.show({ kind: "black" });
    live.release("bible");
  }

  /** O slide do versículo atual, para a prévia e o telão. */
  get slide(): Slide | null {
    const book = this.bookInfo;
    if (this.index === null || !book || this.chapter === null) return null;
    const first = this.verses[this.index];
    if (!first) return null;
    const last = this.rangeEnd !== null ? this.verses[this.rangeEnd] : null;
    const chosen = last ? this.verses.slice(this.index, (this.rangeEnd ?? this.index) + 1) : [first];
    const text = chosen.length > 1 ? chosen.map((v) => `${v.number} ${v.text}`).join(" ") : first.text;
    const abbreviation = this.versionInfo?.abbreviation;
    return {
      kind: "verse",
      text,
      aux: `${formatReference(book, this.chapter, first.number, last?.number)}${abbreviation ? ` · ${abbreviation}` : ""}`,
      image: null,
      imagePosition: 5,
      repeat: false,
      timeMs: null,
      instrumentalTimeMs: null,
    };
  }

  #push() {
    const slide = this.slide;
    const scene: Scene =
      this.live && !this.blackout && slide ? { kind: "slide", slide, title: slide.aux ?? "" } : { kind: "black" };
    void api.show(scene);
  }

  async download() {
    if (!this.#listening) {
      this.#listening = true;
      await on("bible://progress", (p) => (this.progress = p));
    }
    this.downloading = true;
    this.progress = null;
    try {
      await api.downloadBible();
      await this.load();
      toasts.show("Bíblia instalada. Ela funciona sem internet.", "success");
    } catch (e) {
      toasts.error(errorMessage(e));
    } finally {
      this.downloading = false;
    }
  }
}

export const bible = new BiblePresenter();
live.register("bible", bible);
