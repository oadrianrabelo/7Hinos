// A escala da igreja: carrega uma vez e grava sozinha a cada mudança.

import { api, errorMessage } from "../api";
import { emptyRoster, normalizeRoster, type RosterDoc } from "../roster";
import { toasts } from "./toasts.svelte";

class RosterState {
  doc = $state<RosterDoc>(emptyRoster());
  loaded = $state(false);
  #timer: ReturnType<typeof setTimeout> | undefined;

  async init() {
    try {
      this.doc = normalizeRoster(await api.rosterLoad());
    } catch {
      this.doc = emptyRoster();
    }
    this.loaded = true;
  }

  /** Muda a escala e grava logo depois (várias mudanças seguidas viram uma). */
  update(change: (doc: RosterDoc) => void) {
    change(this.doc);
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => void this.#save(), 400);
  }

  async #save() {
    try {
      await api.rosterSave($state.snapshot(this.doc));
    } catch (e) {
      toasts.error(`Não deu para guardar a escala: ${errorMessage(e)}`);
    }
  }
}

export const roster = new RosterState();
