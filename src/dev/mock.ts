// Backend simulado para desenvolver a interface no navegador, sem Tauri e sem
// internet: `npm run dev:mock`. Nunca entra no build de produção.

import { emit } from "@tauri-apps/api/event";
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import type {
  Album,
  AlbumSummary,
  Collection,
  Media,
  Output,
  Settings,
  Slide,
  Song,
  SongSummary,
  StageState,
} from "../lib/types";

const art = (a: string, b: string, shape = "") =>
  "data:image/svg+xml," +
  encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 160 90" preserveAspectRatio="xMidYMid slice"><defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="${a}"/><stop offset="1" stop-color="${b}"/></linearGradient><radialGradient id="r" cx=".75" cy=".2" r=".6"><stop offset="0" stop-color="#fff" stop-opacity=".55"/><stop offset="1" stop-color="#fff" stop-opacity="0"/></radialGradient></defs><rect width="160" height="90" fill="url(#g)"/><rect width="160" height="90" fill="url(#r)"/>${shape}</svg>`,
  );

const hills = (c: string) => `<path d="M0 70 Q40 45 80 62 T160 55 V90 H0Z" fill="${c}" opacity=".55"/><path d="M0 78 Q50 60 100 74 T160 70 V90 H0Z" fill="${c}" opacity=".8"/>`;

const media = (url: string): Media => ({ url, path: null });

const backgrounds = [
  art("#f6a35b", "#7b3f98", hills("#2b1745")),
  art("#5ec2f2", "#1d4f91", hills("#0d2547")),
  art("#9be15d", "#00a38c", hills("#064a3c")),
  art("#ffd86f", "#fc6262", hills("#5a1e2c")),
];

const covers: Record<number, string> = {
  727: art("#1d1d1b", "#c2442d"),
  726: art("#ffb347", "#ff5e62"),
  725: art("#43cea2", "#185a9d"),
  724: art("#8e2de2", "#4a00e0"),
  701: art("#1e3c72", "#2a5298"),
  702: art("#7f5539", "#3e2723"),
  690: art("#f7971e", "#ffd200"),
  691: art("#56ccf2", "#2f80ed"),
};

const lyrics: Record<number, { title: string; parts: string[]; aux?: string[] }> = {
  1: {
    title: "Santo! Santo! Santo!",
    parts: [
      "Santo! Santo! Santo!\nDeus onipotente!",
      "Cedo de manhã\ncantaremos Teu louvor",
      "Santo! Santo! Santo!\nDeus Jeová triúno,",
      "És um só Deus,\nexcelso Criador",
      "Santo! Santo! Santo!\nDeus onipotente!",
    ],
  },
  18: {
    title: "Castelo Forte",
    parts: [
      "Castelo forte é nosso Deus,\nespada e bom escudo;",
      "Com Seu poder defende os Seus\nem todo transe agudo.",
      "Com fúria pertinaz\npersegue Satanás,",
      "Com artimanhas tais\ne astúcias tão cruéis,",
      "Que iguais não há na Terra.",
    ],
  },
  101: {
    title: "Meu Lugar no Mundo",
    parts: [
      "Eu encontrei\nmeu lugar no mundo",
      "Nos braços do Pai\neu descanso enfim",
      "Meu lugar é contigo,\nSenhor",
      "Meu lugar é contigo,\nSenhor",
      "Pra sempre Te seguirei",
    ],
    aux: ["", "", "Refrão", "Refrão", ""],
  },
  102: { title: "Brilha Jesus", parts: ["Brilha, Jesus,\nno meu coração", "Ilumina meu caminho", "Brilha, Jesus,\nno meu coração"] },
  103: { title: "Nosso Sol é Jesus", parts: ["O nosso sol\nveio iluminar", "A escuridão\nque havia em nós", "Nosso sol é Jesus!"] },
  104: { title: "Ele Está ao Leme", parts: ["Quando o mar\nse levanta", "Ele está ao leme", "Não temerei"] },
  105: { title: "Grandioso És Tu", parts: ["Senhor, meu Deus,\nquando eu maravilhado", "Contemplo a Tua\nimensa criação", "Então minha alma\ncanta a Ti, Senhor:", "Quão grande és Tu!\nQuão grande és Tu!"] },
  106: { title: "Jesus, Meu Melhor Amigo", parts: ["Jesus é meu\nmelhor amigo", "Ele nunca me deixa", "Jesus é meu\nmelhor amigo"] },
};

