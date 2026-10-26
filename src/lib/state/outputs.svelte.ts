import { api, errorMessage, on } from "../api";
import { resolve, snapshot } from "../profiles";
import type { Display, Output, OutputRole, ScreenProfile } from "../types";
import { settings } from "./settings.svelte";
import { toasts } from "./toasts.svelte";

class OutputsState {
  displays = $state<Display[]>([]);
  outputs = $state<Output[]>([]);

  async init() {
    await on("displays://changed", () => this.refresh());
    await on("displays://lost", () =>
      toasts.error("Uma tela de projeção foi desconectada. Ela volta sozinha quando for religada."),
    );
    await on("displays://restored", () => toasts.show("A tela de projeção voltou.", "success"));
    await this.refresh();
  }

  async refresh() {
    [this.displays, this.outputs] = await Promise.all([api.displays(), api.outputs()]);
  }

  /** Há uma tela além daquela onde está o controle (projetor, TV). */
  get externalDisplays() {
    return this.displays.filter((d) => !d.hostsController);
  }

  /** Telas que a igreja vê (sem os monitores de retorno). */
  /** Todas as telas abertas, telões primeiro (o primeiro toca o som dos vídeos). */
  get forVideo() {
    return [...this.outputs].sort((a, b) => Number(a.role === "monitor") - Number(b.role === "monitor"));
  }

  get audience() {
    return this.outputs.filter((o) => o.role === "audience");
  }

  get isProjecting() {
    return this.audience.length > 0;
  }

  async open(displayId: string | null, role: OutputRole = "audience") {
    try {
      await api.openOutput(displayId, role);
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

  /** Fecha e abre de novo todas as telas, nos mesmos lugares (o vídeo continua). */
  async reopen() {
    try {
      await api.reopenOutputs();
      toasts.show("Telas reabertas.", "success", undefined, 2000);
    } catch (e) {
      toasts.error(errorMessage(e));
    }
    await this.refresh();
  }

  // Perfis de telas

  get profiles(): ScreenProfile[] {
    return settings.value?.screenProfiles ?? [];
  }

  /** O perfil que corresponde exatamente às telas abertas agora, se houver. */
  get currentProfile(): ScreenProfile | null {
    const key = (list: { displayId: string | null; role: OutputRole }[]) =>
      list.map((o) => `${o.displayId ?? "janela"}:${o.role}`).sort().join("|");
    const now = key(this.outputs);
    return this.profiles.find((p) => key(resolve(p.outputs, this.displays)) === now && p.outputs.length > 0) ?? null;
  }

  async saveProfile(name: string) {
    const list = this.profiles;
    const id = Math.max(0, ...list.map((p) => p.id)) + 1;
    await settings.save({ screenProfiles: [...list, { id, name: name.trim() || `Perfil ${id}`, outputs: snapshot(this.outputs, this.displays) }] });
    return id;
  }

  /** Atualiza um perfil com as telas abertas agora. */
  updateProfile(id: number) {
    return settings.save({
      screenProfiles: this.profiles.map((p) => (p.id === id ? { ...p, outputs: snapshot(this.outputs, this.displays) } : p)),
    });
  }

  renameProfile(id: number, name: string) {
    return settings.save({ screenProfiles: this.profiles.map((p) => (p.id === id ? { ...p, name: name.trim() || p.name } : p)) });
  }

  deleteProfile(id: number) {
    return settings.save({
      screenProfiles: this.profiles.filter((p) => p.id !== id),
      ...(settings.current.defaultProfile === id ? { defaultProfile: null } : {}),
    });
  }

  setDefault(id: number | null) {
    return settings.save({ defaultProfile: id });
  }

  /** Deixa as telas exatamente como no perfil: abre o que falta, fecha o resto. */
  async applyProfile(profile: ScreenProfile) {
    await this.refresh();
    const plan = resolve(profile.outputs, this.displays);
    try {
      for (const o of this.outputs) {
        const keep = o.displayId !== null && plan.some((p) => p.displayId === o.displayId);
        if (!keep) await api.closeOutput(o.label);
      }
      for (const p of plan) await api.openOutput(p.displayId, p.role);
    } catch (e) {
      toasts.error(errorMessage(e));
    }
    await this.refresh();
    const missing = profile.outputs.filter((o) => o.displayId !== null).length - plan.filter((p) => p.displayId !== null).length;
    if (missing > 0) toasts.error(`${missing === 1 ? "Uma tela" : `${missing} telas`} do perfil “${profile.name}” não está conectada.`);
  }
}

export const outputs = new OutputsState();
