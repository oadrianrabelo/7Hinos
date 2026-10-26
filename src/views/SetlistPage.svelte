<!-- A programação do culto: monte antes, apresente na ordem. -->
<script lang="ts">
  import {
    ArrowDown,
    ArrowUp,
    Check,
    CircleCheck,
    CloudOff,
    Film,
    LayoutTemplate,
    ListMusic,
    Megaphone,
    Music,
    NotebookPen,
    Pencil,
    Play,
    Plus,
    Presentation,
    Timer,
    Trash2,
    X,
  } from "@lucide/svelte";
  import Popover from "../components/Popover.svelte";
  import { COMMON_PARTS, TEMPLATES } from "../lib/liturgy";
  import { itemKey } from "../lib/setlist";
  import { screen } from "../lib/state/screen.svelte";
  import DownloadButton from "../components/DownloadButton.svelte";
  import { availability } from "../lib/state/availability.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import { openSetItem, playNow } from "../lib/actions";
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
  const isCurrent = (item: SetItem) => {
    switch (item.kind) {
      case "song":
        return player.song?.id === item.id;
      case "deck":
        return decks.live && decks.deck?.id === item.id;
      case "video":
        return video.state?.path === resolveVideo(item, videos).path;
      case "part":
        return setlist.current?.kind === "part" && setlist.current.id === item.id;
    }
  };

  const kindLabel: Record<SetItem["kind"], string> = { song: "Hino", deck: "Apresentação", video: "Vídeo", part: "Momento" };

  // Acrescentar momentos (oração, sermão…) e modelos de programa.
  let partsOpen = $state(false);
  let partsAnchor = $state<HTMLButtonElement>();
  let templatesOpen = $state(false);
  let templatesAnchor = $state<HTMLButtonElement>();
  let customPart = $state("");

  function addPart(title: string, minutes?: number) {
    void setlist.addPart(title, minutes);
    partsOpen = false;
    customPart = "";
  }

  // Edição no lugar: anotação de qualquer item; nome e tempo de um momento.
  let editing = $state<{ index: number; title: string; note: string; minutes: string } | null>(null);

  function edit(item: SetItem, index: number) {
    editing = {
      index,
      title: item.title,
      note: item.note ?? "",
      minutes: item.kind === "part" && item.minutes ? String(item.minutes) : "",
    };
  }

  function saveEdit(item: SetItem) {
    if (!editing) return;
    const patch: Record<string, unknown> = { note: editing.note.trim() };
    if (item.kind === "part") {
      patch.title = editing.title.trim() || item.title;
      const minutes = Number(editing.minutes.replace(",", "."));
      patch.minutes = Number.isFinite(minutes) && minutes > 0 ? Math.round(minutes) : null;
    }
    void setlist.update(editing.index, patch as Partial<SetItem>);
    editing = null;
  }

  /** Mostra o momento no telão como um aviso (nome em cima, anotação embaixo). */
  function showPart(item: SetItem) {
    screen.showNotice({ id: 0, title: item.title, body: item.note?.trim() || item.title });
  }

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

<div class="slots">
  <span class="muted">Acrescentar:</span>
  <button class="chip plain" bind:this={partsAnchor} aria-expanded={partsOpen} onclick={() => (partsOpen = !partsOpen)}>
    <Plus size={14} /> Momento
  </button>
  <button class="chip plain" bind:this={templatesAnchor} aria-expanded={templatesOpen} onclick={() => (templatesOpen = !templatesOpen)}>
    <LayoutTemplate size={14} /> Modelo de programa
  </button>
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

