import { api, errorMessage, on } from "../api";
import { open } from "@tauri-apps/plugin-dialog";
import type { AlbumSummary, CatalogStatus, Collection, Install, SongSummary } from "../types";
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

  /** Instalações do LouvorJA encontradas neste computador. */
  async detectLocal(): Promise<Install[]> {
    return api.detectLouvorja().catch(() => []);
  }

  /** Pede a pasta do LouvorJA (instalação, pendrive, pasta copiada). */
  async pickLocal(): Promise<string | null> {
    const picked = await open({ directory: true, title: "Escolha a pasta do LouvorJA (a que tem a pasta config)" });
    return typeof picked === "string" ? picked : null;
  }

  /** Importa o LouvorJA desktop, sem internet. Devolve `true` se deu certo. */
  async importLocal(path: string | null = null): Promise<boolean> {
    if (this.syncing) return false;
    this.syncing = true;
    this.error = null;
    this.progress = null;
    try {
      this.status = await api.importLouvorja(path);
      await this.reload();
      return true;
    } catch (e) {
      this.error = errorMessage(e);
      return false;
    } finally {
      this.syncing = false;
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
