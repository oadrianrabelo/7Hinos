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

const firstLine = (text: string) => text.split("\n").find((l) => l.trim())?.trim() ?? "";

class RemoteControl {
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

  async run(command: { action: string; index?: number; query?: string }) {
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
        if (!command.query?.trim()) return;
        const [first] = await api.search(command.query.trim());
        if (!first) return;
        // Com um hino no ar, o novo vai para "Próximo" (nunca interrompe).
        const loaded = await player.open(first.id);
        if (loaded && !player.live) await player.start(loaded.audio ? "voice" : "lyrics");
        return;
      }
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
