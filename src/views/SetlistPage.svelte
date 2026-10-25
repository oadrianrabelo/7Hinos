<!-- A programação do culto: monte antes, apresente na ordem. -->
<script lang="ts">
  import { ArrowDown, ArrowUp, CircleCheck, CloudOff, Film, ListMusic, Music, Play, Plus, Presentation, Trash2, X } from "@lucide/svelte";
  import DownloadButton from "../components/DownloadButton.svelte";
  import { availability } from "../lib/state/availability.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import { openSetItem } from "../lib/actions";
  import { player } from "../lib/state/player.svelte";
  import { setlist } from "../lib/state/setlist.svelte";
  import { video } from "../lib/state/video.svelte";
  import { decks } from "../lib/state/decks.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import type { SetItem } from "../lib/types";
  import { itemTag, resolveVideo, WEEKLY_TAGS } from "../lib/videoTags";

  let { simple = false }: { simple?: boolean } = $props();

  const songIds = $derived(setlist.items.flatMap((i) => (i.kind === "song" ? [i.id] : [])));

  const videos = $derived(settings.current.videos);
  const isCurrent = (item: SetItem) =>
    item.kind === "song"
      ? player.song?.id === item.id
      : item.kind === "deck"
        ? decks.live && decks.deck?.id === item.id
        : video.state?.path === resolveVideo(item, videos).path;

  /** Lugares semanais que ainda não estão na lista: "Informativo de hoje" etc. */
  const weeklySlots = $derived(
    WEEKLY_TAGS.map((tag) => ({ tag, today: videos.find((v) => v.tag === tag.id) ?? null })).filter(
      ({ tag }) => !setlist.items.some((i) => i.kind === "video" && i.tag === tag.id),
    ),
  );
</script>

<PageHeader title="Culto de hoje" subtitle="Monte a ordem antes do culto e vá apresentando um por um">
  {#snippet actions()}
    {#if songIds.length}
      <DownloadButton key="setlist" scope={{ kind: "songs", ids: songIds }} label="Culto de hoje" text="Preparar para sem internet" large={simple} />
    {/if}
    {#if setlist.items.length}
      <button class="btn" class:btn-lg={simple} onclick={() => setlist.clear()}><Trash2 size={15} /> Limpar lista</button>
    {/if}
  {/snippet}
</PageHeader>

{#if weeklySlots.length}
  <div class="slots">
    <span class="muted">Acrescentar:</span>
    {#each weeklySlots as { tag, today } (tag.id)}
      <button
        class="chip"
        style:--tint={tag.tint}
        disabled={!today}
        title={today ? `Toca o vídeo marcado como ${tag.label} de hoje` : `Marque um vídeo como ${tag.label} em Vídeos`}
        onclick={() => today && setlist.toggle({ kind: "video", path: today.path, title: tag.label, tag: tag.id })}
      >
        <Plus size={14} /> {tag.short} de hoje
      </button>
    {/each}
  </div>
{/if}

{#if setlist.items.length}
  <ol class="setlist">
    {#each setlist.items as item, i (item.kind + (item.kind === "song" || item.kind === "deck" ? item.id : `${item.path}|${item.tag ?? ""}`))}
      {@const tag = item.kind === "video" ? itemTag(item, videos) : null}
      <li class="entry" class:current={isCurrent(item)} class:simple>
        <span class="n">{i + 1}</span>
        <span class="icon" class:video={item.kind === "video"} class:deck={item.kind === "deck"}>
          {#if item.kind === "song"}<Music size={18} />{:else if item.kind === "deck"}<Presentation size={18} />{:else}<Film size={18} />{/if}
        </span>
        <span class="info">
          <span class="title truncate">{item.title}</span>
          <span class="kind">
            {item.kind === "song" ? "Hino" : item.kind === "deck" ? "Apresentação" : "Vídeo"}
            {#if tag}
              <span class="tag" style:--tint={tag.tint}>{tag.short}{tag.weekly ? " · de hoje" : ""}</span>
            {/if}
            {#if item.kind === "video" && tag?.weekly}
              {@const today = resolveVideo(item, videos)}
              {#if today.title !== item.title}<span class="truncate">{today.title}</span>{/if}
            {/if}
            {#if item.kind === "song"}
              {@const ready = availability.get(item.id)}
              {#if ready === true}
                <span class="ok"><CircleCheck size={12} /> toca sem internet</span>
              {:else if ready === false}
                <span class="need"><CloudOff size={12} /> precisa de internet</span>
              {/if}
            {/if}
          </span>
        </span>
        <span class="order">
          <button class="btn btn-plain btn-icon" aria-label="Subir" title="Subir" disabled={i === 0} onclick={() => setlist.move(i, -1)}><ArrowUp size={16} /></button>
          <button class="btn btn-plain btn-icon" aria-label="Descer" title="Descer" disabled={i === setlist.items.length - 1} onclick={() => setlist.move(i, 1)}><ArrowDown size={16} /></button>
          <button class="btn btn-plain btn-icon" aria-label="Tirar da lista" title="Tirar da lista" onclick={() => setlist.remove(i)}><X size={16} /></button>
        </span>
        <button class="btn btn-primary" class:btn-lg={simple} onclick={() => openSetItem(item)}>
          <Play size={15} fill="currentColor" />
          {item.kind === "video" ? "Exibir" : "Abrir"}
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
    background: linear-gradient(145deg, #4f9cc4, var(--accent));
  }

  .slots {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.35rem 0.8rem;
    border-radius: 999px;
    border: 1px solid var(--tint);
    background: color-mix(in srgb, var(--tint) 12%, transparent);
    color: var(--text);
    font: inherit;
    font-size: 0.88rem;
    font-weight: 600;
    cursor: pointer;
  }

  .chip:disabled {
    opacity: 0.45;
    cursor: default;
  }

  .tag {
    padding: 0.05rem 0.45rem;
    border-radius: 999px;
    font-size: 0.72rem;
    font-weight: 700;
    color: #fff;
    background: var(--tint);
  }

  .icon.deck {
    background: linear-gradient(145deg, #ffb340, #ff8a00);
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
    display: flex;
    align-items: center;
    gap: 0.6rem;
    font-size: 0.8rem;
    color: var(--text-2);
  }

  .ok,
  .need {
    display: inline-flex;
    align-items: center;
    gap: 0.2rem;
  }

  .ok {
    color: var(--success);
  }

  .need {
    color: var(--warning);
  }

  .order {
    display: flex;
    gap: 2px;
  }
</style>
