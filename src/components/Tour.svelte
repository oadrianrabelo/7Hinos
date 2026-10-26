<!--
  O passeio: escurece a janela, deixa à vista só o pedaço explicado e mostra
  um cartão ao lado com Voltar / Próximo / Sair. Também desenha o convite
  depois da primeira configuração.
-->
<script lang="ts">
  import { Compass, X } from "@lucide/svelte";
  import { fade, fly } from "svelte/transition";
  import { tour } from "../lib/state/tour.svelte";

  const PAD = 6;
  const CARD_W = 340;
  const GAP = 14;

  let rect = $state<DOMRect | null>(null);
  let vw = $state(window.innerWidth);
  let vh = $state(window.innerHeight);
  let cardH = $state(180);

  // Procura o pedaço da tela do passo atual (e de novo se a janela mudar).
  $effect(() => {
    const step = tour.current;
    if (!step) return;
    const measure = () => {
      vw = window.innerWidth;
      vh = window.innerHeight;
      const el = document.querySelector<HTMLElement>(`[data-tour="${step.anchor}"]`);
      const r = el?.getBoundingClientRect();
      rect = r && r.width && r.height ? r : null;
    };
    // Espera a página trocar (o passeio começa no Início).
    const frame = requestAnimationFrame(measure);
    window.addEventListener("resize", measure);
    return () => {
      cancelAnimationFrame(frame);
      window.removeEventListener("resize", measure);
    };
  });

  const hole = $derived(
    rect && {
      left: rect.left - PAD,
      top: rect.top - PAD,
      width: rect.width + PAD * 2,
      height: rect.height + PAD * 2,
    },
  );

  // O cartão vai onde couber: à direita, à esquerda, embaixo ou em cima.
  const card = $derived.by(() => {
    const clampX = (x: number) => Math.max(12, Math.min(vw - CARD_W - 12, x));
    const clampY = (y: number) => Math.max(12, Math.min(vh - cardH - 12, y));
    if (!hole) return { left: (vw - CARD_W) / 2, top: (vh - cardH) / 2 };
    if (hole.left + hole.width + GAP + CARD_W < vw) return { left: hole.left + hole.width + GAP, top: clampY(hole.top) };
    if (hole.left - GAP - CARD_W > 0) return { left: hole.left - GAP - CARD_W, top: clampY(hole.top) };
    if (hole.top + hole.height + GAP + cardH < vh) return { left: clampX(hole.left), top: hole.top + hole.height + GAP };
    return { left: clampX(hole.left), top: clampY(hole.top - GAP - cardH) };
  });

  function onKey(e: KeyboardEvent) {
    if (tour.offer && e.key === "Escape") {
      tour.skip();
    } else if (tour.active) {
      if (e.key === "Escape") tour.close();
      else if (e.key === "ArrowRight" || e.key === "Enter") tour.next();
      else if (e.key === "ArrowLeft") tour.back();
      else return;
    } else return;
    // O passeio não deixa a tecla chegar aos atalhos de projeção.
    e.preventDefault();
    e.stopImmediatePropagation();
  }
</script>

<svelte:window onkeydowncapture={onKey} />

{#if tour.offer}
  <div class="backdrop" transition:fade={{ duration: 200 }}>
    <div class="offer card" role="dialog" aria-modal="true" aria-labelledby="tour-offer-title" in:fly={{ y: 16, duration: 300 }}>
      <span class="badge"><Compass size={28} /></span>
      <h2 id="tour-offer-title">Quer um passeio rápido?</h2>
      <p class="muted">Em menos de um minuto mostramos onde fica cada coisa. Dá para ver de novo depois, na Ajuda.</p>
      <div class="choices">
        <button class="btn btn-primary btn-lg" onclick={() => tour.start()}>Me mostre como usar</button>
        <button class="btn btn-lg" onclick={() => tour.skip()}>Já sei usar, pular</button>
      </div>
    </div>
  </div>
{/if}

{#if tour.current}
  {@const step = tour.current}
  <div class="tour" transition:fade={{ duration: 150 }}>
    {#if hole}
      <div class="hole" style:left="{hole.left}px" style:top="{hole.top}px" style:width="{hole.width}px" style:height="{hole.height}px"></div>
    {:else}
      <div class="dim"></div>
    {/if}
    <div
      class="step card"
      role="dialog"
      aria-modal="true"
      aria-labelledby="tour-step-title"
      bind:clientHeight={cardH}
      style:left="{card.left}px"
      style:top="{card.top}px"
      style:width="{CARD_W}px"
    >
      <header>
        <span class="count">{tour.step + 1} de {tour.steps.length}</span>
        <button class="btn btn-plain btn-icon" aria-label="Sair do passeio" onclick={() => tour.close()}><X size={18} /></button>
      </header>
      <h3 id="tour-step-title">{step.title}</h3>
      <p>{step.text}</p>
      <div class="dots" aria-hidden="true">
        {#each tour.steps as _, i (i)}<span class:on={i === tour.step}></span>{/each}
      </div>
      <footer>
        {#if tour.step > 0}<button class="btn" onclick={() => tour.back()}>Voltar</button>{/if}
        <span class="spacer"></span>
        <button class="btn btn-primary" onclick={() => tour.next()}>
          {tour.step === tour.steps.length - 1 ? "Terminar" : "Próximo"}
        </button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 200;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.45);
  }

  .offer {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
    width: min(30rem, calc(100vw - 2rem));
    padding: 2rem 1.75rem 1.5rem;
    text-align: center;
    box-shadow: var(--shadow-l);
  }

  .offer h2 {
    margin: 0;
    font-size: 1.5rem;
  }

  .offer p {
    margin: 0 0 0.5rem;
    line-height: 1.5;
  }

  .badge {
    display: grid;
    place-items: center;
    width: 3.6rem;
    height: 3.6rem;
    border-radius: 999px;
    background: var(--accent-soft);
    color: var(--accent);
  }

  /* Os dois caminhos com o mesmo tamanho: pular não é "pior". */
  .choices {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.6rem;
    width: 100%;
  }

  .choices .btn {
    justify-content: center;
  }

  .tour {
    position: fixed;
    inset: 0;
    z-index: 200;
  }

  .dim {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
  }

  /* O buraco no escuro: uma sombra enorme em volta do que se explica. */
  .hole {
    position: absolute;
    border-radius: 12px;
    box-shadow:
      0 0 0 3px var(--accent),
      0 0 0 9999px rgba(0, 0, 0, 0.55);
    transition: all 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
    pointer-events: none;
  }

  .step {
    position: absolute;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.9rem 1rem 1rem 1.1rem;
    box-shadow: var(--shadow-l);
    transition:
      left 0.3s cubic-bezier(0.2, 0.8, 0.2, 1),
      top 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .count {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--accent);
  }

  h3 {
    margin: 0;
    font-size: 1.15rem;
  }

  p {
    margin: 0;
    line-height: 1.5;
  }

  .dots {
    display: flex;
    gap: 0.3rem;
    margin-top: 0.25rem;
  }

  .dots span {
    width: 0.4rem;
    height: 0.4rem;
    border-radius: 999px;
    background: var(--border);
  }

  .dots span.on {
    width: 1.1rem;
    background: var(--accent);
  }

  footer {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.25rem;
  }

  .spacer {
    flex: 1;
  }
</style>
