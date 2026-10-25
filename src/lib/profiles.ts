// Perfis de telas: funções puras, testadas.

import type { Display, Output, ProfileOutput } from "./types";

/** As telas abertas agora, no formato de um perfil. */
export function snapshot(outputs: Output[], displays: Display[]): ProfileOutput[] {
  return outputs.map((o) => {
    const d = displays.find((x) => x.id === o.displayId);
    return { displayId: o.displayId, displayName: d?.name ?? "", width: d?.width ?? 0, height: d?.height ?? 0, role: o.role };
  });
}

/**
 * Onde abrir cada tela do perfil com os monitores de agora. Procura pelo
 * identificador; se o sistema o mudou (cabo em outra porta, adaptador USB),
 * pelo mesmo tamanho, sem repetir monitor e nunca no do controle. Telas que
 * não acharem lugar ficam de fora.
 */
export function resolve(profile: ProfileOutput[], displays: Display[]): { displayId: string | null; role: ProfileOutput["role"] }[] {
  const used = new Set<string>();
  const plan: { displayId: string | null; role: ProfileOutput["role"] }[] = [];
  // Primeiro os que batem pelo identificador, para não "roubarem" monitor.
  const exact = profile.map((p) => (p.displayId && displays.some((d) => d.id === p.displayId) ? p.displayId : null));
  exact.forEach((id) => id && used.add(id));
  profile.forEach((p, i) => {
    if (p.displayId === null) {
      plan.push({ displayId: null, role: p.role });
      return;
    }
    const id =
      exact[i] ??
      displays.find((d) => !used.has(d.id) && !d.hostsController && d.width === p.width && d.height === p.height)?.id ??
      displays.find((d) => !used.has(d.id) && !d.hostsController)?.id;
    if (!id) return;
    used.add(id);
    plan.push({ displayId: id, role: p.role });
  });
  return plan;
}

/** Descrição curta para a lista: "Telão na Tela 2 · Retorno na Tela 3". */
export function describe(profile: ProfileOutput[]): string {
  if (!profile.length) return "Nenhuma tela";
  return profile
    .map((p) => `${p.role === "monitor" ? "Retorno" : "Telão"} ${p.displayId ? `na ${p.displayName || "tela"}` : "em janela"}`)
    .join(" · ");
}
