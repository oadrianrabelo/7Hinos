<!--
  A letra de um hino ao lado da busca, como no LouvorJA: dá para conferir
  sem tocar e sem ocupar a tela toda.
-->
<script lang="ts">
  import { ListChecks, ListPlus, LoaderCircle, PanelRight, Play } from "@lucide/svelte";
  import { api } from "../lib/api";
  import { playSong, selectSong } from "../lib/actions";
  import { setlist } from "../lib/state/setlist.svelte";
  import type { Song, SongSummary } from "../lib/types";

  let { summary }: { summary: SongSummary } = $props();

  let song = $state<Song | null>(null);
  let failed = $state(false);

  $effect(() => {
    const id = summary.id;
    let cancelled = false;
    song = null;
    failed = false;
    void api
      .song(id)
      .then((s) => {
        if (!cancelled) song = s;
      })
      .catch(() => {
        if (!cancelled) failed = true;
      });
    return () => (cancelled = true);
  });

  // Só as estrofes: o slide de título já está no cabeçalho, e o refrão
  // repetido aparece uma vez (marcado), como numa folha de hinário.
  const stanzas = $derived.by(() => {
    if (!song) return [];
    const seen = new Set<string>();
    return song.slides
      .filter((s) => s.kind !== "title" && s.text.trim())
      .filter((s) => {
        const key = s.text.trim();
        if (s.repeat && seen.has(key)) return false;
        seen.add(key);
        return true;
      });
  });

  const item = $derived({ kind: "song" as const, id: summary.id, title: summary.title });
  const inSetlist = $derived(setlist.has(item));
</script>

<aside class="preview card">
  <header>
    {#if summary.number}<span class="number">{summary.number}</span>{/if}
    <div class="titles">
      <h2>{summary.title}</h2>
      {#if summary.subtitle}<span class="muted">{summary.subtitle}</span>{/if}
    </div>
  </header>

  <div class="actions">
    <button class="btn btn-primary" onclick={() => playSong(summary)}><Play size={15} fill="currentColor" /> Tocar</button>
    <button class="btn" onclick={() => selectSong(summary)} title="Abre no painel ao lado para escolher cantado, playback ou só letra">
      <PanelRight size={15} /> Abrir no painel
    </button>
    <button class="btn btn-icon" class:on={inSetlist} title={inSetlist ? "Está no culto de hoje" : "Adicionar ao culto de hoje"} aria-label={inSetlist ? "Tirar do culto de hoje" : "Adicionar ao culto de hoje"} onclick={() => setlist.toggle(item)}>
      {#if inSetlist}<ListChecks size={16} />{:else}<ListPlus size={16} />{/if}
    </button>
  </div>

  <div class="lyrics">
    {#if failed}
      <p class="muted">Não deu para abrir a letra.</p>
    {:else if !song}
      <p class="muted loading"><LoaderCircle size={16} class="spin" /> Abrindo a letra…</p>
    {:else if !stanzas.length}
      <p class="muted">Esta música não tem letra cadastrada.</p>
    {:else}
      {#each stanzas as s, i (i)}
        <p class="stanza" class:chorus={s.repeat}>
          {#if s.aux}<span class="aux">{s.aux}</span>{/if}
          {s.text}
        </p>
      {/each}
    {/if}
  </div>
</aside>

<style>
  .preview {
    position: sticky;
    top: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    max-height: calc(100vh - 6rem);
    padding: 1rem 1.1rem;
  }

  header {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
  }

  .number {
    flex: none;
    min-width: 2.4rem;
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-s);
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    text-align: center;
  }

  .titles {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  h2 {
    margin: 0;
    font-size: 1.15rem;
    line-height: 1.25;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .actions .on {
    color: var(--accent);
  }

  .lyrics {
    overflow-y: auto;
    padding-right: 0.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }

  .stanza {
    margin: 0;
    white-space: pre-line;
    line-height: 1.45;
  }

  /* O refrão em destaque, como o dourado do telão. */
  .stanza.chorus {
    padding-left: 0.7rem;
    border-left: 3px solid #e0a82e;
    font-style: italic;
  }

  .aux {
    display: block;
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text-2);
  }

  .loading {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }
</style>
