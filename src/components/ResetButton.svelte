<!-- "Voltar ao padrão": só aparece quando algo do grupo mudou. -->
<script lang="ts">
  import { RotateCcw } from "@lucide/svelte";
  import { isDefault } from "../lib/defaults";

  let {
    current,
    defaults,
    onreset,
    label = "Voltar ao padrão",
  }: {
    current: object;
    defaults: object;
    onreset: () => void;
    label?: string;
  } = $props();

  const changed = $derived(!isDefault(current, defaults));
</script>

{#if changed}
  <button class="btn btn-plain reset" title="Volta ao jeito que o 7Hinos vem instalado" onclick={onreset}>
    <RotateCcw size={14} />
    {label}
  </button>
{/if}

<style>
  .reset {
    flex: none;
    color: var(--text-2);
  }
</style>
