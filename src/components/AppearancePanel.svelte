<!-- Aparência dos avisos, da tela de espera e do relógio no telão, com prévia. -->
<script lang="ts">
  import { ImagePlus, Palette, RotateCcw, X } from "@lucide/svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { api, errorMessage } from "../lib/api";
  import { settings } from "../lib/state/settings.svelte";
  import { toasts } from "../lib/state/toasts.svelte";
  import type { Scene, ScreenFont, ScreenStyle } from "../lib/types";
  import SceneView from "./SceneView.svelte";

  const s = $derived(settings.current);
  const st = $derived(s.screenStyle);
  const save = (patch: Partial<ScreenStyle>) => settings.save({ screenStyle: patch });

  const fonts: [ScreenFont, string, string][] = [
    ["condensed", "Condensada", "var(--font-slide)"],
    ["sans", "Moderna", "var(--font-ui)"],
    ["serif", "Clássica", 'Georgia, "Times New Roman", serif'],
  ];
  const textColors = [
    ["#ffffff", "Branco"],
    ["#f1e4c6", "Creme"],
    ["#ffe9a8", "Amarelo claro"],
  ];
  const accents = [
    ["#ffc94a", "Dourado"],
    ["#ffffff", "Branco"],
    ["#8fd0f2", "Azul claro"],
    ["#ff8a80", "Vermelho claro"],
  ];
  const backgrounds = [
    ["gradient", "Azul do 7Hinos", "radial-gradient(ellipse at 50% 35%, #1d4560 0%, #0b1d2b 80%)"],
    ["#000000", "Preto", "#000000"],
    ["#173b53", "Azul-marinho", "#173b53"],
    ["#4a1c24", "Vinho", "#4a1c24"],
    ["#1f3d2b", "Verde-escuro", "#1f3d2b"],
  ];

  const sample: Scene = {
    kind: "slide",
    title: "Aviso",
    slide: {
      kind: "text",
      text: "Neste sábado, às 10h30.\nTraga um amigo!",
      aux: "Santa Ceia",
      image: null,
      imagePosition: 5,
      repeat: false,
      timeMs: null,
      instrumentalTimeMs: null,
    },
  };

  async function pickImage() {
    const picked = await open({
      title: "Escolha a imagem de fundo dos avisos",
      filters: [{ name: "Imagem", extensions: ["jpg", "jpeg", "png", "webp", "bmp"] }],
    });
    if (typeof picked !== "string") return;
    try {
      await save({ backgroundImage: await api.addUserMedia(picked) });
    } catch (e) {
      toasts.error(errorMessage(e));
    }
  }

  const isDefault = $derived(
    st.font === "condensed" &&
      st.textColor === "#ffffff" &&
      st.accentColor === "#ffc94a" &&
      st.background === "gradient" &&
      !st.backgroundImage,
  );
</script>

<section class="card panel">
  <header class="line">
    <h2 class="grow"><Palette size={18} /> Aparência no telão</h2>
    {#if !isDefault}
      <button
        class="btn btn-plain"
        onclick={() =>
          save({ font: "condensed", textColor: "#ffffff", accentColor: "#ffc94a", background: "gradient", backgroundImage: null })}
      >
        <RotateCcw size={14} /> Voltar ao padrão
      </button>
    {/if}
  </header>
  <p class="muted">Vale para os avisos, a tela de espera, o relógio e a contagem. Os hinos seguem o estilo dos slides.</p>

  <div class="layout">
    <div class="preview">
      <SceneView scene={sample} style={s.slide} idle={s.idle} screen={st} />
    </div>

    <div class="options">
      <div class="option">
        <span class="section-title">Letra</span>
        <div class="segmented">
          {#each fonts as [value, label, family] (value)}
            <button aria-pressed={st.font === value} style:font-family={family} onclick={() => save({ font: value })}>{label}</button>
          {/each}
        </div>
      </div>

      <div class="option">
        <span class="section-title">Cor do texto</span>
        <div class="swatches">
          {#each textColors as [value, label] (value)}
            <button class="swatch" class:on={st.textColor === value} style:background={value} title={label} aria-label={label} onclick={() => save({ textColor: value })}></button>
          {/each}
        </div>
      </div>

      <div class="option">
        <span class="section-title">Cor dos títulos</span>
        <div class="swatches">
          {#each accents as [value, label] (value)}
            <button class="swatch" class:on={st.accentColor === value} style:background={value} title={label} aria-label={label} onclick={() => save({ accentColor: value })}></button>
          {/each}
        </div>
      </div>

      <div class="option">
        <span class="section-title">Fundo</span>
        <div class="swatches">
          {#each backgrounds as [value, label, paint] (value)}
            <button
              class="swatch"
              class:on={!st.backgroundImage && st.background === value}
              style:background={paint}
              title={label}
              aria-label={label}
              onclick={() => save({ background: value, backgroundImage: null })}
            ></button>
          {/each}
        </div>
        <div class="line">
          <button class="btn" onclick={pickImage}><ImagePlus size={15} /> {st.backgroundImage ? "Trocar imagem" : "Usar uma imagem"}</button>
          {#if st.backgroundImage}
            <button class="btn btn-plain" onclick={() => save({ backgroundImage: null })}><X size={14} /> Tirar imagem</button>
          {/if}
        </div>
      </div>
    </div>
  </div>
</section>

<style>
  .panel {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 1rem 1.1rem;
  }

  h2 {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    margin: 0;
    font-size: 1.05rem;
  }

  .line {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .grow {
    flex: 1;
  }

  .layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 1rem;
    align-items: start;
  }

  .preview {
    aspect-ratio: 16 / 9;
    border-radius: var(--radius);
    overflow: hidden;
  }

  .options {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }

  .option {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .swatches {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
  }

  .swatch {
    width: 2rem;
    height: 2rem;
    border-radius: 999px;
    border: 1px solid var(--border-strong);
    cursor: pointer;
  }

  .swatch.on {
    box-shadow:
      0 0 0 2px var(--bg),
      0 0 0 4px var(--accent);
  }

  @media (max-width: 760px) {
    .layout {
      grid-template-columns: 1fr;
    }
  }
</style>
