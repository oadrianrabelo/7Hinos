<!--
  Uma página de apresentação com as ferramentas do apresentador por cima:
  zoom, foco, caneta, marca-texto e laser. O telão, o retorno e a prévia do
  controle usam este mesmo componente, então tudo aparece igual.
-->
<script lang="ts">
  import { cubicOut } from "svelte/easing";
  import { strokePath, zoomTransform } from "../lib/overlay";
  import type { PageOverlay } from "../lib/types";

  let {
    src,
    overlay = null,
    animate = true,
    onbox,
  }: {
    src: string;
    overlay?: PageOverlay | null;
    animate?: boolean;
    /** Avisa o tamanho e a posição da página na tela (para quem desenha por cima). */
    onbox?: (box: { el: HTMLElement; aspect: number }) => void;
  } = $props();

  let cw = $state(0);
  let ch = $state(0);
  // Proporção da imagem (largura/altura); até carregar, a de um slide.
  let aspect = $state(16 / 9);

  const box = $derived.by(() => {
    if (!cw || !ch) return { w: 0, h: 0 };
    return cw / ch > aspect ? { w: ch * aspect, h: ch } : { w: cw, h: cw / aspect };
  });

  const z = $derived(zoomTransform(overlay?.zoom));
  const strokes = $derived(overlay?.strokes ?? []);
  const spot = $derived(overlay?.spotlight ?? null);
  const laser = $derived(overlay?.laser ?? null);
  const transition = $derived(animate ? (overlay?.transition ?? "fade") : "none");
  const dir = $derived(overlay?.dir ?? 1);

  let boxEl = $state<HTMLElement>();
  $effect(() => {
    if (boxEl && onbox) onbox({ el: boxEl, aspect });
  });

  /** Entrada e saída da página, conforme a transição escolhida. */
  function page(_node: Element, { out = false } = {}) {
    const kind = transition;
    if (kind === "none") return { duration: 0 };
    const sign = out ? -dir : dir;
    return {
      duration: kind === "fade" ? 350 : 450,
      easing: cubicOut,
      css: (t: number, u: number) => {
        if (kind === "slide") return `transform: translateX(${u * 100 * sign}%);`;
        if (kind === "zoom") return `opacity: ${t}; transform: scale(${out ? 1 + u * 0.08 : 0.92 + t * 0.08});`;
        return `opacity: ${t};`;
      },
    };
  }
</script>

<div class="page-view" bind:clientWidth={cw} bind:clientHeight={ch}>
  {#key src}
    <div class="layer" in:page out:page={{ out: true }}>
      <div class="box" bind:this={boxEl} style:width="{box.w}px" style:height="{box.h}px">
        <div
          class="zoomer"
          class:smooth={!laser}
          style:transform="translate({z.tx * box.w}px, {z.ty * box.h}px) scale({z.scale})"
        >
          <img
            {src}
            alt=""
            onload={(e) => {
              const img = e.currentTarget as HTMLImageElement;
              if (img.naturalWidth && img.naturalHeight) aspect = img.naturalWidth / img.naturalHeight;
            }}
          />
          {#if strokes.length}
            <svg viewBox="0 0 {box.w} {box.h}" preserveAspectRatio="none" aria-hidden="true">
              {#each strokes as s, i (i)}
                <path
                  d={strokePath(s, box.w, box.h)}
                  class={s.tool}
                  stroke={s.color}
                  stroke-width={(s.width / 1000) * box.w}
                />
              {/each}
            </svg>
          {/if}
          {#if spot}
            <div
              class="spotlight"
              style:background="radial-gradient(circle at {spot.x * 100}% {spot.y * 100}%, transparent {spot.r * box.w}px, rgba(0, 0, 0, 0.72) {spot.r * box.w + 2}px)"
            ></div>
          {/if}
          {#if laser}
            <span class="laser" style:left="{laser.x * 100}%" style:top="{laser.y * 100}%" style:--s={1 / z.scale}></span>
          {/if}
        </div>
      </div>
    </div>
  {/key}
</div>

<style>
  .page-view {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: #000;
  }

  .layer {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
  }

  .box {
    position: relative;
    overflow: hidden;
  }

  .zoomer {
    position: absolute;
    inset: 0;
    transform-origin: 0 0;
  }

  .zoomer.smooth {
    transition: transform 0.35s cubic-bezier(0.2, 0.8, 0.2, 1);
  }

  img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: fill;
    user-select: none;
    -webkit-user-drag: none;
  }

  svg {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    overflow: visible;
  }

  path {
    fill: none;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  /* Marca-texto: translúcido e "por baixo" do texto, como no papel. */
  path.marker {
    opacity: 0.38;
    mix-blend-mode: multiply;
    stroke-linecap: square;
  }

  .spotlight {
    position: absolute;
    inset: -1px;
    pointer-events: none;
    transition: background 0.15s;
  }

  .laser {
    position: absolute;
    width: 1.6%;
    aspect-ratio: 1;
    border-radius: 999px;
    transform: translate(-50%, -50%) scale(var(--s, 1));
    background: radial-gradient(circle, #fff 0 18%, #ff2a2a 30%, rgba(255, 42, 42, 0.5) 55%, transparent 72%);
    box-shadow: 0 0 1.2vw 0.4vw rgba(255, 30, 30, 0.55);
    pointer-events: none;
  }
</style>
