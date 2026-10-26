// Quem está no telão agora: um hino, a Bíblia, um aviso, o relógio... Só um
// por vez. Quem começa a apresentar "reivindica" o palco e o anterior para.
// Os atalhos de teclado vão para quem está no palco.

export interface Presenter {
  stop(): void;
  next(): void;
  previous(): void;
  toggleBlackout(): void;
  togglePlay?(): void;
}

export type Source = "song" | "bible" | "text" | "clock" | "deck" | "draw";

class Live {
  source = $state<Source | null>(null);
  #presenters = new Map<Source, Presenter>();

  register(source: Source, presenter: Presenter) {
    this.#presenters.set(source, presenter);
  }

  /** Assume o palco, encerrando quem estava antes. */
  claim(source: Source) {
    const previous = this.source;
    this.source = source;
    if (previous && previous !== source) this.#presenters.get(previous)?.stop();
  }

  release(source: Source) {
    if (this.source === source) this.source = null;
  }

  get current(): Presenter | null {
    return this.source ? (this.#presenters.get(this.source) ?? null) : null;
  }
}

export const live = new Live();
