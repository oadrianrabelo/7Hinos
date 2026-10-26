<!--
  Editor de cor: quadro de saturação/brilho, faixa de matiz e o código
  (#rrggbb). Avisa a cada mudança; quem usa decide quando salvar.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { hexToHsv, hsvToHex, normalizeHex } from "../lib/colors";

  let { value, oninput }: { value: string; oninput: (hex: string) => void } = $props();

  // O HSV é guardado aqui: converter de volta do hex perderia o matiz no
  // preto e no branco (o ponteiro pularia).
  let hsv = $state(untrack(() => hexToHsv(value)));
  let typed = $state(untrack(() => value));
  let last = untrack(() => value);
  let editedAt = 0;

  // Uma cor vinda de fora (outra bolinha) reposiciona o editor; as que
  // voltam do próprio arraste, atrasadas pelo salvamento, são ignoradas.
  $effect(() => {
    if (value !== last && Date.now() - editedAt > 500) {
      last = value;
      hsv = hexToHsv(value);
      typed = value;
    }
  });

  const current = $derived(hsvToHex(hsv[0], hsv[1], hsv[2]));
  const pure = $derived(hsvToHex(hsv[0], 1, 1));

  function emit() {
    editedAt = Date.now();
    last = current;
    typed = current;
    oninput(current);
  }

  let area = $state<HTMLElement>();
  function drag(e: PointerEvent) {
    if (!area) return;
    const r = area.getBoundingClientRect();
    const s = Math.min(1, Math.max(0, (e.clientX - r.left) / r.width));
    const v = 1 - Math.min(1, Math.max(0, (e.clientY - r.top) / r.height));
    hsv = [hsv[0], s, v];
    emit();
  }

  function startDrag(e: PointerEvent) {
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    drag(e);
  }

  function nudge(e: KeyboardEvent) {
    const step = e.shiftKey ? 0.1 : 0.02;
    const [h, s, v] = hsv;
    const moves: Record<string, [number, number]> = {
      ArrowLeft: [-step, 0],
      ArrowRight: [step, 0],
      ArrowUp: [0, step],
      ArrowDown: [0, -step],
    };
    const move = moves[e.key];
    if (!move) return;
    e.preventDefault();
    hsv = [h, Math.min(1, Math.max(0, s + move[0])), Math.min(1, Math.max(0, v + move[1]))];
    emit();
  }

  function commitTyped() {
    const hex = normalizeHex(typed);
    if (!hex) return void (typed = current);
    hsv = hexToHsv(hex);
    emit();
  }
</script>

<div class="editor">
  <div
    class="area"
    bind:this={area}
    style:background-color={pure}
    role="slider"
    tabindex="0"
    aria-label="Saturação e brilho"
    aria-valuenow={Math.round(hsv[2] * 100)}
    onpointerdown={startDrag}
    onpointermove={(e) => e.buttons && drag(e)}
    onkeydown={nudge}
  >
    <span class="thumb" style:left="{hsv[1] * 100}%" style:top="{(1 - hsv[2]) * 100}%" style:background={current}></span>
  </div>
  <input
    class="hue"
    type="range"
    min="0"
    max="359"
    step="1"
    value={hsv[0]}
    aria-label="Matiz"
    oninput={(e) => {
      hsv = [Number(e.currentTarget.value), hsv[1], hsv[2]];
      emit();
    }}
  />
  <div class="code">
    <span class="preview" style:background={current}></span>
    <input
      class="input"
      bind:value={typed}
      spellcheck="false"
      aria-label="Código da cor"
      onkeydown={(e) => e.key === "Enter" && commitTyped()}
      onblur={commitTyped}
    />
  </div>
</div>

<style>
  .editor {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .area {
    position: relative;
    height: 9.5rem;
    border-radius: var(--radius);
    background-image: linear-gradient(to top, #000, transparent), linear-gradient(to right, #fff, transparent);
    cursor: crosshair;
    touch-action: none;
  }

  .area:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .thumb {
    position: absolute;
    width: 1rem;
    height: 1rem;
    margin: -0.5rem 0 0 -0.5rem;
    border: 2px solid #fff;
    border-radius: 999px;
    box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.35), 0 1px 4px rgba(0, 0, 0, 0.4);
    pointer-events: none;
  }

  .hue {
    appearance: none;
    width: 100%;
    height: 0.8rem;
    margin: 0;
    border-radius: 999px;
    background: linear-gradient(to right, #f00, #ff0, #0f0, #0ff, #00f, #f0f, #f00);
    cursor: pointer;
  }

  .hue::-webkit-slider-thumb {
    appearance: none;
    width: 1.1rem;
    height: 1.1rem;
    border: 2px solid #fff;
    border-radius: 999px;
    background: transparent;
    box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.35), 0 1px 4px rgba(0, 0, 0, 0.4);
  }

  .hue::-moz-range-thumb {
    width: 1rem;
    height: 1rem;
    border: 2px solid #fff;
    border-radius: 999px;
    background: transparent;
  }

  .code {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .preview {
    flex: none;
    width: 2rem;
    height: 2rem;
    border-radius: var(--radius-s);
    border: 1px solid var(--border-strong);
  }

  .code .input {
    flex: 1;
    min-width: 0;
    font-family: ui-monospace, "SF Mono", Consolas, monospace;
    text-transform: lowercase;
  }
</style>
