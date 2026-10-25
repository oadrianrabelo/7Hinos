<!--
  Primeira abertura: escolher o modo e de onde vêm os hinos. Com o LouvorJA
  instalado no computador, tudo funciona sem internet.
-->
<script lang="ts">
  import { CloudDownload, FolderOpen, HardDrive, LayoutGrid, LoaderCircle, MousePointerClick, Usb, WifiOff } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { formatBytes } from "../lib/format";
  import { catalog } from "../lib/state/catalog.svelte";
  import { transfer } from "../lib/state/transfer.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import type { Install, UiMode } from "../lib/types";

  let step = $state<"mode" | "source">(settings.current.onboarded ? "source" : "mode");
  let installs = $state<Install[] | null>(null);
  let action = $state<"download" | "import" | "package" | null>(null);

  onMount(async () => {
    installs = await catalog.detectLocal();
  });

  async function choose(mode: UiMode) {
    await settings.save({ mode });
    step = "source";
  }

  async function download() {
    action = "download";
    if (await catalog.sync(true)) await settings.save({ onboarded: true });
  }

  async function importLocal(path: string | null) {
    action = "import";
    if (await catalog.importLocal(path)) await settings.refresh();
  }

  /** Pacote salvo por outro 7Hinos (o caminho de quem não tem internet). */
  async function importPackage() {
    const path = await transfer.pickFolder("Escolha o pendrive com o pacote do 7Hinos");
    if (!path) return;
    action = "package";
    const ok = await transfer.import(path);
    if (!ok) catalog.error = transfer.error;
  }

  async function pickFolder() {
    const path = await catalog.pickLocal();
    if (path) await importLocal(path);
  }

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
      <div class="options" in:fade>
        <button class="option card" onclick={() => choose("simple")}>
          <MousePointerClick size={32} />
          <span class="title">Simples</span>
          <span class="muted">Botões grandes e só o essencial. Ideal para quem tem pouca prática com computador.</span>
        </button>
        <button class="option card" onclick={() => choose("full")}>
          <LayoutGrid size={32} />
          <span class="title">Completo</span>
          <span class="muted">Biblioteca, painel de projeção ao lado e atalhos de teclado.</span>
        </button>
      </div>
      <p class="subtle small">Dá para trocar depois em Ajustes.</p>
    {:else if catalog.error}
      <div class="text" in:fade>
        <WifiOff size={36} class="warn" />
        <h1>{action === "download" ? "Sem conexão" : "Não deu para importar"}</h1>
        <p class="muted">{catalog.error}</p>
      </div>
      <div class="actions">
        {#if installs?.length && action !== "import"}
          <button class="btn btn-primary btn-lg" onclick={() => importLocal(installs![0].root)}>
            <HardDrive size={18} /> Usar o LouvorJA deste computador
          </button>
        {/if}
        <button class="btn btn-lg" class:btn-primary={!installs?.length} onclick={importPackage}>
          <Usb size={18} /> Trazer de um pendrive (pacote do 7Hinos)…
        </button>
        <button class="btn btn-lg" onclick={pickFolder}>
          <FolderOpen size={18} /> Escolher a pasta do LouvorJA…
        </button>
        <button class="btn btn-lg btn-plain" onclick={download}>
          <CloudDownload size={18} /> Tentar baixar de novo
        </button>
      </div>
      <p class="subtle small hint">
        Sem internet na igreja? Copie a pasta do LouvorJA de outro computador (normalmente
        <code>C:\Arquivos de Programas (x86)\Louvor JA</code>) para um pendrive e escolha-a aqui.
      </p>
    {:else if transfer.busy === "import"}
      <div class="text" in:fade>
        <Usb size={36} class="accent" />
        <h1>Trazendo do pendrive</h1>
        <p class="muted">Copiando os hinos e as músicas para este computador. Não precisa de internet.</p>
        <div class="bar"><div class="fill" style:width="{transfer.percent}%"></div></div>
        <p class="subtle small">{transfer.percent}%</p>
      </div>
    {:else if catalog.syncing}
      <div class="text" in:fade>
        {#if action === "import"}
          <HardDrive size={36} class="accent" />
          <h1>Importando do LouvorJA</h1>
          <p class="muted">Lendo os hinos e as músicas do computador. Não precisa de internet.</p>
          <div class="bar indeterminate"><div class="fill" style:width="30%"></div></div>
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
    {:else if installs !== null}
      <div class="text" in:fade>
        <h1>De onde vêm os hinos?</h1>
        <p class="muted">
          {installs.length ? "Encontramos o LouvorJA instalado neste computador." : "Escolha como trazer os hinos para este computador."}
        </p>
      </div>
      <div class="options" in:fade>
        {#if installs.length}
          <button class="option card recommended" onclick={() => importLocal(installs![0].root)}>
            <HardDrive size={32} />
            <span class="title">Usar o LouvorJA deste computador</span>
            <span class="muted">Funciona sem internet, com as músicas que já estão aqui.</span>
            <span class="path subtle truncate">{installs[0].root}</span>
          </button>
        {/if}
        <button class="option card" onclick={importPackage}>
          <Usb size={32} />
          <span class="title">Trazer de um pendrive</span>
          <span class="muted">Um pacote salvo pelo 7Hinos em outro computador.</span>
        </button>
        <button class="option card" onclick={download}>
          <CloudDownload size={32} />
          <span class="title">Baixar da internet</span>
          <span class="muted">O acervo mais recente. As músicas tocam pela internet ou depois de baixadas.</span>
        </button>
      </div>
      <button class="btn btn-plain" onclick={pickFolder}>
        <FolderOpen size={16} /> Tenho o LouvorJA numa pasta ou pendrive
      </button>
    {:else}
      <div class="text"><LoaderCircle size={32} class="spin accent" /></div>
    {/if}
  </div>
</div>

<style>
  .welcome {
    display: grid;
    place-items: center;
    height: 100%;
    padding: 2rem;
    overflow-y: auto;
    background:
      radial-gradient(ellipse at 30% 0%, var(--accent-soft), transparent 60%),
      var(--surface-2);
  }

  .panel {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.5rem;
    width: min(46rem, 100%);
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
    max-width: 32rem;
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

  .options {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(13rem, 1fr));
    gap: 1rem;
    width: 100%;
  }

  .option {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.5rem;
    min-width: 0;
    padding: 1.5rem;
    text-align: left;
    cursor: pointer;
    transition:
      transform var(--fast) var(--ease),
      border-color var(--normal),
      box-shadow var(--normal);
  }

  .option :global(svg) {
    color: var(--accent);
  }

  .option:hover,
  .option.recommended {
    border-color: var(--accent);
    box-shadow: 0 0 0 4px var(--accent-soft);
  }

  .option:active {
    transform: scale(0.98);
  }

  .title {
    font-size: 1.3rem;
    font-weight: 700;
  }

  .path {
    max-width: 100%;
    font-size: 0.8rem;
  }

  .actions {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.6rem;
  }

  .small {
    font-size: 0.9rem;
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
  }

  .hint {
    display: block;
    max-width: 32rem;
    line-height: 1.5;
  }

  code {
    font-size: 0.85em;
    padding: 0.1em 0.35em;
    border-radius: 4px;
    background: var(--surface-3);
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
