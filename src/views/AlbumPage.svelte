<script lang="ts">
  import { LoaderCircle, Play } from "@lucide/svelte";
  import Cover from "../components/Cover.svelte";
  import DownloadButton from "../components/DownloadButton.svelte";
  import SongList from "../components/SongList.svelte";
  import { api, errorMessage } from "../lib/api";
  import { playSong, selectSong } from "../lib/actions";
  import { plural } from "../lib/format";
  import type { Album } from "../lib/types";

  let { id }: { id: number } = $props();

  let album = $state<Album | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    const albumId = id;
    album = null;
    error = null;
    api
      .album(albumId)
      .then((a) => (album = a))
      .catch((e) => (error = errorMessage(e)));
  });
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
        <DownloadButton key="album:{album.id}" scope={{ kind: "album", id: album.id }} label={album.name} large />
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
