<!--
  Tela de projeção: só desenha o palco. Não tem estado próprio além do que
  recebe, então pode ser aberta, fechada ou duplicada a qualquer momento.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import ErrorBoundary from "../components/ErrorBoundary.svelte";
  import SceneView from "../components/SceneView.svelte";
  import VideoLayer from "../components/VideoLayer.svelte";
  import { api, on } from "../lib/api";
  import type { Settings, StageState } from "../lib/types";

  const win = getCurrentWebviewWindow();
  const label = win.label;

  let stage = $state<StageState | null>(null);
  let prefs = $state<Settings | null>(null);
  let idle = $state(false);
  let idleTimer: ReturnType<typeof setTimeout>;

  onMount(() => {
    document.body.style.background = "#000";
    void (async () => {
      await on("stage://changed", (s) => (stage = s));
      await on("settings://changed", (s) => (prefs = s));
      [stage, prefs] = await Promise.all([api.stage(), api.settings()]);
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
      {#if showVideo && stage.video}
        <VideoLayer state={stage.video} muted={stage.video.targets[0] !== label} />
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
