<!-- Recado e cronômetro que só quem está no púlpito vê (no monitor de retorno). -->
<script lang="ts">
  import ResetButton from "./ResetButton.svelte";
  import { MESSAGE_DEFAULTS } from "../lib/defaults";
  import { Brush, MessageSquareText, Minus, MonitorSpeaker, Pause, Play, Plus, RotateCcw, Send, Timer, X } from "@lucide/svelte";
  import { fontFamily } from "../lib/fonts";
  import { nav } from "../lib/state/nav.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import type { MonitorOptions } from "../lib/types";
  import ColorPicker from "./ColorPicker.svelte";
  import FontPicker from "./FontPicker.svelte";
  import { outputs } from "../lib/state/outputs.svelte";
  import { pulpit } from "../lib/state/pulpit.svelte";
  import { timerDisplay } from "../lib/timer";

  let draft = $state("");
  const quick = ["Faltam 5 minutos", "Pode encerrar", "Mais perto do microfone", "Próximo hino: "];
  const limits: [number | null, string][] = [
    [null, "Sem limite"],
    [5, "5 min"],
    [10, "10"],
    [15, "15"],
    [20, "20"],
    [30, "30"],
    [45, "45"],
    [60, "1 hora"],
  ];
  const presetMinutes = limits.map(([m]) => m);

  // Recado: aparência no retorno (fonte e cores da faixa).
  const mon = $derived(settings.current.monitor);
  const setMon = (patch: Partial<MonitorOptions>) => settings.save({ monitor: patch });
  let styling = $state(false);
  const messageText = [
    { value: "#000000", label: "Preto" },
    { value: "#ffffff", label: "Branco" },
    { value: "#5c1d1d", label: "Vinho" },
    { value: "#173b53", label: "Azul-marinho" },
  ];
  const messageBg = [
    { value: "#ffd36b", label: "Amarelo" },
    { value: "#ffffff", label: "Branco" },
    { value: "#ff8a80", label: "Vermelho claro" },
    { value: "#d93025", label: "Vermelho" },
    { value: "#8fd18f", label: "Verde" },
    { value: "#8fd0f2", label: "Azul claro" },
    { value: "#173b53", label: "Azul-marinho" },
  ];

  const hasMonitor = $derived(outputs.outputs.some((o) => o.role === "monitor"));
  const message = $derived(pulpit.current.message);
  const timer = $derived(pulpit.current.timer);
  const limitMin = $derived(timer?.limitMs != null ? timer.limitMs / 60_000 : null);
  const isCustom = $derived(limitMin !== null && !presetMinutes.includes(limitMin));
  let customOpen = $state(false);
  let customDraft = $state("");

  function applyCustom() {
    // Aceita "25", "25,5", "1:30" (hora:minuto) ou "90".
    const text = customDraft.trim().replace(",", ".");
    const [a, b] = text.split(":");
    const minutes = b !== undefined ? Number(a) * 60 + Number(b) : Number(a);
    if (!Number.isFinite(minutes) || minutes <= 0 || minutes > 600) return;
    void pulpit.setLimit(Math.round(minutes * 60) * 1000);
    customOpen = false;
  }

  /** Mais ou menos tempo com o cronômetro andando (o pregador pediu mais 5 min). */
  function adjust(minutes: number) {
    if (limitMin === null) return;
    void pulpit.setLimit(Math.max(60_000, (limitMin + minutes) * 60_000));
  }

  const limitText = (m: number) => (m >= 60 && m % 60 === 0 ? `${m / 60} h` : m >= 60 ? `${Math.floor(m / 60)}h${String(Math.round(m % 60)).padStart(2, "0")}` : `${Math.round(m * 10) / 10} min`);

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
      <span class="grow"></span>
      <button class="btn btn-plain" aria-expanded={styling} onclick={() => (styling = !styling)}><Brush size={14} /> Aparência</button>
    </div>
    {#if styling}
      <div class="style-box">
        <p
          class="message-sample"
          style:color={mon.messageColor}
          style:background={mon.messageBackground}
          style:font-family={mon.messageFont ? fontFamily(mon.messageFont) : null}
        >
          {message || draft || "Faltam 5 minutos"}
        </p>
        <ResetButton current={mon} defaults={MESSAGE_DEFAULTS} onreset={() => setMon({ ...MESSAGE_DEFAULTS })} />
        <div class="style-grid">
          <span class="section-title">Letra</span>
          <span class="font-cell">
            <FontPicker label="Letra do recado" value={mon.messageFont || "sans"} onchange={(messageFont) => setMon({ messageFont })} sample="Faltam 5 minutos" />
          </span>
          <span class="section-title">Cor do texto</span>
          <ColorPicker label="Cor do texto do recado" value={mon.messageColor} swatches={messageText} onchange={(messageColor) => setMon({ messageColor })} />
          <span class="section-title">Cor da faixa</span>
          <ColorPicker label="Cor da faixa do recado" value={mon.messageBackground} swatches={messageBg} onchange={(messageBackground) => setMon({ messageBackground })} />
        </div>
      </div>
    {/if}
  </div>

  <div class="block">
    <span class="section-title"><Timer size={14} /> Cronômetro — quanto falta para acabar esta parte do culto</span>
    <div class="line wrap">
      <div class="segmented">
        {#each limits as [minutes, text] (text)}
          <button aria-pressed={limitMin === minutes} onclick={() => ((customOpen = false), pulpit.setLimit(minutes === null ? null : minutes * 60_000))}>{text}</button>
        {/each}
        <button
          aria-pressed={isCustom || customOpen}
          onclick={() => {
            customOpen = !customOpen;
            customDraft = limitMin !== null ? String(limitMin) : "";
          }}
        >
          {isCustom && limitMin !== null ? limitText(limitMin) : "Outro…"}
        </button>
      </div>
    </div>
    {#if customOpen}
      <form class="line" onsubmit={(e) => (e.preventDefault(), applyCustom())}>
        <input class="input minutes" bind:value={customDraft} inputmode="decimal" placeholder="Ex.: 25" aria-label="Minutos" />
        <span class="muted">minutos <span class="hint-text">(ou 1:30 para hora e meia)</span></span>
        <button class="btn btn-primary" type="submit" disabled={!customDraft.trim()}>Usar</button>
      </form>
    {/if}
    <div class="line">
      <span class="clock" class:warn={shown?.warn} class:over={shown?.over}>
        <span class="clock-label">{shown?.label ?? (limitMin ? "Faltam" : "Tempo")}</span>
        {shown?.text ?? (limitMin ? `${limitMin}:00` : "00:00")}
      </span>
      <span class="grow"></span>
      {#if limitMin !== null}
        <span class="adjust">
          <button class="btn btn-icon" aria-label="Menos 1 minuto" title="Menos 1 minuto" disabled={limitMin <= 1} onclick={() => adjust(-1)}><Minus size={15} /></button>
          <span class="adjust-label">1 min</span>
          <button class="btn btn-icon" aria-label="Mais 1 minuto" title="Mais 1 minuto" onclick={() => adjust(1)}><Plus size={15} /></button>
        </span>
      {/if}
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

  .minutes {
    width: 6rem;
  }

  .hint-text {
    font-size: 0.82rem;
  }

  .adjust {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .adjust-label {
    font-size: 0.8rem;
    color: var(--text-2);
  }

  .style-box > :global(.reset) {
    align-self: flex-end;
  }

  .style-box {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 0.75rem;
    border-radius: var(--radius);
    background: var(--surface-2);
  }

  .message-sample {
    margin: 0;
    padding: 0.6rem 0.9rem;
    border-radius: var(--radius-s);
    font-size: 1.4rem;
    font-weight: 700;
    text-align: center;
  }

  .style-grid {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    align-items: center;
    gap: 0.6rem 1rem;
  }

  .clock-label {
    display: block;
    font-size: 0.72rem;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--text-2);
  }

  .clock {
    line-height: 1.05;
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
