<script lang="ts">
  import { BookOpen, Clock, Film, Heart } from "@lucide/svelte";
  import AlbumCard from "../components/AlbumCard.svelte";
  import SongList from "../components/SongList.svelte";
  import { api } from "../lib/api";
  import { playSong, selectSong } from "../lib/actions";
  import { catalog } from "../lib/state/catalog.svelte";
  import { nav } from "../lib/state/nav.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import type { SongSummary } from "../lib/types";

  let recent = $state<SongSummary[]>([]);
  $effect(() => {
    const ids = settings.current.recent.slice(0, 5);
    void api.songs(ids).then((s) => (recent = s));
  });

  const hour = new Date().getHours();
  const greeting = hour < 12 ? "Bom dia" : hour < 18 ? "Boa tarde" : "Boa noite";

  const shortcuts = [
    { label: "Hinário", detail: "Hinos numerados", icon: BookOpen, route: { name: "hymnal" }, tint: "#5b57e0" },
    { label: "Favoritos", detail: "Os que você marcou", icon: Heart, route: { name: "favorites" }, tint: "#ff2d55" },
    { label: "Recentes", detail: "Tocados por último", icon: Clock, route: { name: "recent" }, tint: "#ff9500" },
    { label: "Vídeos", detail: "Projetar vídeos", icon: Film, route: { name: "videos" }, tint: "#30b0c7" },
  ] as const;
</script>

<div class="home">
  <header>
    <h1>{greeting}</h1>
    <p class="muted">
      {catalog.status?.songs.toLocaleString("pt-BR")} músicas em {catalog.status?.albums.toLocaleString("pt-BR")} álbuns
    </p>
  </header>

  <div class="shortcuts">
    {#each shortcuts as s (s.label)}
      <button class="shortcut card" onclick={() => nav.reset(s.route)} style:--tint={s.tint}>
        <span class="icon"><s.icon size={20} /></span>
        <span class="label">{s.label}</span>
        <span class="detail">{s.detail}</span>
      </button>
    {/each}
  </div>

  {#if recent.length}
    <section>
      <div class="section-head">
        <h2>Tocados recentemente</h2>
        <button class="btn btn-plain" onclick={() => nav.reset({ name: "recent" })}>Ver todos</button>
      </div>
      <SongList songs={recent} onselect={selectSong} onplay={playSong} numbered={false} />
    </section>
  {/if}

  {#each catalog.collections as collection (collection.id)}
    <section>
      <div class="section-head">
        <h2>{collection.name}</h2>
        <button class="btn btn-plain" onclick={() => nav.push({ name: "collection", id: collection.id })}>
          Ver todos
        </button>
      </div>
      <div class="shelf">
        {#each collection.albums.slice(0, 12) as album (album.id)}
          <div class="shelf-item">
            <AlbumCard {album} onclick={() => nav.push({ name: "album", id: album.id })} />
          </div>
        {/each}
      </div>
    </section>
  {/each}
</div>

<style>
  .home {
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }

  header {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .shortcuts {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(10rem, 1fr));
    gap: 0.75rem;
  }

  .shortcut {
    display: grid;
    grid-template-columns: auto 1fr;
    grid-template-rows: auto auto;
    column-gap: 0.75rem;
    align-items: center;
    padding: 0.9rem 1rem;
    text-align: left;
    cursor: pointer;
    transition:
      transform var(--fast) var(--ease),
      box-shadow var(--normal) var(--ease);
  }

  .shortcut:hover {
    box-shadow: var(--shadow);
    transform: translateY(-1px);
  }

  .shortcut:active {
    transform: scale(0.98);
  }

  .icon {
    grid-row: span 2;
    display: grid;
    place-items: center;
    width: 2.4rem;
    height: 2.4rem;
    border-radius: var(--radius);
    color: #fff;
    background: linear-gradient(145deg, color-mix(in srgb, var(--tint) 80%, white), var(--tint));
  }

  .label {
    font-weight: 650;
  }

  .detail {
    font-size: 0.82rem;
    color: var(--text-2);
  }

  .section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.75rem;
  }

  .shelf {
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: minmax(9.5rem, 10.5rem);
    gap: 1.1rem;
    overflow-x: auto;
    padding-bottom: 0.5rem;
    scroll-snap-type: x proximity;
  }

  .shelf-item {
    scroll-snap-align: start;
    min-width: 0;
  }
</style>