<Popover bind:open={partsOpen} anchor={partsAnchor} width={20}>
  <form class="custom-part" onsubmit={(e) => (e.preventDefault(), customPart.trim() && addPart(customPart))}>
    <input class="input" bind:value={customPart} placeholder="Outro momento…" aria-label="Nome do momento" />
    <button class="btn btn-primary" type="submit" disabled={!customPart.trim()}>Pôr</button>
  </form>
  <ul class="menu">
    {#each COMMON_PARTS as part (part.title)}
      <li>
        <button onclick={() => addPart(part.title, part.minutes)}>
          <span class="grow">{part.title}</span>
          {#if part.minutes}<span class="muted">{part.minutes} min</span>{/if}
        </button>
      </li>
    {/each}
  </ul>
</Popover>

<Popover bind:open={templatesOpen} anchor={templatesAnchor} width={22}>
  <p class="muted menu-hint">Acrescenta os momentos no fim da lista. Depois é só pôr os hinos entre eles e anotar quem dirige.</p>
  <ul class="menu">
    {#each TEMPLATES as template (template.name)}
      <li>
        <button onclick={() => (setlist.addTemplate(template.parts), (templatesOpen = false))}>
          <span class="grow">
            <strong>{template.name}</strong>
            <span class="muted block">{template.parts.map((p) => p.title).join(" · ")}</span>
          </span>
        </button>
      </li>
    {/each}
  </ul>
</Popover>

{#if setlist.items.length}
  <ol class="setlist">
    {#each setlist.items as item, i (itemKey(item))}
      {@const tag = item.kind === "video" ? itemTag(item, videos) : null}
      <li class="entry" class:current={isCurrent(item)} class:simple ondblclick={(e) => !(e.target as HTMLElement).closest("button") && playNow(item)} title={item.kind === "song" ? "Clique duplo para começar já" : undefined}>
        <span class="n">{i + 1}</span>
        <span class="icon" class:video={item.kind === "video"} class:deck={item.kind === "deck"} class:part={item.kind === "part"}>
          {#if item.kind === "song"}<Music size={18} />{:else if item.kind === "deck"}<Presentation size={18} />{:else if item.kind === "part"}<NotebookPen size={17} />{:else}<Film size={18} />{/if}
        </span>
        {#if editing?.index === i}
          <form class="edit" onsubmit={(e) => (e.preventDefault(), saveEdit(item))}>
            {#if item.kind === "part"}
              <input class="input" bind:value={editing.title} aria-label="Nome do momento" />
              <input class="input minutes" bind:value={editing.minutes} inputmode="numeric" placeholder="min" aria-label="Minutos (cronômetro)" title="Minutos: liga o cronômetro do púlpito ao começar" />
            {/if}
            <!-- svelte-ignore a11y_autofocus -->
            <input class="input grow" bind:value={editing.note} placeholder="Anotação: quem dirige, o que lembrar…" aria-label="Anotação" autofocus />
            <button class="btn btn-primary btn-icon" type="submit" aria-label="Salvar"><Check size={16} /></button>
            <button class="btn btn-plain btn-icon" type="button" aria-label="Cancelar" onclick={() => (editing = null)}><X size={16} /></button>
          </form>
        {:else}
        <span class="info">
          <span class="title truncate">{item.title}</span>
          {#if item.note}<span class="note truncate">{item.note}</span>{/if}
          <span class="kind">
            {kindLabel[item.kind]}
            {#if item.kind === "part" && item.minutes}
              <span class="ok-time"><Timer size={12} /> {item.minutes} min no cronômetro</span>
            {/if}
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
        {/if}
        <span class="order">
          <button class="btn btn-plain btn-icon" aria-label="Anotar" title={item.kind === "part" ? "Editar momento e anotação" : "Anotar (quem dirige, o que lembrar)"} onclick={() => edit(item, i)}><Pencil size={15} /></button>
          <button class="btn btn-plain btn-icon" aria-label="Subir" title="Subir" disabled={i === 0} onclick={() => setlist.move(i, -1)}><ArrowUp size={16} /></button>
          <button class="btn btn-plain btn-icon" aria-label="Descer" title="Descer" disabled={i === setlist.items.length - 1} onclick={() => setlist.move(i, 1)}><ArrowDown size={16} /></button>
          <button class="btn btn-plain btn-icon" aria-label="Tirar da lista" title="Tirar da lista" onclick={() => setlist.remove(i)}><X size={16} /></button>
        </span>
        {#if item.kind === "part"}
          <button class="btn btn-icon" class:btn-lg={simple} aria-label="Mostrar no telão" title="Mostrar no telão (nome e anotação)" onclick={() => showPart(item)}><Megaphone size={15} /></button>
          <button class="btn btn-primary" class:btn-lg={simple} title={item.minutes ? `Marca como agora e liga o cronômetro com ${item.minutes} min` : "Marca como agora"} onclick={() => openSetItem(item)}>
            <Play size={15} fill="currentColor" /> Começar
          </button>
        {:else}
          <button class="btn btn-primary" class:btn-lg={simple} onclick={() => openSetItem(item)}>
            <Play size={15} fill="currentColor" />
            {item.kind === "video" ? "Exibir" : "Abrir"}
          </button>
        {/if}
      </li>
    {/each}
  </ol>
{:else}
  <div class="empty">
    <ListMusic size={44} strokeWidth={1.4} />
    <h2>A lista está vazia</h2>
    <p>
      Toque no ícone <ListMusic size={14} /> ao lado de um hino ou vídeo para colocá-lo aqui,<br />
      ou comece por um <strong>modelo de programa</strong> (acima). Na hora do culto, é só seguir a ordem.
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

  .icon.part {
    background: linear-gradient(145deg, #a48bd6, #6b4fb0);
  }

  .chip.plain {
    --tint: var(--border-strong);
    background: var(--surface);
  }

  .note {
    font-size: 0.86rem;
    font-style: italic;
    color: var(--text-2);
  }

  .ok-time {
    display: inline-flex;
    align-items: center;
    gap: 0.2rem;
    color: #6b4fb0;
  }

  .edit {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .edit .grow {
    flex: 1;
    min-width: 0;
  }

  .edit .minutes {
    width: 4.5rem;
  }

  .custom-part {
    display: flex;
    gap: 0.4rem;
    margin-bottom: 0.5rem;
  }

  .custom-part .input {
    flex: 1;
    min-width: 0;
  }

  .menu {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
  }

  .menu button {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.45rem 0.55rem;
    border: 0;
    border-radius: var(--radius-s);
    background: none;
    color: var(--text);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .menu button:hover {
    background: var(--hover);
  }

  .menu .grow {
    flex: 1;
    min-width: 0;
  }

  .menu .block {
    display: block;
    font-size: 0.8rem;
  }

  .menu-hint {
    margin: 0 0 0.5rem;
    font-size: 0.85rem;
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
