<script lang="ts">
  import { AppWindow, Monitor, MonitorOff, MonitorPlay, MonitorSpeaker, RefreshCw } from "@lucide/svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import { outputs } from "../lib/state/outputs.svelte";

  $effect(() => {
    void outputs.refresh();
  });

  const windowed = $derived(outputs.outputs.filter((o) => o.displayId === null));
  const roleOf = (label: string | null) => outputs.outputs.find((o) => o.label === label)?.role ?? "audience";
</script>

<PageHeader title="Telas" subtitle="Escolha onde a letra aparece para a igreja e, se quiser, um retorno para quem está à frente">
  {#snippet actions()}
    <button class="btn" onclick={() => outputs.refresh()}><RefreshCw size={15} /> Atualizar</button>
  {/snippet}
</PageHeader>

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

<section class="windowed">
  <div class="row-head">
    <div>
      <h2>Janela de projeção</h2>
      <p class="muted">Para conferir os slides ou arrastar para uma tela manualmente.</p>
    </div>
    <button class="btn" onclick={() => outputs.open(null)}><AppWindow size={16} /> Abrir em uma janela</button>
  </div>
  {#each windowed as o (o.label)}
    <div class="row group">
      <AppWindow size={18} />
      <span class="grow">Janela de projeção</span>
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

  .row-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }
</style>
