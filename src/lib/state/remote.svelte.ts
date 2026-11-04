// Controle pelo celular: executa os comandos que chegam pela rede local e
// conta ao celular o que está no ar. A lógica toda continua aqui na janela de
// controle; o celular só aperta botões.

import { openSetItem } from "../actions";
import { api, on } from "../api";
import { bible } from "./bible.svelte";
import { decks } from "./decks.svelte";
import { live } from "./live.svelte";
import { player } from "./player.svelte";
import { screen } from "./screen.svelte";
import { setlist } from "./setlist.svelte";
import { stage } from "./stage.svelte";
import { video } from "./video.svelte";
import { assignedNames, serviceRoles, shortTime, todayServices } from "../roster";
import { roster } from "./roster.svelte";
import { settings } from "./settings.svelte";
import type { SongSummary } from "../types";

const firstLine = (text: string) => text.split("\n").find((l) => l.trim())?.trim() ?? "";

class RemoteControl {
  /** Última busca feita pelo celular (aparece lá como lista para tocar). */
  search = $state<{ query: string; results: SongSummary[] } | null>(null);

  async init() {
    await on("remote://command", (command) => void this.run(command));
    // Publica o que está no ar sempre que algo muda (com um respiro).
    $effect.root(() => {
      $effect(() => {
        const status = JSON.stringify(this.status());
        const timer = setTimeout(() => void api.remotePublish(status).catch(() => {}), 120);
        return () => clearTimeout(timer);
      });
    });
  }

  /** Quem recebe os comandos: quem está no telão, ou o hino aberto no painel. */
  #presenter() {
    return live.current ?? (player.song ? player : null);
  }

  async run(command: { action: string; index?: number; query?: string; id?: number }) {
    const presenter = this.#presenter();
    switch (command.action) {
      case "next":
        return presenter?.next();
      case "previous":
        return presenter?.previous();
      case "blackout":
        return presenter?.toggleBlackout();
      case "togglePlay":
        return presenter?.togglePlay?.();
      case "stop":
        if (video.state) video.stop();
        return presenter?.stop();
      case "goto":
        if (command.index === undefined) return;
        if (live.source === "deck") return decks.show(command.index);
        if (player.song) return player.go(command.index);
        return;
      case "setlist": {
        const item = command.index !== undefined ? setlist.items[command.index] : undefined;
        if (item) await openSetItem(item);
        return;
      }
      case "song": {
        const query = command.query?.trim() ?? "";
        if (!query && command.id === undefined) return;
        let id = command.id;
        if (id === undefined) {
          const results = await api.search(query);
          // Número: toca o primeiro. Nome ou letra: o celular escolhe.
          if (!/^\d+$/.test(query)) {
            this.search = { query, results: results.slice(0, 12) };
            return;
          }
          id = results[0]?.id;
        }
        if (id === undefined) return;
        this.search = null;
        // Com um hino no ar, o novo vai para "Próximo" (nunca interrompe).
        const loaded = await player.open(id);
        if (loaded && !player.live) await player.start(loaded.audio ? "voice" : "lyrics");
        return;
      }
      case "clearSearch":
        this.search = null;
        return;
      case "notice": {
        const notice = command.index !== undefined ? settings.current.notices[command.index] : undefined;
        if (notice) screen.showNotice(notice);
        return;
      }
      case "idle":
        return screen.rest();
    }
  }

  /** O que o celular mostra. */
  status() {
    const source = live.source;
    const items = setlist.items;
    const base = {
      setlist: items.map((i) => ({ kind: i.kind, title: i.note ? `${i.title} — ${i.note}` : i.title })),
      setlistCurrent:
        setlist.currentIndex >= 0
          ? setlist.currentIndex
          : items.findIndex(
              (i) => (i.kind === "song" && player.song?.id === i.id) || (i.kind === "deck" && decks.deck?.id === i.id && decks.live),
            ),
      notices: settings.current.notices.map((n) => n.title || n.body.slice(0, 40)),
      noticeCurrent: live.source === "text" && screen.notice ? settings.current.notices.findIndex((n) => n.id === screen.notice?.id) : -1,
      search: this.search
        ? { query: this.search.query, results: this.search.results.map((r) => ({ id: r.id, title: r.number ? `${r.number} · ${r.title}` : r.title, sub: r.subtitle })) }
        : null,
      roster: todayServices(roster.doc).map((x) => ({
        title: `${x.name} · ${shortTime(x.time)}`,
        rows: serviceRoles(roster.doc, x)
          .map((r) => ({ role: r.name, who: assignedNames(roster.doc, x.assigned[r.id]) }))
          .filter((r) => r.who),
      })),
    };
    if (source === "deck" && decks.deck) {
      return {
        ...base,
        live: true,
        blackout: decks.blackout,
        title: decks.deck.title,
        text: `Página ${decks.index + 1}`,
        index: decks.index,
        total: decks.deck.pages.length,
        slides: decks.deck.pages.map((_, i) => `Página ${i + 1}`),
      };
    }
    if (source === "bible") {
      return { ...base, live: true, blackout: bible.blackout, title: bible.slide?.aux ?? "Bíblia", text: bible.slide?.text ?? "" };
    }
    if (source === "text" || source === "clock") {
      return { ...base, live: true, title: source === "clock" ? "Relógio" : "Aviso", text: screen.notice?.title ?? "" };
    }
    if (player.song) {
      return {
        ...base,
        live: player.live,
        blackout: player.blackout,
        canPause: player.live && player.hasAudio,
        playing: player.playing,
        title: player.song.title,
        text: player.slide?.text ?? "",
        index: player.index,
        total: player.song.slides.length,
        slides: player.song.slides.map((s) => firstLine(s.text) || s.aux || "—"),
      };
    }
    if (video.state) return { ...base, live: true, title: `Vídeo: ${video.state.title}`, text: "" };
    return { ...base, live: false, title: stage.current.scene.kind === "idle" ? "Tela de espera" : "", text: "" };
  }
}

export const remote = new RemoteControl();
