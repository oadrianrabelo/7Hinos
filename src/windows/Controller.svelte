<!-- Janela de controle: carrega o estado, aplica tema/escala e escolhe o modo. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { Sparkles } from "@lucide/svelte";
  import DownloadsStrip from "../components/DownloadsStrip.svelte";
  import Toasts from "../components/Toasts.svelte";
  import FullShell from "../shell/FullShell.svelte";
  import SimpleShell from "../shell/SimpleShell.svelte";
  import Onboarding from "./Onboarding.svelte";
  import { on } from "../lib/api";
  import { catalog } from "../lib/state/catalog.svelte";
  import { downloads } from "../lib/state/downloads.svelte";
  import { nav } from "../lib/state/nav.svelte";
  import { outputs } from "../lib/state/outputs.svelte";
  import { player } from "../lib/state/player.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import { updater } from "../lib/state/updater.svelte";
  import { video } from "../lib/state/video.svelte";

  let ready = $state(false);
  let full = $state<FullShell>();

  onMount(async () => {
    await Promise.all([settings.init(), catalog.init(), outputs.init(), video.init(), downloads.init()]);
    await on("stage://changed", (stage) => video.receive(stage.video));
    ready = true;
    if (settings.current.autoUpdateApp) setTimeout(() => updater.checkInBackground(), 5000);
  });

  // Tema e tamanho do texto valem para a janela inteira.
  $effect(() => {
    const s = settings.value;
    if (!s) return;
    const root = document.documentElement;
    root.dataset.theme = s.theme;
    const scale = s.mode === "simple" ? Math.max(s.uiScale, 1.15) : s.uiScale;
    root.style.setProperty("--scale", String(scale));
  });

  // Ao trocar de modo, recomeça do início para não cair numa tela que o
  // outro modo não tem.
  let lastMode: string | undefined;
  $effect(() => {
    const mode = settings.value?.mode;
    if (lastMode && mode !== lastMode) nav.reset({ name: "home" });
    lastMode = mode;
  });

  const needsOnboarding = $derived(ready && (!settings.current.onboarded || !catalog.installed));

  /**
   * Atalhos compatíveis com passadores de slide (PageUp/PageDown, B e ponto).
   * Ignorados enquanto se digita.
   */
  function onKey(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    const typing = target.closest("input, textarea, select") && !(target as HTMLInputElement).type?.match(/range|checkbox/);
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "f") {
      e.preventDefault();
      full?.focusSearch();
      return;
    }
    if (typing || e.metaKey || e.ctrlKey || e.altKey) return;
    switch (e.key) {
      case "ArrowRight":
      case "ArrowDown":
      case "PageDown":
        if (!player.song) return;
        player.next();
        break;
      case "ArrowLeft":
      case "ArrowUp":
      case "PageUp":
        if (!player.song) return;
        player.previous();
        break;
      case " ":
        if (!player.live) return;
        player.togglePlay();
        break;
      case "b":
      case "B":
      case ".":
        player.toggleBlackout();
        break;
      case "/":
        full?.focusSearch();
        break;
      default:
        return;
    }
    e.preventDefault();
  }
</script>

<svelte:window onkeydown={onKey} />

{#if !ready}
  <div class="splash" data-tauri-drag-region></div>
{:else if needsOnboarding}
  <Onboarding />
{:else if settings.current.mode === "simple"}
  <SimpleShell />
{:else}
  <FullShell bind:this={full} />
{/if}

{#if updater.ready}
  <div class="update card">
    <Sparkles size={18} />
    <span>A versão {updater.ready.version} está pronta.</span>
    <button class="btn btn-primary" onclick={() => updater.installAndRestart()} disabled={updater.installing}>Reiniciar agora</button>
    <button class="btn btn-plain" onclick={() => (updater.ready = null)}>Depois</button>
  </div>
{/if}

<DownloadsStrip />
<Toasts />

<style>
  .splash {
    height: 100%;
  }

  .update {
    position: fixed;
    left: 1rem;
    bottom: 1rem;
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.6rem 0.6rem 0.6rem 0.9rem;
    box-shadow: var(--shadow-l);
    z-index: 90;
  }

  .update :global(svg) {
    color: var(--accent);
  }
</style>
