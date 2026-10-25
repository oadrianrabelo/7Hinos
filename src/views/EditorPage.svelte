<!--
  Editor de letras: escreva cada estrofe separada por uma linha em branco e
  veja os slides ao lado, já com a aparência do telão.
-->
<script lang="ts">
  import { ask, open } from "@tauri-apps/plugin-dialog";
  import { Image, LoaderCircle, Music, RotateCcw, Save, X } from "@lucide/svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import SlideView from "../components/SlideView.svelte";
  import { api, errorMessage } from "../lib/api";
  import { lyricsToSlides, slidesToLyrics } from "../lib/lyrics";
  import { nav } from "../lib/state/nav.svelte";
  import { player } from "../lib/state/player.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import { toasts } from "../lib/state/toasts.svelte";
  import type { Slide, UserSong } from "../lib/types";

  let { id, basedOn }: { id?: number; basedOn?: number } = $props();

  let draft = $state<UserSong | null>(null);
  let lyrics = $state("");
  let saving = $state(false);
  let existing = $state(false);

  $effect(() => {
    void load(id, basedOn);
  });

  async function load(id?: number, basedOn?: number) {
    const empty: UserSong = { id: 0, title: "", author: "", slides: [], basedOn: basedOn ?? null, audio: null, background: null, updatedAt: 0 };
    try {
      const saved = await api.userSong(id ?? basedOn ?? -1);
      if (saved) {
        draft = saved;
        existing = true;
        lyrics = slidesToLyrics(saved.slides);
        return;
      }
      if (basedOn) {
        // Primeira correção: parte da letra do acervo.
        const song = await api.song(basedOn);
        draft = { ...empty, title: song.title };
        lyrics = slidesToLyrics(song.slides.filter((s) => s.kind === "lyric"));
        return;
      }
    } catch (e) {
      toasts.error(errorMessage(e));
    }
    draft = empty;
  }

  const preview = $derived<Slide[]>(
    draft
      ? [
          { kind: "title", text: draft.title || "Título", aux: draft.author || null, image: null, imagePosition: 5, repeat: false, timeMs: null, instrumentalTimeMs: null },
          ...lyricsToSlides(lyrics).map((s, i, all) => ({
            kind: "lyric" as const,
            text: s.text,
            aux: null,
            image: null,
            imagePosition: 5,
            repeat: i > 0 && all[i - 1].text === s.text,
            timeMs: null,
            instrumentalTimeMs: null,
          })),
        ]
      : [],
  );

  async function pick(kind: "audio" | "background") {
    const picked = await open({
      title: kind === "audio" ? "Escolha a música (áudio)" : "Escolha a imagem de fundo",
      filters: [
        kind === "audio"
          ? { name: "Áudio", extensions: ["mp3", "m4a", "ogg", "opus", "wav", "aac"] }
          : { name: "Imagem", extensions: ["jpg", "jpeg", "png", "webp", "bmp"] },
      ],
    });
    if (typeof picked !== "string" || !draft) return;
    try {
      draft[kind] = await api.addUserMedia(picked);
    } catch (e) {
      toasts.error(errorMessage(e));
    }
  }

  async function save() {
    if (!draft) return;
    saving = true;
    try {
      const saved = await api.saveUserSong({ ...$state.snapshot(draft), slides: lyricsToSlides(lyrics) });
      toasts.show(draft.basedOn ? "Letra corrigida." : `“${saved.title}” salva.`, "success");
      await player.refresh(saved.basedOn ?? saved.id);
      nav.back();
    } catch (e) {
      toasts.error(errorMessage(e));
    } finally {
      saving = false;
    }
  }

  async function discard() {
    if (!draft || !existing) return;
    const what = draft.basedOn ? "Voltar à letra original do acervo?" : `Apagar “${draft.title}”?`;
    if (!(await ask(what, { title: "Confirmar", kind: "warning" }))) return;
    await api.deleteUserSong(draft.id);
    await player.refresh(draft.basedOn ?? draft.id);
    nav.back();
  }
