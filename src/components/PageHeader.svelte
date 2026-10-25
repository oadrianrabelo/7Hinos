<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    subtitle = "",
    actions,
  }: { title: string; subtitle?: string; actions?: Snippet } = $props();
</script>

<header class="page-header">
  <div class="text">
    <h1>{title}</h1>
    {#if subtitle}<p class="muted">{subtitle}</p>{/if}
  </div>
  {#if actions}<div class="actions">{@render actions()}</div>{/if}
</header>

<style>
  .page-header {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 1.25rem;
  }

  .text {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    min-width: 0;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
    flex: none;
  }
</style>
