<script lang="ts">
  import { LayoutGrid, LoaderCircle, MousePointerClick, RefreshCw, Trash2 } from "@lucide/svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import SlideView from "../components/SlideView.svelte";
  import { api } from "../lib/api";
  import { formatBytes } from "../lib/format";
  import { catalog } from "../lib/state/catalog.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import { toasts } from "../lib/state/toasts.svelte";
  import { updater } from "../lib/state/updater.svelte";
  import type { Lang, Slide, Theme, UiMode } from "../lib/types";

  const s = $derived(settings.current);

  let version = $state("");
  let usage = $state<number | null>(null);
  $effect(() => {
    void api.appVersion().then((v) => (version = v));
    void api.mediaUsage().then((u) => (usage = u));
  });

  const sample: Slide = {
    kind: "lyric",
    text: "Castelo forte é nosso Deus,\nespada e bom escudo",
    aux: null,
    image: null,
    imagePosition: 5,
    repeat: false,
    timeMs: null,
    instrumentalTimeMs: null,
  };

  const modes: { value: UiMode; title: string; detail: string; icon: typeof LayoutGrid }[] = [
    { value: "simple", title: "Simples", detail: "Botões grandes e só o essencial. Ideal para quem tem pouca prática com computador.", icon: MousePointerClick },
    { value: "full", title: "Completo", detail: "Biblioteca, painel de projeção ao lado e atalhos de teclado.", icon: LayoutGrid },
  ];
  const themes: [Theme, string][] = [["system", "Automático"], ["light", "Claro"], ["dark", "Escuro"]];
  const sizes: [number, string][] = [[1, "Normal"], [1.2, "Grande"], [1.4, "Muito grande"]];
  const transitions: [number, string][] = [[0, "Nenhuma"], [450, "Suave"], [900, "Lenta"]];
  const langs: [Lang, string][] = [["pt", "Português"], ["es", "Español"]];

  let checking = $state(false);
  async function checkCatalog() {
    checking = true;
    const before = catalog.status?.version;
    const ok = await catalog.sync(false);
    checking = false;
    if (!ok) toasts.error(catalog.error ?? "Não foi possível verificar.");
    else if (catalog.status?.version === before) toasts.show("Os hinos já estão na versão mais recente.", "success");
  }

  async function clearMedia() {
    await api.clearMedia();
    usage = await api.mediaUsage();
    toasts.show("Arquivos baixados apagados.", "success");
  }
</script>

<PageHeader title="Ajustes" />

