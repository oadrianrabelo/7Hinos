<script lang="ts">
  import Cover from "./Cover.svelte";
  import type { AlbumSummary } from "../lib/types";

  let { album, onclick }: { album: AlbumSummary; onclick: () => void } = $props();
</script>

<button class="card-album" {onclick}>
  <Cover media={album.cover} color={album.color} alt={album.name} />
  <span class="name truncate">{album.name}</span>
  {#if album.subtitle}<span class="sub truncate">{album.subtitle}</span>{/if}
</button>

<style>
  .card-album {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
    padding: 0;
    border: 0;
    background: none;
    text-align: left;
    cursor: pointer;
  }

  .card-album :global(.cover) {
    margin-bottom: 0.45rem;
    transition:
      transform var(--normal) var(--ease),
      box-shadow var(--normal) var(--ease);
  }

  .card-album:hover :global(.cover) {
    transform: translateY(-2px);
    box-shadow:
      inset 0 0 0 1px var(--border),
      var(--shadow);
  }

  .card-album:active :global(.cover) {
    transform: scale(0.98);
  }

  .name {
    font-weight: 550;
    font-size: 0.95rem;
  }

  .sub {
    font-size: 0.82rem;
    color: var(--text-2);
  }
</style>
