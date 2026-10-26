<script lang="ts">
  import { BookOpen, ChevronLeft, ChevronRight, CloudDownload, LoaderCircle, Search, Square, SunMoon } from "@lucide/svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import Tip from "../components/Tip.svelte";
  import { api } from "../lib/api";
  import { formatBytes } from "../lib/format";
  import { bible } from "../lib/state/bible.svelte";
  import { toasts } from "../lib/state/toasts.svelte";
  import type { BibleHit } from "../lib/types";

  let { simple = false }: { simple?: boolean } = $props();

  let query = $state("");
  let hits = $state<BibleHit[] | null>(null);

  $effect(() => {
    if (!bible.loaded) void bible.load();
  });

  async function submit() {
    const text = query.trim();
    if (!text) return;
    hits = null;
    if (await bible.goTo(text, true)) return;
    if (bible.version === null) return;
    hits = await api.bibleSearch(bible.version, text).catch(() => []);
    if (!hits.length) toasts.show("Nada encontrado. Tente “João 3:16”, “Sl 23” ou uma palavra do texto.");
  }

  async function openHit(hit: BibleHit) {
    hits = null;
    await bible.open(hit.book, hit.chapter);
    const index = bible.verses.findIndex((v) => v.number === hit.verse);
    if (index >= 0) bible.show(index);
  }

  const oldTestament = $derived(bible.books.filter((b) => b.testament === 1));
  const newTestament = $derived(bible.books.filter((b) => b.testament !== 1));
  const bookName = (n: number) => bible.books.find((b) => b.number === n)?.name ?? "";

  let list = $state<HTMLElement>();
  $effect(() => {
    const i = bible.index;
    if (i !== null) list?.querySelector(`[data-verse="${i}"]`)?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  });
</script>

