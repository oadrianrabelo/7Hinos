<script lang="ts">
  import { ChevronLeft } from "@lucide/svelte";
  import RouteView from "./RouteView.svelte";
  import Sidebar from "./Sidebar.svelte";
  import StagePanel from "./StagePanel.svelte";
  import ErrorBoundary from "../components/ErrorBoundary.svelte";
  import { nav } from "../lib/state/nav.svelte";

  let sidebar: Sidebar;
  let page: HTMLElement;

  export function focusSearch(initial?: string) {
    sidebar?.focusSearch(initial);
  }

  // Cada página nova começa do topo.
  $effect(() => {
    void nav.current;
    page?.scrollTo({ top: 0 });
  });
</script>

<div class="full">
  <Sidebar bind:this={sidebar} />
  <main>
    <div class="topbar" data-tauri-drag-region>
      <!-- As seções da barra lateral são raízes; só subpáginas têm "voltar". -->
      {#if nav.stack.length > 2}
        <button class="btn btn-plain btn-icon" aria-label="Voltar" onclick={() => nav.back()}><ChevronLeft size={20} /></button>
      {/if}
    </div>
    <div class="page scroll" bind:this={page}>
      <div class="content"><RouteView /></div>
    </div>
  </main>
  <ErrorBoundary where="painel ao vivo"><StagePanel /></ErrorBoundary>
</div>

<style>
  .full {
    display: flex;
    height: 100%;
  }

  main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    background: var(--bg);
  }

  .topbar {
    display: flex;
    align-items: center;
    height: 2.9rem;
    flex: none;
    padding: 0.4rem 1rem 0;
  }

  .page {
    flex: 1;
    padding: 0.25rem 2rem 2.5rem;
  }

  /* Em telas largas o conteúdo fica no meio, não colado na barra lateral. */
  .content {
    max-width: 76rem;
    margin: 0 auto;
  }
</style>
