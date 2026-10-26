<!--
  Uma página de apresentação com as ferramentas do apresentador por cima:
  zoom, foco, caneta, marca-texto e laser. O telão, o retorno e a prévia do
  controle usam este mesmo componente, então tudo aparece igual.
-->
<script lang="ts">
  import { cubicOut } from "svelte/easing";
  import { camera, strokePath, type Geometry } from "../lib/overlay";
  import type { PageOverlay } from "../lib/types";

  let {
    src,
    overlay = null,
    animate = true,
    smooth = true,
    onbox,
  }: {
    src: string;
    overlay?: PageOverlay | null;
    animate?: boolean;
    /** Zoom e rolagem deslizam até o lugar (desligado enquanto se arrasta com a mão). */
    smooth?: boolean;
    /** Avisa a tela e a proporção da página (para quem desenha por cima). */
    onbox?: (box: { el: HTMLElement; geometry: Geometry }) => void;
  } = $props();

  let cw = $state(0);
  let ch = $state(0);
  // Proporção da imagem (largura/altura); até carregar, a de um slide.
  let aspect = $state(16 / 9);

  // A página é desenhada no tamanho da "câmera": com zoom ela ocupa a tela
  // toda (inclusive onde seriam as barras pretas de um PDF em pé).
  const geometry = $derived<Geometry>({ cw, ch, aspect });
  const cam = $derived(camera(overlay?.zoom, geometry));
  const strokes = $derived(overlay?.strokes ?? []);
  const spot = $derived(overlay?.spotlight ?? null);
  const laser = $derived(overlay?.laser ?? null);
  const transition = $derived(animate ? (overlay?.transition ?? "fade") : "none");
  const dir = $derived(overlay?.dir ?? 1);

  let screenEl = $state<HTMLElement>();
  $effect(() => {
    if (screenEl && onbox) onbox({ el: screenEl, geometry });
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

<div class="page-view" bind:this={screenEl} bind:clientWidth={cw} bind:clientHeight={ch}>
  {#key src}
    <div class="layer" in:page out:page={{ out: true }}>
      <div
        class="sheet"
        class:smooth={smooth && !laser}
        style:width="{cam.w}px"
        style:height="{cam.h}px"
        style:transform="translate({cam.left}px, {cam.top}px)"
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
          <svg viewBox="0 0 {cam.w} {cam.h}" preserveAspectRatio="none" aria-hidden="true">
            {#each strokes as s, i (i)}
              <path d={strokePath(s, cam.w, cam.h)} class={s.tool} stroke={s.color} stroke-width={(s.width / 1000) * cam.w} />
            {/each}
          </svg>
        {/if}
        {#if spot}
          <div
            class="spotlight"
            style:background="radial-gradient(circle at {spot.x * 100}% {spot.y * 100}%, transparent {spot.r * cam.w}px, rgba(0, 0, 0, 0.72) {spot.r * cam.w + 2}px)"
          ></div>
        {/if}
        {#if laser}
          <span class="laser" style:left="{laser.x * 100}%" style:top="{laser.y * 100}%" style:--s={1 / cam.scale}></span>
        {/if}
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
  }

  .sheet {
    position: absolute;
    top: 0;
    left: 0;
    overflow: hidden;
  }

  .sheet.smooth {
    transition:
      transform 0.3s cubic-bezier(0.2, 0.8, 0.2, 1),
      width 0.3s cubic-bezier(0.2, 0.8, 0.2, 1),
      height 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
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
