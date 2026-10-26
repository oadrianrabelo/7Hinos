<script lang="ts">
  import { BookMarked, BookOpen, Clock, Film, Heart, Library, ListMusic, Megaphone, Presentation } from "@lucide/svelte";
  import SearchField from "../components/SearchField.svelte";
  import SongList from "../components/SongList.svelte";
  import { api } from "../lib/api";
  import { selectSong } from "../lib/actions";
  import { nav, type Route } from "../lib/state/nav.svelte";
  import { setlist } from "../lib/state/setlist.svelte";
  import type { SongSummary } from "../lib/types";

  let query = $state("");
  let results = $state<SongSummary[]>([]);

  $effect(() => {
    const q = query;
    if (!q.trim()) {
      results = [];
      return;
    }
    let cancelled = false;
    void api.search(q).then((r) => {
      if (!cancelled) results = r;
    });
    return () => (cancelled = true);
  });

  const tiles = [
    { label: "Culto de hoje", icon: ListMusic, route: { name: "setlist" }, tint: "#af52de" },
    { label: "Avisos e relógio", icon: Megaphone, route: { name: "screen" }, tint: "#ff6b3d" },
    { label: "Hinário", icon: BookOpen, route: { name: "hymnal" }, tint: "#1f6a90" },
    { label: "Bíblia", icon: BookMarked, route: { name: "bible" }, tint: "#a2845e" },
    { label: "Coletâneas e CDs", icon: Library, route: { name: "collections" }, tint: "#34a853" },
    { label: "Favoritos", icon: Heart, route: { name: "favorites" }, tint: "#ff2d55" },
    { label: "Tocados por último", icon: Clock, route: { name: "recent" }, tint: "#ff9500" },
    { label: "Vídeos", icon: Film, route: { name: "videos" }, tint: "#30b0c7" },
    { label: "Apresentações e lição", icon: Presentation, route: { name: "decks" }, tint: "#ff8a00" },
  ] satisfies { label: string; icon: unknown; route: Route; tint: string }[];
</script>

<div class="home">
  <h1>O que vamos cantar?</h1>
  <div data-tour="simple-search"><SearchField bind:value={query} large autofocus placeholder="Digite o nome, o número ou um pedaço da letra" onsubmit={() => results[0] && selectSong(results[0])} /></div>

  {#if query.trim()}
    {#if results.length}
      <div class="results card"><SongList songs={results} onselect={selectSong} /></div>
    {:else}
      <p class="muted none">Nenhum hino encontrado com “{query}”.</p>
    {/if}
  {:else}
    <div class="tiles" data-tour="simple-tiles">
      {#each tiles as t (t.label)}
        <button class="tile card" class:wide={t.route.name === "setlist"} class:full={t.route.name === "decks"} data-tour={t.route.name === "setlist" ? "simple-setlist" : undefined} style:--tint={t.tint} onclick={() => nav.push(t.route)}>
          <span class="icon"><t.icon size={30} /></span>
          <span class="label">
            {t.label}
            {#if t.route.name === "setlist"}
              <span class="muted small">
                {setlist.items.length ? `${setlist.items.length} na lista` : "Monte a ordem do culto"}
              </span>
            {/if}
          </span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .home {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    padding-top: 1rem;
  }

  h1 {
    font-size: 2.3rem;
    text-align: center;
  }

  .results {
    padding: 0.5rem;
  }

  .none {
    text-align: center;
    font-size: 1.15rem;
    padding: 2rem;
  }

  .tiles {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }

  .tile {
    display: flex;
    align-items: center;
    gap: 1rem;
    min-height: 6rem;
    padding: 1.1rem 1.25rem;
    font-size: 1.2rem;
    font-weight: 650;
    text-align: left;
    cursor: pointer;
    transition:
      transform var(--fast) var(--ease),
      box-shadow var(--normal);
  }

  .tile.full {
    grid-column: 1 / -1;
  }

  .tile.wide {
    grid-column: span 2;
  }

  .tile:hover {
    box-shadow: var(--shadow);
    transform: translateY(-2px);
  }

  .tile:active {
    transform: scale(0.98);
  }

  .label {
    display: flex;
    flex-direction: column;
  }

  .small {
    font-size: 0.85rem;
    font-weight: 500;
  }

  .icon {
    display: grid;
    place-items: center;
    width: 3.6rem;
    height: 3.6rem;
    flex: none;
    border-radius: var(--radius-l);
    color: #fff;
    background: linear-gradient(145deg, color-mix(in srgb, var(--tint) 80%, white), var(--tint));
  }
</style>
