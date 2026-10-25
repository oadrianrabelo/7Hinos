<!--
  Isola falhas: se uma parte da tela quebrar, só ela mostra o aviso e o resto
  do app continua. Na projeção (`silent`), a falha vira tela preta, nunca uma
  mensagem técnica no telão.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { RotateCcw, TriangleAlert } from "@lucide/svelte";
  import { report } from "../lib/errors";

  let {
    children,
    where,
    silent = false,
    onhome,
  }: { children: Snippet; where: string; silent?: boolean; onhome?: () => void } = $props();
</script>

<svelte:boundary onerror={(error) => report(error, where)}>
  {@render children()}

  {#snippet failed(_error, reset)}
    {#if !silent}
      <div class="empty">
        <TriangleAlert size={40} strokeWidth={1.5} />
        <h2>Esta parte teve um problema</h2>
        <p>O restante do 7Hinos continua funcionando. O erro foi registrado.</p>
        <div class="actions">
          <button class="btn btn-primary" onclick={reset}><RotateCcw size={15} /> Tentar de novo</button>
          {#if onhome}<button class="btn" onclick={onhome}>Ir para o início</button>{/if}
        </div>
      </div>
    {/if}
  {/snippet}
</svelte:boundary>

<style>
  .actions {
    display: flex;
    gap: 0.5rem;
  }
</style>
