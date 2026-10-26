<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    subtitle = "",
    actions,
    narrow = false,
  }: { title: string; subtitle?: string; actions?: Snippet; narrow?: boolean } = $props();
</script>

<header class="page-header" class:narrow>
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

  /* Acompanha as páginas de formulário (Ajustes, Avisos), centralizadas. */
  .narrow {
    max-width: 52rem;
    margin-inline: auto;
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
