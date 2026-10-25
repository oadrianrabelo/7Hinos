// Espelho dos tipos que o backend em Rust envia (camelCase).

export type Lang = "pt" | "es";
export type UiMode = "simple" | "full";
export type Theme = "system" | "light" | "dark";

export interface Media {
  url: string;
  path: string | null;
}

export interface AlbumSummary {
  id: number;
  name: string;
  subtitle: string | null;
  color: string | null;
  cover: Media | null;
}

export interface Collection {
  id: number;
  name: string;
  slug: string;
  albums: AlbumSummary[];
}

export interface SongSummary {
  id: number;
  title: string;
  subtitle: string;
  number: number | null;
  hasInstrumental: boolean;
  durationMs: number | null;
}

export interface Album {
  id: number;
  name: string;
  color: string | null;
  cover: Media | null;
  songs: SongSummary[];
}

export interface Slide {
  kind: "title" | "lyric" | "verse" | "text";
  text: string;
  aux: string | null;
  image: Media | null;
  imagePosition: number;
  repeat: boolean;
  timeMs: number | null;
  instrumentalTimeMs: number | null;
}

export interface Song {
  id: number;
  title: string;
  subtitle: string;
  cover: Media | null;
  audio: Media | null;
  instrumental: Media | null;
  durationMs: number | null;
  slides: Slide[];
  /** Música própria ou letra corrigida pela igreja. */
  custom: boolean;
}

export interface SlideStyle {
  uppercase: boolean;
  textScale: number;
  showImages: boolean;
  transitionMs: number;
}

/** Um item da programação do culto. */
export type SetItem =
  | { kind: "song"; id: number; title: string }
  | { kind: "video"; path: string; title: string };

export interface VideoItem {
  path: string;
  title: string;
}

export interface Settings {
  onboarded: boolean;
  mode: UiMode;
  theme: Theme;
  language: Lang;
  uiScale: number;
  slide: SlideStyle;
  favorites: number[];
  recent: number[];
  videos: VideoItem[];
  setlist: SetItem[];
  idle: IdleScreen;
  notices: Notice[];
  bibleVersion: number | null;
  keepPlayed: boolean;
  autoUpdateCatalog: boolean;
  autoUpdateApp: boolean;
  /** Instalações do LouvorJA e pastas de músicas no computador. */
  localSources: string[];
  legacyMigrated: boolean;
  pendingFavorites: { title: string; album: string }[];
}

/** Uma instalação do LouvorJA desktop encontrada no disco. */
export interface Install {
  root: string;
  database: string;
}

export interface CatalogStatus {
  installed: boolean;
  version: number | null;
  songs: number;
  albums: number;
  /** Última vez (epoch em segundos) que o LouvorJA respondeu. */
  lastCheck: number | null;
  /** Versão publicada vista na última resposta. */
  available: number | null;
  /** Erro da última tentativa, se falhou. */
  lastError: string | null;
}

export interface Diagnostics {
  version: string;
  os: string;
  arch: string;
  catalog: CatalogStatus;
  localSources: string[];
  outputs: number;
  logDir: string | null;
}

/** Mudança parcial nas preferências (objetos aninhados também parciais). */
export type SettingsPatch = Partial<Omit<Settings, "slide">> & { slide?: Partial<SlideStyle> };

/** O que baixar para usar sem internet. */
export type Scope =
  | { kind: "all" }
  | { kind: "collection"; id: number }
  | { kind: "album"; id: number }
  | { kind: "songs"; ids: number[] };

export interface DownloadProgress {
  job: number;
  label: string;
  cached: number;
  total: number;
  finished: boolean;
  cancelled: boolean;
  error: string | null;
}

export interface OfflineStatus {
  cached: number;
  total: number;
}

export type Scene =
  | { kind: "black" }
  /** `next`: o slide seguinte, para o monitor de retorno. */
  | { kind: "slide"; slide: Slide; title: string; next?: Slide | null }
  /** Tela de espera (logo e texto da igreja). */
  | { kind: "idle" }
  /** Relógio, ou contagem regressiva até `until` (epoch em ms). */
  | { kind: "clock"; until: number | null; label: string };

export interface IdleScreen {
  /** Arquivo guardado dentro do app (logo ou imagem). */
  image: string | null;
  text: string;
  showClock: boolean;
}

export interface Notice {
  id: number;
  title: string;
  body: string;
}

export interface VideoState {
  path: string;
  title: string;
  playing: boolean;
  /** Segundos. */
  position: number;
  /** Epoch em ms em que `position` era verdade. */
  updatedAt: number;
  looping: boolean;
  volume: number;
  /** Telas que mostram o vídeo; a primeira toca o som. */
  targets: string[];
}

export interface StageState {
  scene: Scene;
  video: VideoState | null;
}

export interface Display {
  id: string;
  name: string;
  width: number;
  height: number;
  primary: boolean;
  hostsController: boolean;
  output: string | null;
}

/** Telão (o que a igreja vê) ou retorno (slide atual, próximo e horas). */
export type OutputRole = "audience" | "monitor";

export interface Output {
  label: string;
  displayId: string | null;
  name: string;
  role: OutputRole;
}

/** O que um pacote de pendrive contém. */
export interface PackageManifest {
  format: number;
  /** Epoch em segundos. */
  createdAt: number;
  appVersion: string;
  catalogVersion: number | null;
  songs: number;
  mediaFiles: number;
  videos: number;
  bytes: number;
}

export interface BibleVersion {
  id: number;
  abbreviation: string;
  name: string;
}

export interface BibleBook {
  /** 1 (Gênesis) a 66 (Apocalipse). */
  number: number;
  name: string;
  abbreviation: string;
  chapters: number;
  /** 1 = Antigo, 2 = Novo Testamento. */
  testament: number;
}

export interface Verse {
  number: number;
  text: string;
}

export interface BibleHit {
  book: number;
  chapter: number;
  verse: number;
  text: string;
}

export interface UserSlide {
  text: string;
  aux: string | null;
}

/** Música da igreja ou letra corrigida de um hino (`basedOn`). */
export interface UserSong {
  id: number;
  title: string;
  author: string;
  slides: UserSlide[];
  basedOn: number | null;
  /** Nomes de arquivo guardados dentro do app. */
  audio: string | null;
  background: string | null;
  updatedAt: number;
}
