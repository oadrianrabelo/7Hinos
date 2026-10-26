<!--
  Cores prontas e, no fim, a bolinha do arco-íris: abre o seletor para
  qualquer cor. Uma cor personalizada fica marcada na própria bolinha.
-->
<script lang="ts">
  import type { Swatch } from "../lib/colors";
  import HsvEditor from "./HsvEditor.svelte";
  import Popover from "./Popover.svelte";

  let {
    value,
    swatches,
    onchange,
    label,
  }: { value: string; swatches: Swatch[]; onchange: (hex: string) => void; label: string } = $props();

  let open = $state(false);
  let trigger = $state<HTMLButtonElement>();
  const custom = $derived(!swatches.some((s) => s.value === value));

  // Arrastar no seletor gera muitas mudanças: salva no máximo a cada 120 ms
  // (e sempre a última), para a prévia acompanhar sem pesar.
  let pending: string | null = null;
  let timer: ReturnType<typeof setTimeout> | null = null;
  function live(hex: string) {
    pending = hex;
    if (timer) return;
    timer = setTimeout(() => {
      timer = null;
      if (pending) onchange(pending);
    }, 120);
  }
</script>

<div class="swatches" role="group" aria-label={label}>
  {#each swatches as s (s.value)}
    <button class="swatch" class:on={value === s.value} style:background={s.value} title={s.label} aria-label={s.label} onclick={() => onchange(s.value)}></button>
  {/each}
  <button
    class="swatch any"
    class:on={custom}
    bind:this={trigger}
    style:--custom={custom ? value : "transparent"}
    title="Outra cor…"
    aria-label="Escolher outra cor"
    aria-expanded={open}
    onclick={() => (open = !open)}
  >
  </button>
</div>

<Popover bind:open anchor={trigger} width={16}>
  <HsvEditor {value} oninput={live} />
</Popover>

<style>
  .swatches {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
  }

  .swatch {
    display: grid;
    place-items: center;
    width: 2rem;
    height: 2rem;
    padding: 0;
    border-radius: 999px;
    border: 1px solid var(--border-strong);
    cursor: pointer;
  }

  .swatch.on {
    box-shadow:
      0 0 0 2px var(--bg),
      0 0 0 4px var(--accent);
  }

  /* Arco-íris em volta; no meio, a cor personalizada (se houver). */
  .any {
    color: #fff;
    background:
      radial-gradient(circle, var(--custom) 0 48%, transparent 50%),
      conic-gradient(#f44, #fb3, #ee4, #4d6, #3cf, #66f, #c5f, #f44);
  }
</style>