<div class="settings">
  <section>
    <h2>Modo de uso</h2>
    <div class="modes">
      {#each modes as m (m.value)}
        <button class="mode card" class:selected={s.mode === m.value} aria-pressed={s.mode === m.value} onclick={() => settings.save({ mode: m.value })}>
          <m.icon size={26} />
          <span class="mode-title">{m.title}</span>
          <span class="muted">{m.detail}</span>
        </button>
      {/each}
    </div>
  </section>

  <section>
    <h2>Aparência</h2>
    <div class="group">
      <div class="row">
        <span class="grow">Tema</span>
        <div class="segmented">
          {#each themes as [value, label] (value)}
            <button aria-pressed={s.theme === value} onclick={() => settings.save({ theme: value })}>{label}</button>
          {/each}
        </div>
      </div>
      <div class="row">
        <span class="grow">Tamanho do texto e dos botões</span>
        <div class="segmented">
          {#each sizes as [value, label] (value)}
            <button aria-pressed={Math.abs(s.uiScale - value) < 0.01} onclick={() => settings.save({ uiScale: value })}>{label}</button>
          {/each}
        </div>
      </div>
    </div>
  </section>

  <section>
    <h2>Slides</h2>
    <div class="slides">
      <div class="preview"><SlideView slide={sample} style={s.slide} /></div>
      <div class="group">
        <div class="row">
          <span class="grow">Letras maiúsculas</span>
          <input type="checkbox" class="switch" checked={s.slide.uppercase} onchange={(e) => settings.saveSlide({ uppercase: e.currentTarget.checked })} />
        </div>
        <div class="row">
          <span class="grow">Imagens de fundo</span>
          <input type="checkbox" class="switch" checked={s.slide.showImages} onchange={(e) => settings.saveSlide({ showImages: e.currentTarget.checked })} />
        </div>
        <div class="row">
          <span class="grow">Tamanho da letra</span>
          <input
            type="range"
            min="0.6"
            max="1.6"
            step="0.05"
            value={s.slide.textScale}
            style:--progress="{((s.slide.textScale - 0.6) / 1) * 100}%"
            onchange={(e) => settings.saveSlide({ textScale: Number(e.currentTarget.value) })}
            aria-label="Tamanho da letra"
          />
        </div>
        <div class="row">
          <span class="grow">Transição</span>
          <div class="segmented">
            {#each transitions as [value, label] (value)}
              <button aria-pressed={s.slide.transitionMs === value} onclick={() => settings.saveSlide({ transitionMs: value })}>{label}</button>
            {/each}
          </div>
        </div>
      </div>
    </div>
  </section>

  <section>
    <h2>Hinos</h2>
    <div class="group">
      <div class="row">
        <span class="grow">Idioma dos hinos</span>
        <div class="segmented">
          {#each langs as [value, label] (value)}
            <button aria-pressed={s.language === value} onclick={() => settings.save({ language: value })}>{label}</button>
          {/each}
        </div>
      </div>
      <div class="row">
        <span class="grow">
          Acervo do LouvorJA
          <span class="muted block">
            {catalog.status?.songs.toLocaleString("pt-BR")} músicas · versão {catalog.status?.version ?? "—"}
          </span>
        </span>
        <button class="btn" onclick={checkCatalog} disabled={checking}>
          {#if checking}<LoaderCircle size={15} class="spin" /> Verificando…{:else}<RefreshCw size={15} /> Procurar novidades{/if}
        </button>
      </div>
      <div class="row">
        <span class="grow">Atualizar os hinos automaticamente</span>
        <input type="checkbox" class="switch" checked={s.autoUpdateCatalog} onchange={(e) => settings.save({ autoUpdateCatalog: e.currentTarget.checked })} />
      </div>
    </div>
  </section>

  <section>
    <h2>Sem internet</h2>
    <div class="group">
      <div class="row">
        <span class="grow">
          Guardar os hinos tocados
          <span class="muted block">Na próxima vez eles tocam mesmo sem internet.</span>
        </span>
        <input type="checkbox" class="switch" checked={s.keepPlayed} onchange={(e) => settings.save({ keepPlayed: e.currentTarget.checked })} />
      </div>
      <div class="row">
        <span class="grow">
          Arquivos baixados
          <span class="muted block">{usage === null ? "Calculando…" : formatBytes(usage)}</span>
        </span>
        <button class="btn" onclick={clearMedia} disabled={!usage}><Trash2 size={15} /> Apagar</button>
      </div>
    </div>
  </section>

  <section>
    <h2>Sobre</h2>
    <div class="group">
      <div class="row">
        <span class="grow">7Hinos <span class="muted">versão {version}</span></span>
        {#if updater.ready}
          <button class="btn btn-primary" onclick={() => updater.installAndRestart()} disabled={updater.installing}>
            Instalar versão {updater.ready.version}
          </button>
        {/if}
      </div>
      <div class="row">
        <span class="grow">Atualizar o aplicativo automaticamente</span>
        <input type="checkbox" class="switch" checked={s.autoUpdateApp} onchange={(e) => settings.save({ autoUpdateApp: e.currentTarget.checked })} />
      </div>
      <div class="row">
        <span class="grow muted small">Hinos, áudios e imagens: acervo público do LouvorJA (louvorja.com.br).</span>
      </div>
    </div>
  </section>
</div>

<style>
  .settings {
    display: flex;
    flex-direction: column;
    gap: 2rem;
    max-width: 52rem;
  }

  section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .modes {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }

  .mode {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    padding: 1.1rem;
    text-align: left;
    cursor: pointer;
    transition:
      border-color var(--fast),
      box-shadow var(--fast);
  }

  .mode.selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .mode.selected :global(svg) {
    color: var(--accent);
  }

  .mode-title {
    font-weight: 650;
    font-size: 1.1rem;
    margin-top: 0.3rem;
  }

  .slides {
    display: grid;
    grid-template-columns: minmax(12rem, 20rem) 1fr;
    gap: 1rem;
    align-items: start;
  }

  .preview {
    aspect-ratio: 16 / 9;
    border-radius: var(--radius);
    overflow: hidden;
    box-shadow: var(--shadow);
  }

  .block {
    display: block;
    font-size: 0.85rem;
  }

  .small {
    font-size: 0.85rem;
  }
</style>
