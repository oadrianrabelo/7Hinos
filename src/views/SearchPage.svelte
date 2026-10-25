<script lang="ts">
  import { SearchX } from "@lucide/svelte";
  import SongList from "../components/SongList.svelte";
  import { api } from "../lib/api";
  import { playSong, selectSong } from "../lib/actions";
  import type { SongSummary } from "../lib/types";

  let { query }: { query: string } = $props();

  let results = $state<SongSummary[]>([]);
  let searched = $state(false);

  $effect(() => {
    const q = query;
    let cancelled = false;
    void api.search(q).then((r) => {
      if (cancelled) return;
      results = r;
      searched = true;
    });
    return () => (cancelled = true);
  });
</script>

{#if results.length}
  <p class="section-title count">
    {results.length === 60 ? "Mais de 60 resultados" : results.length === 1 ? "1 resultado" : `${results.length} resultados`} para “{query}”
    <span class="tip">· Enter toca o primeiro · dois cliques tocam qualquer um</span>
  </p>
  <SongList songs={results} onselect={selectSong} onplay={playSong} />
{:else if searched && query.trim()}
  <div class="empty">
    <SearchX size={44} strokeWidth={1.4} />
    <h2>Nada encontrado</h2>
    <p>Tente o número do hino, outra palavra do título ou um trecho da letra.</p>
  </div>
{/if}

<style>
  .count {
    margin: 0 0 0.5rem 0.75rem;
  }

  .tip {
    text-transform: none;
    letter-spacing: 0;
    font-weight: 400;
  }
</style>
