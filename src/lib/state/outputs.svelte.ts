import { api, errorMessage, on } from "../api";
import type { Display, Output } from "../types";
import { toasts } from "./toasts.svelte";

class OutputsState {
  displays = $state<Display[]>([]);
  outputs = $state<Output[]>([]);

  async init() {
    await on("displays://changed", () => this.refresh());
    await this.refresh();
  }

  async refresh() {
    [this.displays, this.outputs] = await Promise.all([api.displays(), api.outputs()]);
  }

  /** Há uma tela além daquela onde está o controle (projetor, TV). */
  get externalDisplays() {
    return this.displays.filter((d) => !d.hostsController);
  }

  get isProjecting() {
    return this.outputs.length > 0;
  }

  async open(displayId: string | null) {
    try {
      await api.openOutput(displayId);
      await this.refresh();
    } catch (e) {
      toasts.error(errorMessage(e));
    }
  }

  async close(label: string) {
    await api.closeOutput(label);
    await this.refresh();
  }

  async closeAll() {
    await api.closeOutputs();
    await this.refresh();
  }

  /**
   * Garante uma projeção aberta quando há projetor/TV conectado. Com uma tela
   * só, não abre nada sozinho: cobriria a janela de controle.
   */
  async ensureOpen(): Promise<boolean> {
    if (this.isProjecting) return true;
    await this.refresh();
    const target = this.externalDisplays.find((d) => !d.output);
    if (!target) return false;
    await this.open(target.id);
    return this.isProjecting;
  }
}

export const outputs = new OutputsState();
