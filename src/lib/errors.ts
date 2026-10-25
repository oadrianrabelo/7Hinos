// Rede de segurança: qualquer erro não tratado vai para o arquivo de registro
// e vira um aviso discreto, em vez de deixar a tela quebrada sem explicação.

import { error as logError } from "@tauri-apps/plugin-log";
import { errorMessage } from "./api";
import { toasts } from "./state/toasts.svelte";

const FRIENDLY = "Algo deu errado, mas o 7Hinos continua funcionando. O problema foi registrado.";

export function report(error: unknown, where = "interface") {
  const detail = error instanceof Error ? `${error.message}\n${error.stack ?? ""}` : errorMessage(error);
  void logError(`[${where}] ${detail}`).catch(() => console.error(where, error));
}

export function installErrorHandlers() {
  window.addEventListener("error", (event) => {
    report(event.error ?? event.message, "erro");
    toasts.error(FRIENDLY);
  });
  window.addEventListener("unhandledrejection", (event) => {
    report(event.reason, "promessa");
    toasts.error(FRIENDLY);
  });
}
