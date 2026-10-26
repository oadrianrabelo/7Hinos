<!--
  Menu flutuante preso a um botão: abre embaixo dele (ou em cima, se não
  couber) e fecha com Esc ou clicando fora. Fica por cima de tudo, mesmo
  dentro de cartões com rolagem.
-->
<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    open = $bindable(false),
    anchor,
    width = 26,
    children,
  }: { open: boolean; anchor: HTMLElement | undefined; width?: number; children: Snippet } = $props();

  let panel = $state<HTMLElement>();
  let place = $state({ top: 0, left: 0, maxHeight: 400 });

  function position() {
    if (!anchor) return;
    const r = anchor.getBoundingClientRect();
    const rem = parseFloat(getComputedStyle(document.documentElement).fontSize) || 16;
    const w = Math.min(width * rem, window.innerWidth - 16);
    const left = Math.max(8, Math.min(r.left, window.innerWidth - w - 8));
    const below = window.innerHeight - r.bottom - 12;
    const above = r.top - 12;
    // Embaixo, a não ser que em cima caiba bem mais.
    if (below >= 280 || below >= above) place = { top: r.bottom + 6, left, maxHeight: below };
    else place = { top: Math.max(8, r.top - 6 - Math.min(above, 560)), left, maxHeight: Math.min(above, 560) };
  }

  $effect(() => {
    if (!open) return;
    position();
    const outside = (e: PointerEvent) => {
      const t = e.target as Node;
      if (!panel?.contains(t) && !anchor?.contains(t)) open = false;
    };
    const key = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.stopPropagation();
        open = false;
        anchor?.focus();
      }
    };
    const reflow = () => position();
    document.addEventListener("pointerdown", outside, true);
    document.addEventListener("keydown", key, true);
    window.addEventListener("resize", reflow);
    window.addEventListener("scroll", reflow, true);
    return () => {
      document.removeEventListener("pointerdown", outside, true);
      document.removeEventListener("keydown", key, true);
      window.removeEventListener("resize", reflow);
      window.removeEventListener("scroll", reflow, true);
    };
  });
</script>

{#if open}
  <div
    class="popover"
    bind:this={panel}
    role="dialog"
    style:top="{place.top}px"
    style:left="{place.left}px"
    style:width="min({width}rem, calc(100vw - 16px))"
    style:max-height="{place.maxHeight}px"
  >
    {@render children()}
  </div>
{/if}

<style>
  .popover {
    position: fixed;
    z-index: 60;
    overflow-y: auto;
    padding: 0.75rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-l);
    background: var(--surface);
    box-shadow:
      0 12px 32px rgba(0, 0, 0, 0.18),
      0 2px 6px rgba(0, 0, 0, 0.08);
    animation: pop 0.12s ease-out;
  }

  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
  }
</style>
