<!-- Horas, cronômetro e recado por cima do vídeo no retorno. -->
<script lang="ts">
  import { clockText } from "../lib/clock";
  import { timerDisplay } from "../lib/timer";
  import { fontFamily } from "../lib/fonts";
  import type { MonitorOptions, Pulpit } from "../lib/types";

  let { pulpit, options }: { pulpit: Pulpit; options: MonitorOptions } = $props();
  const show = $derived(options.showClock);

  let now = $state(Date.now());
  $effect(() => {
    const t = setInterval(() => (now = Date.now()), 500);
    return () => clearInterval(t);
  });
  const timer = $derived(pulpit.timer ? timerDisplay(pulpit.timer, now) : null);
</script>

<div class="corner">
  {#if timer}
    <span class="chip" class:warn={timer.warn} class:over={timer.over}>
      <small>{timer.label}</small>{timer.text}
    </span>
  {/if}
  {#if show}
    <span class="chip clock"><small>Agora</small>{clockText(new Date(now))}</span>
  {/if}
</div>
{#if pulpit.message}
  <p
    class="message"
    style:color={options.messageColor || null}
    style:background={options.messageBackground || null}
    style:font-family={options.messageFont ? fontFamily(options.messageFont) : null}
  >
    {pulpit.message}
  </p>
{/if}

<style>
  .corner {
    position: fixed;
    top: 2vh;
    right: 2vw;
    display: flex;
    gap: 1vw;
    z-index: 5;
  }

  .chip {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 0.8vh 1.2vw;
    border-radius: 1.2vh;
    background: rgba(0, 0, 0, 0.6);
    color: #fff;
    font: 700 5vh/1 var(--font-ui, system-ui);
    font-variant-numeric: tabular-nums;
  }

  .chip small {
    margin-bottom: 0.5vh;
    font-size: 1.7vh;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    opacity: 0.85;
  }

  .clock {
    color: #ffd36b;
  }

  .warn {
    background: #c62828;
  }

  .over {
    background: #d93025;
  }

  .message {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    margin: 0;
    padding: 1.6vh 2.5vw;
    z-index: 5;
    background: #ffd36b;
    color: #000;
    font: 700 5.5vh/1.15 var(--font-ui, system-ui);
    text-align: center;
  }
</style>
