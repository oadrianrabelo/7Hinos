<script lang="ts">
  import { Heart, LoaderCircle, Mic, Music, Pencil, Type } from "@lucide/svelte";
  import Cover from "../components/Cover.svelte";
  import { api } from "../lib/api";
  import type { PlayMode } from "../lib/slides";
  import { nav } from "../lib/state/nav.svelte";
  import { player } from "../lib/state/player.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import type { Song } from "../lib/types";

  let { id }: { id: number } = $props();
  let song = $state<Song | null>(null);
  let failed = $state(false);

  $effect(() => {
    const songId = id;
    song = null;
    failed = false;
    api
      .song(songId)
      .then((s) => (song = s))
      .catch(() => (failed = true));
  });

  async function start(mode: PlayMode) {
    if (!song) return;
    nav.push({ name: "live" });
    await player.present(song, mode);
  }
</script>

{#if song}
  <div class="song">
    <div class="art"><Cover media={song.cover} radius="var(--radius-xl)" /></div>
    <div class="head">
      <h1>{song.title}</h1>
      {#if song.subtitle}<p class="muted">{song.subtitle}</p>{/if}
      <button class="btn btn-lg fav" class:on={settings.isFavorite(song.id)} onclick={() => song && settings.toggleFavorite(song.id)}>
        <Heart size={20} fill={settings.isFavorite(song.id) ? "currentColor" : "none"} />
        {settings.isFavorite(song.id) ? "Nos favoritos" : "Adicionar aos favoritos"}
      </button>
      <button
        class="btn btn-plain"
        onclick={() => song && nav.push(song.id >= 900_000_000 ? { name: "editor", id: song.id } : { name: "editor", basedOn: song.id })}
      >
        <Pencil size={16} /> Corrigir a letra
      </button>
    </div>

    <div class="choices">
      <button class="choice card" disabled={!song.audio} onclick={() => start("voice")}>
        <span class="icon voice"><Mic size={30} /></span>
        <span class="label">Cantar com a música</span>
        <span class="muted">Com as vozes gravadas. A letra passa sozinha.</span>
      </button>
      <button class="choice card" disabled={!song.instrumental} onclick={() => start("playback")}>
        <span class="icon playback"><Music size={30} /></span>
        <span class="label">Só o acompanhamento</span>
        <span class="muted">{song.instrumental ? "Playback sem vozes. A letra passa sozinha." : "Este hino não tem playback."}</span>
      </button>
      <button class="choice card" onclick={() => start("lyrics")}>
        <span class="icon lyrics"><Type size={30} /></span>
        <span class="label">Só a letra</span>
        <span class="muted">Sem som. Você passa os slides.</span>
      </button>
    </div>
  </div>
{:else if failed}
  <div class="empty"><h2>Não foi possível abrir este hino</h2><p>Confira a internet e tente de novo.</p></div>
{:else}
  <div class="empty"><LoaderCircle size={32} class="spin" /></div>
{/if}

<style>
  .song {
    display: grid;
    grid-template-columns: 15rem 1fr;
    gap: 1.5rem 2rem;
    padding-top: 1rem;
  }

  .art :global(.cover) {
    box-shadow:
      inset 0 0 0 1px var(--border),
      var(--shadow-l);
  }

  .head {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: flex-start;
    gap: 0.5rem;
  }

  h1 {
    font-size: 2.4rem;
  }

  .head p {
    font-size: 1.2rem;
  }

  .fav {
    margin-top: 0.75rem;
  }

  .fav.on {
    color: #ff2d55;
  }

  .choices {
    grid-column: 1 / -1;
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }

  .choice {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.5rem;
    padding: 1.4rem;
    text-align: left;
    cursor: pointer;
    transition:
      transform var(--fast) var(--ease),
      box-shadow var(--normal),
      border-color var(--normal);
  }

  .choice:hover {
    border-color: var(--accent);
    box-shadow: 0 0 0 4px var(--accent-soft);
  }

  .choice:active {
    transform: scale(0.98);
  }

  .choice:disabled {
    opacity: 0.45;
    pointer-events: none;
  }

  .label {
    font-size: 1.3rem;
    font-weight: 700;
    margin-top: 0.4rem;
  }

  .icon {
    display: grid;
    place-items: center;
    width: 4rem;
    height: 4rem;
    border-radius: 50%;
    color: #fff;
  }

  .voice {
    background: linear-gradient(145deg, #3d8bb5, #1f6a90);
  }

  .playback {
    background: linear-gradient(145deg, #4cd4a0, #1f9d6c);
  }

  .lyrics {
    background: linear-gradient(145deg, #ffb340, #ff8a00);
  }
</style>
