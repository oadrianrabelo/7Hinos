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
      { path: "/Users/igreja/Videos/Informativo Mundial das Missões.mp4", title: "Informativo Mundial das Missões" },
    ],
    keepPlayed: true,
    autoUpdateCatalog: true,
    autoUpdateApp: true,
  };
  let installed = params.get("installed") !== "0";
  let stage: StageState = { scene: { kind: "black" }, video: null };
  let outputs: Output[] = params.get("outputs") === "1" ? [{ label: "output-1", displayId: "tv", name: "Tela 2" }] : [];

  const changed = (event: string, payload: unknown) => void emit(event, payload);

  mockIPC(
    async (cmd, args) => {
      const a = (args ?? {}) as Record<string, any>;
      switch (cmd) {
        case "app_version":
          return "0.5.0";
        case "settings_get":
          return settings;
        case "settings_set":
          settings = a.settings;
          changed("settings://changed", settings);
          return settings;
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
        case "catalog_status":
          return { installed, version: installed ? 185 : null, songs: installed ? 2482 : 0, albums: installed ? 318 : 0 };
        case "catalog_sync":
          for (let done = 0; done <= 18_000_000; done += 1_500_000) {
            changed("catalog://progress", { done, total: 18_000_000 });
            await new Promise((r) => setTimeout(r, 120));
          }
          installed = true;
          return { installed, version: 185, songs: 2482, albums: 318 };
        case "collections":
          return collections;
        case "hymnal":
          return [summary(1, 1), summary(18, 18)];
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
        case "album_offline":
          return { cached: a.id === 727 ? 12 : 0, total: 12 };
        case "album_download":
          return { cached: 12, total: 12 };
        case "media_usage":
          return 734_003_200;
        case "stage_get":
          return stage;
        case "stage_show":
          stage = { ...stage, scene: a.scene };
          changed("stage://changed", stage);
          return;
        case "stage_video":
          stage = { ...stage, video: a.video };
          changed("stage://changed", stage);
          return;
        case "displays_list":
          return [
            { id: "builtin", name: "Tela 1", width: 2880, height: 1800, primary: true, hostsController: true, output: null },
            { id: "tv", name: "Tela 2", width: 1920, height: 1080, primary: false, hostsController: false, output: outputs.find((o) => o.displayId === "tv")?.label ?? null },
          ];
        case "outputs_list":
          return outputs;
        case "output_open":
          if (!outputs.some((o) => o.displayId === a.displayId)) {
            outputs = [...outputs, { label: `output-${outputs.length + 1}`, displayId: a.displayId, name: a.displayId ? "Tela 2" : "Janela" }];
          }
          changed("displays://changed", null);
          return outputs[outputs.length - 1].label;
        case "output_close":
          outputs = outputs.filter((o) => o.label !== a.label);
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
