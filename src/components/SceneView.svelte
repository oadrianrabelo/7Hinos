<!-- Desenha qualquer cena do palco. O telão e a prévia usam o mesmo componente. -->
<script lang="ts">
  import { clockText, countdownText } from "../lib/clock";
  import { ensureFont, fontFamily } from "../lib/fonts";
  import { fileSrc, userMediaSrc } from "../lib/media";
  import { fitText } from "../lib/slides";
  import type { IdleScreen, Scene, ScreenStyle, SlideStyle } from "../lib/types";
  import SlideView from "./SlideView.svelte";

  let {
    scene,
    style,
    idle,
    screen,
  }: { scene: Scene; style: SlideStyle; idle: IdleScreen; screen: ScreenStyle } = $props();

  // O relógio anda sozinho em cada tela, sem depender de mensagens.
  let now = $state(Date.now());
  $effect(() => {
    if (scene.kind !== "clock" && !(scene.kind === "idle" && idle.showClock)) return;
    const timer = setInterval(() => (now = Date.now()), 250);
    return () => clearInterval(timer);
  });

  const logo = $derived(userMediaSrc(idle.image));

  // Avisos, tela de espera e relógio seguem a aparência escolhida em
  // "Avisos e relógio"; os hinos seguem o estilo dos slides.
  $effect(() => ensureFont(screen.font));
  const backdrop = $derived.by(() => {
    const image = userMediaSrc(screen.backgroundImage);
    if (image) return `linear-gradient(rgba(0, 0, 0, 0.4), rgba(0, 0, 0, 0.4)), url("${image}") center / cover`;
    if (screen.background === "gradient")
      return "radial-gradient(ellipse at 50% 35%, #1d4560 0%, #0b1d2b 70%, #050b11 100%)";
    return screen.background;
  });
  const notice = $derived(scene.kind === "slide" && scene.slide.kind === "text" ? scene.slide : null);
  const noticeSize = $derived(notice ? fitText(notice.text, "text", style.textScale) : null);
</script>

{#snippet themed(kind: string, body: import("svelte").Snippet)}
  <div
    class="screen {kind}"
    style:background={backdrop}
    style:color={screen.textColor}
    style:--accent={screen.accentColor}
    style:font-family={fontFamily(screen.font)}
  >
    {@render body()}
  </div>
{/snippet}

{#if notice && noticeSize}
  {#snippet noticeBody()}
    {#if notice.aux}<p class="label">{notice.aux}</p>{/if}
    <p class="notice-text" style:font-size="min({noticeSize.cqw}cqw, {noticeSize.cqh}cqh)">{notice.text}</p>
  {/snippet}
  {@render themed("notice", noticeBody)}
{:else if scene.kind === "slide"}
  <SlideView slide={scene.slide} {style} />
{:else if scene.kind === "image"}
  <div class="screen page"><img class="page-img" src={fileSrc(scene.src)} alt="" /></div>
{:else if scene.kind === "idle"}
  {#snippet idleBody()}
    {#if logo}<img src={logo} alt="" />{/if}
    {#if idle.text}<p class="idle-text">{idle.text}</p>{/if}
    {#if idle.showClock}<p class="corner-clock">{clockText(new Date(now))}</p>{/if}
  {/snippet}
  {@render themed("idle", idleBody)}
{:else if scene.kind === "clock"}
  {@const until = scene.until}
  {@const label = scene.label}
  {#snippet clockBody()}
    {#if label}<p class="label">{label}</p>{/if}
    <p class="time" class:done={until !== null && now >= until}>
      {until !== null ? countdownText(until, now) : clockText(new Date(now))}
    </p>
  {/snippet}
  {@render themed("clock", clockBody)}
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

  .page-img {
    width: 100%;
    height: 100%;
    max-width: none;
    max-height: none;
    object-fit: contain;
    filter: none;
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

  .notice-text {
    max-width: 88cqw;
    margin: 0;
    text-align: center;
    font-weight: 600;
    line-height: 1.25;
    text-wrap: balance;
    white-space: pre-line;
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
    margin: 0;
    font-size: min(7cqh, 5.5cqw);
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--accent, #ffc94a);
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
    color: var(--accent, #ffd36b);
  }
</style>
