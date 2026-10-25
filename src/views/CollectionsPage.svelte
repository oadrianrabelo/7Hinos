<script lang="ts">
  import Cover from "../components/Cover.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import { catalog } from "../lib/state/catalog.svelte";
  import { nav } from "../lib/state/nav.svelte";
</script>

<PageHeader title="Coletâneas" subtitle="CDs oficiais, hinários e coleções do LouvorJA" />

<div class="grid">
  {#each catalog.collections as c (c.id)}
    <button class="collection card" onclick={() => nav.push({ name: "collection", id: c.id })}>
      <div class="covers">
        {#each c.albums.slice(0, 3) as album (album.id)}
          <Cover media={album.cover} color={album.color} radius="var(--radius-s)" />
        {/each}
      </div>
      <span class="name">{c.name}</span>
      <span class="muted">{c.albums.length} {c.albums.length === 1 ? "álbum" : "álbuns"}</span>
    </button>
  {/each}
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(17rem, 1fr));
    gap: 1rem;
  }

  .collection {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    padding: 1rem;
    text-align: left;
    cursor: pointer;
    transition:
      transform var(--fast) var(--ease),
      box-shadow var(--normal);
  }

  .collection:hover {
    box-shadow: var(--shadow);
    transform: translateY(-1px);
  }

  .covers {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.4rem;
    margin-bottom: 0.7rem;
  }

  .name {
    font-weight: 650;
    font-size: 1.1rem;
  }
</style>
