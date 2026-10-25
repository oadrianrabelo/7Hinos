<!-- A programação do culto: monte antes, apresente na ordem. -->
<script lang="ts">
  import { ArrowDown, ArrowUp, Film, ListMusic, Music, Play, Trash2, X } from "@lucide/svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import { openSetItem } from "../lib/actions";
  import { player } from "../lib/state/player.svelte";
  import { setlist } from "../lib/state/setlist.svelte";
  import { video } from "../lib/state/video.svelte";
  import type { SetItem } from "../lib/types";

  let { simple = false }: { simple?: boolean } = $props();

  const isCurrent = (item: SetItem) =>
    item.kind === "song" ? player.song?.id === item.id : video.state?.path === item.path;
</script>

<PageHeader title="Culto de hoje" subtitle="Monte a ordem antes do culto e vá apresentando um por um">
  {#snippet actions()}
    {#if setlist.items.length}
      <button class="btn" class:btn-lg={simple} onclick={() => setlist.clear()}><Trash2 size={15} /> Limpar lista</button>
    {/if}
  {/snippet}
</PageHeader>

{#if setlist.items.length}
  <ol class="setlist">
    {#each setlist.items as item, i (item.kind + (item.kind === "song" ? item.id : item.path))}
      <li class="entry" class:current={isCurrent(item)} class:simple>
        <span class="n">{i + 1}</span>
        <span class="icon" class:video={item.kind === "video"}>
          {#if item.kind === "song"}<Music size={18} />{:else}<Film size={18} />{/if}
        </span>
        <span class="info">
          <span class="title truncate">{item.title}</span>
          <span class="kind">{item.kind === "song" ? "Hino" : "Vídeo"}</span>
        </span>
        <span class="order">
          <button class="btn btn-plain btn-icon" aria-label="Subir" title="Subir" disabled={i === 0} onclick={() => setlist.move(i, -1)}><ArrowUp size={16} /></button>
          <button class="btn btn-plain btn-icon" aria-label="Descer" title="Descer" disabled={i === setlist.items.length - 1} onclick={() => setlist.move(i, 1)}><ArrowDown size={16} /></button>
          <button class="btn btn-plain btn-icon" aria-label="Tirar da lista" title="Tirar da lista" onclick={() => setlist.remove(i)}><X size={16} /></button>
        </span>
        <button class="btn btn-primary" class:btn-lg={simple} onclick={() => openSetItem(item)}>
          <Play size={15} fill="currentColor" />
          {item.kind === "song" ? "Abrir" : "Exibir"}
        </button>
      </li>
    {/each}
  </ol>
{:else}
  <div class="empty">
    <ListMusic size={44} strokeWidth={1.4} />
    <h2>A lista está vazia</h2>
    <p>
      Toque no ícone <ListMusic size={14} /> ao lado de um hino ou vídeo para colocá-lo aqui.<br />
      Na hora do culto, é só seguir a ordem.
    </p>
  </div>
{/if}

<style>
  .setlist {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .entry {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    padding: 0.55rem 0.7rem;
    border-radius: var(--radius-l);
    background: var(--surface);
    border: 1px solid var(--border);
    transition:
      border-color var(--normal),
      box-shadow var(--normal);
  }

  .entry.simple {
    padding: 0.9rem 1rem;
    font-size: 1.1rem;
  }

  .entry.current {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .n {
    width: 1.6rem;
    text-align: right;
    font-weight: 650;
    color: var(--text-3);
    font-variant-numeric: tabular-nums;
  }

  .icon {
    display: grid;
    place-items: center;
    width: 2.3rem;
    height: 2.3rem;
    flex: none;
    border-radius: var(--radius);
    color: #fff;
    background: linear-gradient(145deg, #8c88ff, var(--accent));
  }

  .icon.video {
    background: linear-gradient(145deg, #5fd0e0, #2a9db0);
  }

  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .title {
    font-weight: 600;
  }

  .kind {
    font-size: 0.8rem;
    color: var(--text-2);
  }

  .order {
    display: flex;
    gap: 2px;
  }
</style>
