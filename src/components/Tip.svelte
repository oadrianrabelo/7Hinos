<!-- Dica de uma página: aparece até a pessoa clicar em "Entendi". -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Lightbulb } from "@lucide/svelte";
  import { slide } from "svelte/transition";
  import { nav } from "../lib/state/nav.svelte";
  import { tour } from "../lib/state/tour.svelte";

  let { id, article, children }: { id: string; article?: string; children: Snippet } = $props();
</script>

{#if tour.tipVisible(id)}
  <aside class="tip" transition:slide={{ duration: 200 }}>
    <Lightbulb size={18} />
    <p>{@render children()}</p>
    <div class="actions">
      {#if article}<button class="btn btn-plain" onclick={() => nav.push({ name: "help", article })}>Saiba mais</button>{/if}
      <button class="btn" onclick={() => tour.dismissTip(id)}>Entendi</button>
    </div>
  </aside>
{/if}

<style>
  .tip {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1rem;
    padding: 0.65rem 0.75rem 0.65rem 0.9rem;
    border: 1px solid color-mix(in srgb, var(--accent) 35%, transparent);
    border-radius: var(--radius-l);
    background: var(--accent-soft);
  }

  .tip > :global(svg) {
    flex: none;
    color: var(--accent);
  }

  p {
    flex: 1;
    margin: 0;
    line-height: 1.45;
  }

  .actions {
    display: flex;
    gap: 0.35rem;
    flex: none;
  }
</style>
