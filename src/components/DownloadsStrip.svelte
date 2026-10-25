<!-- Downloads em andamento, discretos no canto, com opção de cancelar. -->
<script lang="ts">
  import { fly } from "svelte/transition";
  import { CloudDownload, X } from "@lucide/svelte";
  import { downloads, percent } from "../lib/state/downloads.svelte";

  const active = $derived(downloads.active.filter((d) => !d.finished));
</script>

{#if active.length}
  <div class="strip card" transition:fly={{ y: 12, duration: 200 }}>
    {#each active as d (d.job)}
      <div class="job">
        <CloudDownload size={16} />
        <span class="label truncate">{d.label}</span>
        <span class="bar"><span class="fill" style:width="{percent(d)}%"></span></span>
        <span class="pct">{percent(d)}%</span>
        <button class="btn btn-plain btn-icon" aria-label="Cancelar" title="Cancelar" onclick={() => downloads.cancel(d.job)}><X size={14} /></button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .strip {
    position: fixed;
    right: 1rem;
    bottom: 1rem;
    z-index: 80;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    width: min(24rem, 90vw);
    padding: 0.45rem 0.45rem 0.45rem 0.8rem;
    box-shadow: var(--shadow-l);
  }

  .job {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    font-size: 0.88rem;
  }

  .job :global(svg) {
    color: var(--accent);
    flex: none;
  }

  .label {
    flex: 1;
    min-width: 0;
    font-weight: 550;
  }

  .bar {
    width: 5rem;
    height: 5px;
    border-radius: 5px;
    background: var(--surface-3);
    overflow: hidden;
  }

  .fill {
    display: block;
    height: 100%;
    background: var(--accent);
    transition: width var(--normal) var(--ease);
  }

  .pct {
    width: 2.6rem;
    text-align: right;
    font-variant-numeric: tabular-nums;
    color: var(--text-2);
  }
</style>
