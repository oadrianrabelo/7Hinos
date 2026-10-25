<!-- Recado e cronômetro que só quem está no púlpito vê (no monitor de retorno). -->
<script lang="ts">
  import { MessageSquareText, MonitorSpeaker, Pause, Play, RotateCcw, Send, Timer, X } from "@lucide/svelte";
  import { nav } from "../lib/state/nav.svelte";
  import { outputs } from "../lib/state/outputs.svelte";
  import { pulpit } from "../lib/state/pulpit.svelte";
  import { timerDisplay } from "../lib/timer";

  let draft = $state("");
  const quick = ["Faltam 5 minutos", "Pode encerrar", "Mais perto do microfone", "Próximo hino: "];
  const limits: [number | null, string][] = [[null, "Sem limite"], [15, "15 min"], [30, "30 min"], [45, "45 min"], [60, "1 hora"]];

  const hasMonitor = $derived(outputs.outputs.some((o) => o.role === "monitor"));
  const message = $derived(pulpit.current.message);
  const timer = $derived(pulpit.current.timer);
  const limitMin = $derived(timer?.limitMs != null ? timer.limitMs / 60_000 : null);

  let now = $state(Date.now());
  $effect(() => {
    if (!timer) return;
    const t = setInterval(() => (now = Date.now()), 500);
    return () => clearInterval(t);
  });
  const shown = $derived(timer ? timerDisplay(timer, now) : null);

  function send(text = draft) {
    if (!text.trim()) return;
    void pulpit.say(text);
    draft = "";
  }

  function useQuick(text: string) {
    // "Próximo hino: " é um começo de frase: completa antes de enviar.
    if (text.endsWith(": ")) draft = text;
    else send(text);
  }
</script>

<section class="card panel">
  <h2><MonitorSpeaker size={18} /> Para o púlpito <span class="muted small">— só no retorno, a igreja não vê</span></h2>

  {#if !hasMonitor}
    <div class="hint">
      <span class="grow">Abra um monitor de retorno (a TV de quem prega ou canta) para usar o recado e o cronômetro.</span>
      <button class="btn" onclick={() => nav.reset({ name: "displays" })}>Abrir em Telas</button>
    </div>
  {/if}

  <div class="block">
    <span class="section-title"><MessageSquareText size={14} /> Recado</span>
    {#if message}
      <div class="on-air">
        <span class="grow truncate">“{message}”</span>
        <button class="btn" onclick={() => pulpit.clearMessage()}><X size={14} /> Tirar</button>
      </div>
    {/if}
    <form class="line" onsubmit={(e) => (e.preventDefault(), send())}>
      <input class="input grow" bind:value={draft} placeholder="Escreva um recado para quem está à frente" />
      <button class="btn btn-primary" type="submit" disabled={!draft.trim()}><Send size={15} /> Enviar</button>
    </form>
    <div class="line wrap">
      {#each quick as text (text)}
        <button class="chip" onclick={() => useQuick(text)}>{text.replace(/: $/, "…")}</button>
      {/each}
    </div>
  </div>

  <div class="block">
    <span class="section-title"><Timer size={14} /> Cronômetro do pregador</span>
    <div class="line wrap">
      <div class="segmented">
        {#each limits as [minutes, text] (text)}
          <button aria-pressed={limitMin === minutes} onclick={() => pulpit.setLimit(minutes === null ? null : minutes * 60_000)}>{text}</button>
        {/each}
      </div>
    </div>
    <div class="line">
      <span class="clock" class:warn={shown?.warn} class:over={shown?.over}>{shown?.text ?? (limitMin ? `${limitMin}:00` : "00:00")}</span>
      <span class="grow"></span>
      {#if pulpit.running}
        <button class="btn btn-lg" onclick={() => pulpit.pause()}><Pause size={17} /> Pausar</button>
      {:else}
        <button class="btn btn-lg btn-primary" onclick={() => pulpit.start()}><Play size={17} /> {timer && timer.elapsedMs > 0 ? "Continuar" : "Começar"}</button>
      {/if}
      {#if timer}
        <button class="btn btn-lg btn-icon" aria-label="Zerar e esconder" title="Zerar e esconder" onclick={() => pulpit.reset()}><RotateCcw size={17} /></button>
      {/if}
    </div>
  </div>
</section>

<style>
  .panel {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1rem 1.1rem;
  }

  h2 {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.45rem;
    margin: 0;
    font-size: 1.05rem;
  }

  .small {
    font-size: 0.85rem;
    font-weight: 400;
  }

  .block {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .section-title {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .line {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .wrap {
    flex-wrap: wrap;
  }

  .grow {
    flex: 1;
    min-width: 0;
  }

  .hint,
  .on-air {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.6rem 0.75rem;
    border-radius: var(--radius);
    background: var(--surface-2);
  }

  .on-air {
    background: #fff4d1;
    color: #5c4300;
    font-weight: 600;
  }

  .chip {
    padding: 0.35rem 0.75rem;
    border-radius: 999px;
    border: 1px solid var(--border-strong);
    background: var(--surface);
    color: var(--text);
    font: inherit;
    font-size: 0.88rem;
    cursor: pointer;
  }

  .chip:hover {
    border-color: var(--accent);
  }

  .clock {
    font-size: 2.2rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    color: var(--text);
  }

  .clock.warn {
    color: #c98a00;
  }

  .clock.over {
    color: #d93025;
  }
</style>