const summary = (id: number, number: number | null = null, subtitle = "Hinário Adventista"): SongSummary => ({
  id,
  title: lyrics[id].title,
  subtitle,
  number,
  hasInstrumental: id % 2 === 0,
  durationMs: 150_000 + id * 1000,
});

const albums: Record<number, Album> = {
  727: { id: 727, name: "Meu Lugar no Mundo", color: "#C2442D", cover: media(covers[727]), songs: [101, 104, 105].map((id, i) => ({ ...summary(id, i + 1, "Meu Lugar no Mundo") })) },
  726: { id: 726, name: "Brilha Jesus", color: "#ff5e62", cover: media(covers[726]), songs: [102, 103, 106].map((id, i) => ({ ...summary(id, i + 1, "Brilha Jesus") })) },
  725: { id: 725, name: "Já é Tempo", color: "#185a9d", cover: media(covers[725]), songs: [104, 105].map((id, i) => summary(id, i + 1, "Já é Tempo")) },
  724: { id: 724, name: "Tempo de Esperança", color: "#4a00e0", cover: media(covers[724]), songs: [103].map((id, i) => summary(id, i + 1, "Tempo de Esperança")) },
  701: { id: 701, name: "Hinário Adventista", color: "#2a5298", cover: media(covers[701]), songs: [summary(1, 1), summary(18, 18)] },
  702: { id: 702, name: "Hinário 1996", color: "#3e2723", cover: media(covers[702]), songs: [summary(18, 18)] },
  690: { id: 690, name: "Músicas Infantis", color: "#ffd200", cover: media(covers[690]), songs: [summary(106, 1, "Músicas Infantis")] },
  691: { id: 691, name: "Cantinho Feliz", color: "#2f80ed", cover: media(covers[691]), songs: [summary(102, 1, "Cantinho Feliz")] },
};

const albumSummary = (id: number, subtitle: string | null): AlbumSummary => ({
  id,
  name: albums[id].name,
  subtitle,
  color: albums[id].color,
  cover: albums[id].cover,
});

const collections: Collection[] = [
  { id: 6, name: "CDs Oficiais/Ano", slug: "aym", albums: [albumSummary(727, "2026"), albumSummary(726, "2025"), albumSummary(725, "2024"), albumSummary(724, "2023")] },
  { id: 2, name: "Hinários", slug: "hymnal", albums: [albumSummary(701, "Atual"), albumSummary(702, "1996")] },
  { id: 9, name: "Infantis", slug: "children", albums: [albumSummary(690, null), albumSummary(691, null)] },
];

const allSongs: SongSummary[] = Object.keys(lyrics).map((id) => summary(Number(id), Number(id) < 100 ? Number(id) : null));

