<script lang="ts">
  import { fly } from "svelte/transition";
  import { CircleAlert, CircleCheck, Info } from "@lucide/svelte";
  import { toasts } from "../lib/state/toasts.svelte";
</script>

<div class="toasts" aria-live="polite">
  {#each toasts.items as toast (toast.id)}
    <div class="toast {toast.kind}" transition:fly={{ y: 16, duration: 220 }}>
      {#if toast.kind === "error"}<CircleAlert size={18} />
      {:else if toast.kind === "success"}<CircleCheck size={18} />
      {:else}<Info size={18} />{/if}
      <span>{toast.message}</span>
      {#if toast.action}
        <button
          class="btn btn-plain"
          onclick={() => {
            toast.action?.run();
            toasts.dismiss(toast.id);
          }}>{toast.action.label}</button
        >
      {/if}
    </div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    left: 50%;
    bottom: 1.5rem;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    z-index: 100;
    pointer-events: none;
  }

  .toast {
    pointer-events: auto;
    display: flex;
    align-items: center;
    gap: 0.6rem;
    max-width: min(34rem, 90vw);
    padding: 0.7rem 1rem;
    border-radius: var(--radius-l);
    background: color-mix(in srgb, var(--surface) 88%, transparent);
    backdrop-filter: blur(20px) saturate(1.6);
    -webkit-backdrop-filter: blur(20px) saturate(1.6);
    border: 1px solid var(--border);
    box-shadow: var(--shadow-l);
    font-weight: 500;
  }

  .toast.error :global(svg) {
    color: var(--danger);
    flex: none;
  }

  .toast.success :global(svg) {
    color: var(--success);
    flex: none;
  }

  .toast.info :global(svg) {
    color: var(--accent);
    flex: none;
  }
</style>
