// Levar o 7Hinos de um computador para outro por pendrive.

import { open } from "@tauri-apps/plugin-dialog";
import { api, errorMessage, on } from "../api";
import type { PackageManifest } from "../types";
import { catalog } from "./catalog.svelte";
import { settings } from "./settings.svelte";

class Transfer {
  busy = $state<"export" | "import" | null>(null);
  progress = $state<{ done: number; total: number } | null>(null);
  error = $state<string | null>(null);
  #listening = false;

  async #listen() {
    if (this.#listening) return;
    this.#listening = true;
    await on("package://progress", (p) => (this.progress = p));
  }

  get percent() {
    return this.progress?.total ? Math.round((this.progress.done / this.progress.total) * 100) : 0;
  }

  async pickFolder(title: string): Promise<string | null> {
    const picked = await open({ directory: true, title });
    return typeof picked === "string" ? picked : null;
  }

  async export(dir: string, includeVideos: boolean): Promise<PackageManifest | null> {
    return this.#run("export", () => api.exportPackage(dir, includeVideos));
  }

  inspect(path: string) {
    return api.inspectPackage(path);
  }

  async import(path: string): Promise<PackageManifest | null> {
    const manifest = await this.#run("import", () => api.importPackage(path));
    if (manifest) {
      await settings.refresh();
      catalog.status = await api.catalogStatus();
      await catalog.reload();
    }
    return manifest;
  }

  async #run<T>(kind: "export" | "import", task: () => Promise<T>): Promise<T | null> {
    await this.#listen();
    this.busy = kind;
    this.error = null;
    this.progress = null;
    try {
      return await task();
    } catch (e) {
      this.error = errorMessage(e);
      return null;
    } finally {
      this.busy = null;
    }
  }
}

export const transfer = new Transfer();
