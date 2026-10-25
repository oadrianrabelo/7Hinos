// O que está no telão agora, como o backend conta. A prévia "ao vivo" do
// painel desenha exatamente isto.

import { api, on } from "../api";
import type { StageState } from "../types";
import { video } from "./video.svelte";

class StageMirror {
  current = $state<StageState>({ scene: { kind: "black" }, video: null, pulpit: { message: null, timer: null }, rev: 0 });

  async init() {
    await on("stage://changed", (s) => {
      this.current = s;
      video.receive(s.video);
    });
    this.current = await api.stage();
    // Ao abrir o app, o telão começa na tela de espera.
    if (this.current.scene.kind === "black") void api.show({ kind: "idle" });
  }
}

export const stage = new StageMirror();
