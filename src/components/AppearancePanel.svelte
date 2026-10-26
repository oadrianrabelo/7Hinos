<!-- Aparência dos avisos, da tela de espera e do relógio no telão, com prévia. -->
<script lang="ts">
  import { ImagePlus, Palette, X } from "@lucide/svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { api, errorMessage } from "../lib/api";
  import { settings } from "../lib/state/settings.svelte";
  import { toasts } from "../lib/state/toasts.svelte";
  import type { Scene, ScreenStyle } from "../lib/types";
  import { ACCENT_COLORS, TEXT_COLORS } from "../lib/colors";
  import { SCREEN_DEFAULTS } from "../lib/defaults";
  import BackgroundPicker from "./BackgroundPicker.svelte";
  import ResetButton from "./ResetButton.svelte";
  import ColorPicker from "./ColorPicker.svelte";
  import FontPicker from "./FontPicker.svelte";
  import SceneView from "./SceneView.svelte";

  const s = $derived(settings.current);
  const st = $derived(s.screenStyle);
  const save = (patch: Partial<ScreenStyle>) => settings.save({ screenStyle: patch });

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

</script>

<section class="card panel">
  <header class="line">
    <h2 class="grow"><Palette size={18} /> Aparência no telão</h2>
    <ResetButton current={st} defaults={SCREEN_DEFAULTS} onreset={() => save({ ...SCREEN_DEFAULTS })} />
  </header>
  <p class="muted">Vale para os avisos, a tela de espera, o relógio e a contagem. Os hinos seguem o estilo dos slides.</p>

  <div class="layout">
    <div class="preview">
      <SceneView scene={sample} style={s.slide} idle={s.idle} screen={st} />
    </div>

    <div class="options">
      <div class="option">
        <span class="section-title">Letra</span>
        <FontPicker value={st.font} onchange={(font) => save({ font })} sample="Santa Ceia neste sábado" />
      </div>

      <div class="option">
        <span class="section-title">Cor do texto</span>
        <ColorPicker label="Cor do texto" value={st.textColor} swatches={TEXT_COLORS} onchange={(textColor) => save({ textColor })} />
      </div>

      <div class="option">
        <span class="section-title">Cor dos títulos</span>
        <ColorPicker label="Cor dos títulos" value={st.accentColor} swatches={ACCENT_COLORS} onchange={(accentColor) => save({ accentColor })} />
      </div>

      <div class="option">
        <span class="section-title">Fundo</span>
        <BackgroundPicker value={st.background} active={!st.backgroundImage} onchange={(background) => save({ background, backgroundImage: null })} />
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


  @media (max-width: 760px) {
    .layout {
      grid-template-columns: 1fr;
    }
  }
</style>
