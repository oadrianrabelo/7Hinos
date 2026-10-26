<!--
  Fundo do telão: degradês circulares, cores lisas e um personalizado (liso
  ou degradê, com o centro e a borda escolhidos no seletor).
-->
<script lang="ts">
  import {
    backgroundCss,
    isHex,
    RADIAL_BACKGROUNDS,
    radialStops,
    radialValue,
    SOLID_BACKGROUNDS,
  } from "../lib/colors";
  import HsvEditor from "./HsvEditor.svelte";
  import Popover from "./Popover.svelte";

  let { value, active = true, onchange }: { value: string; active?: boolean; onchange: (bg: string) => void } = $props();

  const normalized = $derived(value === "gradient" ? RADIAL_BACKGROUNDS[0].value : value);
  const presets = [...RADIAL_BACKGROUNDS, ...SOLID_BACKGROUNDS];
  const custom = $derived(!presets.some((p) => p.value === normalized));

  let open = $state(false);
  let trigger = $state<HTMLButtonElement>();
  const stops = $derived(radialStops(normalized));
  const solid = $derived(isHex(normalized) ? normalized : null);
  let editing = $state<"center" | "edge">("center");
  const editingColor = $derived(stops ? (editing === "center" ? stops[0] : stops[1]) : (solid ?? "#000000"));

  let pending: string | null = null;
  let timer: ReturnType<typeof setTimeout> | null = null;
  function save(bg: string) {
    pending = bg;
    if (timer) return;
    timer = setTimeout(() => {
      timer = null;
      if (pending) onchange(pending);
    }, 120);
  }

  function edit(hex: string) {
    if (!stops) return save(hex);
    save(editing === "center" ? radialValue(hex, stops[1]) : radialValue(stops[0], hex));
  }
</script>

<div class="group-block">
  <span class="sub">Degradê</span>
  <div class="swatches">
    {#each RADIAL_BACKGROUNDS as s (s.value)}
      <button class="swatch" class:on={active && normalized === s.value} style:background={backgroundCss(s.value)} title={s.label} aria-label={s.label} onclick={() => onchange(s.value)}></button>
    {/each}
  </div>
  <span class="sub">Cor lisa</span>
  <div class="swatches">
    {#each SOLID_BACKGROUNDS as s (s.value)}
      <button class="swatch" class:on={active && normalized === s.value} style:background={s.value} title={s.label} aria-label={s.label} onclick={() => onchange(s.value)}></button>
    {/each}
    <button
      class="swatch any"
      class:on={active && custom}
      bind:this={trigger}
      style:--custom={custom ? backgroundCss(normalized) : "transparent"}
      title="Personalizar…"
      aria-label="Personalizar o fundo"
      aria-expanded={open}
      onclick={() => (open = !open)}
    >
    </button>
  </div>
</div>

<Popover bind:open anchor={trigger} width={17}>
  <div class="custom">
    <div class="segmented">
      <button aria-pressed={!!stops} onclick={() => !stops && onchange(radialValue(solid ?? "#1d4560", "#050b11"))}>Degradê</button>
      <button aria-pressed={!stops} onclick={() => stops && onchange(stops[0])}>Lisa</button>
    </div>
    {#if stops}
      <div class="stops">
        <button class="stop" class:on={editing === "center"} onclick={() => (editing = "center")}>
          <span class="dot" style:background={stops[0]}></span> Centro
        </button>
        <button class="stop" class:on={editing === "edge"} onclick={() => (editing = "edge")}>
          <span class="dot" style:background={stops[1]}></span> Bordas
        </button>
      </div>
    {/if}
    {#key stops ? editing : "solid"}
      <HsvEditor value={editingColor} oninput={edit} />
    {/key}
    <div class="sample" style:background={backgroundCss(normalized)}></div>
  </div>
</Popover>

<style>
  .group-block {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .sub {
    font-size: 0.75rem;
    color: var(--text-2);
  }

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

  .any {
    color: #fff;
    background:
      radial-gradient(circle, var(--custom) 0 48%, transparent 50%),
      conic-gradient(#f44, #fb3, #ee4, #4d6, #3cf, #66f, #c5f, #f44);
  }

  .custom {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .custom .segmented {
    align-self: flex-start;
  }

  .stops {
    display: flex;
    gap: 0.4rem;
  }

  .stop {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.35rem 0.5rem;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    color: var(--text);
    font: inherit;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .stop.on {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-soft);
  }

  .dot {
    width: 1rem;
    height: 1rem;
    border-radius: 999px;
    border: 1px solid var(--border-strong);
  }

  .sample {
    aspect-ratio: 16 / 9;
    border-radius: var(--radius);
  }
</style>
