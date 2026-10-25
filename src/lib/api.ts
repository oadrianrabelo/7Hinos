// Única porta de entrada para o backend. Os componentes nunca chamam
// `invoke` diretamente: assim os nomes e tipos ficam num lugar só.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  Album,
  CatalogStatus,
  Collection,
  Diagnostics,
  DownloadProgress,
  Display,
  Install,
  OfflineStatus,
  Output,
  Scope,
  Scene,
  Settings,
  SettingsPatch,
  Song,
  SongSummary,
  StageState,
  VideoState,
} from "./types";

export const api = {
  appVersion: () => invoke<string>("app_version"),
  diagnostics: () => invoke<Diagnostics>("diagnostics"),
  openLogs: () => invoke<void>("logs_open"),

  settings: () => invoke<Settings>("settings_get"),
  /** Só os campos enviados mudam; o backend mescla com o que já está salvo. */
  updateSettings: (patch: SettingsPatch) => invoke<Settings>("settings_update", { patch }),
  toggleFavorite: (id: number) => invoke<Settings>("favorite_toggle", { id }),
  addVideos: (paths: string[]) => invoke<Settings>("videos_add", { paths }),
  removeVideo: (path: string) => invoke<Settings>("videos_remove", { path }),

  catalogStatus: () => invoke<CatalogStatus>("catalog_status"),
  syncCatalog: (force = false) => invoke<CatalogStatus>("catalog_sync", { force }),
  checkCatalogNow: () => invoke<void>("catalog_check_now"),
  detectLouvorja: () => invoke<Install[]>("louvorja_detect"),
  importLouvorja: (path: string | null = null) => invoke<CatalogStatus>("louvorja_import", { path }),
  collections: () => invoke<Collection[]>("collections"),
  hymnal: (edition: "current" | "1996" = "current") =>
    invoke<SongSummary[]>("hymnal", { edition: edition === "1996" ? "1996" : null }),
  search: (query: string) => invoke<SongSummary[]>("search", { query }),
  songs: (ids: number[]) => invoke<SongSummary[]>("songs", { ids }),
  album: (id: number) => invoke<Album>("album", { id }),
  song: (id: number) => invoke<Song>("song", { id }),
  songPlayed: (id: number) => invoke<void>("song_played", { id }),
  offlineStatus: (scope: Scope) => invoke<OfflineStatus>("offline_status", { scope }),
  songsAvailable: (ids: number[]) => invoke<number[]>("songs_available", { ids }),
  startDownload: (scope: Scope, label: string) => invoke<number>("download_start", { scope, label }),
  cancelDownload: (job: number) => invoke<void>("download_cancel", { job }),
  mediaUsage: () => invoke<number>("media_usage"),
  clearMedia: () => invoke<void>("media_clear"),

  stage: () => invoke<StageState>("stage_get"),
  show: (scene: Scene) => invoke<void>("stage_show", { scene }),
  setVideo: (video: VideoState | null) => invoke<void>("stage_video", { video }),

  displays: () => invoke<Display[]>("displays_list"),
  outputs: () => invoke<Output[]>("outputs_list"),
  openOutput: (displayId: string | null) => invoke<string>("output_open", { displayId }),
  closeOutput: (label: string) => invoke<void>("output_close", { label }),
  closeOutputs: () => invoke<void>("outputs_close_all"),
};

export interface Events {
  "stage://changed": StageState;
  "settings://changed": Settings;
  "catalog://changed": { status: CatalogStatus; updated: boolean };
  "catalog://progress": { done: number; total: number | null };
  "download://progress": DownloadProgress;
  "displays://changed": null;
}

export function on<K extends keyof Events>(
  event: K,
  handler: (payload: Events[K]) => void,
): Promise<UnlistenFn> {
  return listen<Events[K]>(event, (e) => handler(e.payload));
}

/** Mensagem legível de qualquer erro vindo do backend ou do navegador. */
export function errorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return "Algo deu errado.";
}
