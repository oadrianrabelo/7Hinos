<!--
  Tela de projeção: só desenha o palco. Não tem estado próprio além do que
  recebe, então pode ser aberta, fechada ou duplicada a qualquer momento.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import ErrorBoundary from "../components/ErrorBoundary.svelte";
  import MonitorView from "../components/MonitorView.svelte";
  import SceneView from "../components/SceneView.svelte";
  import VideoLayer from "../components/VideoLayer.svelte";
  import { api, on, tell } from "../lib/api";
  import type { OutputRole, Settings, StageState } from "../lib/types";

  const win = getCurrentWebviewWindow();
  const label = win.label;

  let stage = $state<StageState | null>(null);
  let prefs = $state<Settings | null>(null);
  let role = $state<OutputRole>("audience");
  let identify = $state<string | null>(null);
  let identifyTimer: ReturnType<typeof setTimeout>;
  let idle = $state(false);
  let idleTimer: ReturnType<typeof setTimeout>;

  onMount(() => {
    document.body.style.background = "#000";
    void (async () => {
      await on("stage://changed", (s) => (stage = s));
      await on("settings://changed", (s) => (prefs = s));
      // O papel (telão ou retorno) pode mudar com a tela aberta.
      const readRole = async () => (role = (await api.outputs()).find((o) => o.label === label)?.role ?? "audience");
      await on("displays://changed", () => void readRole());
      await on("displays://identify", async () => {
        const me = (await api.outputs()).find((o) => o.label === label);
        identify = me ? `${me.name} · ${me.role === "monitor" ? "Retorno" : "Telão"}` : label;
        clearTimeout(identifyTimer);
        identifyTimer = setTimeout(() => (identify = null), 4000);
      });
      [stage, prefs] = await Promise.all([api.stage(), api.settings(), readRole()]);
    })();
    wake();

    // Rede de segurança: se um aviso do controle se perder (janela minimizada,
    // computador lento), a tela percebe pela revisão e se acerta sozinha, sem
    // precisar ser fechada e aberta.
    const resync = async () => {
      try {
        const now = await api.stage();
        if (!stage || now.rev !== stage.rev) stage = now;
      } catch {
        // O controle pode estar fechando; tenta de novo no próximo ciclo.
      }
    };
    const timer = setInterval(resync, 4000);
    const onVisible = () => {
      if (document.visibilityState === "visible") void resync();
    };
    document.addEventListener("visibilitychange", onVisible);
    window.addEventListener("focus", onVisible);
    return () => {
      clearInterval(timer);
      document.removeEventListener("visibilitychange", onVisible);
      window.removeEventListener("focus", onVisible);
    };
  });

  // O cursor some depois de parado, para não aparecer no telão.
  function wake() {
    idle = false;
    clearTimeout(idleTimer);
    idleTimer = setTimeout(() => (idle = true), 2000);
  }

  async function toggleFullscreen() {
    await win.setFullscreen(!(await win.isFullscreen()));
  }

  const showVideo = $derived(stage?.video?.targets.includes(label) ?? false);
</script>

<div class="output" class:idle onmousemove={wake} ondblclick={toggleFullscreen} role="presentation">
  {#if stage && prefs}
    <ErrorBoundary where="projeção {label}" silent>
      {#if role === "monitor"}
        <MonitorView {stage} options={prefs.monitor} />
      {:else if showVideo && stage.video}
        {@const v = stage.video}
        <VideoLayer
          state={v}
          muted={v.targets[0] !== label}
          onerror={() => void tell("video://failed", { path: v.path, title: v.title })}
          onended={() => v.targets[0] === label && void tell("video://ended", { path: v.path })}
        />
      {:else}
        <SceneView scene={stage.scene} style={prefs.slide} idle={prefs.idle} screen={prefs.screenStyle} />
      {/if}
    </ErrorBoundary>
  {/if}
  {#if identify}<div class="identify">{identify}</div>{/if}
</div>

<style>
  .output {
    position: fixed;
    inset: 0;
    background: #000;
  }

  .idle {
    cursor: none;
  }

  .identify {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    background: rgba(31, 106, 144, 0.94);
    color: #fff;
    font: 700 12vh/1 var(--font-ui, system-ui);
    text-align: center;
  }
</style>
