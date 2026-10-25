<!-- Favoritos e recentes: mesma tela, listas diferentes. -->
<script lang="ts">
  import { Clock, Heart } from "@lucide/svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import SongList from "../components/SongList.svelte";
  import { api } from "../lib/api";
  import { playSong, selectSong } from "../lib/actions";
  import { settings } from "../lib/state/settings.svelte";
  import type { SongSummary } from "../lib/types";

  let { kind }: { kind: "favorites" | "recent" } = $props();

  let songs = $state<SongSummary[]>([]);
  $effect(() => {
    const ids = kind === "favorites" ? settings.current.favorites : settings.current.recent;
    void api.songs(ids).then((s) => (songs = s));
  });
</script>

<PageHeader
  title={kind === "favorites" ? "Favoritos" : "Recentes"}
  subtitle={kind === "favorites" ? "Os hinos que você marcou com ♥" : "Os últimos hinos tocados"}
/>

{#if songs.length}
  <SongList {songs} onselect={selectSong} onplay={playSong} numbered={false} />
{:else}
  <div class="empty">
    {#if kind === "favorites"}
      <Heart size={44} strokeWidth={1.4} />
      <h2>Nenhum favorito ainda</h2>
      <p>Toque no coração ao lado de um hino para encontrá-lo aqui depois.</p>
    {:else}
      <Clock size={44} strokeWidth={1.4} />
      <h2>Nada tocado ainda</h2>
      <p>Os hinos que você tocar aparecem aqui.</p>
    {/if}
  </div>
{/if}
