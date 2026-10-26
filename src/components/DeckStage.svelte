<!--
  Prévia da apresentação com as ferramentas do apresentador: zoom, foco,
  marca-texto, caneta, laser e borracha. O que se faz aqui aparece ao vivo no
  telão e no retorno (se a apresentação estiver no ar).
-->
<script lang="ts">
  import {
    Crosshair,
    Eraser,
    Focus,
    Highlighter,
    MousePointer2,
    PenLine,
    RotateCcw,
    Trash2,
    Undo2,
    ZoomIn,
    ZoomOut,
  } from "@lucide/svelte";
  import { addPoint, strokeAt, toPage, zoomAt } from "../lib/overlay";
  import { decks, type DeckTool } from "../lib/state/decks.svelte";
  import type { PageTransition } from "../lib/types";
  import PageView from "./PageView.svelte";

  let { src }: { src: string } = $props();

  let box: { el: HTMLElement; aspect: number } | null = null;
  let drawing: [number, number][] | null = null;

  const tools: [DeckTool, string, typeof ZoomIn, string][] = [
    ["none", "Normal", MousePointer2, "Só olhar (as setas passam as páginas)"],
    ["zoom", "Zoom", ZoomIn, "Clique para aproximar; Alt+clique afasta; a roda do mouse também"],
    ["spotlight", "Foco", Focus, "Clique onde quer destacar; o resto escurece. A roda muda o tamanho"],
    ["marker", "Marca-texto", Highlighter, "Arraste sobre o texto para marcar"],
    ["pen", "Caneta", PenLine, "Desenhe à mão livre"],
    ["laser", "Laser", Crosshair, "Um ponto vermelho segue o mouse no telão"],
    ["eraser", "Borracha", Eraser, "Toque num traço para apagar"],
  ];
  const penColors = ["#ff3b30", "#1e6fd9", "#1f9d55", "#ffffff", "#111111"];
  const markerColors = ["#ffe14d", "#7cf29a", "#ff8fd0", "#8fd3ff"];
  const transitions: [PageTransition, string][] = [
    ["none", "Nenhuma"],
    ["fade", "Suave"],
    ["slide", "Deslizar"],
    ["zoom", "Aproximar"],
  ];

  /** Ponto do evento na página (0 a 1), já descontando o zoom. */
  function point(e: PointerEvent | WheelEvent): { px: number; py: number; x: number; y: number } | null {
    if (!box) return null;
    const r = box.el.getBoundingClientRect();
    if (!r.width || !r.height) return null;
    const px = (e.clientX - r.left) / r.width;
    const py = (e.clientY - r.top) / r.height;
    const [x, y] = toPage(px, py, decks.zoom);
    return { px, py, x, y };
  }

  const inside = (x: number, y: number) => x >= 0 && x <= 1 && y >= 0 && y <= 1;

  function down(e: PointerEvent) {
    const p = point(e);
    if (!p || e.button !== 0) return;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    switch (decks.tool) {
      case "zoom":
        decks.setZoom(zoomAt(decks.zoom, p.px, p.py, e.altKey || e.shiftKey ? 0.5 : 2));
        break;
      case "spotlight":
        if (inside(p.x, p.y)) decks.setSpotlight({ x: p.x, y: p.y, r: decks.spotlight?.r ?? 0.13 });
        break;
      case "marker":
      case "pen": {
        if (!inside(p.x, p.y)) return;
        drawing = [[p.x, p.y]];
        const marker = decks.tool === "marker";
        decks.beginStroke({
          tool: marker ? "marker" : "pen",
          color: marker ? decks.markerColor : decks.penColor,
          width: marker ? 22 : 4,
          points: drawing,
        });
        break;
      }
      case "eraser":
        erase(p.x, p.y);
        break;
    }
  }

  function move(e: PointerEvent) {
    const p = point(e);
    if (!p) return;
    if (decks.tool === "laser") return decks.setLaser(inside(p.x, p.y) ? { x: p.x, y: p.y } : null);
    if (!e.buttons) return;
    if (drawing) {
      const next = addPoint(drawing, Math.min(1, Math.max(0, p.x)), Math.min(1, Math.max(0, p.y)));
      if (next !== drawing) {
        drawing = next;
        decks.extendStroke(drawing);
      }
    } else if (decks.tool === "spotlight" && inside(p.x, p.y)) {
      decks.setSpotlight({ x: p.x, y: p.y, r: decks.spotlight?.r ?? 0.13 });
    } else if (decks.tool === "eraser") {
      erase(p.x, p.y);
    }
  }

  function up() {
    drawing = null;
  }

  function erase(x: number, y: number) {
    const i = strokeAt(decks.marks[decks.index] ?? [], x, y, 1 / (box?.aspect ?? 16 / 9));
    if (i >= 0) decks.eraseStroke(i);
  }

  function wheel(e: WheelEvent) {
    const p = point(e);
    if (!p) return;
    if (decks.tool === "spotlight" && decks.spotlight) {
      e.preventDefault();
      const r = Math.min(0.5, Math.max(0.04, decks.spotlight.r * (e.deltaY < 0 ? 1.1 : 0.9)));
      decks.setSpotlight({ ...decks.spotlight, r });
    } else if (decks.tool === "zoom" || e.ctrlKey) {
      e.preventDefault();
      decks.setZoom(zoomAt(decks.zoom, p.px, p.py, e.deltaY < 0 ? 1.2 : 1 / 1.2));
    }
  }

  function pick(tool: DeckTool) {
    decks.tool = decks.tool === tool ? "none" : tool;
    if (decks.tool !== "laser") decks.setLaser(null);
  }

  const zoomPercent = $derived(Math.round((decks.zoom?.scale ?? 1) * 100));
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key !== "Escape" || (e.target as HTMLElement)?.closest("input, textarea, select")) return;
    // Esc solta a ferramenta; de novo, tira zoom e foco.
    if (decks.tool !== "none") pick("none");
    else if (decks.zoom || decks.spotlight) (decks.setZoom(null), decks.setSpotlight(null));
  }}
