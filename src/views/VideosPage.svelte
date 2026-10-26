<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { Film, ListChecks, ListPlus, Pause, Play, Plus, Repeat, Square, Trash2, Volume2 } from "@lucide/svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import VideoLayer from "../components/VideoLayer.svelte";
  import { api, errorMessage } from "../lib/api";
  import { playVideo } from "../lib/actions";
  import { setlist } from "../lib/state/setlist.svelte";
  import { formatTime } from "../lib/format";
  import { outputs } from "../lib/state/outputs.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import { toasts } from "../lib/state/toasts.svelte";
  import { video } from "../lib/state/video.svelte";
  import type { VideoItem } from "../lib/types";
  import { formatWarning, VIDEO_EXTENSIONS } from "../lib/videoFormats";
  import { guessTag, tagInfo, VIDEO_TAGS, WEEKLY_TAGS } from "../lib/videoTags";

  let { simple = false }: { simple?: boolean } = $props();

  /** Telas escolhidas para o próximo vídeo. Vazio = todas as abertas. */
  let chosen = $state<string[]>([]);
  const targets = $derived(
    chosen.filter((l) => outputs.outputs.some((o) => o.label === l)).length
      ? chosen.filter((l) => outputs.outputs.some((o) => o.label === l))
      : outputs.forVideo.map((o) => o.label),
  );

  async function addVideos() {
    const picked = await open({
      multiple: true,
      title: "Escolha os vídeos",
      filters: [{ name: "Vídeos", extensions: VIDEO_EXTENSIONS }],
    });
    const paths = Array.isArray(picked) ? picked : picked ? [picked] : [];
    if (!paths.length) return;
    try {
      await api.addVideos(paths);
    } catch (e) {
      toasts.error(errorMessage(e));
      return;
    }
    const warning = paths.map(formatWarning).find((w) => w !== null);
    if (warning) toasts.show(warning, "info", undefined, 12_000);
    // "Informativo…" e "Provai e Vede…" viram os de hoje sozinhos.
    for (const path of paths) {
      const tag = guessTag(path);
      if (!tag) continue;
      await setTag(path, tag);
      toasts.show(`“${tagInfo(tag)?.label}” de hoje atualizado.`, "success");
    }
  }

  function play(item: VideoItem) {
    return playVideo(item, targets);
  }

  function toggleTarget(label: string) {
    const current = targets;
    const next = current.includes(label) ? current.filter((l) => l !== label) : [...current, label];
    chosen = next;
    if (video.state) video.setTargets(next);
  }

  async function setTag(path: string, tag: string | null) {
    try {
      settings.value = await api.tagVideo(path, tag);
    } catch (e) {
      toasts.error(errorMessage(e));
    }
  }

  /** O vídeo "de hoje" de cada tipo semanal. */
  const weekly = $derived(
    WEEKLY_TAGS.map((tag) => ({ tag, item: settings.current.videos.find((v) => v.tag === tag.id) ?? null })),
  );

  const live = $derived(video.state);
  const progress = $derived(video.duration ? (video.position / video.duration) * 100 : 0);
</script>

