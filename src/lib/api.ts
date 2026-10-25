// Única porta de entrada para o backend. Os componentes nunca chamam
// `invoke` diretamente: assim os nomes e tipos ficam num lugar só.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  Album,
  CatalogStatus,
  Collection,
  Display,
  OfflineStatus,
  Output,
  Scene,
  Settings,
  Song,
  SongSummary,
  StageState,
  VideoState,
} from "./types";

export const api = {
  appVersion: () => invoke<string>("app_version"),

  settings: () => invoke<Settings>("settings_get"),
  saveSettings: (settings: Settings) => invoke<Settings>("settings_set", { settings }),
  toggleFavorite: (id: number) => invoke<Settings>("favorite_toggle", { id }),
  addVideos: (paths: string[]) => invoke<Settings>("videos_add", { paths }),
  removeVideo: (path: string) => invoke<Settings>("videos_remove", { path }),

  catalogStatus: () => invoke<CatalogStatus>("catalog_status"),
  syncCatalog: (force = false) => invoke<CatalogStatus>("catalog_sync", { force }),
  collections: () => invoke<Collection[]>("collections"),
  hymnal: () => invoke<SongSummary[]>("hymnal"),
  search: (query: string) => invoke<SongSummary[]>("search", { query }),
  songs: (ids: number[]) => invoke<SongSummary[]>("songs", { ids }),
  album: (id: number) => invoke<Album>("album", { id }),
  song: (id: number) => invoke<Song>("song", { id }),
  songPlayed: (id: number) => invoke<void>("song_played", { id }),
  albumOffline: (id: number) => invoke<OfflineStatus>("album_offline", { id }),
  downloadAlbum: (id: number) => invoke<OfflineStatus>("album_download", { id }),
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
  "download://progress": { albumId: number; cached: number; total: number; finished: boolean };
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
