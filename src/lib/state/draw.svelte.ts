// Sorteio no telão: números (de… até…) ou uma lista de nomes. O resultado é
// escolhido aqui; as telas só fazem a animação (ver lib/draw.ts).

import { api } from "../api";
import { available, numberRange, parseNames, pick, sampleCandidates } from "../draw";
import { live } from "./live.svelte";
import { outputs } from "./outputs.svelte";
import { toasts } from "./toasts.svelte";

const KEY = "7hinos.draw";

interface Saved {
  mode: "numbers" | "names";
  from: number;
  to: number;
  names: string;
  title: string;
  allowRepeat: boolean;
  seconds: number;
}

const defaults: Saved = { mode: "numbers", from: 1, to: 100, names: "", title: "Sorteio", allowRepeat: false, seconds: 5 };

function load(): Saved {
  try {
    return { ...defaults, ...JSON.parse(localStorage.getItem(KEY) ?? "{}") };
  } catch {
    return defaults;
  }
}

class Draw {
  #saved = load();
  mode = $state(this.#saved.mode);
  from = $state(this.#saved.from);
  to = $state(this.#saved.to);
  names = $state(this.#saved.names);
  title = $state(this.#saved.title);
  allowRepeat = $state(this.#saved.allowRepeat);
  seconds = $state(this.#saved.seconds);
  /** Quem já saiu, na ordem (some ao fechar o app ou em "Recomeçar"). */
  drawn = $state<string[]>([]);
  /** O telão está embaralhando (o resultado ainda não apareceu). */
  rolling = $state(false);
  #timer: ReturnType<typeof setTimeout> | undefined;

  constructor() {
    $effect.root(() => {
      $effect(() => {
        const saved: Saved = {
          mode: this.mode,
          from: this.from,
          to: this.to,
          names: this.names,
          title: this.title,
          allowRepeat: this.allowRepeat,
          seconds: this.seconds,
        };
        try {
          localStorage.setItem(KEY, JSON.stringify(saved));
        } catch {
          // Sem armazenamento, só não lembra na próxima vez.
        }
      });
    });
  }

  get pool(): string[] {
    return this.mode === "numbers" ? numberRange(this.from, this.to) : parseNames(this.names);
  }

  get remaining(): string[] {
    return available(this.pool, this.drawn, this.allowRepeat);
  }

  run() {
    const pool = this.remaining;
    const result = pick(pool);
    if (result === null) {
      toasts.error(this.pool.length ? "Todos já foram sorteados. Toque em Recomeçar." : "Coloque os números ou os nomes antes de sortear.");
      return;
    }
    const durationMs = Math.round(this.seconds * 1000);
    const startedAt = Date.now() + 150;
    this.rolling = true;
    clearTimeout(this.#timer);
    // Quem acompanha no controle vê o resultado só quando o telão revela.
    this.#timer = setTimeout(() => {
      this.rolling = false;
      this.drawn = [...this.drawn, result];
    }, durationMs + 150);
    live.claim("draw");
    void outputs.ensureOpen();
    void api.show({
      kind: "draw",
      title: this.title.trim(),
      result,
      candidates: sampleCandidates(pool, result),
      startedAt,
      durationMs,
    });
  }

  restart() {
    this.drawn = [];
  }

  /** Tira do telão, voltando à tela de espera. */
  rest() {
    live.release("draw");
    void api.show({ kind: "idle" });
  }

  readonly presenter = {
    stop: () => {},
    next: () => this.run(),
    previous: () => {},
    toggleBlackout: () => {},
  };
}

export const draw = new Draw();
live.register("draw", draw.presenter);
