<!-- Primeira abertura: escolher o modo e baixar os hinos. Dois passos, só. -->
<script lang="ts">
  import { CloudDownload, LayoutGrid, LoaderCircle, MousePointerClick, WifiOff } from "@lucide/svelte";
  import { fade, fly } from "svelte/transition";
  import { formatBytes } from "../lib/format";
  import { catalog } from "../lib/state/catalog.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import type { UiMode } from "../lib/types";

  let step = $state<"mode" | "download">(settings.current.onboarded ? "download" : "mode");

  async function choose(mode: UiMode) {
    await settings.save({ mode });
    step = "download";
    void download();
  }

  async function download() {
    if (await catalog.sync(true)) await settings.save({ onboarded: true });
  }

  $effect(() => {
    if (step === "download" && !catalog.syncing && !catalog.error && !catalog.installed) void download();
  });

  const percent = $derived(
    catalog.progress?.total ? Math.min(100, (catalog.progress.done / catalog.progress.total) * 100) : null,
  );
</script>

<div class="welcome" data-tauri-drag-region>
  <div class="panel" in:fly={{ y: 20, duration: 400 }}>
    <img class="logo" src="/icon.png" alt="" width="88" height="88" />
    {#if step === "mode"}
      <div class="text" in:fade>
        <h1>Bem-vindo ao 7Hinos</h1>
        <p class="muted">Como você prefere usar?</p>
      </div>
      <div class="modes" in:fade>
        <button class="mode card" onclick={() => choose("simple")}>
          <MousePointerClick size={32} />
          <span class="title">Simples</span>
          <span class="muted">Botões grandes e só o essencial. Ideal para quem tem pouca prática com computador.</span>
        </button>
        <button class="mode card" onclick={() => choose("full")}>
          <LayoutGrid size={32} />
          <span class="title">Completo</span>
          <span class="muted">Biblioteca, painel de projeção ao lado e atalhos de teclado.</span>
        </button>
      </div>
      <p class="subtle small">Dá para trocar depois em Ajustes.</p>
    {:else}
      <div class="text" in:fade>
        {#if catalog.error}
          <WifiOff size={36} class="warn" />
          <h1>Sem conexão</h1>
          <p class="muted">{catalog.error}</p>
          <button class="btn btn-primary btn-lg" onclick={download}>Tentar de novo</button>
        {:else}
          <CloudDownload size={36} class="accent" />
          <h1>Baixando os hinos</h1>
          <p class="muted">O acervo do LouvorJA está vindo para o seu computador. Só é preciso fazer isso uma vez.</p>
          <div class="bar" class:indeterminate={percent === null}>
            <div class="fill" style:width="{percent ?? 30}%"></div>
          </div>
          <p class="subtle small">
            {#if catalog.progress}
              {formatBytes(catalog.progress.done)}{catalog.progress.total ? ` de ${formatBytes(catalog.progress.total)}` : ""}
            {:else}
              <LoaderCircle size={14} class="spin" /> Conectando…
            {/if}
          </p>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .welcome {
    display: grid;
    place-items: center;
    height: 100%;
    padding: 2rem;
    background:
      radial-gradient(ellipse at 30% 0%, var(--accent-soft), transparent 60%),
      var(--surface-2);
  }

  .panel {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.5rem;
    width: min(44rem, 100%);
    text-align: center;
  }

  .logo {
    border-radius: 22px;
    filter: drop-shadow(0 12px 30px rgba(67, 56, 202, 0.35));
  }

  .text {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.6rem;
    max-width: 30rem;
  }

  .text :global(.accent) {
    color: var(--accent);
  }

  .text :global(.warn) {
    color: var(--warning);
  }

  h1 {
    font-size: 2.2rem;
  }

  .text p {
    font-size: 1.1rem;
  }

  .modes {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
    width: 100%;
  }

  .mode {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.5rem;
    padding: 1.5rem;
    text-align: left;
    cursor: pointer;
    transition:
      transform var(--fast) var(--ease),
      border-color var(--normal),
      box-shadow var(--normal);
  }

  .mode :global(svg) {
    color: var(--accent);
  }

  .mode:hover {
    border-color: var(--accent);
    box-shadow: 0 0 0 4px var(--accent-soft);
  }

  .mode:active {
    transform: scale(0.98);
  }

  .title {
    font-size: 1.35rem;
    font-weight: 700;
  }

  .small {
    font-size: 0.9rem;
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
  }

  .bar {
    position: relative;
    width: 100%;
    height: 8px;
    margin-top: 0.75rem;
    border-radius: 8px;
    background: var(--surface-3);
    overflow: hidden;
  }

  .fill {
    height: 100%;
    border-radius: 8px;
    background: var(--accent);
    transition: width var(--normal) var(--ease);
  }

  .indeterminate .fill {
    position: absolute;
    animation: slide 1.4s ease-in-out infinite;
  }

  @keyframes slide {
    from {
      left: -30%;
    }
    to {
      left: 100%;
    }
  }
</style>
