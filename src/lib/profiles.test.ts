import { describe as group, expect, it } from "vitest";
import { describe, resolve, snapshot } from "./profiles";
import type { Display } from "./types";

const display = (id: string, name: string, w = 1920, h = 1080, host = false): Display => ({
  id, name, width: w, height: h, primary: host, hostsController: host, output: null,
});

group("perfis de telas", () => {
  const now = [display("vga@0,0", "Tela 1", 1366, 768, true), display("hdmi@1366,0", "Tela 2"), display("usb@3286,0", "Tela 3", 1280, 720)];

  it("guarda as telas abertas com nome e tamanho", () => {
    const snap = snapshot([{ label: "output-1", displayId: "hdmi@1366,0", name: "Tela 2", role: "audience" }], now);
    expect(snap).toEqual([{ displayId: "hdmi@1366,0", displayName: "Tela 2", width: 1920, height: 1080, role: "audience" }]);
  });

  it("reabre nos mesmos monitores", () => {
    const plan = resolve(
      [
        { displayId: "hdmi@1366,0", displayName: "Tela 2", width: 1920, height: 1080, role: "audience" },
        { displayId: "usb@3286,0", displayName: "Tela 3", width: 1280, height: 720, role: "monitor" },
      ],
      now,
    );
    expect(plan).toEqual([
      { displayId: "hdmi@1366,0", role: "audience" },
      { displayId: "usb@3286,0", role: "monitor" },
    ]);
  });

  it("reconhece pelo tamanho quando o identificador muda, sem usar a tela do controle", () => {
    const plan = resolve([{ displayId: "usb@9999,0", displayName: "Tela 3", width: 1280, height: 720, role: "monitor" }], now);
    expect(plan).toEqual([{ displayId: "usb@3286,0", role: "monitor" }]);
  });

  it("deixa de fora o que não tem onde abrir e mantém janelas", () => {
    const only = [display("vga@0,0", "Tela 1", 1366, 768, true)];
    const plan = resolve(
      [
        { displayId: "hdmi@1366,0", displayName: "Tela 2", width: 1920, height: 1080, role: "audience" },
        { displayId: null, displayName: "", width: 0, height: 0, role: "monitor" },
      ],
      only,
    );
    expect(plan).toEqual([{ displayId: null, role: "monitor" }]);
  });

  it("descreve o perfil em uma linha", () => {
    expect(describe([{ displayId: "x", displayName: "Tela 2", width: 0, height: 0, role: "audience" }])).toBe("Telão na Tela 2");
  });
});
