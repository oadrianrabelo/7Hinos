<!--
  Busca como no LouvorJA: o resultado mais exato já vem escolhido, com a letra
  ao lado; os parecidos (no título ou na letra) ficam na lista. Um clique
  mostra a letra, dois cliques (ou Enter) tocam.
-->
<script lang="ts">
  import { SearchX } from "@lucide/svelte";
  import LyricsPreview from "../components/LyricsPreview.svelte";
  import SongList from "../components/SongList.svelte";
  import { api } from "../lib/api";
  import { playSong } from "../lib/actions";
  import { searchSelection } from "../lib/state/search.svelte";
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
      // O primeiro (o mais exato) já vem com a letra.
      searchSelection.selected = r[0] ?? null;
    });
    return () => {
      cancelled = true;
    };
  });

  $effect(() => () => (searchSelection.selected = null));

  const selected = $derived(searchSelection.selected);

  /** Setas para cima e para baixo trocam a letra mostrada, mesmo digitando. */
  function onKey(e: KeyboardEvent) {
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
    if (!results.length) return;
    const target = e.target as HTMLElement;
    if (target.closest("textarea, select") || (target.closest("input") && !target.closest("[data-search]"))) return;
    e.preventDefault();
    e.stopPropagation();
    const i = results.findIndex((s) => s.id === selected?.id);
    const next = e.key === "ArrowDown" ? Math.min(results.length - 1, i + 1) : Math.max(0, i - 1);
    searchSelection.selected = results[next];
    document.querySelector(`[data-song="${results[next].id}"]`)?.scrollIntoView({ block: "nearest" });
  }
</script>

<svelte:window onkeydowncapture={onKey} />

{#if results.length}
  <p class="section-title count">
    {results.length === 60 ? "Mais de 60 resultados" : results.length === 1 ? "1 resultado" : `${results.length} resultados`} para “{query}”
    <span class="tip">· um clique mostra a letra · Enter ou dois cliques tocam · ↑ ↓ trocam</span>
  </p>
  <div class="split" class:single={results.length === 1}>
    <div class="results">
      <SongList songs={results} selected={selected?.id ?? null} onselect={(s) => (searchSelection.selected = s)} onplay={playSong} />
    </div>
    {#if selected}
      {#key selected.id}
        <LyricsPreview summary={selected} />
      {/key}
    {/if}
  </div>
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

  .split {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(18rem, 24rem);
    gap: 1rem;
    align-items: start;
  }

  .results {
    min-width: 0;
  }

  /* Tela estreita: a letra vai para baixo da lista. */
  @media (max-width: 900px) {
    .split {
      grid-template-columns: 1fr;
    }
  }
</style>
