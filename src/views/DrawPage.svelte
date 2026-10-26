<!-- Sorteio no telão: números ou nomes, com animação e sem repetir. -->
<script lang="ts">
  import { Dices, Hash, RotateCcw, Users } from "@lucide/svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import SceneView from "../components/SceneView.svelte";
  import { draw } from "../lib/state/draw.svelte";
  import { live } from "../lib/state/live.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import { stage } from "../lib/state/stage.svelte";

  const s = $derived(settings.current);
  const onAir = $derived(live.source === "draw" && stage.current.scene.kind === "draw");
  const pool = $derived(draw.pool);
  const remaining = $derived(draw.remaining);
  const speeds: [number, string][] = [
    [3, "Rápido"],
    [5, "Normal"],
    [8, "Suspense"],
  ];

  // Enquanto o telão embaralha, o botão espera (e o último sorteado aparece só no fim).
  const rolling = $derived(draw.rolling);
  const last = $derived(draw.drawn.at(-1) ?? null);
</script>

<PageHeader narrow title="Sorteio" subtitle="Sorteie números ou nomes no telão, com suspense — para brindes, programas e momentos especiais" />

<div class="page">
  <section class="card panel">
    <div class="segmented">
      <button aria-pressed={draw.mode === "numbers"} onclick={() => (draw.mode = "numbers")}><Hash size={15} /> Números</button>
      <button aria-pressed={draw.mode === "names"} onclick={() => (draw.mode = "names")}><Users size={15} /> Nomes</button>
    </div>

    {#if draw.mode === "numbers"}
      <div class="line">
        <label class="field-inline">De <input class="input num" type="number" bind:value={draw.from} /></label>
        <label class="field-inline">até <input class="input num" type="number" bind:value={draw.to} /></label>
        <span class="muted">{pool.length} números</span>
      </div>
    {:else}
      <label class="stack">
        <span class="form-label">Um nome por linha</span>
        <textarea class="input names" bind:value={draw.names} rows="7" placeholder={"Ana\nJoão\nMaria"}></textarea>
        <span class="muted">{pool.length} {pool.length === 1 ? "nome" : "nomes"}</span>
      </label>
    {/if}

    <div class="grid2">
      <label class="stack">
        <span class="form-label">Frase em cima (opcional)</span>
        <input class="input" bind:value={draw.title} placeholder="Sorteio" />
      </label>
      <div class="stack">
        <span class="form-label">Duração</span>
        <div class="segmented">
          {#each speeds as [seconds, label] (seconds)}
            <button aria-pressed={draw.seconds === seconds} onclick={() => (draw.seconds = seconds)}>{label}</button>
          {/each}
        </div>
      </div>
    </div>

    <label class="line">
      <input type="checkbox" class="switch" bind:checked={draw.allowRepeat} />
      <span>Pode sair de novo quem já foi sorteado</span>
    </label>

    <div class="actions">
      <button class="btn btn-lg btn-live big" disabled={rolling || !remaining.length} onclick={() => draw.run()}>
        <Dices size={20} /> {rolling ? "Sorteando…" : draw.drawn.length ? "Sortear outro" : "Sortear no telão"}
      </button>
      {#if onAir}
        <button class="btn btn-lg" onclick={() => draw.rest()}>Tirar do telão</button>
      {/if}
      <span class="muted">{remaining.length} {remaining.length === 1 ? "pode sair" : "podem sair"}</span>
    </div>
  </section>

  <div class="side">
    <div class="preview">
      {#if onAir}
        <SceneView scene={stage.current.scene} style={s.slide} idle={s.idle} screen={s.screenStyle} />
      {:else}
        <SceneView
          scene={{ kind: "draw", title: draw.title.trim(), result: last ?? "?", candidates: [], startedAt: 0, durationMs: 1 }}
          style={s.slide}
          idle={s.idle}
          screen={s.screenStyle}
        />
      {/if}
    </div>

    <section class="card panel">
      <header class="line">
        <h2 class="grow">Já sorteados</h2>
        {#if draw.drawn.length}
          <button class="btn btn-plain" onclick={() => draw.restart()}><RotateCcw size={14} /> Recomeçar</button>
        {/if}
      </header>
      {#if draw.drawn.length}
        <ol class="history">
          {#each draw.drawn as name, i (i)}
            <li><span class="n">{i + 1}º</span> {name}</li>
          {/each}
        </ol>
      {:else}
        <p class="muted">Ninguém ainda. A aparência segue “Avisos e relógio”.</p>
      {/if}
    </section>
  </div>
</div>

<style>
  .page {
    display: grid;
    grid-template-columns: minmax(0, 1.1fr) minmax(0, 1fr);
    gap: 1rem;
    align-items: start;
    max-width: 52rem;
    margin-inline: auto;
  }

  .panel {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    padding: 1rem 1.1rem;
  }

  .segmented {
    align-self: flex-start;
  }

  .line {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .grow {
    flex: 1;
  }

  h2 {
    margin: 0;
    font-size: 1rem;
  }

  .field-inline {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .num {
    width: 6.5rem;
  }

  .stack {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .names {
    resize: vertical;
    font: inherit;
  }

  .grid2 {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .actions {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.6rem;
  }

  .big {
    font-size: 1.05rem;
  }

  .side {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .preview {
    aspect-ratio: 16 / 9;
    border-radius: var(--radius-l);
    overflow: hidden;
  }

  .history {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    max-height: 16rem;
    overflow-y: auto;
  }

  .history li {
    display: flex;
    gap: 0.6rem;
    padding: 0.35rem 0.5rem;
    border-radius: var(--radius-s);
    background: var(--surface-2);
    font-weight: 600;
  }

  .n {
    min-width: 2rem;
    color: var(--text-2);
    font-variant-numeric: tabular-nums;
  }

  @media (max-width: 900px) {
    .page {
      grid-template-columns: 1fr;
    }
  }
</style>
