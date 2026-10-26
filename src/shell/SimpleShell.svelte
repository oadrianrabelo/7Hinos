<!--
  Modo simples: uma coisa por tela, botões grandes, sempre um "Voltar" visível
  e nenhuma ação escondida em menu ou atalho.
-->
<script lang="ts">
  import { ChevronLeft, CircleHelp, House, Presentation, Settings } from "@lucide/svelte";
  import RouteView from "./RouteView.svelte";
  import SimpleHome from "./SimpleHome.svelte";
  import SimpleLive from "./SimpleLive.svelte";
  import SimpleSong from "./SimpleSong.svelte";
  import ErrorBoundary from "../components/ErrorBoundary.svelte";
  import { nav } from "../lib/state/nav.svelte";
  import { player } from "../lib/state/player.svelte";

  let page: HTMLElement;
  $effect(() => {
    void nav.current;
    page?.scrollTo({ top: 0 });
  });

  const route = $derived(nav.current);
</script>

<div class="simple">
  <header data-tauri-drag-region>
    {#if nav.canGoBack}
      <button class="btn btn-lg" onclick={() => nav.back()}><ChevronLeft size={22} /> Voltar</button>
      {#if nav.stack.length > 2}
        <button class="btn btn-lg btn-plain" onclick={() => nav.reset({ name: "home" })}><House size={20} /> Início</button>
      {/if}
    {:else}
      <div class="brand" data-tauri-drag-region>
        <img src="/icon.png" alt="" width="34" height="34" />
        <span>7Hinos</span>
      </div>
    {/if}
    <div class="spacer" data-tauri-drag-region></div>
    {#if route.name === "home"}
      <button class="btn btn-lg btn-plain" data-tour="simple-help" onclick={() => nav.push({ name: "help" })}><CircleHelp size={20} /> Ajuda</button>
      <button class="btn btn-lg btn-plain" data-tour="simple-settings" onclick={() => nav.push({ name: "settings" })}><Settings size={20} /> Ajustes</button>
    {/if}
    {#if player.live && route.name !== "live"}
      <button class="btn btn-live btn-lg" onclick={() => nav.push({ name: "live" })}>
        <Presentation size={20} /> Voltar à apresentação
      </button>
    {/if}
  </header>

  <main class="scroll" bind:this={page}>
    <div class="content">
      {#if route.name === "home" || route.name === "song" || route.name === "live"}
        {#key route}
          <ErrorBoundary where="modo simples: {route.name}" onhome={() => nav.reset({ name: "home" })}>
            {#if route.name === "home"}
              <SimpleHome />
            {:else if route.name === "song"}
              <SimpleSong id={route.id} />
            {:else}
              <SimpleLive />
            {/if}
          </ErrorBoundary>
        {/key}
      {:else}
        <RouteView simple />
      {/if}
    </div>
  </main>
</div>

<style>
  .simple {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface-2);
  }

  header {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    min-height: 4.75rem;
    padding: 1.6rem 1.75rem 0.6rem;
    flex: none;
  }

  :global([data-platform="mac"]) header {
    padding-left: 5.5rem;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 0.7rem;
    font-size: 1.4rem;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .brand img {
    border-radius: 9px;
    pointer-events: none;
  }

  .spacer {
    flex: 1;
    align-self: stretch;
  }

  main {
    flex: 1;
  }

  .content {
    max-width: 64rem;
    margin: 0 auto;
    padding: 0.75rem 1.75rem 3rem;
  }

  /* Listas e cartões maiores e mais espaçados neste modo. */
  .content :global(.item) {
    min-height: 3.8rem;
    font-size: 1.1rem;
  }

  .content :global(.item .fav) {
    opacity: 1;
  }
</style>