<PageHeader title="Vídeos" subtitle="Projete vídeos do computador em uma ou mais telas">
  {#snippet actions()}
    <button class="btn btn-primary" class:btn-lg={simple} onclick={addVideos}><Plus size={16} /> Adicionar vídeos</button>
  {/snippet}
</PageHeader>

{#if live}
  <section class="player card">
    <div class="preview">
      <VideoLayer
        state={live}
        muted
        bind:duration={video.duration}
        bind:position={video.position}
        onended={() => video.ended()}
        onerror={() => video.failed(live.path, live.title)}
      />
    </div>
    <div class="controls">
      <div class="title-row">
        <span class="pill live">No telão</span>
        <h3 class="truncate">{live.title}</h3>
      </div>
      <div class="seek">
        <span class="time">{formatTime(video.position)}</span>
        <input
          type="range"
          min="0"
          max={video.duration || 1}
          step="0.1"
          value={video.position}
          style:--progress="{progress}%"
          oninput={(e) => video.seek(Number(e.currentTarget.value))}
          aria-label="Posição do vídeo"
        />
        <span class="time">{formatTime(video.duration)}</span>
      </div>
      <div class="buttons">
        <button class="btn btn-primary btn-lg" onclick={() => video.toggle()}>
          {#if live.playing}<Pause size={18} fill="currentColor" /> Pausar{:else}<Play size={18} fill="currentColor" /> Continuar{/if}
        </button>
        <button class="btn btn-lg" onclick={() => video.stop()}><Square size={16} /> Parar</button>
        <button
          class="btn btn-lg btn-icon"
          class:on={live.looping}
          title="Repetir"
          aria-label="Repetir"
          aria-pressed={live.looping}
          onclick={() => video.setLooping(!live.looping)}><Repeat size={18} /></button
        >
        <label class="volume" title="Volume">
          <Volume2 size={18} />
          <input
            type="range"
            min="0"
            max="1"
            step="0.05"
            value={live.volume}
            style:--progress="{live.volume * 100}%"
            oninput={(e) => video.setVolume(Number(e.currentTarget.value))}
            aria-label="Volume"
          />
        </label>
      </div>
    </div>
  </section>
{/if}

<section class="weekly">
  {#each weekly as { tag, item } (tag.id)}
    <div class="week card" style:--tint={tag.tint}>
      <span class="week-icon"><Film size={22} /></span>
      <span class="week-info">
        <span class="week-label">{tag.label} <span class="muted">de hoje</span></span>
        <span class="truncate muted">{item ? item.title : "Escolha o vídeo na lista abaixo, em “Tipo”."}</span>
      </span>
      {#if item}
        <button class="btn btn-primary" class:btn-lg={simple} onclick={() => play(item)}><Play size={15} fill="currentColor" /> Exibir</button>
      {/if}
    </div>
  {/each}
</section>

<section class="targets">
  <span class="section-title">Exibir em</span>
  <div class="chips">
    {#each outputs.outputs as o (o.label)}
      <button class="chip" class:on={targets.includes(o.label)} aria-pressed={targets.includes(o.label)} onclick={() => toggleTarget(o.label)}>
        {o.name}{o.role === "monitor" ? " (retorno)" : ""}
      </button>
    {:else}
      <span class="muted">Nenhuma tela aberta — ao tocar, a projeção abre sozinha.</span>
    {/each}
  </div>
</section>

{#if settings.current.videos.length}
  <ul class="videos">
    {#each settings.current.videos as item (item.path)}
      {@const inSetlist = setlist.has({ kind: "video", ...item })}
      {@const tag = tagInfo(item.tag)}
      <li class="video" class:playing={live?.path === item.path}>
        <span class="thumb"><Film size={20} /></span>
        <span class="info">
          <span class="truncate name">
            {item.title}
            {#if tag}<span class="tag" style:--tint={tag.tint}>{tag.short}{tag.weekly ? " · hoje" : ""}</span>{/if}
          </span>
          <span class="truncate path">{item.path}</span>
        </span>
        <select
          class="kind"
          aria-label="Tipo do vídeo"
          title="Tipo do vídeo"
          value={item.tag ?? ""}
          onchange={(e) => setTag(item.path, e.currentTarget.value || null)}
        >
          <option value="">Tipo…</option>
          {#each VIDEO_TAGS as t (t.id)}
            <option value={t.id}>{t.short}{t.weekly ? " (de hoje)" : ""}</option>
          {/each}
        </select>
        <button
          class="btn btn-plain btn-icon"
          class:on={inSetlist}
          aria-label={inSetlist ? "Tirar do culto de hoje" : "Adicionar ao culto de hoje"}
          title={inSetlist ? "Está no culto de hoje" : "Adicionar ao culto de hoje"}
          onclick={() => setlist.toggle({ kind: "video", ...item })}
        >
          {#if inSetlist}<ListChecks size={17} />{:else}<ListPlus size={17} />{/if}
        </button>
        <button class="btn btn-primary" class:btn-lg={simple} onclick={() => play(item)}><Play size={15} fill="currentColor" /> Exibir</button>
        <button class="btn btn-plain btn-icon" aria-label="Remover da lista" title="Remover da lista" onclick={() => api.removeVideo(item.path)}>
          <Trash2 size={16} />
        </button>
      </li>
    {/each}
  </ul>
{:else}
  <div class="empty">
    <Film size={44} strokeWidth={1.4} />
    <h2>Nenhum vídeo ainda</h2>
    <p>Adicione vídeos do computador (MP4, MOV, WebM) para exibir no telão.</p>
  </div>
{/if}

<style>
  .weekly {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(17rem, 1fr));
    gap: 0.75rem;
    margin-bottom: 1.25rem;
  }

  .week {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    min-width: 0;
    padding: 0.8rem 0.9rem;
  }

  .week-icon {
    display: grid;
    place-items: center;
    flex: none;
    width: 2.6rem;
    height: 2.6rem;
    border-radius: var(--radius);
    color: #fff;
    background: var(--tint);
  }

  .week-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .week-label {
    font-weight: 600;
  }

  .tag {
    margin-left: 0.4rem;
    padding: 0.05rem 0.45rem;
    border-radius: 999px;
    font-size: 0.75rem;
    font-weight: 600;
    color: #fff;
    background: var(--tint);
    vertical-align: middle;
  }

  .kind {
    flex: none;
    max-width: 11rem;
    padding: 0.35rem 0.5rem;
    border-radius: var(--radius);
    border: 1px solid var(--border-strong);
    background: var(--surface);
    color: var(--text);
    font: inherit;
    font-size: 0.85rem;
  }

  .player {
    display: grid;
    grid-template-columns: minmax(14rem, 22rem) 1fr;
    gap: 1.25rem;
    padding: 1rem;
    margin-bottom: 1.5rem;
  }

  .preview {
    aspect-ratio: 16 / 9;
    border-radius: var(--radius);
    overflow: hidden;
    background: #000;
  }

  .controls {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 0.9rem;
    min-width: 0;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    min-width: 0;
  }

  .seek {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .seek input {
    flex: 1;
  }

  .time {
    font-size: 0.82rem;
    color: var(--text-2);
    font-variant-numeric: tabular-nums;
    min-width: 2.8rem;
  }

  .buttons {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
  }

  .volume {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin-left: auto;
    color: var(--text-2);
  }

  .targets {
    display: flex;
    align-items: center;
    gap: 0.9rem;
    margin-bottom: 1rem;
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .chip {
    height: 2rem;
    padding: 0 0.9rem;
    border-radius: 1rem;
    border: 1px solid var(--border-strong);
    background: transparent;
    font-weight: 550;
    cursor: pointer;
    transition: all var(--fast) var(--ease);
  }

  .chip.on {
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
  }

  .videos {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .video {
    display: flex;
    align-items: center;
    gap: 0.9rem;
    padding: 0.6rem 0.75rem;
    border-radius: var(--radius-l);
    background: var(--surface);
    border: 1px solid var(--border);
  }

  .video.playing {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .thumb {
    display: grid;
    place-items: center;
    width: 3.2rem;
    height: 2.2rem;
    flex: none;
    border-radius: var(--radius-s);
    background: linear-gradient(145deg, #3a3a4a, #15151d);
    color: rgba(255, 255, 255, 0.8);
  }

  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .name {
    font-weight: 550;
  }

  .path {
    font-size: 0.8rem;
    color: var(--text-3);
  }
</style>
