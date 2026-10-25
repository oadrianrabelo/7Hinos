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
      [stage, prefs] = await Promise.all([api.stage(), api.settings(), readRole()]);
    })();
    wake();
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
        <MonitorView {stage} />
      {:else if showVideo && stage.video}
        {@const v = stage.video}
        <VideoLayer state={v} muted={v.targets[0] !== label} onerror={() => void tell("video://failed", { path: v.path, title: v.title })} />
      {:else}
        <SceneView scene={stage.scene} style={prefs.slide} idle={prefs.idle} />
      {/if}
    </ErrorBoundary>
  {/if}
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
</style>
