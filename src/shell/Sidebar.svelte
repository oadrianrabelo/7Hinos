<script lang="ts">
  import { BookMarked, BookOpen, Clock, Film, Heart, House, Library, ListMusic, Megaphone, MonitorPlay, Music4, Settings } from "@lucide/svelte";
  import SearchField from "../components/SearchField.svelte";
  import { nav, type Route } from "../lib/state/nav.svelte";
  import { outputs } from "../lib/state/outputs.svelte";
  import { setlist } from "../lib/state/setlist.svelte";

  let query = $state("");
  let search: SearchField;

  export function focusSearch() {
    search?.focus();
  }

  const items = [
    { route: { name: "home" }, label: "Início", icon: House },
    { route: { name: "setlist" }, label: "Culto de hoje", icon: ListMusic },
    { route: { name: "hymnal" }, label: "Hinário", icon: BookOpen },
    { route: { name: "bible" }, label: "Bíblia", icon: BookMarked },
    { route: { name: "screen" }, label: "Avisos e relógio", icon: Megaphone },
    { route: { name: "collections" }, label: "Coletâneas", icon: Library },
    { route: { name: "favorites" }, label: "Favoritos", icon: Heart },
    { route: { name: "recent" }, label: "Recentes", icon: Clock },
    { route: { name: "mysongs" }, label: "Minhas músicas", icon: Music4 },
    { route: { name: "videos" }, label: "Vídeos", icon: Film },
  ] satisfies { route: Route; label: string; icon: unknown }[];

  /** Seção ativa: a primeira rota da pilha depois do início. */
  const section = $derived(nav.stack[1]?.name ?? "home");

  function onSearch(value: string) {
    if (value.trim()) nav.push({ name: "search", query: value });
    else if (nav.current.name === "search") nav.back();
  }

  $effect(() => {
    if (nav.current.name !== "search") query = "";
  });
</script>

<nav class="sidebar">
  <div class="drag" data-tauri-drag-region></div>
  <div class="brand" data-tauri-drag-region>
    <img src="/icon.png" alt="" width="26" height="26" />
    <span>7Hinos</span>
  </div>
  <div class="search"><SearchField bind:this={search} bind:value={query} placeholder="Buscar hinos" oninput={onSearch} /></div>

  <ul>
    {#each items as item (item.label)}
      <li>
        <button class="item" class:active={section === item.route.name} onclick={() => nav.reset(item.route)}>
          <item.icon size={17} />
          {item.label}
          {#if item.route.name === "setlist" && setlist.items.length}
            <span class="count">{setlist.items.length}</span>
          {/if}
        </button>
      </li>
    {/each}
  </ul>

  <div class="spacer"></div>

  <ul>
    <li>
      <button class="item" class:active={section === "displays"} onclick={() => nav.reset({ name: "displays" })}>
        <MonitorPlay size={17} />
        Telas
        {#if outputs.isProjecting}<span class="dot" title="Projetando"></span>{/if}
      </button>
    </li>
    <li>
      <button class="item" class:active={section === "settings"} onclick={() => nav.reset({ name: "settings" })}>
        <Settings size={17} />
        Ajustes
      </button>
    </li>
  </ul>
</nav>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    width: 14.5rem;
    flex: none;
    padding: 0 0.75rem 0.9rem;
    background: var(--sidebar);
    border-right: 1px solid var(--border);
  }

  /* Espaço para os botões de janela do macOS. */
  .drag {
    height: 1.6rem;
    flex: none;
  }

  :global([data-platform="mac"]) .drag {
    height: 2.6rem;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    padding: 0.4rem 0.5rem 0.9rem;
    font-weight: 700;
    font-size: 1.05rem;
    letter-spacing: -0.01em;
  }

  .brand img {
    border-radius: 7px;
    pointer-events: none;
  }

  .search {
    margin-bottom: 0.9rem;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .item {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    width: 100%;
    height: 2.15rem;
    padding: 0 0.65rem;
    border: 0;
    border-radius: var(--radius-s);
    background: transparent;
    text-align: left;
    font-weight: 500;
    cursor: pointer;
    transition: background var(--fast);
  }

  .item :global(svg) {
    color: var(--accent);
  }

  .item:hover {
    background: var(--hover);
  }

  .item.active {
    background: var(--active);
    font-weight: 600;
  }

  .spacer {
    flex: 1;
  }

  .count {
    margin-left: auto;
    min-width: 1.3rem;
    padding: 0 0.35rem;
    border-radius: 1rem;
    background: var(--accent);
    color: #fff;
    font-size: 0.72rem;
    font-weight: 700;
    text-align: center;
  }

  .dot {
    width: 0.5rem;
    height: 0.5rem;
    margin-left: auto;
    border-radius: 50%;
    background: var(--live);
    animation: pulse 1.6s ease-in-out infinite;
  }
</style>
