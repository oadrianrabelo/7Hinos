// Atualização automática do app: verifica em silêncio, baixa em segundo plano
// e só pede para reiniciar quando já está tudo pronto.

import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";

class Updater {
  ready = $state<Update | null>(null);
  installing = $state(false);

  async checkInBackground() {
    try {
      const update = await check();
      if (!update) return;
      await update.download();
      this.ready = update;
    } catch {
      // Sem internet ou sem release publicada: tenta na próxima abertura.
    }
  }

  async installAndRestart() {
    if (!this.ready) return;
    this.installing = true;
    try {
      await this.ready.install();
      await relaunch();
    } finally {
      this.installing = false;
    }
  }
}

export const updater = new Updater();
