<!-- Apresentações: PDF do pregador, imagens e a lição da Escola Sabatina. -->
<script lang="ts">
  import {
    ArrowLeft,
    BookOpenText,
    ChevronLeft,
    ChevronRight,
    FileText,
    Images,
    ListChecks,
    ListPlus,
    LoaderCircle,
    Pencil,
    Presentation,
    Square,
    SunMoon,
    Trash2,
  } from "@lucide/svelte";
  import { ask, open } from "@tauri-apps/plugin-dialog";
  import PageHeader from "../components/PageHeader.svelte";
  import { api, errorMessage } from "../lib/api";
  import { fileSrc } from "../lib/media";
  import { decks } from "../lib/state/decks.svelte";
  import { setlist } from "../lib/state/setlist.svelte";
  import type { Deck, LessonQuarter } from "../lib/types";

  let { simple = false }: { simple?: boolean } = $props();

  $effect(() => {
    void decks.load();
  });

  const deck = $derived(decks.deck);

  async function importPdf() {
    const picked = await open({ title: "Escolha o PDF", filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (typeof picked !== "string") return;
    const created = await decks.importPdf(picked);
    if (created) decks.open(created);
  }

  async function importImages() {
    const picked = await open({
      multiple: true,
      title: "Escolha as imagens (na ordem dos nomes)",
      filters: [{ name: "Imagens", extensions: ["jpg", "jpeg", "png", "webp", "bmp", "gif"] }],
    });
    const paths = Array.isArray(picked) ? picked : picked ? [picked] : [];
    if (!paths.length) return;
    const created = await decks.importImages(paths);
    if (created) decks.open(created);
  }

  async function importOffice() {
    const picked = await open({
      title: "Escolha a apresentação do PowerPoint",
      filters: [{ name: "Apresentações", extensions: ["pptx", "ppt", "odp", "key"] }],
    });
    if (typeof picked !== "string") return;
    const created = await decks.importOffice(picked);
    if (created) decks.open(created);
  }

  let renaming = $state<{ id: string; title: string } | null>(null);

  async function saveRename(d: Deck) {
    if (renaming?.title.trim()) await decks.rename(d, renaming.title);
    renaming = null;
  }

  async function remove(d: Deck) {
    if (await ask(`Apagar “${d.title}”? As páginas guardadas são apagadas.`, { title: "Apagar apresentação", kind: "warning" })) {
      await decks.remove(d);
    }
  }

  // Lição da Escola Sabatina
  let lesson = $state<LessonQuarter | null>(null);
  let lessonError = $state<string | null>(null);
  let loadingLesson = $state(false);

  async function loadLesson() {
    loadingLesson = true;
    lessonError = null;
    try {
      lesson = await api.lessonQuarter();
    } catch (e) {
      lessonError = errorMessage(e);
    } finally {
      loadingLesson = false;
    }
  }

  const lessonDeck = (number: string) => decks.list.find((d) => d.source === "lesson" && d.title.startsWith(`Lição ${number} —`));
  const sourceLabel: Record<string, string> = { pdf: "PDF", images: "Imagens", lesson: "Lição" };
</script>

{#if deck}
  <!-- Uma apresentação aberta: passar as páginas -->
  <PageHeader title={deck.title} subtitle="{deck.pages.length} páginas · setas do teclado ou passador também funcionam">
    {#snippet actions()}
      <button class="btn" onclick={() => (decks.deck = null)}><ArrowLeft size={15} /> Todas</button>
    {/snippet}
  </PageHeader>

  <div class="viewer">
    <div class="stage-preview" class:dim={decks.live && decks.blackout}>
      {#if deck.pages.length}
        <img src={fileSrc(decks.pagePath(deck, decks.index))} alt="Página {decks.index + 1}" />
      {/if}
      {#if decks.live}<span class="pill live on-air">{decks.blackout ? "Tela preta" : "No telão"}</span>{/if}
    </div>
    <div class="controls">
      <span class="count">{decks.index + 1} / {deck.pages.length}</span>
      <button class="btn btn-lg btn-icon" aria-label="Anterior" disabled={decks.index === 0} onclick={() => (decks.live ? decks.previous() : (decks.index = Math.max(0, decks.index - 1)))}>
        <ChevronLeft size={20} />
      </button>
      {#if decks.live}
        <button class="btn btn-lg btn-primary" disabled={decks.index >= deck.pages.length - 1} onclick={() => decks.next()}>Próxima <ChevronRight size={18} /></button>
        <button class="btn btn-lg btn-icon" class:on={decks.blackout} aria-label="Tela preta" title="Tela preta (B)" onclick={() => decks.toggleBlackout()}><SunMoon size={18} /></button>
        <button class="btn btn-lg" onclick={() => decks.stop()}><Square size={15} /> Encerrar</button>
      {:else}
        <button class="btn btn-lg btn-icon" aria-label="Próxima" disabled={decks.index >= deck.pages.length - 1} onclick={() => (decks.index = Math.min(deck.pages.length - 1, decks.index + 1))}>
          <ChevronRight size={20} />
        </button>
        <button class="btn btn-lg btn-live" onclick={() => decks.show(decks.index)}><Presentation size={18} /> Projetar</button>
      {/if}
    </div>
  </div>

  <div class="thumbs">
    {#each deck.pages as _, i (i)}
      <button class="thumb" class:current={i === decks.index} onclick={() => (decks.live ? decks.show(i) : (decks.index = i))}>
        <img src={fileSrc(decks.pagePath(deck, i))} alt="" loading="lazy" />
        <span>{i + 1}</span>
      </button>
    {/each}
  </div>
{:else}
  <PageHeader title="Apresentações" subtitle="PDF do pregador, imagens e a lição da Escola Sabatina — tudo funciona sem internet depois de importado">
    {#snippet actions()}
      <button class="btn btn-primary" class:btn-lg={simple} onclick={importPdf}><FileText size={16} /> PDF</button>
      <button class="btn" class:btn-lg={simple} onclick={importImages}><Images size={16} /> Imagens</button>
      <button class="btn" class:btn-lg={simple} onclick={importOffice} title="Precisa do LibreOffice instalado; ou salve como PDF no PowerPoint">
        <Presentation size={16} /> PowerPoint
      </button>
    {/snippet}
  </PageHeader>

  {#if decks.importing}
    {@const imp = decks.importing}
    <div class="importing card">
      <LoaderCircle size={18} class="spin" />
      <span class="grow">
        Preparando “{imp.title}”{imp.total ? ` — página ${imp.done} de ${imp.total}` : "…"}
        {#if imp.total}<span class="bar"><span style:width="{(imp.done / imp.total) * 100}%"></span></span>{/if}
      </span>
    </div>
  {/if}

  {#if decks.list.length}
    <div class="grid">
      {#each decks.list as d (d.id)}
        {@const inSetlist = setlist.has({ kind: "deck", id: d.id, title: d.title })}
        <div class="deck card">
          <button class="cover" onclick={() => decks.open(d)} aria-label="Abrir {d.title}">
            {#if d.pages.length}<img src={fileSrc(decks.pagePath(d, 0))} alt="" loading="lazy" />{/if}
          </button>
          <div class="meta">
            {#if renaming?.id === d.id}
              <form onsubmit={(e) => (e.preventDefault(), void saveRename(d))}>
                <!-- svelte-ignore a11y_autofocus -->
                <input class="input" bind:value={renaming.title} autofocus onblur={() => saveRename(d)} />
              </form>
            {:else}
              <span class="truncate title">{d.title}</span>
            {/if}
            <span class="muted small">{sourceLabel[d.source] ?? "Apresentação"} · {d.pages.length} páginas</span>
          </div>
          <div class="actions">
            <button class="btn btn-plain btn-icon" class:on={inSetlist} title={inSetlist ? "Está no culto de hoje" : "Adicionar ao culto de hoje"} aria-label="Culto de hoje" onclick={() => setlist.toggle({ kind: "deck", id: d.id, title: d.title })}>
              {#if inSetlist}<ListChecks size={16} />{:else}<ListPlus size={16} />{/if}
            </button>
            <button class="btn btn-plain btn-icon" title="Renomear" aria-label="Renomear" onclick={() => (renaming = { id: d.id, title: d.title })}><Pencil size={15} /></button>
            <button class="btn btn-plain btn-icon" title="Apagar" aria-label="Apagar" onclick={() => remove(d)}><Trash2 size={15} /></button>
            <button class="btn btn-primary" onclick={() => decks.open(d)}>Abrir</button>
          </div>
        </div>
      {/each}
    </div>
  {:else if !decks.importing}
    <div class="empty">
      <Presentation size={44} strokeWidth={1.4} />
      <h2>Nenhuma apresentação ainda</h2>
      <p>Importe o PDF ou as imagens do pregador. Do PowerPoint: Arquivo → Salvar como → PDF.</p>
    </div>
  {/if}

  <section class="lesson card">
    <header>
      <h2><BookOpenText size={18} /> Lição da Escola Sabatina</h2>
      {#if !lesson}
        <button class="btn" onclick={loadLesson} disabled={loadingLesson}>
          {#if loadingLesson}<LoaderCircle size={15} class="spin" />{/if} Ver lições do trimestre
        </button>
      {/if}
    </header>
    <p class="muted">Baixa o PDF da lição (precisa de internet uma vez) e abre como apresentação. Leve no pacote de pendrive para a igreja.</p>
    {#if lessonError}<p class="error">{lessonError}</p>{/if}
    {#if lesson}
      <div class="lesson-body">
        {#if lesson.cover}<img class="lesson-cover" src={lesson.cover} alt="" />{/if}
        <div class="grow">
          <h3>{lesson.title || `Trimestre ${lesson.quarter}`}</h3>
          <ul>
            {#each lesson.lessons as l (l.number)}
              {@const ready = lessonDeck(l.number)}
              <li>
                <span class="n">{l.number}</span>
                <span class="grow truncate">{l.title}</span>
                {#if ready}
                  <button class="btn" onclick={() => decks.open(ready)}>Abrir</button>
                {:else}
                  <button class="btn btn-primary" disabled={!!decks.importing} onclick={async () => { const d = await decks.importLesson(lesson!.quarter, l); if (d) decks.open(d); }}>Baixar e abrir</button>
                {/if}
              </li>
            {/each}
          </ul>
        </div>
      </div>
    {/if}
  </section>
{/if}

<style>
  .viewer {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }

  .stage-preview {
    position: relative;
    aspect-ratio: 16 / 9;
    max-height: 52vh;
    border-radius: var(--radius-l);
    overflow: hidden;
    background: #000;
    display: grid;
    place-items: center;
  }

  .stage-preview.dim img {
    opacity: 0.25;
  }

  .stage-preview img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
  }

  .on-air {
    position: absolute;
    top: 0.6rem;
    left: 0.6rem;
  }

  .controls {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
  }

  .count {
    min-width: 4.5rem;
    font-variant-numeric: tabular-nums;
    color: var(--text-2);
  }

  .thumbs {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(9rem, 1fr));
    gap: 0.6rem;
  }

  .thumb {
    position: relative;
    aspect-ratio: 16 / 9;
    padding: 0;
    border: 2px solid transparent;
    border-radius: var(--radius);
    overflow: hidden;
    background: #000;
    cursor: pointer;
  }

  .thumb.current {
    border-color: var(--accent);
  }

  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .thumb span {
    position: absolute;
    right: 0.3rem;
    bottom: 0.25rem;
    padding: 0 0.35rem;
    border-radius: 0.4rem;
    font-size: 0.72rem;
    color: #fff;
    background: rgba(0, 0, 0, 0.6);
  }

  .importing {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.8rem 1rem;
    margin-bottom: 1rem;
  }

  .bar {
    display: block;
    height: 4px;
    margin-top: 0.4rem;
    border-radius: 2px;
    background: var(--surface-3);
    overflow: hidden;
  }

  .bar span {
    display: block;
    height: 100%;
    background: var(--accent);
    transition: width var(--normal);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(15rem, 1fr));
    gap: 0.9rem;
    margin-bottom: 1.5rem;
  }

  .deck {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .cover {
    aspect-ratio: 16 / 9;
    padding: 0;
    border: 0;
    background: #111;
    cursor: pointer;
  }

  .cover img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .meta {
    display: flex;
    flex-direction: column;
    padding: 0.6rem 0.75rem 0;
    min-width: 0;
  }

  .title {
    font-weight: 600;
  }

  .small {
    font-size: 0.82rem;
  }

  .actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.25rem;
    padding: 0.5rem 0.6rem 0.6rem;
  }

  .lesson {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    padding: 1rem 1.1rem;
  }

  .lesson header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
  }

  .lesson h2 {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    margin: 0;
    font-size: 1.05rem;
  }

  .lesson p {
    margin: 0;
  }

  .error {
    color: var(--danger);
  }

  .lesson-body {
    display: flex;
    gap: 1rem;
    align-items: flex-start;
  }

  .lesson-cover {
    width: 8rem;
    border-radius: var(--radius);
    box-shadow: var(--shadow);
  }

  .lesson-body h3 {
    margin: 0 0 0.5rem;
  }

  .lesson ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .lesson li {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    min-width: 0;
  }

  .n {
    width: 1.8rem;
    font-variant-numeric: tabular-nums;
    color: var(--text-2);
  }

  .grow {
    flex: 1;
    min-width: 0;
  }
</style>
