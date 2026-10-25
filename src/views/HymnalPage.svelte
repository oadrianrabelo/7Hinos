<script lang="ts">
  import PageHeader from "../components/PageHeader.svelte";
  import SearchField from "../components/SearchField.svelte";
  import SongList from "../components/SongList.svelte";
  import { playSong, selectSong } from "../lib/actions";
  import { normalize } from "../lib/text";
  import { catalog } from "../lib/state/catalog.svelte";

  let filter = $state("");
  $effect(() => {
    void catalog.loadHymnal();
  });

  const songs = $derived.by(() => {
    const all = catalog.hymnal ?? [];
    const q = normalize(filter);
    if (!q) return all;
    if (/^\d+$/.test(q)) return all.filter((s) => String(s.number ?? "").startsWith(q));
    return all.filter((s) => normalize(s.title).includes(q));
  });
</script>

<PageHeader title="Hinário" subtitle="{(catalog.hymnal ?? []).length} hinos">
  {#snippet actions()}
    <div class="filter"><SearchField bind:value={filter} placeholder="Número ou nome" /></div>
  {/snippet}
</PageHeader>

<SongList {songs} onselect={selectSong} onplay={playSong} showSubtitle={false} />

<style>
  .filter {
    width: 15rem;
  }
</style>
