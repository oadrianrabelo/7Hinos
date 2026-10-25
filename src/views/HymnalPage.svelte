<script lang="ts">
  import PageHeader from "../components/PageHeader.svelte";
  import SearchField from "../components/SearchField.svelte";
  import SongList from "../components/SongList.svelte";
  import { playSong, selectSong } from "../lib/actions";
  import { normalize } from "../lib/text";
  import { catalog } from "../lib/state/catalog.svelte";

  let filter = $state("");
  let edition = $state<"current" | "1996">("current");
  $effect(() => {
    void catalog.loadHymnal();
  });

  const all = $derived((edition === "1996" ? catalog.hymnal1996 : catalog.hymnal) ?? []);
  const songs = $derived.by(() => {
    const q = normalize(filter);
    if (!q) return all;
    if (/^\d+$/.test(q)) return all.filter((s) => String(s.number ?? "").startsWith(q));
    return all.filter((s) => normalize(s.title).includes(q));
  });
</script>

<PageHeader title="Hinário" subtitle="{all.length} hinos">
  {#snippet actions()}
    {#if catalog.hymnal1996?.length}
      <div class="segmented">
        <button aria-pressed={edition === "current"} onclick={() => (edition = "current")}>Atual</button>
        <button aria-pressed={edition === "1996"} onclick={() => (edition = "1996")}>1996</button>
      </div>
    {/if}
    <div class="filter"><SearchField bind:value={filter} placeholder="Número ou nome" /></div>
  {/snippet}
</PageHeader>

<SongList {songs} onselect={selectSong} onplay={playSong} showSubtitle={false} />

<style>
  .filter {
    width: 15rem;
  }
</style>