/>

<div class="stage-wrap">
  <div class="stage-preview" class:dim={decks.live && decks.blackout}>
    <PageView {src} overlay={decks.overlay} animate={false} onbox={(b) => (box = b)} />
    <div
      class="capture tool-{decks.tool}"
      role="presentation"
      onpointerdown={down}
      onpointermove={move}
      onpointerup={up}
      onpointercancel={up}
      onpointerleave={() => decks.tool === "laser" && decks.setLaser(null)}
      onwheel={wheel}
    ></div>
    {#if decks.live}<span class="pill live on-air">{decks.blackout ? "Tela preta" : "No telão"}</span>{/if}
  </div>

  <div class="toolbar">
    <div class="tools" role="toolbar" aria-label="Ferramentas do apresentador">
      {#each tools as [id, label, Icon, hint] (id)}
        <button class="tool" class:on={decks.tool === id} aria-pressed={decks.tool === id} title="{label}: {hint}" onclick={() => pick(id)}>
          <Icon size={17} />
          <span>{label}</span>
        </button>
      {/each}
    </div>

    <div class="options">
      {#if decks.tool === "pen" || decks.tool === "marker"}
        {@const colors = decks.tool === "pen" ? penColors : markerColors}
        {@const current = decks.tool === "pen" ? decks.penColor : decks.markerColor}
        <div class="colors" role="group" aria-label="Cor">
          {#each colors as c (c)}
            <button
              class="dot"
              class:on={current === c}
              style:background={c}
              aria-label="Cor {c}"
              onclick={() => (decks.tool === "pen" ? (decks.penColor = c) : (decks.markerColor = c))}
            ></button>
          {/each}
        </div>
      {/if}
      {#if decks.tool === "zoom" || decks.zoom}
        <div class="zoom">
          <button class="btn btn-plain btn-icon" aria-label="Afastar" title="Afastar" disabled={!decks.zoom} onclick={() => decks.setZoom(zoomAt(decks.zoom, 0.5, 0.5, 1 / 1.5))}><ZoomOut size={16} /></button>
          <span class="pct">{zoomPercent}%</span>
          <button class="btn btn-plain btn-icon" aria-label="Aproximar" title="Aproximar" onclick={() => decks.setZoom(zoomAt(decks.zoom, 0.5, 0.5, 1.5))}><ZoomIn size={16} /></button>
          {#if decks.zoom}
            <button class="btn btn-plain" onclick={() => decks.setZoom(null)}><RotateCcw size={14} /> Página inteira</button>
          {/if}
        </div>
      {/if}
      {#if decks.spotlight}
        <button class="btn btn-plain" onclick={() => decks.setSpotlight(null)}>Tirar o foco</button>
      {/if}
      <span class="grow"></span>
      <button class="btn btn-plain btn-icon" aria-label="Desfazer" title="Desfazer o último traço" disabled={!(decks.marks[decks.index]?.length)} onclick={() => decks.undo()}><Undo2 size={16} /></button>
      <button class="btn btn-plain" disabled={!decks.hasMarks} title="Apaga desenhos, foco e zoom desta página" onclick={() => decks.clearPage()}><Trash2 size={15} /> Limpar</button>
      <label class="transition">
        <span class="muted">Transição</span>
        <select class="input" value={decks.transition} onchange={(e) => decks.setTransition(e.currentTarget.value as PageTransition)}>
          {#each transitions as [value, label] (value)}<option {value}>{label}</option>{/each}
        </select>
      </label>
    </div>
  </div>
</div>

<style>
  .stage-wrap {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .stage-preview {
    position: relative;
    aspect-ratio: 16 / 9;
    max-height: 52vh;
    border-radius: var(--radius-l);
    overflow: hidden;
    background: #000;
  }

  .stage-preview.dim :global(.page-view) {
    opacity: 0.25;
  }

  .capture {
    position: absolute;
    inset: 0;
    touch-action: none;
  }

  .tool-zoom {
    cursor: zoom-in;
  }

  .tool-spotlight,
  .tool-laser {
    cursor: crosshair;
  }

  .tool-pen,
  .tool-marker {
    cursor: crosshair;
  }

  .tool-eraser {
    cursor: cell;
  }

  .on-air {
    position: absolute;
    top: 0.6rem;
    left: 0.6rem;
    pointer-events: none;
  }

  .toolbar {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .tools {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    padding: 0.25rem;
    border-radius: var(--radius-l);
    background: var(--surface-2);
    align-self: center;
  }

  .tool {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.4rem 0.65rem;
    border: 0;
    border-radius: var(--radius);
    background: none;
    color: var(--text-2);
    font: inherit;
    font-size: 0.86rem;
    font-weight: 550;
    cursor: pointer;
  }

  .tool:hover {
    color: var(--text);
    background: var(--hover);
  }

  .tool.on {
    color: var(--accent);
    background: var(--surface);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.12);
  }

  .options {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
    min-height: 2.4rem;
  }

  .grow {
    flex: 1;
  }

  .colors {
    display: flex;
    gap: 0.35rem;
  }

  .dot {
    width: 1.6rem;
    height: 1.6rem;
    padding: 0;
    border-radius: 999px;
    border: 1px solid var(--border-strong);
    cursor: pointer;
  }

  .dot.on {
    box-shadow:
      0 0 0 2px var(--bg),
      0 0 0 4px var(--accent);
  }

  .zoom {
    display: flex;
    align-items: center;
    gap: 0.2rem;
  }

  .pct {
    min-width: 3.2rem;
    text-align: center;
    font-variant-numeric: tabular-nums;
    color: var(--text-2);
  }

  .transition {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.88rem;
  }

  .transition select {
    width: auto;
    padding-block: 0.3rem;
  }
</style>
