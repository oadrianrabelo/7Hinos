<!--
  Toca um vídeo do palco seguindo o relógio: posição esperada =
  posição salva + tempo decorrido. Corrige só quando a diferença fica visível.
-->
<script lang="ts">
  import { fileSrc } from "../lib/media";
  import { expectedPosition, needsSeek } from "../lib/videoSync";
  import type { VideoState } from "../lib/types";

  let {
    state,
    muted = false,
    duration = $bindable(0),
    position = $bindable(0),
    onended,
  }: {
    state: VideoState;
    muted?: boolean;
    duration?: number;
    position?: number;
    onended?: () => void;
  } = $props();

  let el: HTMLVideoElement;
  const src = $derived(fileSrc(state.path));

  function align() {
    if (!el || el.readyState < 1) return;
    const expected = expectedPosition(state, Date.now(), el.duration);
    if (needsSeek(el.currentTime, expected)) el.currentTime = expected;
    el.loop = state.looping;
    el.volume = Math.max(0, Math.min(1, state.volume));
    el.muted = muted;
    const atEnd = !state.looping && Number.isFinite(el.duration) && expected >= el.duration - 0.05;
    if (state.playing && !atEnd) {
      if (el.paused) void el.play().catch(() => {});
    } else if (!el.paused) {
      el.pause();
    }
  }

  $effect(() => {
    void state;
    void muted;
    align();
  });

  $effect(() => {
    const timer = setInterval(align, 2000);
    return () => clearInterval(timer);
  });
</script>

<!-- svelte-ignore a11y_media_has_caption -->
<video
  bind:this={el}
  {src}
  preload="auto"
  playsinline
  onloadedmetadata={() => {
    duration = el.duration || 0;
    align();
  }}
  ontimeupdate={() => (position = el.currentTime)}
  onended={() => onended?.()}
></video>

<style>
  video {
    width: 100%;
    height: 100%;
    object-fit: contain;
    background: #000;
    display: block;
  }
</style>
