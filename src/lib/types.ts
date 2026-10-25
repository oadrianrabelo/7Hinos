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
  /** Fonte da letra dos hinos (ver lib/fonts.ts). */
  font: string;
  uppercase: boolean;
  textScale: number;
  showImages: boolean;
  transitionMs: number;
}

/** Um item da programação do culto. */
export type SetItem =
  | { kind: "song"; id: number; title: string }
  | { kind: "deck"; id: string; title: string }
  /** `tag` semanal: na hora toca o vídeo "de hoje" daquele tipo. */
  | { kind: "video"; path: string; title: string; tag?: string | null };

/** O que o monitor de retorno mostra além da letra. */
export interface MonitorOptions {
  showNext: boolean;
  showClock: boolean;
  showTitle: boolean;
  /** A mesma imagem de fundo do telão, escurecida. */
  showBackground: boolean;
  textScale: number;
}

/** Uma tela de um perfil; nome e tamanho ajudam a reconhecê-la depois. */
export interface ProfileOutput {
  displayId: string | null;
  displayName: string;
  width: number;
  height: number;
  role: OutputRole;
}

/** Combinação de telas salva com nome. */
export interface ScreenProfile {
  id: number;
  name: string;
  outputs: ProfileOutput[];
}

/** Ver lib/fonts.ts: "condensed", "lora", "system:Arial", "file:x.ttf"… */
export type ScreenFont = string;

/** Aparência dos avisos, da tela de espera e do relógio. */
export interface ScreenStyle {
  font: ScreenFont;
  textColor: string;
  accentColor: string;
  /** "gradient" ou uma cor #rrggbb. */
  background: string;
  backgroundImage: string | null;
}

export interface VideoItem {
  path: string;
  title: string;
  /** Tipo ("missoes", "provai", "abertura"…); ver lib/videoTags.ts. */
  tag?: string | null;
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
  monitor: MonitorOptions;
  screenStyle: ScreenStyle;
  screenProfiles: ScreenProfile[];
  defaultProfile: number | null;
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
export type SettingsPatch = Partial<Omit<Settings, "slide" | "monitor" | "screenStyle">> & {
  slide?: Partial<SlideStyle>;
  monitor?: Partial<MonitorOptions>;
  screenStyle?: Partial<ScreenStyle>;
};

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
  /** Página de apresentação (PDF, imagem, lição), mostrada inteira. */
  | { kind: "image"; src: string; title: string; page: number; total: number; next?: string | null }
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

/** Cronômetro do pregador: tempo atual = elapsedMs + (agora - startedAt). */
export interface Timer {
  elapsedMs: number;
  /** null = pausado. */
  startedAt: number | null;
  limitMs: number | null;
}

/** Só para quem está à frente: aparece no retorno, nunca no telão. */
export interface Pulpit {
  message: string | null;
  timer: Timer | null;
}

export interface StageState {
  scene: Scene;
  video: VideoState | null;
  pulpit: Pulpit;
  /** Sobe a cada mudança; uma tela que vê outro número pede o estado de novo. */
  rev: number;
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

/** Apresentação: páginas em imagem numa pasta do app. */
export interface Deck {
  id: string;
  title: string;
  /** Nomes dos arquivos, em ordem, dentro da pasta da apresentação. */
  pages: string[];
  createdAt: number;
  source: string;
}

export interface Lesson {
  number: string;
  title: string;
  src: string;
}

export interface LessonQuarter {
  quarter: string;
  title: string;
  cover: string | null;
  lessons: Lesson[];
}