</script>

{#if draft}
  <PageHeader
    title={draft.basedOn ? "Corrigir letra" : existing ? "Editar música" : "Nova música"}
    subtitle={draft.basedOn ? "A correção vale só neste computador (e nos pacotes de pendrive)." : "Cada estrofe separada por uma linha em branco vira um slide."}
  >
    {#snippet actions()}
      {#if existing}
        <button class="btn" onclick={discard}>
          {#if draft?.basedOn}<RotateCcw size={15} /> Voltar ao original{:else}<X size={15} /> Apagar{/if}
        </button>
      {/if}
      <button class="btn btn-primary" onclick={save} disabled={saving}>
        {#if saving}<LoaderCircle size={15} class="spin" />{:else}<Save size={15} />{/if} Salvar
      </button>
    {/snippet}
  </PageHeader>

  <div class="editor">
    <div class="form">
      <label class="label"><span>Título</span>
        <input class="input" bind:value={draft.title} placeholder="Nome da música" />
      </label>
      {#if !draft.basedOn}
        <label class="label"><span>Autor ou origem <span class="muted">(opcional)</span></span>
          <input class="input" bind:value={draft.author} placeholder="Ex.: Ministério de louvor" />
        </label>
      {/if}
      <label class="label grow"><span>Letra</span>
        <textarea class="input lyrics" bind:value={lyrics} placeholder={"Primeira estrofe\nsegunda linha\n\nRefrão\n..."} spellcheck="true"></textarea>
      </label>
      {#if !draft.basedOn}
        <div class="files">
          <button class="btn" onclick={() => pick("audio")}><Music size={15} /> {draft.audio ? draft.audio : "Escolher áudio"}</button>
          {#if draft.audio}<button class="btn btn-plain btn-icon" aria-label="Tirar áudio" onclick={() => draft && (draft.audio = null)}><X size={14} /></button>{/if}
          <button class="btn" onclick={() => pick("background")}><Image size={15} /> {draft.background ? draft.background : "Escolher fundo"}</button>
          {#if draft.background}<button class="btn btn-plain btn-icon" aria-label="Tirar fundo" onclick={() => draft && (draft.background = null)}><X size={14} /></button>{/if}
        </div>
        <p class="muted small">Sem áudio, a música é apresentada só com a letra (você passa os slides).</p>
      {/if}
    </div>

    <div class="preview">
      <span class="section-title">{preview.length - 1} {preview.length - 1 === 1 ? "slide" : "slides"} de letra</span>
      <div class="slides">
        {#each preview as slide, i (i)}
          <div class="thumb"><SlideView {slide} style={settings.current.slide} animate={false} /></div>
        {/each}
      </div>
    </div>
  </div>
{:else}
  <div class="empty"><LoaderCircle size={28} class="spin" /></div>
{/if}

<style>
  .editor {
    display: grid;
    grid-template-columns: minmax(18rem, 1fr) minmax(14rem, 22rem);
    gap: 1.5rem;
    align-items: start;
  }

  .form {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }

  .label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-weight: 600;
  }

  .input {
    padding: 0.6rem 0.75rem;
    border-radius: var(--radius);
    border: 1px solid var(--border-strong);
    background: var(--surface);
    font: inherit;
    font-weight: 400;
    outline: none;
  }

  .input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .lyrics {
    min-height: 22rem;
    resize: vertical;
    line-height: 1.5;
  }

  .files {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.4rem;
  }

  .small {
    font-size: 0.85rem;
  }

  .preview {
    position: sticky;
    top: 0;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .slides {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-height: 70vh;
    overflow-y: auto;
  }

  .thumb {
    aspect-ratio: 16 / 9;
    border-radius: var(--radius-s);
    overflow: hidden;
    flex: none;
  }
</style>
