import { api, errorMessage, on } from "../api";
import type { Settings, SlideStyle } from "../types";
import { toasts } from "./toasts.svelte";

class SettingsState {
  value = $state<Settings | null>(null);

  async init() {
    this.value = await api.settings();
    await on("settings://changed", (s) => (this.value = s));
  }

  /** Relê do backend (depois de ações que mudam preferências lá). */
  async refresh() {
    this.value = await api.settings();
  }

  get current(): Settings {
    if (!this.value) throw new Error("preferências ainda não carregadas");
    return this.value;
  }

  async save(patch: Partial<Settings>) {
    try {
      this.value = await api.saveSettings({ ...this.current, ...patch });
    } catch (e) {
      toasts.error(errorMessage(e));
    }
  }

  saveSlide(patch: Partial<SlideStyle>) {
    return this.save({ slide: { ...this.current.slide, ...patch } });
  }

  isFavorite(id: number) {
    return this.value?.favorites.includes(id) ?? false;
  }

  async toggleFavorite(id: number) {
    this.value = await api.toggleFavorite(id);
  }
}

export const settings = new SettingsState();
