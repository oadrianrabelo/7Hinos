<!--
  Um slide como aparece no telão. Usado na projeção e nas prévias: o tamanho
  do texto é relativo ao próprio quadro (container queries), então a prévia
  pequena é uma miniatura fiel da tela grande.
-->
<script lang="ts">
  import { fade } from "svelte/transition";
  import { mediaSrc } from "../lib/media";
  import { backgroundPosition, fitText } from "../lib/slides";
  import type { Slide, SlideStyle } from "../lib/types";

  let {
    slide,
    style,
    animate = true,
  }: { slide: Slide | null; style: SlideStyle; animate?: boolean } = $props();

  const key = $derived(
    slide ? [slide.kind, slide.text, slide.aux, slide.image?.url, slide.repeat].join("|") : "black",
  );
  const image = $derived(style.showImages ? mediaSrc(slide?.image) : null);
  const size = $derived(slide ? fitText(slide.text, slide.kind, style.textScale) : null);
  const duration = $derived(animate ? style.transitionMs : 0);
</script>

<div class="frame">
  {#key key}
    <div class="layer" transition:fade={{ duration }}>
      {#if slide}
        {#if image}
          <div
            class="bg"
            style:background-image={`url("${image}")`}
            style:background-position={backgroundPosition(slide.imagePosition)}
          ></div>
        {/if}
        <div class="shade" class:plain={!image}></div>
        <div class="content" class:upper={style.uppercase}>
          {#if slide.aux}
            <p class="aux">{slide.aux}</p>
          {/if}
          {#if slide.text}
            <p
              class="text"
              class:title={slide.kind === "title"}
              class:repeat={slide.repeat}
              style:font-size={`min(${size?.cqw}cqw, ${size?.cqh}cqh)`}
            >
              {slide.text}
            </p>
          {/if}
        </div>
      {/if}
    </div>
  {/key}
</div>

<style>
  .frame {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: #000;
    container-type: size;
    isolation: isolate;
  }

  .layer {
    position: absolute;
    inset: 0;
  }

  .bg {
    position: absolute;
    inset: -1px;
    background-size: cover;
    background-repeat: no-repeat;
  }

  /* Escurece o fundo o suficiente para ler, sem as tarjas pretas de antes. */
  .shade {
    position: absolute;
    inset: 0;
    background:
      radial-gradient(ellipse at center, rgba(0, 0, 0, 0.35) 0%, rgba(0, 0, 0, 0.62) 100%),
      linear-gradient(rgba(0, 0, 0, 0.1), rgba(0, 0, 0, 0.25));
  }

  .shade.plain {
    background: radial-gradient(ellipse at 50% 35%, #26264a 0%, #0d0d1a 70%, #050509 100%);
  }

  .content {
    position: absolute;
    inset: 6cqh 5cqw;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2.2cqh;
    text-align: center;
    font-family: var(--font-slide);
    color: #fff;
    text-shadow:
      0 0.04em 0.1em rgba(0, 0, 0, 0.55),
      0 0 0.6em rgba(0, 0, 0, 0.35);
  }

  .upper {
    text-transform: uppercase;
  }

  .text {
    font-weight: 600;
    line-height: 1.12;
    letter-spacing: 0.01em;
    white-space: pre-line;
    text-wrap: balance;
  }

  .text.title {
    font-weight: 700;
    color: #ffd36b;
    letter-spacing: 0.02em;
  }

  /* Refrão repetido em dourado, como o LouvorJA faz. */
  .text.repeat {
    color: #ffe39a;
  }

  .aux {
    font-size: min(4.6cqh, 5cqw);
    font-weight: 600;
    letter-spacing: 0.08em;
    color: #ffc94a;
    opacity: 0.95;
  }
</style>
