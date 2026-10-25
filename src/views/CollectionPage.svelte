<script lang="ts">
  import AlbumCard from "../components/AlbumCard.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import { catalog } from "../lib/state/catalog.svelte";
  import { nav } from "../lib/state/nav.svelte";

  let { id }: { id: number } = $props();
  const collection = $derived(catalog.collections.find((c) => c.id === id));
</script>

{#if collection}
  <PageHeader
    title={collection.name}
    subtitle="{collection.albums.length} {collection.albums.length === 1 ? 'álbum' : 'álbuns'}"
  />
  <div class="grid">
    {#each collection.albums as album (album.id)}
      <AlbumCard {album} onclick={() => nav.push({ name: "album", id: album.id })} />
    {/each}
  </div>
{/if}

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(9.5rem, 1fr));
    gap: 1.5rem 1.1rem;
  }
</style>
