<script lang="ts">
  import { AppWindow, Monitor, MonitorOff, MonitorPlay, MonitorSpeaker, RefreshCw, RotateCw, ScanEye } from "@lucide/svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import ProfilesPanel from "../components/ProfilesPanel.svelte";
  import { api } from "../lib/api";
  import { outputs } from "../lib/state/outputs.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import type { MonitorOptions } from "../lib/types";

  const mon = $derived(settings.current.monitor);
  const setMon = (patch: Partial<MonitorOptions>) => settings.save({ monitor: patch });

  $effect(() => {
    void outputs.refresh();
  });

  const windowed = $derived(outputs.outputs.filter((o) => o.displayId === null));
  const roleOf = (label: string | null) => outputs.outputs.find((o) => o.label === label)?.role ?? "audience";
</script>

<PageHeader title="Telas" subtitle="Escolha onde a letra aparece para a igreja e, se quiser, um retorno para quem está à frente">
  {#snippet actions()}
    {#if outputs.outputs.length}
      <button class="btn" onclick={() => api.identifyDisplays()} title="Cada tela aberta mostra o próprio nome"><ScanEye size={15} /> Identificar telas</button>
      <button class="btn" onclick={() => outputs.reopen()} title="Fecha e abre de novo todas as telas, nos mesmos lugares. Use se alguma travar.">
        <RotateCw size={15} /> Reabrir telas
      </button>
    {/if}
    <button class="btn" onclick={() => outputs.refresh()}><RefreshCw size={15} /> Atualizar</button>
  {/snippet}
</PageHeader>

<ProfilesPanel />

<div class="displays">
  {#each outputs.displays as d (d.id)}
    {@const role = roleOf(d.output)}
    <div class="display card" class:active={!!d.output} class:monitor={!!d.output && role === "monitor"}>
      <div class="screen" style:aspect-ratio="{d.width} / {d.height}">
        {#if d.output && role === "monitor"}<MonitorSpeaker size={30} />{:else if d.output}<MonitorPlay size={30} />{:else}<Monitor size={30} />{/if}
      </div>
      <div class="meta">
        <h3>{d.name}</h3>
        <span class="muted">{d.width} × {d.height}</span>
        <div class="badges">
          {#if d.hostsController}<span class="pill">Este controle está aqui</span>{/if}
          {#if d.primary}<span class="pill">Principal</span>{/if}
          {#if d.output}<span class="pill live">{role === "monitor" ? "Retorno" : "Telão"}</span>{/if}
        </div>
      </div>
      {#if d.output}
        <button class="btn btn-lg" onclick={() => d.output && outputs.close(d.output)}><MonitorOff size={17} /> Parar</button>
        <button class="btn btn-plain" onclick={() => outputs.open(d.id, role === "monitor" ? "audience" : "monitor")}>
          {role === "monitor" ? "Usar como telão" : "Usar como retorno"}
        </button>
      {:else}
        <button class="btn btn-lg btn-primary" onclick={() => outputs.open(d.id)}><MonitorPlay size={17} /> Projetar aqui</button>
        <button class="btn btn-plain" title="Letra atual, a próxima e as horas, para quem canta ou prega" onclick={() => outputs.open(d.id, "monitor")}>
          <MonitorSpeaker size={16} /> Usar como retorno
        </button>
      {/if}
      {#if d.hostsController && !d.output}
        <p class="hint subtle">A projeção cobrirá esta janela. Use outra tela ou uma janela.</p>
      {/if}
    </div>
  {/each}
</div>

<section class="monitor-options">
  <div>
    <h2>Tela de retorno</h2>
    <p class="muted">O que aparece para quem canta ou prega. Vale para todas as telas de retorno.</p>
  </div>
  <div class="group">
    <label class="row">
      <span class="grow">
        Mostrar a próxima letra
        <span class="muted block">Embaixo, em letra menor. Desligue se o pessoal achar confuso.</span>
      </span>
      <input type="checkbox" class="switch" checked={mon.showNext} onchange={(e) => setMon({ showNext: e.currentTarget.checked })} />
    </label>
    <label class="row">
      <span class="grow">
        Mostrar a imagem de fundo
        <span class="muted block">A mesma do telão (também nos avisos e na tela de espera), escurecida para a letra continuar fácil de ler. Desligada, o fundo fica preto.</span>
      </span>
      <input type="checkbox" class="switch" checked={mon.showBackground} onchange={(e) => setMon({ showBackground: e.currentTarget.checked })} />
    </label>
    <label class="row">
      <span class="grow">Mostrar as horas</span>
      <input type="checkbox" class="switch" checked={mon.showClock} onchange={(e) => setMon({ showClock: e.currentTarget.checked })} />
    </label>
    <label class="row">
      <span class="grow">Mostrar o nome do hino ou a referência</span>
      <input type="checkbox" class="switch" checked={mon.showTitle} onchange={(e) => setMon({ showTitle: e.currentTarget.checked })} />
    </label>
    <div class="row">
      <span class="grow">Tamanho da letra</span>
      <div class="segmented">
        {#each [[0.85, "Menor"], [1, "Normal"], [1.2, "Maior"]] as [value, label] (value)}
          <button aria-pressed={mon.textScale === value} onclick={() => setMon({ textScale: Number(value) })}>{label}</button>
        {/each}
      </div>
    </div>
  </div>
</section>

<section class="windowed">
  <div class="row-head">
    <div>
      <h2>Em uma janela</h2>
      <p class="muted">
        Para conferir os slides, ou para uma tela que não aparece acima (alguns adaptadores USB para HDMI): abra a janela,
        arraste para a tela e dê dois cliques para ocupar a tela inteira.
      </p>
    </div>
    <div class="window-buttons">
      <button class="btn" onclick={() => outputs.open(null)}><AppWindow size={16} /> Telão em janela</button>
      <button class="btn" onclick={() => outputs.open(null, "monitor")}><MonitorSpeaker size={16} /> Retorno em janela</button>
    </div>
  </div>
  {#each windowed as o (o.label)}
    <div class="row group">
      {#if o.role === "monitor"}<MonitorSpeaker size={18} />{:else}<AppWindow size={18} />{/if}
      <span class="grow">{o.role === "monitor" ? "Retorno em janela" : "Telão em janela"}</span>
      <button class="btn btn-plain" onclick={() => outputs.close(o.label)}>Fechar</button>
    </div>
  {/each}
</section>

<style>
  .displays {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(16rem, 1fr));
    gap: 1rem;
  }

  .display {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    padding: 1.1rem;
    transition:
      border-color var(--normal),
      box-shadow var(--normal);
  }

  .display.active {
    border-color: var(--live);
    box-shadow: 0 0 0 3px var(--live-soft);
  }

  .display.monitor {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .screen {
    display: grid;
    place-items: center;
    max-height: 9rem;
    border-radius: var(--radius);
    background: linear-gradient(160deg, #2b2b3c, #101018);
    color: rgba(255, 255, 255, 0.75);
    box-shadow: inset 0 0 0 4px rgba(255, 255, 255, 0.06);
  }

  .meta {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .badges {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    margin-top: 0.3rem;
  }

  .hint {
    font-size: 0.82rem;
  }

  .windowed {
    margin-top: 2rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .monitor-options {
    margin-top: 2rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .monitor-options h2 {
    margin: 0;
  }

  .block {
    display: block;
  }

  .window-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    flex: none;
  }

  .row-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }
</style>
