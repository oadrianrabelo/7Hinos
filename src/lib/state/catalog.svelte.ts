import { api, errorMessage, on } from "../api";
import type { AlbumSummary, CatalogStatus, Collection, SongSummary } from "../types";
import { toasts } from "./toasts.svelte";

class CatalogState {
  status = $state<CatalogStatus | null>(null);
  collections = $state<Collection[]>([]);
  hymnal = $state<SongSummary[] | null>(null);
  syncing = $state(false);
  progress = $state<{ done: number; total: number | null } | null>(null);
  error = $state<string | null>(null);

  async init() {
    await on("catalog://progress", (p) => (this.progress = p));
    await on("catalog://changed", async ({ status, updated }) => {
      this.status = status;
      await this.reload();
      if (updated) toasts.show("Hinos atualizados com as novidades do LouvorJA.", "success");
    });
    this.status = await api.catalogStatus();
    await this.reload();
  }

  get installed() {
    return this.status?.installed ?? false;
  }

  async reload() {
    this.hymnal = null;
    this.collections = this.installed ? await api.collections().catch(() => []) : [];
  }

  async loadHymnal() {
    if (!this.hymnal && this.installed) this.hymnal = await api.hymnal().catch(() => []);
    return this.hymnal ?? [];
  }

  album(id: number): AlbumSummary | undefined {
    for (const c of this.collections) {
      const found = c.albums.find((a) => a.id === id);
      if (found) return found;
    }
  }

  /** Baixa o catálogo. Devolve `true` se terminou bem. */
  async sync(force = false): Promise<boolean> {
    if (this.syncing) return false;
    this.syncing = true;
    this.error = null;
    this.progress = null;
    try {
      this.status = await api.syncCatalog(force);
      await this.reload();
      return true;
    } catch (e) {
      this.error = errorMessage(e);
      return false;
    } finally {
      this.syncing = false;
    }
  }
}

export const catalog = new CatalogState();
