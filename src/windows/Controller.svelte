<!-- Janela de controle: carrega o estado, aplica tema/escala e escolhe o modo. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { Sparkles } from "@lucide/svelte";
  import DownloadsStrip from "../components/DownloadsStrip.svelte";
  import Toasts from "../components/Toasts.svelte";
  import FullShell from "../shell/FullShell.svelte";
  import SimpleShell from "../shell/SimpleShell.svelte";
  import Onboarding from "./Onboarding.svelte";
  import Tour from "../components/Tour.svelte";
  import { catalog } from "../lib/state/catalog.svelte";
  import { downloads } from "../lib/state/downloads.svelte";
  import { nav } from "../lib/state/nav.svelte";
  import { outputs } from "../lib/state/outputs.svelte";
  import { live } from "../lib/state/live.svelte";
  import { player } from "../lib/state/player.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import { stage } from "../lib/state/stage.svelte";
  import { updater } from "../lib/state/updater.svelte";
  import { remote } from "../lib/state/remote.svelte";
  import { video } from "../lib/state/video.svelte";
  import { toasts } from "../lib/state/toasts.svelte";
  import { tour } from "../lib/state/tour.svelte";
  import { api } from "../lib/api";

  let ready = $state(false);
  let full = $state<FullShell>();

  onMount(async () => {
    await Promise.all([settings.init(), catalog.init(), outputs.init(), video.init(), downloads.init()]);
    await stage.init();
    await remote.init();
    ready = true;
    // O perfil de telas padrão abre sozinho, se nenhuma tela estiver aberta.
    const preferred = settings.current.screenProfiles.find((p) => p.id === settings.current.defaultProfile);
    if (preferred && settings.current.onboarded && !outputs.outputs.length) void outputs.applyProfile(preferred);
    if (settings.current.autoUpdateApp) setTimeout(() => updater.checkInBackground(), 5000);
    void announceUpdate();
  });

  /** Depois de uma atualização, avisa uma vez e leva às Novidades. */
  async function announceUpdate() {
    const KEY = "7hinos.lastVersion";
    try {
      const version = await api.appVersion();
      const before = localStorage.getItem(KEY);
      localStorage.setItem(KEY, version);
      if (before && before !== version && settings.current.onboarded) {
        toasts.show(`O 7Hinos foi atualizado para a versão ${version}.`, "success", { label: "Ver novidades", run: () => nav.push({ name: "news" }) }, 12_000);
      }
    } catch {
      // Sem armazenamento ou sem versão: só não avisa.
    }
  }

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

  // Depois da primeira configuração, oferece o passeio (uma vez só).
  $effect(() => {
    if (ready && !needsOnboarding) tour.maybeOffer();
  });

  /**
   * Atalhos compatíveis com passadores de slide (PageUp/PageDown, B e ponto).
   * Ignorados enquanto se digita.
   */
  function onKey(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    const typing = target.closest("input, textarea, select") && !(target as HTMLInputElement).type?.match(/range|checkbox/);
    if (e.key === "F1") {
      e.preventDefault();
      if (nav.current.name === "help") return;
      if (settings.current.mode === "simple") nav.push({ name: "help" });
      else nav.reset({ name: "help" });
      return;
    }
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "f") {
      e.preventDefault();
      full?.focusSearch();
      return;
    }
    if (typing || e.metaKey || e.ctrlKey || e.altKey) return;
    // Quem está no telão recebe os atalhos; sem ninguém, o hino do painel.
    const presenter = live.current ?? (player.song ? player : null);
    switch (e.key) {
      case "ArrowRight":
      case "ArrowDown":
      case "PageDown":
        if (!presenter) return;
        void presenter.next();
        break;
      case "ArrowLeft":
      case "ArrowUp":
      case "PageUp":
        if (!presenter) return;
        void presenter.previous();
        break;
      case " ":
        if (!presenter?.togglePlay) return;
        presenter.togglePlay();
        break;
      case "b":
      case "B":
      case ".":
        presenter?.toggleBlackout();
        break;
      case "/":
        full?.focusSearch();
        break;
      default:
        // Digitar um número em qualquer lugar já busca o hino; Enter toca.
        if (!/^[0-9]$/.test(e.key) || !full) return;
        full.focusSearch(e.key);
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
{#if ready && !needsOnboarding}<Tour />{/if}

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
