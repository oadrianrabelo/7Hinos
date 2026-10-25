import { warn } from "@tauri-apps/plugin-log";

export interface Toast {
  id: number;
  message: string;
  kind: "info" | "success" | "error";
  action?: { label: string; run: () => void };
}

class Toasts {
  items = $state<Toast[]>([]);
  #next = 1;

  show(message: string, kind: Toast["kind"] = "info", action?: Toast["action"], ms = 4500) {
    const id = this.#next++;
    this.items.push({ id, message, kind, action });
    if (ms > 0) setTimeout(() => this.dismiss(id), ms);
    return id;
  }

  /** Erros mostrados à pessoa também vão para o arquivo de registro. */
  error(message: string) {
    void warn(`[interface] ${message}`).catch(() => {});
    if (this.items.some((t) => t.kind === "error" && t.message === message)) return;
    return this.show(message, "error", undefined, 7000);
  }

  dismiss(id: number) {
    this.items = this.items.filter((t) => t.id !== id);
  }
}

export const toasts = new Toasts();
