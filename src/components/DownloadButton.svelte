<!-- "Baixar para usar sem internet" com progresso, cancelamento e estado pronto. -->
<script lang="ts">
  import { CircleCheck, CloudDownload, LoaderCircle, X } from "@lucide/svelte";
  import { api } from "../lib/api";
  import { downloads, percent } from "../lib/state/downloads.svelte";
  import type { OfflineStatus, Scope } from "../lib/types";

  let {
    key,
    scope,
    label,
    text = "Baixar para usar sem internet",
    large = false,
  }: { key: string; scope: Scope; label: string; text?: string; large?: boolean } = $props();

  let status = $state<OfflineStatus | null>(null);
  const running = $derived(downloads.running(key));

  // Reconfere ao abrir e sempre que um download desta chave termina.
  $effect(() => {
    void running;
    void JSON.stringify(scope);
    if (!running) void api.offlineStatus(scope).then((s) => (status = s)).catch(() => (status = null));
  });

  const ready = $derived(status !== null && status.total > 0 && status.cached >= status.total);
</script>

{#if running}
  <span class="progress" class:large>
    <LoaderCircle size={16} class="spin" />
    Baixando… {percent(running)}%
    <span class="muted">({running.cached}/{running.total || "?"})</span>
    <button class="btn btn-plain btn-icon" aria-label="Cancelar download" title="Cancelar" onclick={() => downloads.cancel(running.job)}>
      <X size={15} />
    </button>
  </span>
{:else if ready}
  <span class="pill accent"><CircleCheck size={14} /> Pronto para usar sem internet</span>
{:else}
  <button class="btn" class:btn-lg={large} onclick={() => downloads.start(key, scope, label)}>
    <CloudDownload size={17} />
    {text}
    {#if status && status.cached > 0}<span class="muted">({percent(status)}% já aqui)</span>{/if}
  </button>
{/if}

<style>
  .progress {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    font-weight: 550;
  }

  .progress.large {
    font-size: 1.05rem;
  }
</style>