<PageHeader title="Bíblia" subtitle={bible.versionInfo?.name ?? ""}>
  {#snippet actions()}
    {#if bible.versions.length > 1}
      <select
        class="version field"
        value={bible.version}
        onchange={(e) => bible.setVersion(Number(e.currentTarget.value))}
        aria-label="Versão da Bíblia"
      >
        {#each bible.versions as v (v.id)}
          <option value={v.id}>{v.abbreviation} — {v.name}</option>
        {/each}
      </select>
    {/if}
  {/snippet}
</PageHeader>

<Tip id="bible" article="biblia">Escreva a referência, como “João 3:16” ou “Salmos 23”, e o versículo já aparece. Clique nele para mostrar no telão.</Tip>

{#if !bible.loaded}
  <div class="empty"><LoaderCircle size={28} class="spin" /></div>
{:else if !bible.installed}
  <div class="empty">
    <BookOpen size={44} strokeWidth={1.4} />
    <h2>A Bíblia ainda não está neste computador</h2>
    <p>
      Baixe agora (cerca de 26 MB, todas as versões do LouvorJA) ou, sem internet, importe o LouvorJA em
      <strong>Ajustes → Hinos</strong> ou um pacote de pendrive: a Bíblia vem junto.
    </p>
    <button class="btn btn-primary btn-lg" onclick={() => bible.download()} disabled={bible.downloading}>
      {#if bible.downloading}
        <LoaderCircle size={18} class="spin" />
        {#if bible.progress?.phase === "install"}
          Instalando… {bible.progress.total ? Math.round((bible.progress.done / bible.progress.total) * 100) : 0}%
        {:else}
          Baixando… {bible.progress ? formatBytes(bible.progress.done) : ""}
        {/if}
      {:else}
        <CloudDownload size={18} /> Baixar a Bíblia
      {/if}
    </button>
  </div>
{:else}
  <form class="find" onsubmit={(e) => (e.preventDefault(), submit())}>
    <label class="field" class:large={simple}>
      <Search size={simple ? 20 : 15} />
      <input bind:value={query} placeholder="João 3:16, Salmos 23, 1 Co 13:4-7 ou uma palavra" spellcheck="false" />
    </label>
    <button class="btn btn-primary" class:btn-lg={simple} type="submit">Ir</button>
  </form>

  <nav class="crumbs">
    <button class="link" onclick={() => ((bible.book = null), (bible.chapter = null), (hits = null))}>Livros</button>
    {#if bible.bookInfo}
      <ChevronRight size={14} />
      <button class="link" onclick={() => ((bible.chapter = null), (hits = null))}>{bible.bookInfo.name}</button>
    {/if}
    {#if bible.chapter}
      <ChevronRight size={14} />
      <span>Capítulo {bible.chapter}</span>
    {/if}
  </nav>

  {#if hits}
    <ul class="hits">
      {#each hits as hit (hit.book + "-" + hit.chapter + "-" + hit.verse)}
        <li>
          <button class="hit" onclick={() => openHit(hit)}>
            <strong>{bookName(hit.book)} {hit.chapter}:{hit.verse}</strong>
            <span>{hit.text}</span>
          </button>
        </li>
      {/each}
    </ul>
  {:else if !bible.book}
    {#each [["Antigo Testamento", oldTestament], ["Novo Testamento", newTestament]] as const as [title, books] (title)}
      <h3 class="section-title testament">{title}</h3>
      <div class="books" class:simple>
        {#each books as b (b.number)}
          <button class="book" onclick={() => (bible.book = b.number)}>
            <span class="abbr">{b.abbreviation}</span>
            <span class="name truncate">{b.name}</span>
          </button>
        {/each}
      </div>
    {/each}
  {:else if !bible.chapter && bible.bookInfo}
    <div class="chapters" class:simple>
      {#each Array.from({ length: bible.bookInfo.chapters }, (_, i) => i + 1) as c (c)}
        <button class="chapter" onclick={() => bible.book && bible.open(bible.book, c)}>{c}</button>
      {/each}
    </div>
  {:else}
    <ol class="verses" class:simple bind:this={list}>
      {#each bible.verses as v, i (v.number)}
        {@const selected = bible.index !== null && i >= bible.index && i <= (bible.rangeEnd ?? bible.index)}
        <li>
          <button class="verse" class:selected class:live={selected && bible.live} data-verse={i} onclick={() => bible.show(i)}>
            <span class="n">{v.number}</span>
            <span>{v.text}</span>
          </button>
        </li>
      {/each}
    </ol>
  {/if}

  {#if simple && bible.live}
    <div class="livebar card">
      <span class="pill live">{bible.blackout ? "Tela preta" : "No telão"}</span>
      <span class="ref truncate">{bible.slide?.aux}</span>
      <button class="btn btn-xl" onclick={() => bible.previous()}><ChevronLeft size={26} /> Anterior</button>
      <button class="btn btn-xl btn-primary" onclick={() => bible.next()}>Próximo <ChevronRight size={26} /></button>
      <button class="btn btn-lg" class:on={bible.blackout} onclick={() => bible.toggleBlackout()}><SunMoon size={20} /></button>
      <button class="btn btn-lg" onclick={() => bible.stop()}><Square size={18} /> Encerrar</button>
    </div>
  {/if}
{/if}

<style>
  .version {
    height: 2.25rem;
    max-width: 18rem;
  }

  .find {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.9rem;
  }

  .find .field {
    flex: 1;
  }

  .find .large {
    height: 3.2rem;
    font-size: 1.15rem;
  }

  .crumbs {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    margin-bottom: 1rem;
    color: var(--text-2);
    font-weight: 550;
  }

  .link {
    border: 0;
    padding: 0;
    background: none;
    color: var(--accent);
    font-weight: 600;
    cursor: pointer;
  }

  .testament {
    margin: 0.5rem 0 0.6rem;
  }

  .books {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(9rem, 1fr));
    gap: 0.4rem;
    margin-bottom: 1.25rem;
  }

  .books.simple {
    grid-template-columns: repeat(auto-fill, minmax(12rem, 1fr));
  }

  .book {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    padding: 0.5rem 0.6rem;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--surface);
    text-align: left;
    cursor: pointer;
    transition: background var(--fast);
  }

  .simple .book {
    padding: 0.85rem 0.9rem;
    font-size: 1.1rem;
  }

  .book:hover {
    background: var(--hover);
  }

  .abbr {
    min-width: 2.2rem;
    font-weight: 700;
    color: var(--accent);
  }

  .name {
    font-weight: 500;
  }

  .chapters {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(3.4rem, 1fr));
    gap: 0.4rem;
  }

  .chapters.simple {
    grid-template-columns: repeat(auto-fill, minmax(4.6rem, 1fr));
  }

  .chapter {
    height: 3rem;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--surface);
    font-size: 1.05rem;
    font-weight: 600;
    cursor: pointer;
  }

  .simple .chapter {
    height: 4rem;
    font-size: 1.3rem;
  }

  .chapter:hover {
    background: var(--accent-soft);
    color: var(--accent);
  }

  .verses,
  .hits {
    list-style: none;
    margin: 0;
    padding: 0 0 6rem;
  }

  .verse,
  .hit {
    display: flex;
    gap: 0.7rem;
    width: 100%;
    padding: 0.6rem 0.75rem;
    border: 0;
    border-radius: var(--radius);
    background: transparent;
    text-align: left;
    line-height: 1.5;
    cursor: pointer;
  }

  .simple .verse {
    font-size: 1.15rem;
    padding: 0.8rem 0.9rem;
  }

  .hit {
    flex-direction: column;
    gap: 0.15rem;
  }

  .verse:hover,
  .hit:hover {
    background: var(--hover);
  }

  .verse.selected {
    background: var(--accent-soft);
  }

  .verse.live {
    background: var(--accent);
    color: #fff;
  }

  .verse.live .n {
    color: rgba(255, 255, 255, 0.8);
  }

  .n {
    min-width: 1.6rem;
    text-align: right;
    font-weight: 700;
    color: var(--accent);
    font-variant-numeric: tabular-nums;
  }

  .livebar {
    position: sticky;
    bottom: 1rem;
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.75rem;
    box-shadow: var(--shadow-l);
  }

  .ref {
    flex: 1;
    min-width: 0;
    font-weight: 650;
  }
</style>
