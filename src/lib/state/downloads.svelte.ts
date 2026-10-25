// Downloads para usar sem internet: acompanha o progresso de cada um e
// avisa quando termina. Cada download tem uma chave (ex.: "album:12") para a
// página certa mostrar o próprio progresso.

import { api, errorMessage, on } from "../api";
import type { DownloadProgress, Scope } from "../types";
import { availability } from "./availability.svelte";
import { toasts } from "./toasts.svelte";

class Downloads {
  jobs = $state<Record<number, DownloadProgress>>({});
  #keys = $state<Record<string, number>>({});

  async init() {
    await on("download://progress", (p) => {
      this.jobs[p.job] = p;
      if (!p.finished) return;
      availability.clear();
      if (p.error) toasts.error(`“${p.label}” não terminou: ${p.error}`);
      else if (!p.cancelled) toasts.show(`“${p.label}” está pronto para usar sem internet.`, "success");
      setTimeout(() => delete this.jobs[p.job], 4000);
    });
  }

  get active(): DownloadProgress[] {
    return Object.values(this.jobs);
  }

  /** O download em andamento desta chave, se houver. */
  running(key: string): DownloadProgress | null {
    const job = this.#keys[key];
    const progress = job === undefined ? undefined : this.jobs[job];
    return progress && !progress.finished ? progress : null;
  }

  async start(key: string, scope: Scope, label: string) {
    if (this.running(key)) return;
    try {
      const job = await api.startDownload(scope, label);
      this.#keys[key] = job;
      this.jobs[job] ??= { job, label, cached: 0, total: 0, finished: false, cancelled: false, error: null };
    } catch (e) {
      toasts.error(errorMessage(e));
    }
  }

  cancel(job: number) {
    return api.cancelDownload(job);
  }
}

export const downloads = new Downloads();

export function percent(p: { cached: number; total: number } | null): number {
  return p && p.total ? Math.round((p.cached / p.total) * 100) : 0;
}
