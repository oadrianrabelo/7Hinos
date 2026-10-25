<script lang="ts">
  import { CircleCheck, CloudDownload, LoaderCircle, Play } from "@lucide/svelte";
  import Cover from "../components/Cover.svelte";
  import SongList from "../components/SongList.svelte";
  import { api, errorMessage, on } from "../lib/api";
  import { playSong, selectSong } from "../lib/actions";
  import { plural } from "../lib/format";
  import { toasts } from "../lib/state/toasts.svelte";
  import type { Album, OfflineStatus } from "../lib/types";

  let { id }: { id: number } = $props();

  let album = $state<Album | null>(null);
  let error = $state<string | null>(null);
  let offline = $state<OfflineStatus | null>(null);
  let downloading = $state(false);

  $effect(() => {
    const albumId = id;
    album = null;
    error = null;
    offline = null;
    api
      .album(albumId)
      .then((a) => {
        album = a;
        return api.albumOffline(albumId);
      })
      .then((o) => (offline = o))
      .catch((e) => (error = errorMessage(e)));
  });

  $effect(() => {
    const unlisten = on("download://progress", (p) => {
      if (p.albumId !== id) return;
      offline = { cached: p.cached, total: p.total };
      downloading = !p.finished;
    });
    return () => void unlisten.then((f) => f());
  });

  async function download() {
    downloading = true;
    try {
      offline = await api.downloadAlbum(id);
      toasts.show(`“${album?.name}” está disponível sem internet.`, "success");
    } catch (e) {
      toasts.error(errorMessage(e));
    } finally {
      downloading = false;
    }
  }

  const complete = $derived(offline !== null && offline.total > 0 && offline.cached >= offline.total);
  const percent = $derived(offline && offline.total ? Math.round((offline.cached / offline.total) * 100) : 0);
</script>

{#if error}
  <div class="empty"><h2>Não foi possível abrir o álbum</h2><p>{error}</p></div>
{:else if album}
  <header class="hero">
    <div class="art"><Cover media={album.cover} color={album.color} radius="var(--radius-l)" /></div>
    <div class="meta">
      <span class="section-title">Álbum</span>
      <h1>{album.name}</h1>
      <p class="muted">{plural(album.songs.length, "música", "músicas")}</p>
      <div class="actions">
        {#if album.songs.length}
          <button class="btn btn-primary btn-lg" onclick={() => album && playSong(album.songs[0])}>
            <Play size={18} fill="currentColor" /> Tocar
          </button>
        {/if}
        {#if complete}
          <span class="pill accent"><CircleCheck size={14} /> Disponível sem internet</span>
        {:else}
          <button class="btn btn-lg" onclick={download} disabled={downloading}>
            {#if downloading}
              <LoaderCircle size={18} class="spin" /> Baixando… {percent}%
            {:else}
              <CloudDownload size={18} /> Baixar para usar sem internet
            {/if}
          </button>
        {/if}
      </div>
    </div>
  </header>
  <SongList songs={album.songs} onselect={selectSong} onplay={playSong} showSubtitle={false} />
{:else}
  <div class="empty"><LoaderCircle class="spin" size={28} /></div>
{/if}

<style>
  .hero {
    display: flex;
    align-items: flex-end;
    gap: 1.75rem;
    margin-bottom: 1.75rem;
  }

  .art {
    width: 12rem;
    flex: none;
  }

  .art :global(.cover) {
    box-shadow:
      inset 0 0 0 1px var(--border),
      var(--shadow-l);
  }

  .meta {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    min-width: 0;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.6rem;
    margin-top: 0.8rem;
  }
</style>