let silent: string | null = null;
function silentAudio(seconds = 90): string {
  if (silent) return silent;
  const rate = 8000;
  const data = new Uint8Array(44 + rate * seconds).fill(128);
  const view = new DataView(data.buffer);
  const text = (o: number, s: string) => [...s].forEach((c, i) => view.setUint8(o + i, c.charCodeAt(0)));
  text(0, "RIFF");
  view.setUint32(4, 36 + rate * seconds, true);
  text(8, "WAVEfmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, 1, true);
  view.setUint32(24, rate, true);
  view.setUint32(28, rate, true);
  view.setUint16(32, 1, true);
  view.setUint16(34, 8, true);
  text(36, "data");
  view.setUint32(40, rate * seconds, true);
  silent = URL.createObjectURL(new Blob([data], { type: "audio/wav" }));
  return silent;
}

function song(id: number): Song {
  const l = lyrics[id];
  const bg = backgrounds[id % backgrounds.length];
  const slides: Slide[] = [
    { kind: "title", text: l.title, aux: "Meu Lugar no Mundo", image: media(bg), imagePosition: 5, repeat: false, timeMs: 0, instrumentalTimeMs: 0 },
    ...l.parts.map((text, i, all) => ({
      kind: "lyric" as const,
      text,
      aux: l.aux?.[i] || null,
      image: media(backgrounds[(id + (i > 2 ? 1 : 0)) % backgrounds.length]),
      imagePosition: 5,
      repeat: i > 0 && text === all[i - 1],
      timeMs: 6000 + i * 9000,
      instrumentalTimeMs: 6000 + i * 9000,
    })),
  ];
  return {
    id,
    title: l.title,
    subtitle: "Meu Lugar no Mundo",
    cover: media(covers[727]),
    audio: media(silentAudio()),
    instrumental: id % 2 === 0 ? media(silentAudio()) : null,
    durationMs: 90_000,
    slides,
    custom: false,
  };
}

const normalize = (s: string) =>
  s.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLowerCase();

export function install() {
  const params = new URLSearchParams(location.search);
  const label = params.get("window") ?? "main";
  mockWindows(label);

  let settings: Settings = {
    onboarded: params.get("onboarded") !== "0",
    mode: (params.get("mode") as Settings["mode"]) ?? "full",
    theme: (params.get("theme") as Settings["theme"]) ?? "system",
    language: "pt",
    uiScale: 1,
    slide: { uppercase: true, textScale: 1, showImages: true, transitionMs: 450 },
    favorites: [18],
    recent: [101, 1, 18],
    videos: [
      { path: "/Users/igreja/Videos/Abertura Escola Sabatina.mp4", title: "Abertura Escola Sabatina" },
      { path: "/Users/igreja/Videos/Informativo Mundial das Missões.mp4", title: "Informativo Mundial das Missões", tag: "missoes" },
    ],
    setlist:
      params.get("setlist") === "0"
        ? []
        : [
            { kind: "song", id: 101, title: "Meu Lugar no Mundo" },
            { kind: "song", id: 1, title: "Santo! Santo! Santo!" },
            { kind: "video", path: "/Users/igreja/Videos/Informativo Mundial das Missões.mp4", title: "Informativo Mundial das Missões" },
            { kind: "song", id: 18, title: "Castelo Forte" },
          ],
    idle: { image: null, text: "Bem-vindos!", showClock: true },
    monitor: { showNext: params.get("next") !== "0", showClock: true, showTitle: true, showBackground: params.get("bg") === "1", textScale: 1 },
    screenProfiles: [
      {
        id: 1,
        name: "Culto normal",
        outputs: [
          { displayId: "tv", displayName: "Tela 2", width: 1920, height: 1080, role: "audience" },
          { displayId: "stage", displayName: "Tela 3", width: 1920, height: 1080, role: "monitor" },
        ],
      },
      { id: 2, name: "Só o telão", outputs: [{ displayId: "tv", displayName: "Tela 2", width: 1920, height: 1080, role: "audience" }] },
    ],
    defaultProfile: 1,
    screenStyle: { font: "condensed", textColor: "#ffffff", accentColor: "#ffc94a", background: "gradient", backgroundImage: null },
    notices: [
      { id: 1, title: "Santa Ceia", body: "Neste sábado, às 10h30. Prepare o coração." },
    ],
    bibleVersion: null,
    keepPlayed: true,
    autoUpdateCatalog: true,
    autoUpdateApp: true,
    localSources: params.get("local") === "1" ? ["C:\\Program Files (x86)\\Louvor JA"] : [],
    legacyMigrated: true,
    pendingFavorites: [],
  };
  let installed = params.get("installed") !== "0";
  // ?scene=slide: as telas já abrem com um hino no ar (para ver telão e retorno).
  const sample = song(1);
  let stage: StageState = {
    scene: params.get("scene") === "slide" ? { kind: "slide", slide: sample.slides[1], title: sample.title, next: sample.slides[2] } : { kind: "black" },
    video: null,
    pulpit:
      params.get("pulpit") === "1"
        ? { message: "Faltam 5 minutos", timer: { elapsedMs: 26 * 60_000, startedAt: Date.now(), limitMs: 30 * 60_000 } }
        : { message: null, timer: null },
    rev: 0,
  };
  let outputs: Output[] = [
    ...(params.get("outputs") === "1" ? [{ label: "output-1", displayId: "tv", name: "Tela 2", role: "audience" as const }] : []),
    ...(params.get("monitor") === "1" ? [{ label: "output-9", displayId: "stage", name: "Tela 3", role: "monitor" as const }] : []),
  ];

  const changed = (event: string, payload: unknown) => void emit(event, payload);
  let jobs = 0;
  const status = () => ({
    installed,
    version: installed ? 185 : null,
    songs: installed ? 2482 : 0,
    albums: installed ? 318 : 0,
    lastCheck: installed ? Math.floor(Date.now() / 1000) - 600 : null,
    available: 185,
    lastError: params.get("offline") === "1" ? "Não foi possível falar com o servidor." : null,
  });

  mockIPC(
    async (cmd, args) => {
      const a = (args ?? {}) as Record<string, any>;
      switch (cmd) {
        case "media_base":
          return "";
        case "app_version":
          return "0.5.0";
        case "settings_get":
          return settings;
        case "settings_update":
          settings = {
            ...settings,
            ...a.patch,
            slide: { ...settings.slide, ...(a.patch.slide ?? {}) },
            monitor: { ...settings.monitor, ...(a.patch.monitor ?? {}) },
            screenStyle: { ...settings.screenStyle, ...(a.patch.screenStyle ?? {}) },
          };
          changed("settings://changed", settings);
          return settings;
        case "catalog_check_now":
          return;
        case "diagnostics":
          return { version: "0.5.0", os: "macos", arch: "aarch64", catalog: status(), localSources: settings.localSources, outputs: outputs.length, logDir: "/tmp" };
        case "favorite_toggle":
          settings = {
            ...settings,
            favorites: settings.favorites.includes(a.id) ? settings.favorites.filter((f) => f !== a.id) : [a.id, ...settings.favorites],
          };
          changed("settings://changed", settings);
          return settings;
        case "videos_add":
        case "videos_remove":
          return settings;
        case "video_tag": {
          const weekly = ["missoes", "provai"].includes(a.tag);
          settings = {
            ...settings,
            videos: settings.videos.map((v) =>
              v.path === a.path ? { ...v, tag: a.tag } : weekly && v.tag === a.tag ? { ...v, tag: null } : v,
            ),
          };
          changed("settings://changed", settings);
          return settings;
        }
        case "catalog_status":
          return status();
        case "catalog_sync":
          for (let done = 0; done <= 18_000_000; done += 1_500_000) {
            changed("catalog://progress", { done, total: 18_000_000 });
            await new Promise((r) => setTimeout(r, 120));
          }
          installed = true;
          return status();
        case "louvorja_detect":
          return params.get("local") === "1" ? [{ root: "C:\\Program Files (x86)\\Louvor JA", database: "" }] : [];
        case "louvorja_import":
          await new Promise((r) => setTimeout(r, 1500));
          installed = true;
          settings = { ...settings, onboarded: true };
          return status();
        case "collections":
          return collections;
        case "hymnal":
          return a.edition === "1996" ? [summary(18, 1)] : [summary(1, 1), summary(18, 18)];
        case "search": {
          const q = normalize(a.query.trim());
          return allSongs.filter((s) => normalize(s.title).includes(q) || String(s.number) === q);
        }
        case "songs":
          return (a.ids as number[]).filter((id) => lyrics[id]).map((id) => summary(id));
        case "album":
          return albums[a.id];
        case "song":
          return song(a.id);
        case "offline_status":
          return { cached: a.scope.kind === "album" && a.scope.id === 727 ? 12 : 3, total: 12 };
        case "songs_available":
          return (a.ids as number[]).filter((id) => id !== 18);
        case "download_start": {
          const job = ++jobs;
          void (async () => {
            for (let cached = 0; cached <= 40; cached += 4) {
              await new Promise((r) => setTimeout(r, 300));
              changed("download://progress", { job, label: a.label, cached, total: 40, finished: cached === 40, cancelled: false, error: null });
            }
          })();
          return job;
        }
        case "download_cancel":
          return;
        case "user_songs_list":
          return [
            { id: 900000000, title: "Corinho da Família", author: "Ministério jovem", slides: [{ text: "Somos família", aux: null }], basedOn: null, audio: null, background: null, updatedAt: 0 },
            { id: 900000001, title: "Castelo Forte", author: "", slides: [{ text: "Letra corrigida", aux: null }], basedOn: 18, audio: null, background: null, updatedAt: 0 },
          ];
        case "user_song_get":
          return null;
        case "user_song_save":
          return { ...a.song, id: a.song.id || 900000002 };
        case "user_song_delete":
          return;
        case "user_media_dir":
          return "/mock/user-media";
        case "user_media_add":
          return "arquivo.mp3";
        case "bible_versions":
          return [
            { id: 1, abbreviation: "ARA", name: "Almeida Revista e Atualizada" },
            { id: 2, abbreviation: "NVI", name: "Nova Versão Internacional" },
          ];
        case "bible_books":
          return [
            { number: 1, name: "Gênesis", abbreviation: "Gn", chapters: 50, testament: 1 },
            { number: 19, name: "Salmos", abbreviation: "Sl", chapters: 150, testament: 1 },
            { number: 23, name: "Isaías", abbreviation: "Is", chapters: 66, testament: 1 },
            { number: 40, name: "Mateus", abbreviation: "Mt", chapters: 28, testament: 2 },
            { number: 43, name: "João", abbreviation: "Jo", chapters: 21, testament: 2 },
            { number: 45, name: "Romanos", abbreviation: "Rm", chapters: 16, testament: 2 },
          ];
        case "bible_chapter":
          return a.book === 19 && a.chapter === 23
            ? [
                "O Senhor é o meu pastor; nada me faltará.",
                "Ele me faz repousar em pastos verdejantes. Leva-me para junto das águas de descanso;",
                "refrigera-me a alma. Guia-me pelas veredas da justiça por amor do seu nome.",
                "Ainda que eu ande pelo vale da sombra da morte, não temerei mal nenhum, porque tu estás comigo; o teu bordão e o teu cajado me consolam.",
                "Preparas-me uma mesa na presença dos meus adversários, unges-me a cabeça com óleo; o meu cálice transborda.",
                "Bondade e misericórdia certamente me seguirão todos os dias da minha vida; e habitarei na Casa do Senhor para todo o sempre.",
              ].map((text, i) => ({ number: i + 1, text }))
            : Array.from({ length: 12 }, (_, i) => ({ number: i + 1, text: `Texto do versículo ${i + 1}.` }));
        case "bible_search":
          return [{ book: 19, chapter: 23, verse: 1, text: "O Senhor é o meu pastor; nada me faltará." }];
        case "package_inspect":
          return { format: 1, createdAt: Math.floor(Date.now() / 1000) - 86400, appVersion: "0.5.0", catalogVersion: 185, songs: 1889, mediaFiles: 812, videos: 2, bytes: 3_400_000_000 };
        case "package_export":
        case "package_import":
          for (let done = 0; done <= 20; done += 2) {
            changed("package://progress", { done, total: 20 });
            await new Promise((r) => setTimeout(r, 150));
          }
          installed = true;
          settings = { ...settings, onboarded: true };
          return { format: 1, createdAt: Math.floor(Date.now() / 1000), appVersion: "0.5.0", catalogVersion: 185, songs: 1889, mediaFiles: 812, videos: 2, bytes: 3_400_000_000 };
        case "media_usage":
          return 734_003_200;
        case "stage_get":
          return stage;
        case "stage_show":
          stage = { ...stage, scene: a.scene, rev: stage.rev + 1 };
          changed("stage://changed", stage);
          return;
        case "stage_pulpit":
          stage = { ...stage, pulpit: a.pulpit, rev: stage.rev + 1 };
          changed("stage://changed", stage);
          return;
        case "displays_identify":
          changed("displays://identify", null);
          return;
        case "reset_all":
          location.search = "?onboarded=0&installed=0";
          return;
        case "stage_video":
          stage = { ...stage, video: a.video, rev: stage.rev + 1 };
          changed("stage://changed", stage);
          return;
        case "displays_list":
          return [
            { id: "builtin", name: "Tela 1", width: 2880, height: 1800, primary: true, hostsController: true, output: null },
            { id: "tv", name: "Tela 2", width: 1920, height: 1080, primary: false, hostsController: false, output: outputs.find((o) => o.displayId === "tv")?.label ?? null },
            { id: "stage", name: "Tela 3", width: 1920, height: 1080, primary: false, hostsController: false, output: outputs.find((o) => o.displayId === "stage")?.label ?? null },
          ];
        case "outputs_list":
          return outputs;
        case "output_open":
          {
            const role = a.role ?? "audience";
            const existing = outputs.find((o) => a.displayId && o.displayId === a.displayId);
            if (existing) outputs = outputs.map((o) => (o === existing ? { ...o, role } : o));
            else {
              const name = a.displayId === "tv" ? "Tela 2" : a.displayId === "stage" ? "Tela 3" : "Janela";
              outputs = [...outputs, { label: `output-${outputs.length + 1}`, displayId: a.displayId, name, role }];
            }
            changed("displays://changed", null);
            return (existing ?? outputs[outputs.length - 1]).label;
          }
        case "output_close":
          outputs = outputs.filter((o) => o.label !== a.label);
          changed("displays://changed", null);
          return;
        case "outputs_reopen":
          changed("displays://changed", null);
          return;
        case "outputs_close_all":
          outputs = [];
          return;
        case "plugin:updater|check":
          return null;
        case "plugin:dialog|open":
          return null;
        case "plugin:window|is_fullscreen":
          return false;
        default:
          return null;
      }
    },
    { shouldMockEvents: true },
  );
}
