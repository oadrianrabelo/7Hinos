<!--
  Monitor de retorno: o que quem canta ou prega vê. Letra grande e sem fundo,
  o próximo slide embaixo e as horas sempre à vista.
-->
<script lang="ts">
  import { clockText, countdownText } from "../lib/clock";
  import { fitText } from "../lib/slides";
  import { timerDisplay } from "../lib/timer";
  import type { MonitorOptions, StageState } from "../lib/types";

  let { stage, options }: { stage: StageState; options: MonitorOptions } = $props();

  let now = $state(Date.now());
  $effect(() => {
    const timer = setInterval(() => (now = Date.now()), 500);
    return () => clearInterval(timer);
  });

  const scene = $derived(stage.scene);
  const message = $derived(stage.pulpit?.message ?? null);
  const timer = $derived(stage.pulpit?.timer ? timerDisplay(stage.pulpit.timer, now) : null);
  const current = $derived(scene.kind === "slide" ? scene.slide : null);
  const next = $derived(scene.kind === "slide" && options.showNext ? (scene.next ?? null) : null);
  // Sem fundo e sem sombra, cabe um pouco mais de letra que no telão.
  const size = $derived(current ? fitText(current.text, current.kind, 1.3 * options.textScale) : null);

  const status = $derived.by(() => {
    if (stage.video?.playing) return `Vídeo: ${stage.video.title}`;
    switch (scene.kind) {
      case "black":
        return "Tela preta";
      case "idle":
        return "Tela de espera";
      case "clock":
        return scene.until !== null ? scene.label || "Contagem regressiva" : "Relógio no telão";
      default:
        return "";
    }
  });
</script>

<div class="monitor">
  <header>
    <span class="title truncate">
      {#if scene.kind !== "slide"}{status}{:else if options.showTitle}{scene.title}{/if}
    </span>
    {#if timer}
      <span class="timer" class:limited={timer.limited} class:warn={timer.warn} class:over={timer.over}>
        <span class="timer-label">{timer.label}</span>
        <span class="timer-value">{timer.text}</span>
      </span>
    {/if}
    {#if options.showClock}
      <span class="clock">
        <span class="timer-label">Agora</span>
        <span>{clockText(new Date(now))}</span>
      </span>
    {/if}
  </header>

  {#if message}
    {#key message}<p class="message">{message}</p>{/key}
  {/if}

  <main>
    {#if current && size}
      {#if current.aux}<p class="aux">{current.aux}</p>{/if}
      <p class="text" class:prose={current.kind === "verse" || current.kind === "text"} style:font-size="min({size.cqh}cqh, {size.cqw}cqw)">
        {current.text}
      </p>
    {:else if scene.kind === "clock" && scene.until !== null}
      <p class="big">{countdownText(scene.until, now)}</p>
    {:else}
      <p class="status">{status}</p>
    {/if}
  </main>

  {#if options.showNext}
  <footer class:empty={!next}>
    <span class="tag">A seguir</span>
    <span class="next truncate">{next ? next.text.replace(/\n+/g, " / ") : scene.kind === "slide" ? "Fim" : "—"}</span>
  </footer>
  {/if}
</div>

<style>
  .monitor {
    position: fixed;
    inset: 0;
    display: grid;
    grid-template-rows: auto auto 1fr auto;
    background: #000;
    color: #fff;
    font-family: var(--font-slide);
  }

  header,
  footer {
    display: flex;
    align-items: center;
    gap: 2vw;
    padding: 1.6vh 2.5vw;
    font-family: var(--font-ui, system-ui);
  }

  header {
    border-bottom: 1px solid rgba(255, 255, 255, 0.12);
  }

  .title {
    flex: 1;
    min-width: 0;
    font-size: 3.6vh;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.7);
  }

  .clock,
  .timer {
    display: flex;
    flex-direction: column;
    align-items: center;
    line-height: 1;
    font-size: 6vh;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  .clock {
    color: #ffd36b;
  }

  .timer-label {
    margin-bottom: 0.6vh;
    font-size: 1.9vh;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    opacity: 0.85;
  }

  /* Cronômetro: sem tempo combinado, discreto; com, vermelho, cada vez
     mais forte conforme acaba. */
  .timer {
    padding: 0.8vh 1.4vw;
    border-radius: 1.2vh;
    color: #fff;
    background: rgba(255, 255, 255, 0.1);
  }

  .timer.limited {
    color: #ff8a80;
    background: rgba(217, 48, 37, 0.18);
    box-shadow: inset 0 0 0 0.25vh rgba(255, 138, 128, 0.6);
  }

  .timer.warn {
    color: #fff;
    background: #c62828;
    box-shadow: none;
  }

  .timer.over {
    color: #fff;
    background: #d93025;
    box-shadow: none;
    animation: pulse 1s ease-in-out infinite;
  }

  @keyframes pulse {
    50% {
      background: #7f1d1d;
    }
  }

  /* Recado do operador: chama atenção ao chegar, depois fica parado. */
  .message {
    margin: 0;
    padding: 1.8vh 2.5vw;
    font-family: var(--font-ui, system-ui);
    font-size: 6.5vh;
    font-weight: 700;
    line-height: 1.15;
    text-align: center;
    color: #000;
    background: #ffd36b;
    animation: arrive 0.6s ease-out 3;
  }

  @keyframes arrive {
    50% {
      background: #fff;
    }
  }

  main {
    min-height: 0;
    display: flex;
    flex-direction: column;
    justify-content: center;
    padding: 2vh 3vw;
    container-type: size;
    overflow: hidden;
  }

  .aux {
    margin: 0 0 2cqh;
    font-size: 5cqh;
    color: #ffd36b;
  }

  .text {
    margin: 0;
    white-space: pre-line;
    line-height: 1.12;
    font-weight: 600;
    text-transform: uppercase;
  }

  .text.prose {
    text-transform: none;
    line-height: 1.25;
  }

  .big {
    margin: 0;
    text-align: center;
    font-size: 40cqh;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  .status {
    margin: 0;
    text-align: center;
    font-size: 10cqh;
    color: rgba(255, 255, 255, 0.45);
  }

  footer {
    border-top: 1px solid rgba(255, 255, 255, 0.12);
    background: #111;
  }

  .tag {
    flex: none;
    padding: 0.4vh 1vw;
    border-radius: 0.6vh;
    font-size: 2.6vh;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #000;
    background: #8fd18f;
  }

  .empty .tag {
    background: rgba(255, 255, 255, 0.3);
  }

  .next {
    min-width: 0;
    font-size: 4.2vh;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.75);
  }
</style>
