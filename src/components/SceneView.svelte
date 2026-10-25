<!-- Desenha qualquer cena do palco. O telão e a prévia usam o mesmo componente. -->
<script lang="ts">
  import { clockText, countdownText } from "../lib/clock";
  import { userMediaSrc } from "../lib/media";
  import type { IdleScreen, Scene, SlideStyle } from "../lib/types";
  import SlideView from "./SlideView.svelte";

  let { scene, style, idle }: { scene: Scene; style: SlideStyle; idle: IdleScreen } = $props();

  // O relógio anda sozinho em cada tela, sem depender de mensagens.
  let now = $state(Date.now());
  $effect(() => {
    if (scene.kind !== "clock" && !(scene.kind === "idle" && idle.showClock)) return;
    const timer = setInterval(() => (now = Date.now()), 250);
    return () => clearInterval(timer);
  });

  const logo = $derived(userMediaSrc(idle.image));
</script>

{#if scene.kind === "slide"}
  <SlideView slide={scene.slide} {style} />
{:else if scene.kind === "idle"}
  <div class="screen idle">
    {#if logo}<img src={logo} alt="" />{/if}
    {#if idle.text}<p class="idle-text">{idle.text}</p>{/if}
    {#if idle.showClock}<p class="corner-clock">{clockText(new Date(now))}</p>{/if}
  </div>
{:else if scene.kind === "clock"}
  <div class="screen clock">
    {#if scene.label}<p class="label">{scene.label}</p>{/if}
    <p class="time" class:done={scene.until !== null && now >= scene.until}>
      {scene.until !== null ? countdownText(scene.until, now) : clockText(new Date(now))}
    </p>
  </div>
{:else}
  <div class="screen"></div>
{/if}

<style>
  .screen {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: #000;
    container-type: size;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4cqh;
    color: #fff;
    font-family: var(--font-slide);
  }

  .idle,
  .clock {
    background: radial-gradient(ellipse at 50% 35%, #26264a 0%, #0d0d1a 70%, #050509 100%);
  }

  img {
    max-width: 70cqw;
    max-height: 55cqh;
    object-fit: contain;
    filter: drop-shadow(0 1cqh 4cqh rgba(0, 0, 0, 0.5));
  }

  .idle-text {
    max-width: 85cqw;
    text-align: center;
    font-size: min(8cqh, 6cqw);
    font-weight: 600;
    line-height: 1.15;
    text-wrap: balance;
  }

  .corner-clock {
    position: absolute;
    right: 4cqw;
    bottom: 4cqh;
    font-size: 6cqh;
    font-weight: 600;
    opacity: 0.8;
    font-variant-numeric: tabular-nums;
  }

  .label {
    font-size: min(7cqh, 5.5cqw);
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: #ffc94a;
    text-align: center;
  }

  .time {
    font-size: min(30cqh, 24cqw);
    font-weight: 700;
    line-height: 1;
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.02em;
  }

  .time.done {
    color: #ffd36b;
  }
</style>
